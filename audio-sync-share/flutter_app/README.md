# Audio Sync Share - Flutter 移动端应用

## 📱 功能特性

### 现代化 UI 设计
- **Material Design 3** - 遵循最新 Material Design 规范
- **深色主题** - 优雅的深色渐变界面，减少眼睛疲劳
- **流畅动画** - 使用 Flutter Animate 和 Lottie 实现丝滑过渡效果
- **响应式布局** - 适配各种屏幕尺寸

### 核心功能
- **设备发现** - 自动扫描局域网内的可用设备
- **一键连接** - 简单直观的连接流程
- **实时同步** - 显示当前延迟和音频质量
- **播放控制** - 播放/暂停/上一首/下一首
- **音量调节** - 精确的音量控制
- **同步偏移** - 微调音频同步时间
- **音源选择** - 选择特定应用或全部系统音频

### 跨平台支持
- Android (原生支持)
- iOS (待实现)
- 与 Rust 后端无缝集成

## 🏗️ 项目结构

```
flutter_app/
├── lib/
│   ├── main.dart                    # 应用入口
│   ├── models/
│   │   └── device_info.dart         # 设备信息模型
│   ├── screens/
│   │   ├── home_screen.dart         # 主页面（设备列表）
│   │   ├── discovery_screen.dart    # 设备发现页面
│   │   └── connection_screen.dart   # 连接详情页面
│   ├── services/
│   │   ├── audio_service.dart       # 音频捕获/播放服务
│   │   ├── device_discovery_service.dart  # 设备发现服务
│   │   └── sync_service.dart        # 同步控制服务
│   └── widgets/
│       ├── device_card.dart         # 设备卡片组件
│       └── connection_status.dart   # 连接状态栏组件
├── android/
│   └── app/src/main/kotlin/.../MainActivity.kt  # Android 原生代码
├── assets/                          # 资源文件
└── pubspec.yaml                     # 依赖配置
```

## 🚀 快速开始

### 环境要求
- Flutter SDK >= 3.0.0
- Dart SDK >= 3.0.0
- Android Studio / VS Code
- Android 设备或模拟器 (API 21+)

### 安装步骤

1. **克隆项目**
```bash
cd audio-sync-share/flutter_app
```

2. **安装依赖**
```bash
flutter pub get
```

3. **运行应用**
```bash
# 连接到设备后运行
flutter run

# 或者指定设备
flutter devices
flutter run -d <device_id>
```

4. **构建 Release APK**
```bash
flutter build apk --release

# 输出位置：build/app/outputs/flutter-apk/app-release.apk
```

5. **构建 AAB (Google Play)**
```bash
flutter build appbundle --release
```

## 🔧 配置说明

### Android 权限配置

在 `android/app/src/main/AndroidManifest.xml` 中添加：

```xml
<uses-permission android:name="android.permission.INTERNET"/>
<uses-permission android:name="android.permission.ACCESS_NETWORK_STATE"/>
<uses-permission android:name="android.permission.ACCESS_WIFI_STATE"/>
<uses-permission android:name="android.permission.RECORD_AUDIO"/>
<uses-permission android:name="android.permission.MODIFY_AUDIO_SETTINGS"/>
```

### 网络配置

确保设备和手机在同一局域网内。应用使用 mDNS 进行设备发现。

## 🎨 UI 预览

### 主页面
- 渐变背景
- 设备卡片列表
- 实时状态指示器
- 下拉刷新

### 连接页面
- 设备详细信息
- 动画连接按钮
- 音频设置选项
- 播放控制面板

### 连接状态栏
- 实时延迟显示
- 播放/暂停控制
- 音量/同步/音源快捷设置

## 🔌 与 Rust 后端集成

Flutter 应用通过以下方式与 Rust 后端通信：

1. **mDNS 发现** - 自动发现局域网内的 Rust 发送/接收端
2. **TCP/UDP 通信** - 传输音频数据和控制命令
3. **Platform Channels** - 调用原生音频 API

### Platform Channel 示例

```dart
// Flutter 端
const platform = MethodChannel('com.example.audio_sync_share/audio');

await platform.invokeMethod('startAudioCapture', {
  'allApps': true,
});
```

```kotlin
// Android 端
MethodChannel(flutterEngine.dartExecutor.binaryMessenger, CHANNEL)
  .setMethodCallHandler { call, result ->
    when (call.method) {
      "startAudioCapture" -> { /* 实现 */ }
    }
  }
```

## 📦 依赖说明

### UI 组件
- `google_fonts` - 美观的字体
- `flutter_animate` - 流畅动画
- `lottie` - After Effects 动画支持

### 状态管理
- `provider` - 简单的状态管理
- `riverpod` - 更强大的依赖注入

### 网络通信
- `http` - HTTP 请求
- `web_socket_channel` - WebSocket 通信
- `mdns_throttle` - mDNS 服务发现

### 系统功能
- `permission_handler` - 权限管理
- `device_info_plus` - 设备信息
- `connectivity_plus` - 网络状态
- `just_audio` - 音频播放

## 🐛 常见问题

### Q: 无法发现设备？
A: 确保：
- 所有设备在同一 WiFi 网络
- 防火墙允许 mDNS 流量（端口 5353）
- Rust 后端正在运行

### Q: 音频不同步？
A: 在连接状态栏中调整"Sync"偏移值，直到音画同步。

### Q: 构建失败？
A: 运行 `flutter clean && flutter pub get` 清理缓存。

## 📄 许可证

MIT License

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

---

**开发团队**: Audio Sync Share Team  
**版本**: 1.0.0  
**最后更新**: 2024
