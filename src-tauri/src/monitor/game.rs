use crate::db::Database;
use crate::models::{Game, Platform};
use crate::AppError;
use parking_lot::Mutex as PLMutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use windows::Win32::Foundation::*;
use windows::Win32::System::ProcessStatus::*;
use windows::Win32::System::Threading::*;
use windows::Win32::UI::WindowsAndMessaging::*;

#[derive(Clone, Debug)]
pub struct ProcessInfo {
    pub process_name: String,
    pub process_id: u32,
    pub window_handle: HWND,
}

#[derive(Clone, Debug)]
pub struct GameSession {
    pub game_id: String,
    pub start_time: SystemTime,
    pub process_id: u32,
    pub window_handle: HWND,
}

pub type GameMap = PLMutex<HashMap<String, GameSession>>;

pub struct GameMonitor {
    active_games: Arc<GameMap>,
    database: Arc<Database>,
    known_games: Arc<GameMap>,
}

impl GameMonitor {
    pub fn new(database: Arc<Database>) -> Self {
        Self {
            active_games: Arc::new(PLMutex::new(HashMap::new())),
            database,
            known_games: Arc::new(PLMutex::new(HashMap::new())),
        }
    }

    fn sanitize_command(executable: &str) -> String {
        executable
            .split("--")
            .next()
            .unwrap_or(executable)
            .split('\"')
            .next()
            .unwrap_or(executable)
            .trim()
            .to_string()
    }

    pub fn start_monitoring(&self) {
        let active_games = self.active_games.clone();
        let database = self.database.clone();

        tokio::spawn(async move {
            loop {
                let games_to_check = {
                    let games = active_games.lock();
                    if games.is_empty() {
                        break;
                    }
                    games
                        .iter()
                        .map(|(k, v)| (k.clone(), v.clone()))
                        .collect::<Vec<_>>()
                };

                for (game_id, session) in games_to_check {
                    let is_running = unsafe {
                        if let Ok(handle) =
                            OpenProcess(PROCESS_QUERY_INFORMATION, false, session.process_id)
                        {
                            let mut exit_code = 0u32;
                            GetExitCodeProcess(handle, &mut exit_code).as_bool() && exit_code == 259
                        } else {
                            false
                        }
                    };

                    if !is_running {
                        if let Ok(duration) = session.start_time.elapsed() {
                            let duration_secs = duration.as_secs() as i64;
                            if let Ok(session_id) =
                                database.sessions().start_session(&game_id).await
                            {
                                // Passer la durée calculée directement
                                match database
                                    .sessions()
                                    .end_session(session_id, duration_secs)
                                    .await
                                {
                                    // Nouveau paramètre
                                    Ok(_) => println!("✅ Session recorded successfully"),
                                    Err(e) => println!("❌ Error ending session: {}", e),
                                }
                            }
                        }
                        active_games.lock().remove(&game_id);
                    }
                }

                tokio::time::sleep(Duration::from_secs(1)).await;
            }
        });
    }

    unsafe extern "system" fn enum_window_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        let data = &mut *(lparam.0 as *mut CallbackData);
        let mut process_id: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut process_id));

        if let Ok(process) = OpenProcess(
            PROCESS_QUERY_INFORMATION | PROCESS_VM_READ,
            false,
            process_id,
        ) {
            let mut buffer = [0u16; MAX_PATH as usize];

            if GetModuleBaseNameW(process, None, &mut buffer) > 0 {
                let current_name = String::from_utf16_lossy(&buffer)
                    .trim_matches('\0')
                    .to_lowercase();

                let is_main_window = IsWindowVisible(hwnd).as_bool()
                    && GetWindow(hwnd, GW_OWNER).0 == 0
                    && GetWindowTextLengthW(hwnd) > 0;

                if is_main_window && current_name.contains(&data.process_name) {
                    data.result = Some((hwnd, process_id));
                    return BOOL(0);
                }
            }
        }
        BOOL(1)
    }

    pub async fn track_game(&self, game: &Game) -> Option<GameSession> {
        let mut possible_names = Vec::new();
        possible_names.push(game.title.replace(" ", "").to_lowercase() + ".exe");

        if let Some(exe_path) = &game.installation.executable {
            let clean_exe = Self::sanitize_command(exe_path);
            if let Some(exe_name) = std::path::Path::new(&clean_exe)
                .file_name()
                .and_then(|n| n.to_str())
            {
                possible_names.push(exe_name.to_lowercase());
            }
        }

        if game.platform == Platform::BattleNet {
            match game.id.as_str() {
                "battlenet_overwatch" => possible_names.push("overwatch.exe".to_string()),
                "battlenet_wow" => possible_names.push("wow.exe".to_string()),
                "battlenet_d4" => possible_names.push("diablo iv.exe".to_string()),
                _ => {}
            }
        }

        if game.platform == Platform::Steam {
            if let Ok(entries) = std::fs::read_dir(&game.installation.install_path) {
                for entry in entries.flatten() {
                    if let Some(ext) = entry.path().extension() {
                        if ext == "exe" {
                            if let Some(name) = entry.path().file_name().and_then(|n| n.to_str()) {
                                if !name.to_lowercase().contains("unins")
                                    && !name.to_lowercase().contains("crash")
                                    && !name.to_lowercase().contains("unity")
                                    && !name.to_lowercase().contains("ue4")
                                {
                                    possible_names.push(name.to_lowercase());
                                }
                            }
                        }
                    }
                }
            }
        }

        for process_name in possible_names {
            let mut data = CallbackData {
                process_name: process_name.clone(),
                result: None,
            };

            unsafe {
                EnumWindows(
                    Some(Self::enum_window_proc),
                    LPARAM(&mut data as *mut _ as isize),
                );
            }

            if let Some((window_handle, process_id)) = data.result {
                let session = GameSession {
                    game_id: game.id.clone(),
                    start_time: SystemTime::now(),
                    process_id,
                    window_handle,
                };

                {
                    let mut games = self.active_games.lock();
                    let was_empty = games.is_empty();
                    games.insert(game.id.clone(), session.clone());

                    if was_empty {
                        self.start_monitoring();
                    }
                }

                return Some(session);
            }
        }
        None
    }

    fn is_process_running(process_id: u32) -> bool {
        unsafe {
            if let Ok(handle) = OpenProcess(PROCESS_QUERY_INFORMATION, false, process_id) {
                let mut exit_code = 0u32;
                if GetExitCodeProcess(handle, &mut exit_code).as_bool() {
                    return exit_code == 259;
                }
            }
            false
        }
    }

    pub fn get_active_game(&self) -> Option<GameSession> {
        let games = self.active_games.lock();
        games
            .values()
            .find(|session| Self::is_process_running(session.process_id))
            .cloned()
    }

    pub fn is_game_running(&self, game_id: &str) -> bool {
        self.active_games.lock().contains_key(game_id)
    }
}

struct CallbackData {
    process_name: String,
    result: Option<(HWND, u32)>,
}
