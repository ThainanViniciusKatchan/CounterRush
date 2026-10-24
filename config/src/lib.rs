use std::path::Path;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Erro de I/O ao acessar configurações: {0}")]
    Io(#[from] std::io::Error),
    #[error("Falha ao serializar/desserializar configurações: {0}")]
    Serialization(#[from] toml::de::Error),
    #[error("Falha ao processar arquivo JSON de configurações: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub auto_backup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct CustomOptimizationConfig {
    pub launch_args: Vec<String>,
    pub empty_standby_list: bool,
    pub cs2_priority: bool,
    pub cs2_affinity: bool,
    pub disable_gpu_acceleration: bool,
    pub apply_dpi_optimization: bool,
    pub disable_memory_compression: bool,
    pub apply_timer_resolution: bool,
    pub apply_mmcss: bool,
    pub disable_hpet: bool,
    pub disable_game_bar: bool,
    pub enable_msi: bool,
    pub disable_power_saving: bool,
    pub apply_render_queue: bool,
}

impl Default for CustomOptimizationConfig {
    fn default() -> Self {
        Self {
            launch_args: Vec::new(),
            empty_standby_list: true,
            cs2_priority: true,
            cs2_affinity: true,
            disable_gpu_acceleration: false,
            apply_dpi_optimization: true,
            disable_memory_compression: false,
            apply_timer_resolution: true,
            apply_mmcss: true,
            disable_hpet: false,
            disable_game_bar: true,
            enable_msi: false,
            disable_power_saving: true,
            apply_render_queue: true,
        }
    }
}

impl CustomOptimizationConfig {
    pub fn default_with_launch_args(args: Vec<String>) -> Self {
        Self {
            launch_args: args,
            ..Default::default()
        }
    }

    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config = serde_json::from_str(&content)?;
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigError> {
        let path = path.as_ref();
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(())
    }

    pub fn load_or_create<P: AsRef<Path>>(
        path: P,
        default_args: Vec<String>,
    ) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        if path.exists() {
            Self::load_from_file(path)
        } else {
            let default_config = Self::default_with_launch_args(default_args);
            default_config.save_to_file(path)?;
            Ok(default_config)
        }
    }
}
