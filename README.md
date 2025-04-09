# Rust Daemon 代理工具

一个基于Tauri框架开发的跨平台系统托盘代理工具，使用Rust后端和Web前端技术构建。

## 功能特点

- **系统托盘应用**：应用运行在系统托盘中，不占用任务栏空间
- **代理服务**：基于naive代理服务，支持HTTP/HTTPS代理
- **简洁配置界面**：提供简单易用的配置界面，可设置代理地址和监听地址
- **跨平台支持**：支持Windows、macOS和Linux系统
- **自动启动**：应用启动时自动运行代理服务
- **配置持久化**：自动保存和加载用户配置

## 技术栈

- **后端**：Rust + Tauri框架
- **前端**：HTML/CSS/JavaScript
- **代理服务**：naive代理工具
- **配置存储**：JSON文件

## 安装说明

### 从发布版安装
请注意这是一个基于tauri 1.0 框架的程序，需要确保你的当前tauri环境是1.0版本
```
cargo install tauri-cli --version "^1.0.0" --locked
```
1. 前往[Releases](https://github.com/yourusername/rust-daemon/releases)页面下载最新版本
2. 根据您的操作系统选择对应的安装包：
   - Windows: `.msi`或`.exe`
   - macOS: `.dmg`
   - Linux: `.AppImage`或`.deb`
3. 安装后，应用将自动启动并在系统托盘中显示图标

### 从源码构建

#### 前提条件

- [Rust](https://www.rust-lang.org/tools/install) (1.57+)
- [Node.js](https://nodejs.org/) (14+)
- [Tauri CLI](https://tauri.app/v1/guides/getting-started/prerequisites)

#### 构建步骤

```bash
# 克隆仓库
git clone https://github.com/yourusername/rust-daemon.git
cd rust-daemon

# 安装依赖并构建
cargo build --release

# 或使用Tauri CLI构建
cargo tauri build
```

## 使用指南

1. 启动应用后，在系统托盘中找到应用图标
2. 右键点击图标，选择「配置」打开配置窗口
3. 在配置窗口中设置：
   - **代理地址**：格式为`https://user:pass@example.com`
   - **监听地址**：格式为`http://0.0.0.0:1087`
4. 点击「保存」按钮保存配置
5. 重启应用使配置生效（右键点击托盘图标，选择「重启」）

## 配置文件

配置文件位于应用目录下的`config/setting.json`，格式如下：

```json
{
  "proxy": "https://user:pass@example.com",
  "listen": "http://0.0.0.0:1087"
}
```

## 开发说明

### 项目结构

- `src-tauri/`: Rust后端代码
  - `src/main.rs`: 主程序入口
  - `config/`: 配置文件目录
  - `bin/`: 包含naive代理工具的二进制文件
- `src/`: 前端代码
  - `index.html`: 主界面
  - `config.html`: 配置界面
  - `main.js`: JavaScript逻辑

### 开发模式启动

```bash
cargo tauri dev
```

## 许可证

[MIT License](LICENSE)

## 贡献指南

欢迎提交问题和拉取请求，共同改进这个项目。

1. Fork 项目
2. 创建您的特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交您的更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 打开一个 Pull Request