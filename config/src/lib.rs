use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Erro de I/O ao acessar configurações: {0}")]
    Io(#[from] std::io::Error),
    #[error("Falha ao serializar/desserializar configurações: {0}")]
    Serialization(#[from] toml::de::Error),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    pub auto_backup: bool,
}
