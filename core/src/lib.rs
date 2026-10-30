use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
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
    #[error("Erro de I/O ao acessar arquivos: {0}")]
    Io(#[from] std::io::Error),
    #[error("Falha ao processar arquivo JSON: {0}")]
    Json(#[from] serde_json::Error),
}

pub mod classifier;
pub use classifier::{classify_pc, PcCategory};
pub use config;

#[derive(Debug, Clone, Default)]
pub struct CustomLaunchReport {
    pub config: config::CustomOptimizationConfig,
    pub applied_tweaks: Vec<&'static str>,
    pub skipped_tweaks: Vec<&'static str>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakEvaluation {
    pub id: String,
    pub name: String,
    pub active: bool,
    pub supported: bool,
    pub optional_default_off: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareRestrictionsReport {
    pub hardware: windows_api::HardwareInfo,
    pub category: PcCategory,
    pub evaluations: Vec<TweakEvaluation>,
    pub blocked_tweaks: Vec<String>,
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

    pub fn detect_and_save_hardware(
        &self,
        custom_path: Option<&Path>,
    ) -> Result<(PathBuf, windows_api::HardwareInfo, PcCategory), CoreError> {
        let (info, category) = self.get_hardware_profile()?;
        let path = resolve_hardware_config_path(custom_path);
        save_hardware_info(&path, &info)?;
        Ok((path, info, category))
    }

    pub fn get_or_detect_hardware(
        &self,
        custom_path: Option<&Path>,
    ) -> Result<(PathBuf, windows_api::HardwareInfo, PcCategory, bool), CoreError> {
        let path = resolve_hardware_config_path(custom_path);
        if path.exists() {
            let info = load_hardware_info(&path)?;
            let category = classify_pc(&info);
            Ok((path, info, category, true))
        } else {
            let (path, info, category) = self.detect_and_save_hardware(Some(&path))?;
            Ok((path, info, category, false))
        }
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

    pub fn generate_autodetect_config(
        &self,
    ) -> Result<(config::CustomOptimizationConfig, HardwareRestrictionsReport), CoreError> {
        let (_hw_path, hardware, category, _from_cache) = self.get_or_detect_hardware(None)?;
        self.generate_autodetect_config_for_hardware(hardware, category)
    }

    pub fn generate_autodetect_config_for_hardware(
        &self,
        hardware: windows_api::HardwareInfo,
        category: PcCategory,
    ) -> Result<(config::CustomOptimizationConfig, HardwareRestrictionsReport), CoreError> {
        let launch_args = get_default_launch_args_for_category(category);

        let mut evaluations = Vec::new();
        let mut blocked_tweaks = Vec::new();

        evaluations.push(TweakEvaluation {
            id: "cs2_priority".to_string(),
            name: "Prioridade em Tempo Real do CS2".to_string(),
            active: false,
            supported: true,
            optional_default_off: true,
            reason: "Desativada por padrão conforme diretriz do sistema. Mantida como otimização opcional para controle do usuário.".to_string(),
        });

        let affinity_supported = hardware.cpu.physical_cores > 4;
        let affinity_active = affinity_supported;
        let affinity_reason = if affinity_supported {
            format!("Compatível: CPU com {} núcleos físicos permite isolar o Core 0 sem sobrecarregar as threads do CS2.", hardware.cpu.physical_cores)
        } else {
            let msg = format!("Bloqueada por hardware: CPU com {} núcleos físicos (mínimo exigido: mais de 4 núcleos). Isolar o Core 0 causaria perda de desempenho no jogo.", hardware.cpu.physical_cores);
            blocked_tweaks.push(msg.clone());
            msg
        };
        evaluations.push(TweakEvaluation {
            id: "cs2_affinity".to_string(),
            name: "Afinidade de Processador do CS2 (sem Core 0)".to_string(),
            active: affinity_active,
            supported: affinity_supported,
            optional_default_off: false,
            reason: affinity_reason,
        });

        let mem_comp_supported = hardware.ram.total_gb > 16.0;
        let mem_comp_active = mem_comp_supported;
        let mem_comp_reason = if mem_comp_supported {
            format!("Compatível: Sistema possui {:.2} GB de RAM (> 16 GB). Desativar compactação reduz latência de CPU sem risco de exaustão de memória.", hardware.ram.total_gb)
        } else {
            let msg = format!("Bloqueada por hardware: Sistema possui {:.2} GB de RAM (mínimo exigido: mais de 16 GB). A compactação é necessária para evitar esgotamento de RAM.", hardware.ram.total_gb);
            blocked_tweaks.push(msg.clone());
            msg
        };
        evaluations.push(TweakEvaluation {
            id: "disable_memory_compression".to_string(),
            name: "Desativação da Compactação de Memória".to_string(),
            active: mem_comp_active,
            supported: mem_comp_supported,
            optional_default_off: false,
            reason: mem_comp_reason,
        });

        let gpu_lower = hardware.gpu.model.to_lowercase();
        let is_nvidia = gpu_lower.contains("nvidia")
            || gpu_lower.contains("geforce")
            || gpu_lower.contains("rtx")
            || gpu_lower.contains("gtx")
            || tweaks::is_nvidia_smi_available();
        let is_amd = gpu_lower.contains("amd") || gpu_lower.contains("radeon");
        let render_queue_supported = is_nvidia || is_amd;
        let render_queue_active = render_queue_supported;
        let render_queue_reason = if is_nvidia {
            "Compatível: GPU NVIDIA detectada. Modo de persistência do driver suportado para baixa latência.".to_string()
        } else if is_amd {
            "Compatível: GPU AMD detectada. FlipQueueSize = 1 suportado no Registro.".to_string()
        } else {
            let msg = format!("Bloqueada por hardware: GPU dedicada compatível (AMD/NVIDIA) não identificada (detectado: '{}').", hardware.gpu.model);
            blocked_tweaks.push(msg.clone());
            msg
        };
        evaluations.push(TweakEvaluation {
            id: "apply_render_queue".to_string(),
            name: "Fila de Quadros Renderizados (Render Queue)".to_string(),
            active: render_queue_active,
            supported: render_queue_supported,
            optional_default_off: false,
            reason: render_queue_reason,
        });

        let gpu_accel_active = category != PcCategory::Potente;
        let gpu_accel_reason = if gpu_accel_active {
            format!("Ativado: Recomendado para perfil {} para poupar VRAM dedicada e ciclos de decodificação.", category)
        } else {
            "Desativado: Perfil Potente com VRAM abundante; aceleração de hardware mantida nos aplicativos secundários.".to_string()
        };
        evaluations.push(TweakEvaluation {
            id: "disable_gpu_acceleration".to_string(),
            name: "Desativar Aceleração de GPU em Apps".to_string(),
            active: gpu_accel_active,
            supported: true,
            optional_default_off: category == PcCategory::Potente,
            reason: gpu_accel_reason,
        });

        evaluations.push(TweakEvaluation {
            id: "empty_standby_list".to_string(),
            name: "Limpeza da Standby List (RAM)".to_string(),
            active: true,
            supported: true,
            optional_default_off: false,
            reason: "Compatível: Universalmente suportado. Libera memória cache antes da inicialização.".to_string(),
        });

        evaluations.push(TweakEvaluation {
            id: "apply_dpi_optimization".to_string(),
            name: "Otimização de DPI e Modo Janela".to_string(),
            active: true,
            supported: true,
            optional_default_off: false,
            reason: "Compatível: Universalmente suportado. Elimina latência de composição de janelas do DirectX.".to_string(),
        });

        evaluations.push(TweakEvaluation {
            id: "apply_timer_resolution".to_string(),
            name: "Resolução do Temporizador do Sistema (0.5ms)".to_string(),
            active: true,
            supported: true,
            optional_default_off: false,
            reason: "Compatível: Universalmente suportado. Crava o timer do kernel em 0.5ms para menor input lag.".to_string(),
        });

        evaluations.push(TweakEvaluation {
            id: "apply_mmcss".to_string(),
            name: "Otimização do Agendador Multimídia (MMCSS)".to_string(),
            active: true,
            supported: true,
            optional_default_off: false,
            reason: "Compatível: Universalmente suportado com calibração automática conforme núcleos e RAM.".to_string(),
        });

        evaluations.push(TweakEvaluation {
            id: "disable_hpet".to_string(),
            name: "Desativação do HPET".to_string(),
            active: true,
            supported: true,
            optional_default_off: false,
            reason: "Compatível: Universalmente suportado para uso de contadores TSC de alta frequência da CPU.".to_string(),
        });

        evaluations.push(TweakEvaluation {
            id: "disable_game_bar".to_string(),
            name: "Desativação do Xbox Game Bar e GameDVR".to_string(),
            active: true,
            supported: true,
            optional_default_off: false,
            reason: "Compatível: Universalmente suportado. Elimina overlays e gravação em segundo plano.".to_string(),
        });

        evaluations.push(TweakEvaluation {
            id: "enable_msi".to_string(),
            name: "Ativação do Modo MSI".to_string(),
            active: true,
            supported: true,
            optional_default_off: false,
            reason: "Compatível: Ativa interrupções baseadas em mensagem para barramentos PCI.".to_string(),
        });

        evaluations.push(TweakEvaluation {
            id: "disable_power_saving".to_string(),
            name: "Desativação de Economia de Energia PCIe e USB".to_string(),
            active: true,
            supported: true,
            optional_default_off: false,
            reason: "Compatível: Previne estados de suspensão em dispositivos PCIe e USB durante o jogo.".to_string(),
        });

        let config = config::CustomOptimizationConfig {
            launch_args,
            empty_standby_list: true,
            cs2_priority: false,
            cs2_affinity: affinity_active,
            disable_gpu_acceleration: gpu_accel_active,
            apply_dpi_optimization: true,
            disable_memory_compression: mem_comp_active,
            apply_timer_resolution: true,
            apply_mmcss: true,
            disable_hpet: true,
            disable_game_bar: true,
            enable_msi: true,
            disable_power_saving: true,
            apply_render_queue: render_queue_active,
        };

        let report = HardwareRestrictionsReport {
            hardware,
            category,
            evaluations,
            blocked_tweaks,
        };

        Ok((config, report))
    }

    pub fn save_autodetect_config(
        &self,
        custom_path: Option<&Path>,
    ) -> Result<(PathBuf, config::CustomOptimizationConfig, HardwareRestrictionsReport), CoreError> {
        let (config, report) = self.generate_autodetect_config()?;
        let path = resolve_autodetect_config_path(custom_path);
        config.save_to_file(&path)?;
        Ok((path, config, report))
    }

    pub fn load_or_create_autodetect_config(
        &self,
        config_path: Option<&Path>,
    ) -> Result<(PathBuf, config::CustomOptimizationConfig, HardwareRestrictionsReport), CoreError> {
        let path = resolve_autodetect_config_path(config_path);
        if path.exists() {
            let cfg = config::CustomOptimizationConfig::load_from_file(&path)?;
            let (_, report) = self.generate_autodetect_config()?;
            Ok((path, cfg, report))
        } else {
            self.save_autodetect_config(Some(&path))
        }
    }

    pub fn enforce_hardware_restrictions(
        &self,
        config: &mut config::CustomOptimizationConfig,
    ) -> Result<Vec<String>, CoreError> {
        let (hardware, _) = self.get_hardware_profile()?;
        Ok(enforce_hardware_restrictions_for_info(config, &hardware))

    }

    pub fn launch_custom(
        &self,
        config_path: Option<&Path>,
        timeout: std::time::Duration,
    ) -> Result<CustomLaunchReport, CoreError> {
        let (_resolved_path, mut cfg) = self.load_or_create_custom_config(config_path)?;

        let mut report = CustomLaunchReport {
            config: cfg.clone(),
            applied_tweaks: Vec::new(),
            skipped_tweaks: Vec::new(),
        };

        if let Ok((hw_info, _)) = self.get_hardware_profile() {
            let blocked = enforce_hardware_restrictions_for_info(&mut cfg, &hw_info);
            for b in blocked {
                report.skipped_tweaks.push(Box::leak(b.into_boxed_str()));
            }
            report.config = cfg.clone();
        };

        if cfg.empty_standby_list {
            match self.empty_standby_list() {
                Ok(_) => report.applied_tweaks.push("Limpeza da Standby List (RAM)"),
                Err(_) => report.skipped_tweaks.push("Limpeza da Standby List (RAM) [requer Administrador]"),
            }
        }

        if cfg.disable_gpu_acceleration {
            match tweaks::is_all_gpu_accel_disabled() {
                Ok(true) => report.skipped_tweaks.push("Desativar Aceleração de GPU em Apps"),
                Ok(false) => {
                    if self.disable_all_gpu_acceleration().is_ok() {
                        report.applied_tweaks.push("Desativar Aceleração de GPU em Apps");
                    } else {
                        report.skipped_tweaks.push("Desativar Aceleração de GPU em Apps [não aplicado]");
                    }
                }
                Err(_) => report.skipped_tweaks.push("Desativar Aceleração de GPU em Apps [ignorado]"),
            }
        }

        if cfg.apply_dpi_optimization {
            match tweaks::Cs2DpiOptimizationTweak.status() {
                Ok(tweaks::TweakStatus::Applied) => {
                    report.skipped_tweaks.push("Otimização de DPI e Modo Janela");
                }
                _ => {
                    if self.apply_dpi_optimization().is_ok() {
                        report.applied_tweaks.push("Otimização de DPI e Modo Janela");
                    } else {
                        report.skipped_tweaks.push("Otimização de DPI e Modo Janela [não aplicado]");
                    }
                }
            }
        }

        if cfg.disable_memory_compression {
            match self.get_memory_compression_status() {
                Ok(false) => report.skipped_tweaks.push("Desativação da Compactação de Memória"),
                _ => {
                    if self.disable_memory_compression().is_ok() {
                        report.applied_tweaks.push("Desativação da Compactação de Memória");
                    } else {
                        report.skipped_tweaks.push("Desativação da Compactação de Memória [requer Administrador/RAM > 16GB]");
                    }
                }
            }
        }

        if cfg.apply_timer_resolution {
            if self.apply_timer_resolution().is_ok() {
                report.applied_tweaks.push("Resolução do Temporizador do Sistema (0.5ms)");
            } else {
                report.skipped_tweaks.push("Resolução do Temporizador do Sistema [não aplicado]");
            }
        }

        if cfg.apply_mmcss {
            match self.is_mmcss_applied_for_current_hardware() {
                Ok(true) => report.skipped_tweaks.push("Otimização do Agendador Multimídia (MMCSS)"),
                _ => {
                    if self.apply_all_mmcss().is_ok() {
                        report.applied_tweaks.push("Otimização do Agendador Multimídia (MMCSS)");
                    } else {
                        report.skipped_tweaks.push("Otimização do Agendador Multimídia (MMCSS) [requer Administrador]");
                    }
                }
            }
        }

        if cfg.disable_hpet {
            match self.get_hpet_status() {
                Ok(tweaks::HpetStatus::Disabled) => report.skipped_tweaks.push("Desativação do HPET"),
                _ => {
                    if self.disable_hpet().is_ok() {
                        report.applied_tweaks.push("Desativação do HPET");
                    } else {
                        report.skipped_tweaks.push("Desativação do HPET [requer Administrador]");
                    }
                }
            }
        }

        if cfg.disable_game_bar {
            match self.get_game_bar_status() {
                Ok(status) if status.is_all_disabled() => {
                    report.skipped_tweaks.push("Desativação do Xbox Game Bar e GameDVR");
                }
                _ => {
                    if self.disable_all_game_bar_features().is_ok() {
                        report.applied_tweaks.push("Desativação do Xbox Game Bar e GameDVR");
                    } else {
                        report.skipped_tweaks.push("Desativação do Xbox Game Bar e GameDVR [requer Administrador]");
                    }
                }
            }
        }

        if cfg.enable_msi {
            match tweaks::is_all_msi_enabled() {
                Ok(true) => report.skipped_tweaks.push("Ativação do Modo MSI"),
                _ => {
                    if self.enable_all_msi().is_ok() {
                        report.applied_tweaks.push("Ativação do Modo MSI");
                    } else {
                        report.skipped_tweaks.push("Ativação do Modo MSI [requer Administrador]");
                    }
                }
            }
        }

        if cfg.disable_power_saving {
            match self.get_power_saving_status() {
                Ok(status) if status.is_all_optimized() => {
                    report.skipped_tweaks.push("Desativação de Economia de Energia PCIe e USB");
                }
                _ => {
                    if self.disable_all_power_saving().is_ok() {
                        report.applied_tweaks.push("Desativação de Economia de Energia PCIe e USB");
                    } else {
                        report.skipped_tweaks.push("Desativação de Economia de Energia PCIe e USB [requer Administrador]");
                    }
                }
            }
        }

        if cfg.apply_render_queue {
            match self.get_render_queue_status() {
                Ok(status) if status.is_all_optimized() => {
                    report.skipped_tweaks.push("Fila de Quadros Renderizados (Render Queue)");
                }
                _ => {
                    if self.apply_render_queue().is_ok() {
                        report.applied_tweaks.push("Fila de Quadros Renderizados (Render Queue)");
                    } else {
                        report.skipped_tweaks.push("Fila de Quadros Renderizados [requer Administrador]");
                    }
                }
            }
        }

        windows_api::launch_cs2_with_args_slice(&cfg.launch_args)?;
        let process_found = windows_api::wait_for_process("cs2", timeout).is_ok();

        if process_found {
            if cfg.cs2_priority {
                match tweaks::Cs2PriorityTweak.status() {
                    Ok(tweaks::TweakStatus::Applied) => {
                        report.skipped_tweaks.push("Prioridade em Tempo Real do CS2");
                    }
                    _ => {
                        if tweaks::Cs2PriorityTweak.apply().is_ok() {
                            report.applied_tweaks.push("Prioridade em Tempo Real do CS2");
                        } else {
                            report.skipped_tweaks.push("Prioridade em Tempo Real do CS2 [requer Administrador]");
                        }
                    }
                }
            }

            if cfg.cs2_affinity {
                match tweaks::Cs2AffinityTweak.status() {
                    Ok(tweaks::TweakStatus::Applied) => {
                        report.skipped_tweaks.push("Afinidade de Processador do CS2 (sem Core 0)");
                    }
                    _ => {
                        if tweaks::Cs2AffinityTweak.apply().is_ok() {
                            report.applied_tweaks.push("Afinidade de Processador do CS2 (sem Core 0)");
                        } else {
                            report.skipped_tweaks.push("Afinidade de Processador do CS2 [não aplicado]");
                        }
                    }
                }
            }
        } else {
            report.skipped_tweaks.push("Tweaks pós-launch [CS2 inicializando em segundo plano]");
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

pub fn resolve_optimizations_dir() -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let candidate = exe_dir.join("config").join("optimzations");
            if candidate.is_dir() {
                return candidate;
            }
        }
    }

    let direct = PathBuf::from("config/optimzations");
    if direct.is_dir() {
        return direct;
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join("config").join("optimzations");
        }
    }

    direct
}

pub fn list_optimization_configs_in_dir<P: AsRef<Path>>(dir: P) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir.as_ref()) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file()
                && path
                    .extension()
                    .and_then(|s| s.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("json"))
                    == Some(true)
            {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

pub fn list_optimization_configs() -> Vec<PathBuf> {
    list_optimization_configs_in_dir(resolve_optimizations_dir())
}

pub fn resolve_optimization_file_path(name: &str) -> PathBuf {
    let opt_dir = resolve_optimizations_dir();
    let filename = if name.ends_with(".json") {
        name.to_string()
    } else {
        format!("{name}.json")
    };
    opt_dir.join(&filename)
}

pub fn validate_profile_name(name: &str) -> Result<String, String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("O nome do perfil não pode ser vazio.".to_string());
    }

    let base_name = if trimmed.len() >= 5
        && trimmed[trimmed.len() - 5..].eq_ignore_ascii_case(".json")
    {
        trimmed[..trimmed.len() - 5].trim()
    } else {
        trimmed
    };

    if base_name.is_empty() {
        return Err("O nome do perfil não pode ser vazio.".to_string());
    }

    if base_name.contains(|c: char| matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|')) {
        return Err("O nome contém caracteres proibidos pelo sistema operacional.".to_string());
    }

    let reserved = [
        "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7",
        "COM8", "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
    ];
    if reserved.contains(&base_name.to_uppercase().as_str()) {
        return Err("Nome de arquivo reservado pelo sistema operacional.".to_string());
    }

    Ok(base_name.to_string())
}

pub fn resolve_custom_config_path(custom: Option<&Path>) -> PathBuf {
    let opt_dir = resolve_optimizations_dir();

    if let Some(p) = custom {
        if p.exists() {
            return p.to_path_buf();
        }
        let p_json = if p.extension().is_none() {
            p.with_extension("json")
        } else {
            p.to_path_buf()
        };
        if p_json.exists() {
            return p_json;
        }

        let in_opt = opt_dir.join(&p_json);
        if in_opt.exists() {
            return in_opt;
        }

        if p_json.parent().map_or(true, |parent| parent.as_os_str().is_empty()) {
            return opt_dir.join(&p_json);
        }

        return p_json;
    }

    let default_file = opt_dir.join("custom_optimization.json");
    if default_file.exists() || opt_dir.is_dir() {
        return default_file;
    }

    default_file
}

pub fn resolve_autodetect_config_path(custom: Option<&Path>) -> PathBuf {
    let opt_dir = resolve_optimizations_dir();

    if let Some(p) = custom {
        if p.exists() {
            return p.to_path_buf();
        }
        let in_opt = opt_dir.join(p);
        if in_opt.exists() {
            return in_opt;
        }
        return in_opt;
    }

    let default_file = opt_dir.join("AutoDetect.json");
    if default_file.exists() || opt_dir.is_dir() {
        return default_file;
    }

    default_file
}

pub fn resolve_hardware_config_path(custom: Option<&Path>) -> PathBuf {
    if let Some(p) = custom {
        if p.exists() {
            return p.to_path_buf();
        }
        if p.starts_with("config") || p.starts_with("../config") {
            return p.to_path_buf();
        }
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                let in_exe_hw = exe_dir.join("config").join("hardware").join(p);
                if in_exe_hw.exists() {
                    return in_exe_hw;
                }
            }
        }
        let in_hardware = Path::new("config").join("hardware").join(p);
        if in_hardware.exists() {
            return in_hardware;
        }
        let in_config = Path::new("config").join(p);
        if in_config.exists() {
            return in_config;
        }
        if let Ok(exe_path) = std::env::current_exe() {
            if let Some(exe_dir) = exe_path.parent() {
                return exe_dir.join("config").join("hardware").join(p);
            }
        }
        return in_hardware;
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let candidate = exe_dir.join("config").join("hardware").join("Hardware.json");
            if candidate.exists() {
                return candidate;
            }
        }
    }

    let direct = PathBuf::from("config/hardware/Hardware.json");
    if direct.exists() || Path::new("config/hardware").is_dir() {
        return direct;
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join("config").join("hardware").join("Hardware.json");
        }
    }

    direct
}

pub fn save_hardware_info<P: AsRef<Path>>(path: P, info: &windows_api::HardwareInfo) -> Result<(), CoreError> {
    let path = path.as_ref();
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let json = serde_json::to_string_pretty(info)?;
    std::fs::write(path, json)?;
    Ok(())
}

pub fn load_hardware_info<P: AsRef<Path>>(path: P) -> Result<windows_api::HardwareInfo, CoreError> {
    let content = std::fs::read_to_string(path)?;
    let info = serde_json::from_str(&content)?;
    Ok(info)
}

pub fn enforce_hardware_restrictions_for_info(
    config: &mut config::CustomOptimizationConfig,
    hardware: &windows_api::HardwareInfo,
) -> Vec<String> {
    let mut blocked = Vec::new();

    if config.cs2_affinity && hardware.cpu.physical_cores <= 4 {
        config.cs2_affinity = false;
        blocked.push(format!(
            "Afinidade de CPU (cs2_affinity) bloqueada por hardware: requer mais de 4 núcleos físicos (detectado: {}).",
            hardware.cpu.physical_cores
        ));
    }

    if config.disable_memory_compression && hardware.ram.total_gb <= 16.0 {
        config.disable_memory_compression = false;
        blocked.push(format!(
            "Compactação de memória (disable_memory_compression) bloqueada por hardware: requer mais de 16 GB de RAM (detectado: {:.2} GB).",
            hardware.ram.total_gb
        ));
    }

    if config.apply_render_queue {
        let gpu_lower = hardware.gpu.model.to_lowercase();
        let is_nvidia = gpu_lower.contains("nvidia")
            || gpu_lower.contains("geforce")
            || gpu_lower.contains("rtx")
            || gpu_lower.contains("gtx")
            || tweaks::is_nvidia_smi_available();
        let is_amd = gpu_lower.contains("amd") || gpu_lower.contains("radeon");
        if !is_nvidia && !is_amd {
            config.apply_render_queue = false;
            blocked.push(format!(
                "Fila de renderização (apply_render_queue) bloqueada por hardware: GPU dedicada compatível (AMD/NVIDIA) não identificada (detectado: '{}').",
                hardware.gpu.model
            ));
        }
    }

    blocked
}

impl Default for OptimizerCore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_name_validation() {
        // Nomes válidos
        assert_eq!(validate_profile_name("competitivo").unwrap(), "competitivo");
        assert_eq!(validate_profile_name("  treino_cs2  ").unwrap(), "treino_cs2");
        assert_eq!(validate_profile_name("perfil.json").unwrap(), "perfil");
        assert_eq!(validate_profile_name("perfil.JSON").unwrap(), "perfil");
        assert_eq!(validate_profile_name("perfil.JsOn").unwrap(), "perfil");
        assert_eq!(validate_profile_name("meu perfil gamer.json").unwrap(), "meu perfil gamer");

        // Nomes vazios ou somente espaços
        assert!(validate_profile_name("").is_err());
        assert!(validate_profile_name("   ").is_err());
        assert!(validate_profile_name("\t\n").is_err());
        assert!(validate_profile_name(".json").is_err());
        assert!(validate_profile_name(" .JSON ").is_err());
        assert!(validate_profile_name(".Json").is_err());

        // Caracteres ilegais NTFS/Windows (\ / : * ? " < > |)
        for illegal in &['\\', '/', ':', '*', '?', '"', '<', '>', '|'] {
            let bad_name = format!("perfil{}teste", illegal);
            assert!(
                validate_profile_name(&bad_name).is_err(),
                "Deveria rejeitar o caractere ilegal: {}",
                illegal
            );
        }

        // Nomes de dispositivos reservados do Windows
        let reserved_names = [
            "CON", "con", "con.json", "PRN", "prn.JSON", "AUX", "aux.Json", "NUL", "nul",
            "COM1", "com1.json", "COM9", "com9", "LPT1", "lpt1.json", "LPT9", "lpt9",
        ];
        for res in &reserved_names {
            assert!(
                validate_profile_name(res).is_err(),
                "Deveria rejeitar nome reservado: {}",
                res
            );
        }
    }

    #[test]
    fn test_list_optimization_configs_empty_directory() {
        let temp_dir = tempfile::tempdir().expect("Falha ao criar diretório temporário");
        let configs = list_optimization_configs_in_dir(temp_dir.path());
        assert!(configs.is_empty(), "Lista de configs em diretório vazio deve ser vazia");

        // Verifica que nenhum arquivo foi criado no diretório temporário
        let count = std::fs::read_dir(temp_dir.path())
            .expect("Falha ao ler diretório temporário")
            .count();
        assert_eq!(count, 0, "Nenhum arquivo deve ser criado ao listar diretório vazio");
    }

    #[test]
    fn test_create_and_persist_new_profile() {
        let temp_dir = tempfile::tempdir().expect("Falha ao criar diretório temporário");

        let input_name = "ultra_fps.json";
        let clean_name = validate_profile_name(input_name).expect("Nome deveria ser válido");
        assert_eq!(clean_name, "ultra_fps");

        let target_file = temp_dir.path().join(format!("{clean_name}.json"));

        let mut config = config::CustomOptimizationConfig::all_disabled();
        config.launch_args = vec!["-novid".to_string(), "-high".to_string(), "+fps_max 0".to_string()];
        config.apply_timer_resolution = true;
        config.disable_hpet = true;
        config.cs2_priority = true;

        config.save_to_file(&target_file).expect("Falha ao persistir perfil de otimização");
        assert!(target_file.exists());

        let configs = list_optimization_configs_in_dir(temp_dir.path());
        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0], target_file);

        let loaded = config::CustomOptimizationConfig::load_from_file(&target_file)
            .expect("Falha ao recarregar perfil persistido");
        assert_eq!(config, loaded);
        assert_eq!(loaded.launch_args, vec!["-novid", "-high", "+fps_max 0"]);
        assert!(loaded.apply_timer_resolution);
        assert!(loaded.disable_hpet);
        assert!(loaded.cs2_priority);
        assert!(!loaded.apply_render_queue);
    }

    #[test]
    fn test_isolated_build_dir_does_not_escape_to_project_dir() {
        let temp_build_dir = tempfile::tempdir().expect("Falha ao criar diretório temporário");
        let non_existent_opt_dir = temp_build_dir.path().join("config").join("optimzations");

        // Em uma pasta de compilação sem a pasta config/optimzations, a lista de configurações deve ser estritamente vazia
        let configs = list_optimization_configs_in_dir(&non_existent_opt_dir);
        assert!(configs.is_empty(), "Build sem pasta config não deve retornar nenhuma configuração");

        // Criação de perfil no local isolado da build deve ser mantida estritamente no local
        std::fs::create_dir_all(&non_existent_opt_dir).expect("Falha ao criar config/optimzations na build");
        let local_profile = non_existent_opt_dir.join("meu_build_perfil.json");
        let mut cfg = config::CustomOptimizationConfig::all_disabled();
        cfg.cs2_priority = true;
        cfg.save_to_file(&local_profile).expect("Falha ao salvar no build");

        let configs_after = list_optimization_configs_in_dir(&non_existent_opt_dir);
        assert_eq!(configs_after.len(), 1);
        assert_eq!(configs_after[0], local_profile);
    }
}
