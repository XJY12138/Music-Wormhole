//! Audio Sync Share - Core Library
//! 
//! 跨平台低延迟音频流同步共享系统核心库
//! 支持任意设备作为服务端或客户端

pub mod config;
pub mod audio_capture;
pub mod audio_player;
pub mod network;
pub mod media_control;
pub mod sync;
pub mod error;
pub mod device;

pub use config::{Config, AudioConfig, CaptureMode};
pub use audio_capture::AudioCapture;
pub use audio_player::AudioPlayer;
pub use network::{NetworkManager, DeviceInfo, MessageType, NetworkMessage};
pub use sync::{SyncEngine, SyncStats};
pub use media_control::{MediaController, PlaybackState};
pub use device::{DeviceRole, DeviceManager};
pub use error::{Error, Result};

use anyhow::Result as AnyResult;
use log::{info, warn};
use std::sync::Arc;
use tokio::sync::RwLock;

/// 主引擎结构，统一管理音频捕获、播放和网络通信
pub struct AudioSyncEngine {
    device_manager: Arc<RwLock<DeviceManager>>,
    network: Arc<RwLock<NetworkManager>>,
    sync_engine: Arc<RwLock<SyncEngine>>,
    media_controller: MediaController,
}

impl AudioSyncEngine {
    /// 创建新的音频同步引擎实例
    pub async fn new(device_name: String) -> AnyResult<Self> {
        info!("Initializing AudioSyncEngine with device name: {}", device_name);
        
        let device_manager = Arc::new(RwLock::new(DeviceManager::new(device_name)));
        let network = Arc::new(RwLock::new(NetworkManager::new().await?));
        let sync_engine = Arc::new(RwLock::new(SyncEngine::new()));
        let media_controller = MediaController::new();
        
        Ok(Self {
            device_manager,
            network,
            sync_engine,
            media_controller,
        })
    }
    
    /// 启动为服务端（发送音频）
    pub async fn start_as_server(&self, config: AudioConfig) -> AnyResult<()> {
        info!("Starting as server (audio sender)");
        
        let mut device_mgr = self.device_manager.write().await;
        device_mgr.set_role(DeviceRole::Server);
        
        // 启动网络服务
        let network = self.network.read().await;
        network.start_server().await?;
        
        info!("Server started successfully");
        Ok(())
    }
    
    /// 启动为客户端（接收并播放音频）
    pub async fn start_as_client(&self, server_addr: String, config: AudioConfig) -> AnyResult<()> {
        info!("Starting as client, connecting to: {}", server_addr);
        
        let mut device_mgr = self.device_manager.write().await;
        device_mgr.set_role(DeviceRole::Client);
        
        // 连接到服务端
        let network = self.network.read().await;
        network.connect_to_server(&server_addr).await?;
        
        info!("Client started successfully");
        Ok(())
    }
    
    /// 切换角色（服务端 <-> 客户端）
    pub async fn switch_role(&self) -> AnyResult<()> {
        let mut device_mgr = self.device_manager.write().await;
        device_mgr.toggle_role();
        
        let new_role = *device_mgr.get_role();
        info!("Switched role to: {:?}", new_role);
        
        Ok(())
    }
    
    /// 获取当前设备角色
    pub async fn get_role(&self) -> DeviceRole {
        let device_mgr = self.device_manager.read().await;
        *device_mgr.get_role()
    }
    
    /// 发现网络设备
    pub async fn discover_devices(&self) -> Vec<DeviceInfo> {
        let network = self.network.read().await;
        network.discover_devices().await
    }
    
    /// 获取同步统计信息
    pub async fn get_sync_stats(&self) -> SyncStats {
        let sync = self.sync_engine.read().await;
        sync.get_stats()
    }
    
    /// 停止所有服务
    pub async fn stop(&self) -> AnyResult<()> {
        info!("Stopping all services");
        
        let network = self.network.read().await;
        network.stop().await?;
        
        info!("All services stopped");
        Ok(())
    }
    
    /// 获取媒体控制器
    pub fn media_controller(&self) -> &MediaController {
        &self.media_controller
    }
}

/// 初始化日志
pub fn init_logging() {
    env_logger::Builder::from_env(
        env_logger::Env::default().default_filter_or("info")
    ).init();
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_engine_creation() {
        let engine = AudioSyncEngine::new("Test-Device".to_string()).await;
        assert!(engine.is_ok());
    }
}
