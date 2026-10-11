use thiserror::Error;
use windows_api::WindowsApiError;

#[derive(Error, Debug)]
pub enum TweakError {
    #[error("Erro na API do Windows: {0}")]
    WindowsApi(#[from] WindowsApiError),
    #[error("Falha na validação do tweak: {0}")]
    ValidationFailed(String),
    #[error("Operação não suportada nesta versão do Windows")]
    UnsupportedOsVersion,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TweakStatus {
    Applied,
    Default,
    Unknown,
}

pub trait Tweak: Send + Sync {
    fn id(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn requires_admin(&self) -> bool {
        true
    }
    fn status(&self) -> Result<TweakStatus, TweakError>;
    fn apply(&self) -> Result<(), TweakError>;
    fn rollback(&self) -> Result<(), TweakError>;
}

pub mod priority;
pub use priority::Cs2PriorityTweak;

pub mod affinity;
pub use affinity::Cs2AffinityTweak;
