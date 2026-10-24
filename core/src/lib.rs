use std::path::{Path, PathBuf};
use thiserror::Error;
use tweaks::{Tweak, TweakError};

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("Erro ao gerenciar tweak: {0}")]
    Tweak(#[from] TweakError),
    #[error("Tweak com ID '{0}' não foi encontrado")]
    NotFound(String),
    #[error("Erro na API do Windows: {0}")]
    WindowsApi(#[from] windows_api::WindowsApiError),
    #[error("Erro nas configurações: {0}")]
    Config(#[from] config::ConfigError),
}

pub mod classifier;
pub use classifier::{classify_pc, PcCategory};

#[derive(Debug, Clone, Default)]
pub struct CustomLaunchReport {
    pub config: config::CustomOptimizationConfig,
    pub applied_tweaks: Vec<&'static str>,
    pub skipped_tweaks: Vec<&'static str>,
}

pub struct OptimizerCore {
    tweaks: Vec<Box<dyn Tweak>>,
}

impl OptimizerCore {
    pub fn new() -> Self {
        let mut core = Self { tweaks: Vec::new() };
        core.register_default_tweaks();
        core
    }

    pub fn register_default_tweaks(&mut self) {
        self.register(tweaks::Cs2PriorityTweak);
        self.register(tweaks::Cs2AffinityTweak);
    }

    pub fn register<T: Tweak + 'static>(&mut self, tweak: T) {
        self.tweaks.push(Box::new(tweak));
    }

    pub fn tweaks(&self) -> &[Box<dyn Tweak>] {
        &self.tweaks
    }

    pub fn empty_standby_list(&self) -> Result<(), CoreError> {
        tweaks::empty_standby_list()?;
        Ok(())
    }

    pub fn launch_game(&self) -> Result<(), CoreError> {
        windows_api::launch_cs2()?;
        Ok(())
    }

    pub fn apply_tweaks(&self) -> Result<(), CoreError> {
        for tweak in &self.tweaks {
            tweak.apply()?;
        }
        Ok(())
    }

    pub fn launch_and_optimize(&self, timeout: std::time::Duration) -> Result<(), CoreError> {
        let _ = self.empty_standby_list();
        self.launch_game()?;
        windows_api::wait_for_process("cs2", timeout)?;
        self.apply_tweaks()?;
        Ok(())
    }

    pub fn get_hardware_profile(&self) -> Result<(windows_api::HardwareInfo, PcCategory), CoreError> {
        let info = windows_api::detect_hardware()?;
        let category = classify_pc(&info);
        Ok((info, category))
    }

    pub fn launch_economico(&self) -> Result<(), CoreError> {
        windows_api::launch_cs2_economico()?;
        Ok(())
    }

    pub fn launch_intermediario(&self) -> Result<(), CoreError> {
        windows_api::launch_cs2_intermediario()?;
        Ok(())
    }

    pub fn launch_potente(&self) -> Result<(), CoreError> {
        windows_api::launch_cs2_potente()?;
        Ok(())
    }

    pub fn launch_by_category(&self, category: PcCategory) -> Result<(), CoreError> {
        match category {
            PcCategory::Economico => self.launch_economico(),
            PcCategory::Intermediario => self.launch_intermediario(),
            PcCategory::Potente => self.launch_potente(),
        }
    }

    pub fn launch_auto(&self) -> Result<PcCategory, CoreError> {
        let (_, category) = self.get_hardware_profile()?;
        self.launch_by_category(category)?;
        Ok(category)
    }

    pub fn launch_and_optimize_auto(&self, timeout: std::time::Duration) -> Result<PcCategory, CoreError> {
        let _ = self.empty_standby_list();
        let category = self.launch_auto()?;
        windows_api::wait_for_process("cs2", timeout)?;
        self.apply_tweaks()?;
        Ok(category)
    }

    pub fn disable_all_gpu_acceleration(&self) -> Result<(), CoreError> {
        tweaks::disable_all_gpu_accel()?;
        Ok(())
    }

    pub fn rollback_all_gpu_acceleration(&self) -> Result<(), CoreError> {
        tweaks::enable_all_gpu_accel()?;
        Ok(())
    }

    pub fn apply_dpi_optimization(&self) -> Result<(), CoreError> {
        tweaks::apply_cs2_dpi_tweak()?;
        Ok(())
    }

    pub fn rollback_dpi_optimization(&self) -> Result<(), CoreError> {
        tweaks::rollback_cs2_dpi_tweak()?;
        Ok(())
    }

    pub fn launch_with_dpi_optimization(&self) -> Result<(), CoreError> {
        tweaks::launch_cs2_with_dpi_tweak()?;
        Ok(())
    }

    pub fn disable_memory_compression(&self) -> Result<(), CoreError> {
        tweaks::disable_memory_compression()?;
        Ok(())
    }

    pub fn rollback_memory_compression(&self) -> Result<(), CoreError> {
        tweaks::enable_memory_compression()?;
        Ok(())
    }

    pub fn get_memory_compression_status(&self) -> Result<bool, CoreError> {
        let status = tweaks::get_memory_compression_status()?;
        Ok(status)
    }

    pub fn apply_timer_resolution(&self) -> Result<tweaks::TimerApplyResult, CoreError> {
        let result = tweaks::apply_timer_resolution()?;
        Ok(result)
    }

    pub fn test_timer_precision(&self) -> Result<Vec<f64>, CoreError> {
        let pauses = tweaks::test_timer_precision()?;
        Ok(pauses)
    }

    pub fn rollback_timer_resolution(&self) -> Result<f64, CoreError> {
        let restored = tweaks::rollback_timer_resolution()?;
        Ok(restored)
    }

    pub fn rollback_timer_registry(&self) -> Result<(), CoreError> {
        tweaks::rollback_timer_registry()?;
        Ok(())
    }

    pub fn rollback_timer_all(&self) -> Result<(), CoreError> {
        tweaks::rollback_timer_all()?;
        Ok(())
    }

    pub fn is_timer_registry_applied(&self) -> Result<bool, CoreError> {
        let applied = tweaks::is_timer_registry_applied()?;
        Ok(applied)
    }

    pub fn apply_all_mmcss(&self) -> Result<tweaks::MmcssHardwareTier, CoreError> {
        let tier = tweaks::apply_all_mmcss()?;
        Ok(tier)
    }

    pub fn apply_mmcss_with_tier(&self, tier: tweaks::MmcssHardwareTier) -> Result<(), CoreError> {
        tweaks::apply_mmcss_with_tier(tier)?;
        Ok(())
    }

    pub fn rollback_all_mmcss(&self) -> Result<(), CoreError> {
        tweaks::rollback_all_mmcss()?;
        Ok(())
    }

    pub fn get_mmcss_status(&self) -> Result<tweaks::MmcssStatus, CoreError> {
        let status = tweaks::get_mmcss_status()?;
        Ok(status)
    }

    pub fn is_mmcss_applied_for_current_hardware(&self) -> Result<bool, CoreError> {
        let applied = tweaks::is_mmcss_applied_for_current_hardware()?;
        Ok(applied)
    }

    pub fn disable_hpet(&self) -> Result<(), CoreError> {
        tweaks::disable_hpet()?;
        Ok(())
    }

    pub fn rollback_hpet(&self) -> Result<(), CoreError> {
        tweaks::rollback_hpet()?;
        Ok(())
    }

    pub fn get_hpet_status(&self) -> Result<tweaks::HpetStatus, CoreError> {
        let status = tweaks::get_hpet_status()?;
        Ok(status)
    }

    pub fn disable_all_game_bar_features(&self) -> Result<(), CoreError> {
        tweaks::disable_all_game_bar_features()?;
        Ok(())
    }

    pub fn rollback_all_game_bar_features(&self) -> Result<(), CoreError> {
        tweaks::rollback_all_game_bar_features()?;
        Ok(())
    }

    pub fn get_game_bar_status(&self) -> Result<tweaks::GameBarStatus, CoreError> {
        let status = tweaks::get_game_bar_status()?;
        Ok(status)
    }

    pub fn enable_all_msi(&self) -> Result<(), CoreError> {
        tweaks::enable_all_msi()?;
        Ok(())
    }

    pub fn rollback_all_msi(&self) -> Result<(), CoreError> {
        tweaks::rollback_all_msi()?;
        Ok(())
    }

    pub fn get_msi_status(&self) -> Result<Vec<tweaks::MsiDeviceStatus>, CoreError> {
        let status = tweaks::get_msi_status()?;
        Ok(status)
    }

    pub fn disable_all_power_saving(&self) -> Result<(), CoreError> {
        tweaks::disable_all_power_saving()?;
        Ok(())
    }

    pub fn rollback_all_power_saving(&self) -> Result<(), CoreError> {
        tweaks::rollback_all_power_saving()?;
        Ok(())
    }

    pub fn get_power_saving_status(&self) -> Result<tweaks::PowerSavingStatus, CoreError> {
        let status = tweaks::get_power_saving_status()?;
        Ok(status)
    }

    pub fn apply_render_queue(&self) -> Result<(), CoreError> {
        tweaks::apply_render_queue()?;
        Ok(())
    }

    pub fn rollback_render_queue(&self) -> Result<(), CoreError> {
        tweaks::rollback_render_queue()?;
        Ok(())
    }

    pub fn get_render_queue_status(&self) -> Result<tweaks::RenderQueueStatus, CoreError> {
        let status = tweaks::get_render_queue_status()?;
        Ok(status)
    }

    pub fn load_or_create_custom_config(
        &self,
        config_path: Option<&Path>,
    ) -> Result<(PathBuf, config::CustomOptimizationConfig), CoreError> {
        let path = resolve_custom_config_path(config_path);
        let default_args = if !path.exists() {
            let (_, category) = self.get_hardware_profile()?;
            get_default_launch_args_for_category(category)
        } else {
            Vec::new()
        };
        let cfg = config::CustomOptimizationConfig::load_or_create(&path, default_args)?;
        Ok((path, cfg))
    }

    pub fn launch_custom(
        &self,
        config_path: Option<&Path>,
        timeout: std::time::Duration,
    ) -> Result<CustomLaunchReport, CoreError> {
        let (_resolved_path, cfg) = self.load_or_create_custom_config(config_path)?;
        let mut report = CustomLaunchReport {
            config: cfg.clone(),
            applied_tweaks: Vec::new(),
            skipped_tweaks: Vec::new(),
        };

        if cfg.empty_standby_list {
            self.empty_standby_list()?;
            report.applied_tweaks.push("Limpeza da Standby List (RAM)");
        }

        if cfg.disable_gpu_acceleration {
            if tweaks::is_all_gpu_accel_disabled()? {
                report.skipped_tweaks.push("Desativar Aceleração de GPU em Apps");
            } else {
                self.disable_all_gpu_acceleration()?;
                report.applied_tweaks.push("Desativar Aceleração de GPU em Apps");
            }
        }

        if cfg.apply_dpi_optimization {
            if tweaks::Cs2DpiOptimizationTweak.status()? == tweaks::TweakStatus::Applied {
                report.skipped_tweaks.push("Otimização de DPI e Modo Janela");
            } else {
                self.apply_dpi_optimization()?;
                report.applied_tweaks.push("Otimização de DPI e Modo Janela");
            }
        }

        if cfg.disable_memory_compression {
            let is_enabled = self.get_memory_compression_status()?;
            if !is_enabled {
                report.skipped_tweaks.push("Desativação da Compactação de Memória");
            } else {
                self.disable_memory_compression()?;
                report.applied_tweaks.push("Desativação da Compactação de Memória");
            }
        }

        if cfg.apply_timer_resolution {
            if self.is_timer_registry_applied()? {
                report.skipped_tweaks.push("Resolução do Temporizador do Sistema (0.5ms)");
            } else {
                self.apply_timer_resolution()?;
                report.applied_tweaks.push("Resolução do Temporizador do Sistema (0.5ms)");
            }
        }

        if cfg.apply_mmcss {
            if self.is_mmcss_applied_for_current_hardware()? {
                report.skipped_tweaks.push("Otimização do Agendador Multimídia (MMCSS)");
            } else {
                self.apply_all_mmcss()?;
                report.applied_tweaks.push("Otimização do Agendador Multimídia (MMCSS)");
            }
        }

        if cfg.disable_hpet {
            let status = self.get_hpet_status()?;
            if status == tweaks::HpetStatus::Disabled {
                report.skipped_tweaks.push("Desativação do HPET");
            } else {
                self.disable_hpet()?;
                report.applied_tweaks.push("Desativação do HPET");
            }
        }

        if cfg.disable_game_bar {
            let status = self.get_game_bar_status()?;
            if status.is_all_disabled() {
                report.skipped_tweaks.push("Desativação do Xbox Game Bar e GameDVR");
            } else {
                self.disable_all_game_bar_features()?;
                report.applied_tweaks.push("Desativação do Xbox Game Bar e GameDVR");
            }
        }

        if cfg.enable_msi {
            if tweaks::is_all_msi_enabled()? {
                report.skipped_tweaks.push("Ativação do Modo MSI");
            } else {
                self.enable_all_msi()?;
                report.applied_tweaks.push("Ativação do Modo MSI");
            }
        }

        if cfg.disable_power_saving {
            let status = self.get_power_saving_status()?;
            if status.is_all_optimized() {
                report.skipped_tweaks.push("Desativação de Economia de Energia PCIe e USB");
            } else {
                self.disable_all_power_saving()?;
                report.applied_tweaks.push("Desativação de Economia de Energia PCIe e USB");
            }
        }

        if cfg.apply_render_queue {
            let status = self.get_render_queue_status()?;
            if status.is_all_optimized() {
                report.skipped_tweaks.push("Fila de Quadros Renderizados (Render Queue)");
            } else {
                self.apply_render_queue()?;
                report.applied_tweaks.push("Fila de Quadros Renderizados (Render Queue)");
            }
        }

        windows_api::launch_cs2_with_args_slice(&cfg.launch_args)?;
        windows_api::wait_for_process("cs2", timeout)?;

        if cfg.cs2_priority {
            if tweaks::Cs2PriorityTweak.status()? == tweaks::TweakStatus::Applied {
                report.skipped_tweaks.push("Prioridade em Tempo Real do CS2");
            } else {
                tweaks::Cs2PriorityTweak.apply()?;
                report.applied_tweaks.push("Prioridade em Tempo Real do CS2");
            }
        }

        if cfg.cs2_affinity {
            if tweaks::Cs2AffinityTweak.status()? == tweaks::TweakStatus::Applied {
                report.skipped_tweaks.push("Afinidade de Processador do CS2 (sem Core 0)");
            } else {
                tweaks::Cs2AffinityTweak.apply()?;
                report.applied_tweaks.push("Afinidade de Processador do CS2 (sem Core 0)");
            }
        }

        Ok(report)
    }
}

pub fn get_default_launch_args_for_category(category: PcCategory) -> Vec<String> {
    match category {
        PcCategory::Economico => vec![
            "-console".to_string(),
            "-nojoy".to_string(),
            "+fps_max".to_string(),
            "120".to_string(),
            "-vulkan".to_string(),
        ],
        PcCategory::Intermediario => vec![
            "-fullscreen".to_string(),
            "-console".to_string(),
            "-nojoy".to_string(),
            "+engine_low_latency_sleep_after_client_tick".to_string(),
            "true".to_string(),
            "+fps_max".to_string(),
            "240".to_string(),
            "+mat_queue_mode".to_string(),
        ],
        PcCategory::Potente => vec![
            "-console".to_string(),
            "-nojoy".to_string(),
            "-refresh".to_string(),
            "240".to_string(),
            "+fps_max".to_string(),
            "400".to_string(),
            "+engine_low_latency_sleep_after_client_tick".to_string(),
            "true".to_string(),
        ],
    }
}

pub fn resolve_custom_config_path(custom: Option<&Path>) -> PathBuf {
    if let Some(p) = custom {
        return p.to_path_buf();
    }
    let direct = PathBuf::from("config/custom_optimization.json");
    if direct.exists() || Path::new("config").is_dir() {
        return direct;
    }
    let parent_config = PathBuf::from("../config/custom_optimization.json");
    if parent_config.exists() || Path::new("../config").is_dir() {
        return parent_config;
    }
    direct
}

impl Default for OptimizerCore {
    fn default() -> Self {
        Self::new()
    }
}
