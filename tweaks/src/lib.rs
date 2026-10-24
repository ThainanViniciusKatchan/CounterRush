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

pub mod memory_compression;
pub use memory_compression::{
    disable_memory_compression, enable_memory_compression, get_memory_compression_status,
    MemoryCompressionTweak,
};

pub mod timer_resolution;
pub use timer_resolution::{
    apply_timer_registry, apply_timer_resolution, is_precision_reduced,
    is_timer_registry_applied, rollback_timer_all, rollback_timer_registry,
    rollback_timer_resolution, test_timer_precision, TimerApplyResult, TimerResolutionTweak,
    REG_KERNEL_KEY, REG_TIMER_VALUE,
};

pub mod mmcss;
pub use mmcss::{
    apply_all_mmcss, apply_mmcss_with_tier, detect_mmcss_hardware_tier, get_mmcss_status,
    is_mmcss_applied_for_current_hardware, is_mmcss_applied_for_tier, rollback_all_mmcss,
    MmcssHardwareTier, MmcssStatus, MmcssTweak, REG_GAMES_PROFILE_KEY, REG_SYSTEM_PROFILE_KEY,
};

pub mod hpet;
pub use hpet::{disable_hpet, get_hpet_status, rollback_hpet, HpetStatus, HpetTweak};

pub mod game_bar;
pub use game_bar::{
    disable_all_game_bar_features, disable_gamedvr_captures, disable_game_bar_shortcuts,
    disable_game_mode, disable_xbox_mode, enable_gamedvr_captures, enable_game_bar_shortcuts,
    enable_game_mode, enable_xbox_mode, get_game_bar_status, rollback_all_game_bar_features,
    GameBarStatus, GameBarTweak, DEFAULT_XBOX_APP_PACKAGE, REG_APP_CAPTURE_KEY, REG_GAME_BAR_KEY,
    REG_GAME_CONFIG_STORE_KEY, REG_GAMING_CONFIG_KEY, REG_POLICIES_GAMEDVR_KEY,
};

pub mod msi;
pub use msi::{
    enable_all_msi, enable_gpu_msi, enable_network_msi, enable_usb_msi, get_msi_status,
    is_all_msi_enabled, rollback_all_msi, rollback_gpu_msi, rollback_network_msi,
    rollback_usb_msi, MsiDeviceStatus, MsiModeTweak,
};
