#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod layout;
mod settings;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use enigo::{Direction as EnigoDir, Enigo, Key, Keyboard, Settings as EnigoSettings};
use tauri::{
    image::Image,
    menu::{Menu, MenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
    AppHandle, Manager,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt as AutostartExt};
use tauri_plugin_global_shortcut::{Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutState};
use tauri_plugin_notification::NotificationExt;

use crate::settings::{AppSettings, SettingsStore};
use crate::layout::{detect_direction, en_to_ua, ua_to_en, Direction};

/// Основна процедура конвертації: керована хоткеєм або з UI вручну.
fn run_conversion(app: &AppHandle, settings: &AppSettings) {
    // 1) Зберегти поточний clipboard
    let mut clipboard = match arboard::Clipboard::new() {
        Ok(c) => c,
        Err(_) => return,
    };
    let saved = clipboard.get_text().ok();

    // 2) Емулювати Ctrl+C на активному вікні
    if let Ok(mut enigo) = Enigo::new(&EnigoSettings::default()) {
        let _ = enigo.key(Key::Control, EnigoDir::Press);
        let _ = enigo.key(Key::Unicode('c'), EnigoDir::Click);
        let _ = enigo.key(Key::Control, EnigoDir::Release);
    }
    thread::sleep(Duration::from_millis(80));

    // 3) Прочитати виділений текст
    let selected = match clipboard.get_text() {
        Ok(t) if !t.is_empty() => t,
        _ => {
            notify(app, settings, "Нічого не виділено");
            if let Some(s) = saved {
                let _ = clipboard.set_text(s);
            }
            return;
        }
    };

    // 4) Конвертувати
    let direction = match settings.direction.as_str() {
        "en_to_ua" => Direction::EnToUa,
        "ua_to_en" => Direction::UaToEn,
        _ => detect_direction(&selected),
    };
    let converted = match direction {
        Direction::EnToUa => en_to_ua(&selected),
        Direction::UaToEn => ua_to_en(&selected),
    };

    if converted == selected {
        notify(app, settings, "Зміни не потрібні");
        if let Some(s) = saved {
            let _ = clipboard.set_text(s);
        }
        return;
    }

    // 5) Покласти в clipboard і вставити
    if clipboard.set_text(converted.clone()).is_err() {
        return;
    }
    thread::sleep(Duration::from_millis(30));

    if let Ok(mut enigo) = Enigo::new(&EnigoSettings::default()) {
        let _ = enigo.key(Key::Control, EnigoDir::Press);
        let _ = enigo.key(Key::Unicode('v'), EnigoDir::Click);
        let _ = enigo.key(Key::Control, EnigoDir::Release);
    }

    // 6) Відновити оригінальний clipboard (з затримкою щоб Ctrl+V встиг зчитати)
    let app_clone = app.clone();
    let saved_clone = saved.clone();
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(250));
        if let (Ok(mut c), Some(s)) = (arboard::Clipboard::new(), saved_clone) {
            let _ = c.set_text(s);
        }
        let _ = app_clone;
    });

    notify(app, settings, &format!("Конвертовано: {}", truncate(&converted, 40)));
}

fn truncate(s: &str, max: usize) -> String {
    if s.chars().count() <= max { s.to_string() }
    else { s.chars().take(max).collect::<String>() + "…" }
}

fn notify(app: &AppHandle, settings: &AppSettings, msg: &str) {
    if !settings.notifications { return; }
    let _ = app.notification()
        .builder()
        .title("Raskladka Fix")
        .body(msg)
        .show();
}

#[tauri::command]
fn get_settings(store: tauri::State<'_, SettingsStore>) -> AppSettings {
    store.get()
}

#[tauri::command]
fn save_settings(
    app: AppHandle,
    store: tauri::State<'_, SettingsStore>,
    settings: AppSettings,
) -> Result<(), String> {
    let prev = store.get();
    store.set(settings.clone()).map_err(|e| e.to_string())?;

    // Перереєструвати хоткей, якщо змінився
    if prev.hotkey != settings.hotkey {
        let _ = app.global_shortcut().unregister_all();
        register_hotkey(&app, &settings.hotkey);
    }

    // Оновити автозапуск
    let autostart = app.autolaunch();
    let _ = if settings.autostart {
        autostart.enable()
    } else {
        autostart.disable()
    };

    Ok(())
}

fn parse_shortcut(s: &str) -> Option<Shortcut> {
    let mut mods = Modifiers::empty();
    let mut code: Option<Code> = None;
    for raw in s.split('+') {
        let part = raw.trim();
        match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" | "commandorcontrol" | "cmdorctrl" => mods |= Modifiers::CONTROL,
            "shift" => mods |= Modifiers::SHIFT,
            "alt" | "option" => mods |= Modifiers::ALT,
            "super" | "meta" | "win" | "cmd" => mods |= Modifiers::SUPER,
            other => {
                code = match other {
                    "a" => Some(Code::KeyA), "b" => Some(Code::KeyB), "c" => Some(Code::KeyC),
                    "d" => Some(Code::KeyD), "e" => Some(Code::KeyE), "f" => Some(Code::KeyF),
                    "g" => Some(Code::KeyG), "h" => Some(Code::KeyH), "i" => Some(Code::KeyI),
                    "j" => Some(Code::KeyJ), "k" => Some(Code::KeyK), "l" => Some(Code::KeyL),
                    "m" => Some(Code::KeyM), "n" => Some(Code::KeyN), "o" => Some(Code::KeyO),
                    "p" => Some(Code::KeyP), "q" => Some(Code::KeyQ), "r" => Some(Code::KeyR),
                    "s" => Some(Code::KeyS), "t" => Some(Code::KeyT), "u" => Some(Code::KeyU),
                    "v" => Some(Code::KeyV), "w" => Some(Code::KeyW), "x" => Some(Code::KeyX),
                    "y" => Some(Code::KeyY), "z" => Some(Code::KeyZ),
                    "space" => Some(Code::Space),
                    "f1" => Some(Code::F1), "f2" => Some(Code::F2), "f3" => Some(Code::F3),
                    "f4" => Some(Code::F4), "f5" => Some(Code::F5), "f6" => Some(Code::F6),
                    _ => return None,
                };
            }
        }
    }
    code.map(|c| Shortcut::new(Some(mods), c))
}

fn register_hotkey(app: &AppHandle, hotkey_str: &str) {
    let Some(shortcut) = parse_shortcut(hotkey_str) else {
        eprintln!("Не вдалось розпарсити хоткей: {hotkey_str}");
        return;
    };
    let app_handle = app.clone();
    if let Err(e) = app.global_shortcut().on_shortcut(shortcut, move |_app, _sc, ev| {
        if ev.state() == ShortcutState::Pressed {
            let app = app_handle.clone();
            // Невелика затримка, щоб користувач встиг відпустити модифікатори
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(50));
                let settings = app.state::<SettingsStore>().get();
                run_conversion(&app, &settings);
            });
        }
    }) {
        eprintln!("Не вдалось зареєструвати хоткей: {e}");
    }
}

fn build_tray(app: &AppHandle) -> tauri::Result<()> {
    let convert = MenuItem::with_id(app, "convert", "Конвертувати виділене", true, None::<&str>)?;
    let settings_item = MenuItem::with_id(app, "settings", "Налаштування…", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Вихід", true, None::<&str>)?;
    let menu = Menu::with_items(app, &[&convert, &settings_item, &quit])?;

    let _tray = TrayIconBuilder::with_id("main")
        .tooltip("Raskladka Fix")
        .icon(app.default_window_icon().cloned().unwrap_or_else(|| Image::new_owned(vec![], 0, 0)))
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "convert" => {
                let app = app.clone();
                thread::spawn(move || {
                    let s = app.state::<SettingsStore>().get();
                    run_conversion(&app, &s);
                });
            }
            "settings" => {
                if let Some(w) = app.get_webview_window("settings") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::DoubleClick { .. } = event {
                if let Some(w) = tray.app_handle().get_webview_window("settings") {
                    let _ = w.show();
                    let _ = w.set_focus();
                }
            }
        })
        .build(app)?;
    Ok(())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .invoke_handler(tauri::generate_handler![get_settings, save_settings])
        .setup(|app| {
            let handle = app.handle().clone();

            // Завантажити налаштування
            let store = SettingsStore::load(&handle);
            let initial = store.get();
            app.manage(store);

            // Tray
            build_tray(&handle)?;

            // Хоткей
            register_hotkey(&handle, &initial.hotkey);

            // Сховати вікно налаштувань при старті — програма живе в треї
            if let Some(w) = app.get_webview_window("settings") {
                let _ = w.hide();
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            // Не закривати застосунок, коли користувач закриває вікно налаштувань
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "settings" {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_app, event| {
            if let tauri::RunEvent::ExitRequested { api, .. } = event {
                // Залишаємо програму в треї
                let _ = api;
            }
        });
}

fn main() {
    run();
}

// Заглушка для зв'язку з Arc<Mutex<...>>, якщо знадобиться розширювати state.
#[allow(dead_code)]
type SharedState<T> = Arc<Mutex<T>>;