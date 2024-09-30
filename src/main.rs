// Only use the windows subsystem if in release mode (hides the terminal).
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;

use crate::{app::App, win32::clipboard::WindowsClipboard};
use windows::{
    core::w,
    Win32::{
        Foundation::*,
        System::{DataExchange::*, LibraryLoader::GetModuleHandleW},
        UI::{Shell::*, WindowsAndMessaging::*},
    },
};

mod app;
mod clipboard;
mod config;
mod win32;

/// Custom window message used for tray icon events.
const WM_TRAYICON: u32 = WM_USER + 1;

#[derive(Debug, Clone, Copy)]
enum TrayButton {
    OpenDir,
    Exit,
}

impl TryFrom<u16> for TrayButton {
    type Error = ();

    fn try_from(value: u16) -> Result<Self, ()> {
        match value {
            0 => Ok(TrayButton::OpenDir),
            1 => Ok(TrayButton::Exit),
            _ => Err(()),
        }
    }
}

fn high_word(value: usize) -> u16 {
    ((value >> 16) & 0xFFFF) as u16
}

fn low_word(value: usize) -> u16 {
    (value & 0xFFFF) as u16
}

fn main() {
    unsafe {
        let class_name = w!("ClipboardHeraldClass");

        let wc = WNDCLASSW {
            hInstance: GetModuleHandleW(None).unwrap().into(),
            lpszClassName: class_name,
            lpfnWndProc: Some(wndproc),
            ..Default::default()
        };

        let atom = RegisterClassW(&wc);
        assert!(atom != 0, "failed to register window class");

        let window = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            class_name,
            w!("ClipboardHerald"),
            WINDOW_STYLE(0),
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            None,
            None,
            None,
        );
        assert!(window.0 != 0, "failed to create window");

        let config_string = std::fs::read_to_string("./config.toml");

        if config_string.is_err() {
            MessageBoxW(
                None,
                w!("Failed to open config.toml!"),
                w!("Error"),
                MB_OK | MB_ICONERROR,
            );
            return;
        }

        let clipboard_herald =
            App::<WindowsClipboard>::new(toml::from_str(&config_string.unwrap()).unwrap());

        SetWindowLongPtrW(
            window,
            GWLP_USERDATA,
            (&clipboard_herald as *const _) as isize,
        );

        let icon = LoadIconW(GetModuleHandleW(None).unwrap(), w!("icon")).unwrap();

        let mut tooltip = [0u8; 128];
        tooltip[.."ClipboardHerald".len()].copy_from_slice("ClipboardHerald".as_bytes());

        // Not using W because making a UTF-16 array is annoying.
        let nid = &NOTIFYICONDATAA {
            cbSize: std::mem::size_of::<NOTIFYICONDATAA>() as u32,
            hWnd: window,
            uFlags: NIF_TIP | NIF_SHOWTIP | NIF_ICON | NIF_MESSAGE,
            szTip: tooltip,
            hIcon: HICON(icon.0),
            uCallbackMessage: WM_TRAYICON,
            Anonymous: NOTIFYICONDATAA_0 {
                uVersion: NOTIFYICON_VERSION_4,
            },
            uID: 1,
            ..Default::default()
        };

        let ret = Shell_NotifyIconA(NIM_ADD, nid);
        assert!(ret == TRUE);

        let ret = Shell_NotifyIconA(NIM_SETVERSION, nid);
        assert!(ret == TRUE);

        let mut message: MSG = std::mem::zeroed();
        while GetMessageW(&mut message, None, 0, 0) == true {
            TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}

extern "system" fn wndproc(window: HWND, message: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        let instance = &*(GetWindowLongPtrW(window, GWLP_USERDATA) as *const App<WindowsClipboard>);

        match message {
            WM_CREATE => {
                AddClipboardFormatListener(window).unwrap();
                println!("Running...");
            }
            WM_DESTROY => {
                RemoveClipboardFormatListener(window).unwrap();
                PostQuitMessage(0)
            }
            WM_CLIPBOARDUPDATE => match instance.on_clipboard_update() {
                Ok(_) => {}
                Err(e) => log::error!("Error: {}", e),
            },
            WM_TRAYICON => {
                if (lparam.0 & 0xFFFF) == WM_CONTEXTMENU as isize {
                    let menu = CreatePopupMenu().unwrap();

                    AppendMenuW(
                        menu,
                        MF_STRING,
                        TrayButton::OpenDir as usize,
                        w!("Open directory"),
                    )
                    .unwrap();

                    AppendMenuW(menu, MF_STRING, TrayButton::Exit as usize, w!("Exit")).unwrap();

                    let mut point = std::mem::zeroed();

                    GetCursorPos(&mut point).unwrap();

                    SetForegroundWindow(window);

                    TrackPopupMenu(
                        menu,
                        TRACK_POPUP_MENU_FLAGS(0),
                        point.x,
                        point.y,
                        0,
                        window,
                        None,
                    )
                    .unwrap();
                }
            }
            WM_COMMAND => {
                let wparam = wparam.0;

                if high_word(wparam) == 0 {
                    let button = TrayButton::try_from(low_word(wparam)).unwrap();

                    match button {
                        TrayButton::OpenDir => {
                            Command::new("explorer")
                                .arg(std::env::current_dir().unwrap())
                                .spawn()
                                .unwrap();
                        }
                        TrayButton::Exit => {
                            DestroyWindow(window).unwrap();
                        }
                    }
                }
            }
            _ => {}
        }

        DefWindowProcW(window, message, wparam, lparam)
    }
}
