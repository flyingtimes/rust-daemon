// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use tauri::{
    api::process::{Command, CommandChild, CommandEvent}, CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem, Manager
};
use tauri::async_runtime::Receiver;
use std::sync::Mutex;

#[derive(Debug, Serialize, Deserialize)]
struct Config {
    proxy: String,
    listen: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            proxy: "https://user:pass@example.com".to_string(),
            listen: "http://0.0.0.0:1087".to_string(),
        }
    }
}

// 读取配置文件
fn read_config() -> Config {
    let config_path = get_config_path();
    match fs::read_to_string(&config_path) {
        Ok(content) => {
            match serde_json::from_str(&content) {
                Ok(config) => config,
                Err(e) => {
                    println!("解析配置文件失败: {}, 使用默认配置", e);
                    Config::default()
                }
            }
        },
        Err(e) => {
            println!("读取配置文件失败: {}, 使用默认配置", e);
            // 如果配置文件不存在，创建默认配置文件
            let default_config = Config::default();
            let _ = write_config(&default_config);
            default_config
        }
    }
}

// 写入配置文件
fn write_config(config: &Config) -> Result<(), String> {
    let config_path = get_config_path();
    match serde_json::to_string_pretty(config) {
        Ok(content) => {
            match fs::write(&config_path, content) {
                Ok(_) => Ok(()),
                Err(e) => Err(format!("写入配置文件失败: {}", e))
            }
        },
        Err(e) => Err(format!("序列化配置失败: {}", e))
    }
}

// 获取配置文件路径
fn get_config_path() -> PathBuf {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("config");
    path.push("setting.json");
    path
}

// 定义一个结构体来存储naive进程的引用
struct NaiveProcess(Mutex<Option<CommandChild>>);

fn main() {
    // here `"quit".to_string()` defines the menu item id, and the second parameter is the menu item label.
    let config = CustomMenuItem::new("config".to_string(), "配置");
    let restart = CustomMenuItem::new("restart".to_string(), "重启");
    let quit = CustomMenuItem::new("quit".to_string(), "退出");
    let tray_menu = SystemTrayMenu::new()
        .add_item(config)
        .add_item(restart)
        .add_item(quit)
        .add_native_item(SystemTrayMenuItem::Separator);
    let tray = SystemTray::new().with_menu(tray_menu);
    
    tauri::Builder::default()
        // 注册naive进程状态
        .manage(NaiveProcess(Mutex::new(None)))
        .setup(|app| {
            // 读取配置文件
            let config = read_config();
            let url_link = format!("--proxy={}", config.proxy);
            let listen_link = format!("--listen={}", config.listen);
            
            println!("使用代理: {}", url_link);
            println!("监听地址: {}", listen_link);
            
            // 获取naive进程状态的引用
            let naive_process_state = app.state::<NaiveProcess>();
            
            // 启动naive进程并保存引用
            let (_, naive_child) = Command::new_sidecar("naive")
                .expect("failed to setup `app` sidecar")
                .args([
                    url_link,
                    listen_link,
                    "--log".to_string(),
                ])
                .spawn()
                .expect("Failed to spawn packaged node");
                
            // 保存进程引用到状态中
            *naive_process_state.0.lock().unwrap() = Some(naive_child);
            
            // 初始时不创建配置窗口，只在点击托盘菜单时创建
            
            Ok(())
        })
        .system_tray(tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "config" => {
                    // 检查配置窗口是否已存在
                    if let Some(config_window) = app.get_window("config") {
                        // 如果窗口已存在，显示它
                        config_window.show().unwrap();
                        config_window.set_focus().unwrap();
                    } else {
                        // 如果窗口不存在，创建一个新窗口
                        tauri::WindowBuilder::new(
                            app,
                            "config",
                            tauri::WindowUrl::App("config.html".into())
                        )
                        .title("配置")
                        .center()
                        .inner_size(400.0, 300.0)
                        .resizable(false)
                        .build()
                        .expect("无法创建配置窗口");
                    }
                },
                "restart" => {
                    // 重启应用
                    app.restart();
                },
                "quit" => {
                    // 获取naive进程引用并终止它
                    let naive_process_state = app.state::<NaiveProcess>();
                    if let Some(mut child) = naive_process_state.0.lock().unwrap().take() {
                        println!("正在终止naive进程...");
                        // 尝试终止进程
                        if let Err(e) = child.kill() {
                            println!("终止naive进程失败: {}", e);
                        }
                    }
                    // 退出应用程序
                    std::process::exit(0);
                }
                _ => {}
            },
            _ => {}
        })
        // 注册命令
        .invoke_handler(tauri::generate_handler![
            save_config,
            get_config_data
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

// Tauri命令：获取配置数据
#[tauri::command]
fn get_config_data() -> Result<Config, String> {
    Ok(read_config())
}

// Tauri命令：保存配置数据
#[tauri::command]
fn save_config(proxy: String, listen: String) -> Result<(), String> {
    let config = Config {
        proxy,
        listen,
    };
    write_config(&config)?;
    Ok(())
}
