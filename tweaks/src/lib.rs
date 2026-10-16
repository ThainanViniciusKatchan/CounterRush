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

pub mod gpu_acceleration;
pub use gpu_acceleration::{
    disable_all_gpu_accel, disable_chromium_gpu_accel, disable_discord_gpu_accel,
    disable_firefox_gpu_accel, disable_steam_gpu_accel, enable_all_gpu_accel,
    enable_chromium_gpu_accel, enable_discord_gpu_accel, enable_firefox_gpu_accel,
    enable_steam_gpu_accel, ChromiumGpuAccelTweak, DiscordGpuAccelTweak, FirefoxGpuAccelTweak,
    SteamGpuAccelTweak,
};

pub mod memory;
pub use memory::{empty_standby_list, EmptyStandbyListTweak};

pub mod dpi;
pub use dpi::{
    apply_cs2_dpi_tweak, launch_cs2_with_dpi_tweak, rollback_cs2_dpi_tweak,
    Cs2DpiOptimizationTweak, CS2_DPI_APPLY_CMD, CS2_DPI_QUERY_CMD, CS2_DPI_ROLLBACK_CMD,
};
