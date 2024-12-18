// src-tauri/src/overlay/window.rs
use windows::core::{w, Error, Result, PCWSTR};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

pub struct OverlayWindow {
    hwnd: HWND,
}

impl OverlayWindow {
    pub fn new() -> Result<Self> {
        unsafe {
            let instance = GetModuleHandleW(PCWSTR::null())?;
            let class_name = w!("GLaunchOverlay");

            let wc = WNDCLASSEXW {
                cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
                lpfnWndProc: Some(Self::window_proc),
                hInstance: instance,
                lpszClassName: class_name,
                // Paramètres importants pour l'overlay
                style: CS_HREDRAW | CS_VREDRAW,
                ..Default::default()
            };

            RegisterClassExW(&wc);

            let hwnd = CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TRANSPARENT | WS_EX_TOPMOST | WS_EX_TOOLWINDOW,
                class_name,
                w!("Overlay"),
                WS_POPUP | WS_VISIBLE,
                0,
                0,
                GetSystemMetrics(SM_CXSCREEN),
                GetSystemMetrics(SM_CYSCREEN),
                HWND(0),
                HMENU(0),
                instance,
                None,
            );

            if hwnd.0 == 0 {
                return Err(Error::from_win32());
            }

            Ok(Self { hwnd })
        }
    }

    pub fn attach_to_window(&self, game_hwnd: HWND) -> Result<()> {
        unsafe {
            // Obtenir les dimensions de la fenêtre du jeu
            let mut rect = RECT::default();
            GetWindowRect(game_hwnd, &mut rect);

            // Positionner et redimensionner l'overlay pour correspondre au jeu
            SetWindowPos(
                self.hwnd,
                HWND_TOPMOST,
                rect.left,
                rect.top,
                rect.right - rect.left,
                rect.bottom - rect.top,
                SWP_NOACTIVATE | SWP_SHOWWINDOW,
            );

            // Configurer la fenêtre pour l'overlay
            let style = (WS_EX_LAYERED.0 | WS_EX_TRANSPARENT.0 | WS_EX_TOPMOST.0) as i32;
            SetWindowLongW(self.hwnd, GWL_EXSTYLE, style);
        }
        Ok(())
    }

    pub unsafe extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        DefWindowProcW(hwnd, msg, wparam, lparam)
    }

    pub fn show(&self) -> Result<()> {
        unsafe {
            ShowWindow(self.hwnd, SW_SHOW);
        }
        Ok(())
    }

    pub fn hide(&self) -> Result<()> {
        unsafe {
            ShowWindow(self.hwnd, SW_HIDE);
        }
        Ok(())
    }

    pub fn make_transparent(&self) -> Result<()> {
        unsafe {
            SetLayeredWindowAttributes(
                self.hwnd,
                COLORREF(0),
                128, // 50% d'opacité
                LWA_ALPHA,
            );
        }
        Ok(())
    }

    pub fn set_topmost(&self) -> Result<()> {
        unsafe {
            SetWindowPos(self.hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
        }
        Ok(())
    }
}

impl Drop for OverlayWindow {
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.hwnd);
        }
    }
}
