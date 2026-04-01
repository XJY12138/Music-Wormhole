//! 设备管理模块
//! 支持任意设备作为服务端或客户端

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

/// 设备角色枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceRole {
    /// 服务端：捕获并发送音频
    Server,
    /// 客户端：接收并播放音频
    Client,
}

impl Default for DeviceRole {
    fn default() -> Self {
        DeviceRole::Server
    }
}

/// 设备管理器
pub struct DeviceManager {
    device_name: String,
    role: AtomicUsize,
    device_id: u64,
    created_at: u64,
}

impl DeviceManager {
    /// 创建新的设备管理器
    pub fn new(device_name: String) -> Self {
        let device_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        Self {
            device_name,
            role: AtomicUsize::new(DeviceRole::Server as usize),
            device_id,
            created_at: device_id,
        }
    }
    
    /// 获取设备名称
    pub fn name(&self) -> &str {
        &self.device_name
    }
    
    /// 获取设备 ID
    pub fn id(&self) -> u64 {
        self.device_id
    }
    
    /// 获取当前角色
    pub fn get_role(&self) -> DeviceRole {
        match self.role.load(Ordering::SeqCst) {
            0 => DeviceRole::Server,
            _ => DeviceRole::Client,
        }
    }
    
    /// 设置角色
    pub fn set_role(&mut self, role: DeviceRole) {
        self.role.store(role as usize, Ordering::SeqCst);
    }
    
    /// 切换角色
    pub fn toggle_role(&mut self) {
        let current = self.get_role();
        let new_role = match current {
            DeviceRole::Server => DeviceRole::Client,
            DeviceRole::Client => DeviceRole::Server,
        };
        self.set_role(new_role);
    }
    
    /// 获取创建时间戳
    pub fn created_at(&self) -> u64 {
        self.created_at
    }
    
    /// 生成设备标识字符串
    pub fn identifier(&self) -> String {
        format!("{}-{}", self.device_name, self.device_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_device_manager() {
        let mut dm = DeviceManager::new("Test-Device".to_string());
        
        assert_eq!(dm.name(), "Test-Device");
        assert_eq!(dm.get_role(), DeviceRole::Server);
        
        dm.toggle_role();
        assert_eq!(dm.get_role(), DeviceRole::Client);
        
        dm.set_role(DeviceRole::Server);
        assert_eq!(dm.get_role(), DeviceRole::Server);
    }
}
