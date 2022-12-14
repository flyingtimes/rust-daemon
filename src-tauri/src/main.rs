// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use tauri::{
    api::process::Command, CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem,
};

fn main() {
    // here `"quit".to_string()` defines the menu item id, and the second parameter is the menu item label.
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let tray_menu = SystemTrayMenu::new()
        .add_item(quit)
        .add_native_item(SystemTrayMenuItem::Separator);
    let tray = SystemTray::new().with_menu(tray_menu);
    tauri::Builder::default()
        .setup(|_app| {

            tauri::async_runtime::spawn(async move {
                let (_, _) = Command::new_sidecar("naive")
                    .expect("failed to setup `app` sidecar")
                    .args([
                        "--proxy=https://user:pass@dark.21cnai.com",
                        "--listen=http://127.0.0.1:1087",
                        "--log",
                    ])
                    .spawn()
                    .expect("Failed to spawn packaged node");
            });
            Ok(())
        })
        .system_tray(tray)
        .on_system_tray_event(|_, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    std::process::exit(0);
                }
                _ => {}
            },
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
