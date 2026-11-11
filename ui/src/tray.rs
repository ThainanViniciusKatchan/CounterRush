use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use slint::ComponentHandle;

use crate::{AppTray, MainWindow};

#[cfg(target_os = "windows")]
pub fn focus_counterrush_window() {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;

    #[link(name = "user32")]
    extern "system" {
        fn FindWindowW(lpClassName: *const u16, lpWindowName: *const u16) -> isize;
        fn SetForegroundWindow(hWnd: isize) -> i32;
        fn ShowWindow(hWnd: isize, nCmdShow: i32) -> i32;
        fn keybd_event(bVk: u8, bScan: u8, dwFlags: u32, dwExtraInfo: usize);
    }

    const SW_RESTORE: i32 = 9;
    const VK_MENU: u8 = 0x12;
    const KEYEVENTF_KEYUP: u32 = 0x0002;

    let titles = [
        "CounterRush - CS2 Performance",
        "CounterRush - Desempenho CS2",
        "CounterRush - Производительность CS2",
    ];

    unsafe {
        for title_str in titles {
            let title: Vec<u16> = OsStr::new(title_str)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect();

            let hwnd = FindWindowW(std::ptr::null(), title.as_ptr());
            if hwnd != 0 {
                ShowWindow(hwnd, SW_RESTORE);
                keybd_event(VK_MENU, 0, 0, 0);
                keybd_event(VK_MENU, 0, KEYEVENTF_KEYUP, 0);
                SetForegroundWindow(hwnd);
                break;
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
pub fn focus_counterrush_window() {}

pub fn decide_close_request(minimize_to_tray: bool) -> slint::CloseRequestResponse {
    if minimize_to_tray {
        slint::CloseRequestResponse::HideWindow
    } else {
        let _ = slint::quit_event_loop();
        slint::CloseRequestResponse::HideWindow
    }
}

/// Configura o comportamento da bandeja do sistema (SysTray) e o ciclo de vida
/// da janela principal do CounterRush.
pub fn setup_tray(
    main_window: &MainWindow,
    app_tray: &AppTray,
    settings_path: PathBuf,
    initial_minimize_to_tray: bool,
) -> Arc<AtomicBool> {
    let minimize_to_tray_flag = Arc::new(AtomicBool::new(initial_minimize_to_tray));

    // 1. Interceptação de fechamento ("X" da barra de título)
    let tray_flag_for_close = minimize_to_tray_flag.clone();
    main_window.window().on_close_requested(move || {
        let should_minimize = tray_flag_for_close.load(Ordering::SeqCst);
        decide_close_request(should_minimize)
    });

    // 2. Clique simples na bandeja restaura e foca a janela
    let main_window_weak = main_window.as_weak();
    app_tray.on_tray_clicked(move || {
        if let Some(ui) = main_window_weak.upgrade() {
            ui.window().set_minimized(false);
            let _ = ui.show();
            focus_counterrush_window();
        }
    });

    // 3. Seleção de aba no menu de contexto
    let main_window_weak = main_window.as_weak();
    app_tray.on_open_tab(move |tab_idx| {
        if let Some(ui) = main_window_weak.upgrade() {
            ui.set_active_tab(tab_idx);
            let lang = ui.get_active_language();
            let localized_msg = match (tab_idx, lang.as_str()) {
                (0, "en" | "en_US" | "en-US" | "en-EUA") => "Viewing Dashboard",
                (0, "ru" | "ru_RU" | "ru-RU") => "Просмотр панели управления",
                (0, _) => "Visualizando Painel Geral",
                (1, "en" | "en_US" | "en-US" | "en-EUA") => "Advanced Hardware Tweaks",
                (1, "ru" | "ru_RU" | "ru-RU") => "Расширенные настройки оборудования",
                (1, _) => "Ajustes Avançados de Hardware",
                (2, "en" | "en_US" | "en-US" | "en-EUA") => "General Settings",
                (2, "ru" | "ru_RU" | "ru-RU") => "Общие настройки",
                (2, _) => "Configurações Gerais",
                (3, "en" | "en_US" | "en-US" | "en-EUA") => "Valve Leaderboard",
                (3, "ru" | "ru_RU" | "ru-RU") => "Таблица лидеров Valve",
                (3, _) => "Tabela de Classificação Valve",
                _ => "CounterRush",
            };
            ui.set_status_message(localized_msg.into());
            ui.window().set_minimized(false);
            let _ = ui.show();
            focus_counterrush_window();
        }
    });

    // 4. Encerramento definitivo a partir do menu do SysTray
    app_tray.on_quit_app(move || {
        let _ = slint::quit_event_loop();
    });

    // 5. Alternância da configuração de minimização na UI
    let tray_flag_for_toggle = minimize_to_tray_flag.clone();
    let main_window_weak = main_window.as_weak();
    let settings_path_clone = settings_path;
    main_window.on_minimize_to_tray_toggled(move |enabled| {
        tray_flag_for_toggle.store(enabled, Ordering::SeqCst);

        let mut settings =
            counterrush_core::config::AppSettings::load_or_create(&settings_path_clone)
                .unwrap_or_default();
        settings.minimize_to_tray = enabled;
        let _ = settings.save_to_file(&settings_path_clone);

        if let Some(ui) = main_window_weak.upgrade() {
            let lang = ui.get_active_language();
            let msg = match lang.as_str() {
                "en" | "en_US" | "en-US" | "en-EUA" => {
                    if enabled {
                        "Minimize to system tray enabled."
                    } else {
                        "Minimize to system tray disabled."
                    }
                }
                "ru" | "ru_RU" | "ru-RU" => {
                    if enabled {
                        "Сворачивание в системный трей включено."
                    } else {
                        "Сворачивание в системный трей отключено."
                    }
                }
                _ => {
                    if enabled {
                        "Minimizar para a bandeja do sistema ativado."
                    } else {
                        "Minimizar para a bandeja do sistema desativado."
                    }
                }
            };
            ui.set_status_message(msg.into());
        }
    });

    minimize_to_tray_flag
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_focus_window_does_not_panic() {
        focus_counterrush_window();
    }

    #[test]
    fn test_tray_flag_atomic_behavior() {
        let flag = Arc::new(AtomicBool::new(false));
        assert!(!flag.load(Ordering::SeqCst));

        flag.store(true, Ordering::SeqCst);
        assert!(flag.load(Ordering::SeqCst));
    }

    #[test]
    fn test_close_request_response_logic() {
        assert_eq!(decide_close_request(true), slint::CloseRequestResponse::HideWindow);
        assert_eq!(decide_close_request(false), slint::CloseRequestResponse::HideWindow);
    }

    #[test]
    fn test_localized_status_messages() {
        let get_msg = |enabled: bool, lang: &str| match lang {
            "en" | "en_US" | "en-US" | "en-EUA" => {
                if enabled {
                    "Minimize to system tray enabled."
                } else {
                    "Minimize to system tray disabled."
                }
            }
            "ru" | "ru_RU" | "ru-RU" => {
                if enabled {
                    "Сворачивание в системный трей включено."
                } else {
                    "Сворачивание в системный трей отключено."
                }
            }
            _ => {
                if enabled {
                    "Minimizar para a bandeja do sistema ativado."
                } else {
                    "Minimizar para a bandeja do sistema desativado."
                }
            }
        };

        assert_eq!(get_msg(true, "pt_BR"), "Minimizar para a bandeja do sistema ativado.");
        assert_eq!(get_msg(false, "pt_BR"), "Minimizar para a bandeja do sistema desativado.");
        assert_eq!(get_msg(true, "en_US"), "Minimize to system tray enabled.");
        assert_eq!(get_msg(false, "en_US"), "Minimize to system tray disabled.");
        assert_eq!(get_msg(true, "ru_RU"), "Сворачивание в системный трей включено.");
        assert_eq!(get_msg(false, "ru_RU"), "Сворачивание в системный трей отключено.");
    }
}
