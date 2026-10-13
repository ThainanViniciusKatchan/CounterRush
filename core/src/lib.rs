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
        let category = self.launch_auto()?;
        windows_api::wait_for_process("cs2", timeout)?;
        self.apply_tweaks()?;
        Ok(category)
    }
}

impl Default for OptimizerCore {
    fn default() -> Self {
        Self::new()
    }
}
