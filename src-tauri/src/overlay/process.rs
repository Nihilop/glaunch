use windows::Win32::Foundation::*;
use windows::Win32::System::ProcessStatus::GetModuleBaseNameW;
use windows::Win32::System::Threading::*;
use windows::Win32::UI::WindowsAndMessaging::GetWindowLongW;
use windows::Win32::UI::WindowsAndMessaging::GWL_STYLE;
use windows::Win32::UI::WindowsAndMessaging::WS_VISIBLE;
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowExW, GetWindowPlacement, GetWindowThreadProcessId, SW_SHOWMAXIMIZED, WINDOWPLACEMENT,
};

pub struct GameProcess {
    pub process_id: u32,
    pub window_handle: HWND,
}

impl GameProcess {
    pub fn find_game_window(process_name: &str) -> Option<Self> {
        unsafe {
            // Chercher spécifiquement la fenêtre du jeu
            let window = FindWindowExW(None, None, None, None);

            let mut process_id = 0;
            if GetWindowThreadProcessId(window, Some(&mut process_id)) != 0 {
                // Vérifier si c'est vraiment la fenêtre principale du jeu
                let style = GetWindowLongW(window, GWL_STYLE) as u32;
                if (style & (WS_VISIBLE.0)) != 0 {
                    match OpenProcess(PROCESS_QUERY_INFORMATION, false, process_id) {
                        Ok(process) => {
                            let mut name_buf = [0u16; MAX_PATH as usize];

                            if GetModuleBaseNameW(process, None, &mut name_buf) > 0 {
                                let name = String::from_utf16_lossy(&name_buf)
                                    .trim_matches('\0')
                                    .to_lowercase();

                                if name.contains(process_name) {
                                    return Some(Self {
                                        process_id,
                                        window_handle: window,
                                    });
                                }
                            }
                        }
                        Err(_) => return None,
                    }
                }
            }
            None
        }
    }

    pub fn is_fullscreen(&self) -> bool {
        unsafe {
            let mut placement = WINDOWPLACEMENT::default();
            GetWindowPlacement(self.window_handle, &mut placement);

            placement.showCmd.0 == SW_SHOWMAXIMIZED.0
        }
    }
}
