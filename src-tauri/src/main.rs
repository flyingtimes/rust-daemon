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
            let resource_path = _app
                .path_resolver()
                .resolve_resource("./config/settings.json")
                .expect("failed to resolve resource");
            let afile = std::fs::File::open(&resource_path).unwrap();
            let message: serde_json::Value = serde_json::from_reader(afile).unwrap();
            let url_link  = format!("--proxy={}", message["url"].as_str().unwrap());
            println!("{}",url_link);

            let listen_link = format!("--listen=http://0.0.0.0:{}", message.get("port").unwrap() );
            println!("{}",listen_link);
            tauri::async_runtime::spawn(async {
                let (_, _) = Command::new_sidecar("naive")
                    .expect("failed to setup `app` sidecar")
                    .args([
                        url_link,
                        listen_link,
                        "--log".to_string(),
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
