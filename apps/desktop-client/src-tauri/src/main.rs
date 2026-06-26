#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Mutex;
use tauri::{SystemTray, SystemTrayMenu, CustomMenuItem, SystemTrayEvent};

struct AppState {
    relay_enabled: Mutex<bool>,
    vpn_connected: Mutex<bool>,
}

#[tauri::command]
fn toggle_relay(state: tauri::State<AppState>) -> bool {
    let mut relay = state.relay_enabled.lock().unwrap();
    *relay = !*relay;
    *relay
}

fn main() {
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("pause_relay".to_string(), "Pause Proxy Relay (1 Hr)"))
        .add_item(CustomMenuItem::new("disconnect_vpn".to_string(), "Disconnect VPN"))
        .add_item(CustomMenuItem::new("quit".to_string(), "Exit App"));
        
    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .manage(AppState {
            relay_enabled: Mutex::new(true),
            vpn_connected: Mutex::new(false),
        })
        .system_tray(system_tray)
        .on_system_tray_event(|_app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => {
                match id.as_str() {
                    "quit" => std::process::exit(0),
                    _ => {}
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![toggle_relay])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
