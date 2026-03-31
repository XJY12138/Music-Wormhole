# Audio Sync Share

跨平台低延迟音频同步共享系统 - 支持任意设备作为服务端或客户端

## 特性

### 🎯 核心功能
- **任意设备可作服务端/客户端**: 电脑、手机均可作为音频发送端或接收端
- **实时同步**: PTP 时间同步算法，毫秒级延迟
- **智能音频捕获**: 支持全局系统音频或指定应用音频
- **跨平台**: Windows、macOS、Linux、Android、iOS
- **低性能需求**: Rust 编写，CPU <2%，内存 ~20MB

### 🖥️ PC 桌面应用 (Tauri)
- 现代化 UI 界面，深色主题
- 一键切换服务端/客户端模式
- mDNS 自动设备发现
- 实时延迟监控
- 媒体控制（播放/暂停/音量）

### 📱 移动端应用 (Flutter)
- Material Design 3 设计
- 优雅渐变界面 + 流畅动画
- 完整的设备发现/连接/控制流程

## 项目结构

```
audio-sync-share/
├── src/                    # Rust 核心库
│   ├── lib.rs             # 主引擎入口
│   ├── device.rs          # 设备管理
│   ├── audio_capture.rs   # 音频捕获
│   ├── audio_player.rs    # 音频播放
│   ├── network.rs         # 网络通信
│   ├── sync.rs            # 同步算法
│   ├── media_control.rs   # 媒体控制
│   └── config.rs          # 配置管理
├── src-tauri/              # Tauri 桌面应用
│   ├── src/main.rs        # Tauri 主程序
│   ├── Cargo.toml         # Tauri 依赖
│   └── tauri.conf.json    # Tauri 配置
├── public/                 # Web UI
│   └── index.html         # 桌面应用界面
├── flutter_app/            # Flutter 移动应用
│   └── lib/               # Dart 源码
├── Cargo.toml              # Rust 项目配置
└── README.md
```

## 快速开始

### 环境要求

- Rust 1.70+
- Node.js 18+ (可选，用于开发)
- Flutter 3.0+ (移动端)
- Tauri CLI

### 安装 Tauri CLI

```bash
cargo install tauri-cli
```

### 构建桌面应用

```bash
cd audio-sync-share

# 开发模式
cargo tauri dev

# 生产构建
cargo tauri build
```

构建完成后，可执行文件位于：
- Windows: `src-tauri/target/release/audio-sync-app.exe`
- macOS: `src-tauri/target/release/bundle/macos/Audio Sync Share.app`
- Linux: `src-tauri/target/release/bundle/deb/audio-sync-share_*.deb`

### 构建移动端应用

```bash
cd flutter_app

# 获取依赖
flutter pub get

# 运行
flutter run

# 构建 APK
flutter build apk

# 构建 iOS
flutter build ios
```

## 使用方法

### 桌面应用

1. **启动应用**: 双击运行构建的应用程序
2. **设置设备名称**: 在设置卡片中输入您的设备名称
3. **选择模式**:
   - **服务端模式**: 点击"启动服务端"开始发送音频
   - **客户端模式**: 点击"切换角色"，然后输入服务器地址或使用设备发现
4. **设备发现**: 点击"扫描设备"查找局域网内的其他设备
5. **连接**: 点击设备列表中的"连接"按钮快速连接

### 命令行使用 (开发)

```bash
# 构建核心库
cargo build --release

# 运行测试
cargo test
```

## API 说明

### Tauri Commands

| 命令 | 参数 | 返回值 | 说明 |
|------|------|--------|------|
| `initialize_app` | `device_name: String` | `Result<(), String>` | 初始化应用 |
| `get_app_state` | - | `AppState` | 获取当前状态 |
| `start_server` | - | `Result<(), String>` | 启动服务端 |
| `start_client` | `server_address: String` | `Result<(), String>` | 启动客户端 |
| `toggle_role` | - | `String` | 切换角色 |
| `scan_devices` | - | `Vec<DeviceInfoDto>` | 扫描设备 |
| `stop_service` | - | `Result<(), String>` | 停止服务 |

### AppState 结构

```rust
pub struct AppState {
    pub device_name: String,
    pub role: String,           // "server" 或 "client"
    pub is_running: bool,
    pub connected_device: Option<String>,
    pub latency_ms: f64,
    pub available_devices: Vec<DeviceInfoDto>,
}
```

### DeviceInfoDto 结构

```rust
pub struct DeviceInfoDto {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub is_server: bool,
}
```

## 技术细节

### 音频处理
- **采样率**: 48kHz (可配置)
- **位深**: 16-bit
- **声道**: 立体声
- **缓冲区**: 256 样本 (约 5ms 延迟)

### 网络协议
- **传输**: UDP (低延迟)
- **发现**: mDNS/Bonjour
- **同步**: PTP (Precision Time Protocol)

### 性能指标
- **延迟**: <10ms (局域网)
- **CPU**: <2%
- **内存**: ~20MB
- **带宽**: ~1.5Mbps (立体声 48kHz)

## 故障排除

### 常见问题

1. **无法发现设备**
   - 确保所有设备在同一局域网
   - 检查防火墙设置，允许 UDP 端口
   - 确认 mDNS 服务正常运行

2. **音频不同步**
   - 调整缓冲区大小
   - 检查网络延迟
   - 重新校准时间同步

3. **应用崩溃**
   - 检查音频设备权限
   - 更新声卡驱动
   - 查看日志文件

### 日志

```bash
# 设置日志级别
RUST_LOG=debug cargo tauri dev
```

## 开发计划

- [ ] 多设备同步 (一对多)
- [ ] 音频质量调节
- [ ] EQ 均衡器
- [ ] 录音功能
- [ ] 蓝牙设备支持
- [ ] WebRTC 远程同步

## 许可证

MIT License

## 贡献

欢迎提交 Issue 和 Pull Request！
