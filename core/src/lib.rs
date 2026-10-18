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
}

pub mod classifier;
pub use classifier::{classify_pc, PcCategory};

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
}




impl Default for OptimizerCore {
    fn default() -> Self {
        Self::new()
    }
}
