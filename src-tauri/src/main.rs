// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// 在Windows环境下的发布模式中，禁用控制台窗口
// 这个属性确保应用程序在Windows上运行时不会显示命令行窗口
#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

// 导入标准库中的文件系统操作模块
use std::fs;
// 导入路径处理相关模块
use std::path::PathBuf;
// 导入序列化和反序列化功能，用于处理JSON配置文件
use serde::{Deserialize, Serialize};
// 导入Tauri相关组件
use tauri::{
    // 进程管理相关API，用于启动和控制sidecar进程
    api::process::{Command, CommandChild}, 
    // 系统托盘相关组件
    CustomMenuItem, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem, 
    // 窗口管理器
    Manager
};
// 导入互斥锁，用于安全地共享和修改进程引用
use std::sync::Mutex;

// 定义配置结构体，用于存储代理设置
// Debug特性允许使用{:?}格式化输出
// Serialize和Deserialize特性支持JSON序列化和反序列化
#[derive(Debug, Serialize, Deserialize)]
struct Config {
    // 代理服务器地址
    proxy: String,
    // 本地监听地址
    listen: String,
}

// 为Config实现Default特性，提供默认配置值
impl Default for Config {
    fn default() -> Self {
        Self {
            // 默认代理服务器地址（示例值）
            proxy: "https://user:pass@example.com".to_string(),
            // 默认本地监听地址和端口
            listen: "http://0.0.0.0:1087".to_string(),
        }
    }
}

// 读取配置文件函数，返回Config结构体实例
fn read_config() -> Config {
    // 获取配置文件路径
    let config_path = get_config_path();
    // 尝试读取配置文件内容
    match fs::read_to_string(&config_path) {
        // 如果成功读取文件内容
        Ok(content) => {
            // 尝试将JSON字符串解析为Config结构体
            match serde_json::from_str(&content) {
                // 解析成功，返回配置对象
                Ok(config) => config,
                // 解析失败，记录错误并返回默认配置
                Err(e) => {
                    println!("解析配置文件失败: {}, 使用默认配置", e);
                    Config::default()
                }
            }
        },
        // 如果读取文件失败（可能是文件不存在）
        Err(e) => {
            println!("读取配置文件失败: {}, 使用默认配置", e);
            // 创建默认配置对象
            let default_config = Config::default();
            // 尝试将默认配置写入文件，忽略可能的错误
            let _ = write_config(&default_config);
            // 返回默认配置
            default_config
        }
    }
}

// 写入配置文件函数，接收Config引用，返回成功或错误信息
fn write_config(config: &Config) -> Result<(), String> {
    // 获取配置文件路径
    let config_path = get_config_path();
    // 尝试将Config对象序列化为格式化的JSON字符串
    match serde_json::to_string_pretty(config) {
        // 序列化成功
        Ok(content) => {
            // 尝试将JSON内容写入文件
            match fs::write(&config_path, content) {
                // 写入成功
                Ok(_) => Ok(()),
                // 写入失败，返回错误信息
                Err(e) => Err(format!("写入配置文件失败: {}", e))
            }
        },
        // 序列化失败，返回错误信息
        Err(e) => Err(format!("序列化配置失败: {}", e))
    }
}

// 获取配置文件的完整路径
fn get_config_path() -> PathBuf {
    // 从环境变量获取项目根目录路径（编译时确定）
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // 添加config子目录
    path.push("config");
    // 添加配置文件名
    path.push("setting.json");
    // 返回完整路径
    path
}

// 定义一个结构体来存储naive代理进程的引用
// 使用元组结构体包装Mutex，Mutex内部是Option类型，可以为None或Some(CommandChild)
// 这样设计允许安全地在多个地方访问和修改进程引用
struct NaiveProcess(Mutex<Option<CommandChild>>);

// 程序入口函数
fn main() {
    // 创建系统托盘菜单项
    // 第一个参数是菜单项ID（用于识别点击的菜单项），第二个参数是显示的文本
    // 创建配置菜单项
    let config = CustomMenuItem::new("config".to_string(), "配置");
    // 创建浏览菜单项
    let browse = CustomMenuItem::new("browse".to_string(), "浏览");
    // 创建重启菜单项
    let restart = CustomMenuItem::new("restart".to_string(), "重启");
    // 创建退出菜单项
    let quit = CustomMenuItem::new("quit".to_string(), "退出");
    // 创建系统托盘菜单并添加上述菜单项
    let tray_menu = SystemTrayMenu::new()
        .add_item(config)    // 添加配置菜单项
        .add_item(browse)    // 添加浏览菜单项
        .add_item(restart)   // 添加重启菜单项
        .add_item(quit)      // 添加退出菜单项
        .add_native_item(SystemTrayMenuItem::Separator); // 添加分隔线
    // 创建系统托盘并设置菜单
    let tray = SystemTray::new().with_menu(tray_menu);
    
    // 创建Tauri应用构建器
    tauri::Builder::default()
        // 注册naive进程状态，初始化为None（尚未启动进程）
        // 这使得进程状态可以在应用的不同部分共享和访问
        .manage(NaiveProcess(Mutex::new(None)))
        // 设置应用初始化函数，在应用启动时执行
        .setup(|app| {
            // 读取配置文件，获取代理设置
            let config = read_config();
            // 格式化代理参数
            let url_link = format!("--proxy={}", config.proxy);
            // 格式化监听地址参数
            let listen_link = format!("--listen={}", config.listen);
            
            // 打印当前使用的代理设置，便于调试
            println!("使用代理: {}", url_link);
            println!("监听地址: {}", listen_link);
            
            // 获取naive进程状态的引用，用于后续保存进程句柄
            let naive_process_state = app.state::<NaiveProcess>();
            
            // 启动naive代理进程作为sidecar（随应用一起打包的辅助程序）
            // Command::new_sidecar创建一个指向打包在应用内的可执行文件的命令
            let (_, naive_child) = Command::new_sidecar("naive")
                .expect("failed to setup `app` sidecar") // 如果sidecar设置失败，程序会终止
                .args([
                    url_link,      // 传递代理参数
                    listen_link,   // 传递监听地址参数
                    "--log".to_string(), // 启用日志
                ])
                .spawn() // 启动进程
                .expect("Failed to spawn packaged node"); // 如果启动失败，程序会终止
                
            // 获取互斥锁，并将进程引用保存到状态中
            // 这样可以在应用的其他部分（如退出时）访问和控制这个进程
            *naive_process_state.0.lock().unwrap() = Some(naive_child);
            
            // 在macOS环境下设置系统代理
            if cfg!(target_os = "macos") {
                println!("检测到macOS系统，正在设置系统代理...");
                // 从配置中提取端口号
                let port = if let Some(port_str) = config.listen.split(':').last() {
                    port_str
                } else {
                    "1087" // 默认端口
                };
                
                // 执行macOS系统命令设置Wi-Fi代理
                match Command::new("networksetup")
                    .args(["-setwebproxy", "Wi-Fi", "127.0.0.1", port])
                    .spawn() {
                    Ok(_) => println!("成功设置系统代理为 127.0.0.1:{}", port),
                    Err(e) => println!("设置系统代理失败: {}", e),
                }
            }
            
            // 初始时不创建配置窗口，只在点击托盘菜单时创建
            // 这样可以减少资源占用，提高启动速度
            
            Ok(()) // 返回成功
        })
        // 设置系统托盘
        .system_tray(tray)
        // 注册系统托盘事件处理函数
        .on_system_tray_event(|app, event| match event {
            // 处理菜单项点击事件
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                // 处理"配置"菜单项点击
                "config" => {
                    // 检查配置窗口是否已存在
                    if let Some(config_window) = app.get_window("config") {
                        // 如果窗口已存在，显示它并设置焦点
                        config_window.show().unwrap(); // 显示窗口
                        config_window.set_focus().unwrap(); // 设置焦点
                    } else {
                        // 如果窗口不存在，创建一个新窗口
                        tauri::WindowBuilder::new(
                            app, // 应用实例
                            "config", // 窗口标识符
                            tauri::WindowUrl::App("config.html".into()) // 窗口内容URL
                        )
                        .title("配置") // 设置窗口标题
                        .center() // 窗口居中显示
                        .inner_size(400.0, 300.0) // 设置窗口大小
                        .resizable(false) // 禁止调整窗口大小
                        .visible(true)
                        .build() // 构建窗口
                        .expect("无法创建配置窗口"); // 如果创建失败，程序会终止
                    }
                },
                // 处理"重启"菜单项点击
                "restart" => {
                    // 重启整个应用程序
                    // 注意：这会终止当前进程并启动一个新的应用实例
                    app.restart();
                },
                // 处理"浏览"菜单项点击
                "browse" => {
                    // 创建一个指向YouTube的窗口
                    tauri::WindowBuilder::new(
                        app, // 应用实例
                        "youtube", // 窗口标识符
                        tauri::WindowUrl::External("https://xvideos.com".parse().unwrap()) // 窗口内容URL
                    )
                    .title("YouTube") // 设置窗口标题
                    .center() // 窗口居中显示
                    .inner_size(800.0, 400.0) // 设置窗口大小为800x400
                    .resizable(true) // 允许调整窗口大小
                    .visible(true) // 确保窗口可见
                    .build() // 构建窗口
                    .expect("无法创建YouTube窗口"); // 如果创建失败，程序会终止
                },
                // 处理"退出"菜单项点击
                "quit" => {
                    // 获取naive进程状态的引用
                    let naive_process_state = app.state::<NaiveProcess>();
                    // 尝试从状态中取出进程引用
                    // take()方法会将Option中的值取出，并将原Option设为None
                    if let Some(child) = naive_process_state.0.lock().unwrap().take() {
                        println!("正在终止naive进程...");
                        // 尝试终止进程
                        if let Err(e) = child.kill() {
                            // 如果终止失败，记录错误信息
                            println!("终止naive进程失败: {}", e);
                        }
                        // 即使终止失败，也继续执行退出流程
                    }
                    
                    // 在macOS环境下恢复系统代理设置
                    if cfg!(target_os = "macos") {
                        println!("正在恢复系统代理设置...");
                        // 执行macOS系统命令关闭Wi-Fi代理
                        match Command::new("networksetup")
                            .args(["-setwebproxystate", "Wi-Fi", "off"])
                            .spawn() {
                            Ok(_) => println!("成功恢复系统代理设置"),
                            Err(e) => println!("恢复系统代理设置失败: {}", e),
                        }
                    }
                    
                    // 退出整个应用程序
                    // 注意：这会立即终止所有线程，不会执行任何清理代码
                    std::process::exit(0);
                }
                // 忽略其他菜单项点击
                _ => {}
            },
            // 忽略其他系统托盘事件
            _ => {}
        })
        // 注册前端可调用的Tauri命令
        // 这些命令可以从JavaScript通过invoke函数调用
        .invoke_handler(tauri::generate_handler![
            save_config,     // 注册保存配置命令
            get_config_data  // 注册获取配置数据命令
        ])
        // 运行应用程序
        // generate_context!宏会生成应用程序上下文，包含tauri.conf.json中的配置
        .run(tauri::generate_context!())
        // 如果运行失败，程序会终止
        .expect("error while running tauri application");
}

// Tauri命令：获取配置数据
// #[tauri::command]宏标记这个函数可以从前端JavaScript调用
#[tauri::command]
fn get_config_data() -> Result<Config, String> {
    // 调用read_config函数读取配置，并将结果包装在Ok中返回
    // 这个函数会自动将Config序列化为JSON发送给前端
    Ok(read_config())
}

// Tauri命令：保存配置数据
// #[tauri::command]宏标记这个函数可以从前端JavaScript调用
#[tauri::command]
fn save_config(proxy: String, listen: String) -> Result<(), String> {
    // 使用传入的参数创建新的Config对象
    let config = Config {
        proxy,   // 代理服务器地址
        listen,  // 本地监听地址
    };
    // 调用write_config函数保存配置
    // ?操作符会在出错时提前返回错误
    write_config(&config)?;
    // 返回成功
    Ok(())
}
