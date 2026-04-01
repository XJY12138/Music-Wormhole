//! Audio Sync Share - Desktop Application
//! 
//! 跨平台桌面 UI，支持服务端和客户端模式切换

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use audio_sync_share::{
    AudioSyncEngine, AudioConfig, CaptureMode, DeviceRole, DeviceInfo, SyncStats,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, error, warn};

/// 应用状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub device_name: String,
    pub role: String,
    pub is_running: bool,
    pub connected_device: Option<String>,
    pub latency_ms: f64,
    pub available_devices: Vec<DeviceInfoDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfoDto {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub is_server: bool,
}

impl From<DeviceInfo> for DeviceInfoDto {
    fn from(info: DeviceInfo) -> Self {
        DeviceInfoDto {
            name: info.device_name,
            address: info.address,
            port: info.port,
            is_server: info.is_server,
        }
    }
}

/// Tauri 命令：初始化应用
#[tauri::command]
async fn initialize_app(device_name: String, state: tauri::State<'_, Arc<RwLock<Option<AudioSyncEngine>>>>) -> Result<(), String> {
    info!("Initializing app with device name: {}", device_name);
    
    let engine = AudioSyncEngine::new(device_name.clone())
        .await
        .map_err(|e| format!("Failed to create engine: {}", e))?;
    
    *state.write().await = Some(engine);
    
    Ok(())
}

/// Tauri 命令：获取应用状态
#[tauri::command]
async fn get_app_state(
    state: tauri::State<'_, Arc<RwLock<Option<AudioSyncEngine>>>>,
) -> Result<AppState, String> {
    let engine_guard = state.read().await;
    
    if let Some(engine) = engine_guard.as_ref() {
        let role = engine.get_role().await;
        let devices = engine.discover_devices().await;
        
        Ok(AppState {
            device_name: "Device".to_string(), // 从 engine 获取
            role: match role {
                DeviceRole::Server => "server".to_string(),
                DeviceRole::Client => "client".to_string(),
            },
            is_running: true,
            connected_device: None,
            latency_ms: 0.0,
            available_devices: devices.into_iter().map(DeviceInfoDto::from).collect(),
        })
    } else {
        Ok(AppState {
            device_name: "Not Initialized".to_string(),
            role: "server".to_string(),
            is_running: false,
            connected_device: None,
            latency_ms: 0.0,
            available_devices: vec![],
        })
    }
}

/// Tauri 命令：启动为服务端
#[tauri::command]
async fn start_server(
    state: tauri::State<'_, Arc<RwLock<Option<AudioSyncEngine>>>>,
) -> Result<(), String> {
    let engine_guard = state.read().await;
    
    if let Some(engine) = engine_guard.as_ref() {
        let config = AudioConfig {
            sample_rate: 48000,
            channels: 2,
            capture_mode: CaptureMode::SystemDefault,
            buffer_size: 256,
        };
        
        engine.start_as_server(config)
            .await
            .map_err(|e| format!("Failed to start server: {}", e))?;
        
        info!("Started as server");
        Ok(())
    } else {
        Err("Engine not initialized".to_string())
    }
}

/// Tauri 命令：启动为客户端
#[tauri::command]
async fn start_client(
    state: tauri::State<'_, Arc<RwLock<Option<AudioSyncEngine>>>>,
    server_address: String,
) -> Result<(), String> {
    let engine_guard = state.read().await;
    
    if let Some(engine) = engine_guard.as_ref() {
        let config = AudioConfig {
            sample_rate: 48000,
            channels: 2,
            capture_mode: CaptureMode::SystemDefault,
            buffer_size: 256,
        };
        
        engine.start_as_client(server_address, config)
            .await
            .map_err(|e| format!("Failed to start client: {}", e))?;
        
        info!("Started as client connecting to: {}", server_address);
        Ok(())
    } else {
        Err("Engine not initialized".to_string())
    }
}

/// Tauri 命令：切换角色
#[tauri::command]
async fn toggle_role(
    state: tauri::State<'_, Arc<RwLock<Option<AudioSyncEngine>>>>,
) -> Result<String, String> {
    let engine_guard = state.read().await;
    
    if let Some(engine) = engine_guard.as_ref() {
        engine.switch_role()
            .await
            .map_err(|e| format!("Failed to toggle role: {}", e))?;
        
        let new_role = engine.get_role().await;
        let role_str = match new_role {
            DeviceRole::Server => "server",
            DeviceRole::Client => "client",
        };
        
        info!("Toggled role to: {}", role_str);
        Ok(role_str.to_string())
    } else {
        Err("Engine not initialized".to_string())
    }
}

/// Tauri 命令：扫描设备
#[tauri::command]
async fn scan_devices(
    state: tauri::State<'_, Arc<RwLock<Option<AudioSyncEngine>>>>,
) -> Result<Vec<DeviceInfoDto>, String> {
    let engine_guard = state.read().await;
    
    if let Some(engine) = engine_guard.as_ref() {
        let devices = engine.discover_devices().await;
        Ok(devices.into_iter().map(DeviceInfoDto::from).collect())
    } else {
        Err("Engine not initialized".to_string())
    }
}

/// Tauri 命令：停止服务
#[tauri::command]
async fn stop_service(
    state: tauri::State<'_, Arc<RwLock<Option<AudioSyncEngine>>>>,
) -> Result<(), String> {
    let engine_guard = state.read().await;
    
    if let Some(engine) = engine_guard.as_ref() {
        engine.stop()
            .await
            .map_err(|e| format!("Failed to stop: {}", e))?;
        
        info!("Stopped service");
        Ok(())
    } else {
        Err("Engine not initialized".to_string())
    }
}

fn main() {
    // 初始化日志
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();
    
    info!("Starting Audio Sync Share Desktop App");
    
    tauri::Builder::default()
        .manage(Arc::new(RwLock::new(None::<AudioSyncEngine>)))
        .invoke_handler(tauri::generate_handler![
            initialize_app,
            get_app_state,
            start_server,
            start_client,
            toggle_role,
            scan_devices,
            stop_service,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
