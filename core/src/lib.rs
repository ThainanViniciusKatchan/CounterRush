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
pub mod startup;
pub use startup::{
    disable_system_startup, enable_system_startup, is_system_startup_enabled,
    normalize_exe_path, StartupManager, TaskSchedulerStartupManager,
};
pub use config;
pub use windows_api;
pub use tweaks;

pub mod steam_monitor;
pub use steam_monitor::{
    GameMonitorService, MonitorState, ProcessQuery, SteamLogDetector, SteamRegistryDetector,
    Win32ProcessQuery, Win32SteamLogDetector, Win32SteamRegistryDetector, CS2_STEAM_APP_ID,
};
pub use windows_api::{SingleInstanceGuard, SingleInstanceOutcome, DEFAULT_SINGLE_INSTANCE_ID};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakExecutionResult {
    pub id: String,
    pub name: String,
    pub success: bool,
    pub message: String,
}

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
    pub already_applied: bool,
    pub needs_optimization: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareRestrictionsReport {
    pub hardware: windows_api::HardwareInfo,
    pub category: PcCategory,
    pub evaluations: Vec<TweakEvaluation>,
    pub blocked_tweaks: Vec<String>,
    pub already_optimized_tweaks: Vec<String>,
    pub pending_tweaks: Vec<String>,
}

impl HardwareRestrictionsReport {
    /// Retorna `true` se a otimização informada já estiver aplicada no sistema operacional.
    pub fn is_already_applied(&self, id: &str) -> bool {
        self.evaluations
            .iter()
            .find(|e| e.id == id)
            .map(|e| e.already_applied)
            .unwrap_or(false)
    }

    /// Retorna `true` se a otimização estiver recomendada pelo hardware e ainda precisar ser aplicada.
    pub fn needs_optimization(&self, id: &str) -> bool {
        self.evaluations
            .iter()
            .find(|e| e.id == id)
            .map(|e| e.needs_optimization)
            .unwrap_or(false)
    }
}

/// Snapshot do estado atual das otimizações no sistema operacional.
#[derive(Debug, Clone, Default)]
pub struct SystemTweakStatuses {
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

impl SystemTweakStatuses {
    /// Consulta os verificadores reais do sistema reaproveitando o hardware já detectado.
    pub fn query_for_hardware(hardware: &windows_api::HardwareInfo) -> Self {
        let cs2_running = windows_api::is_cs2_process_running_win32().unwrap_or(false);
        let is_applied = |res: Result<tweaks::TweakStatus, tweaks::TweakError>| {
            matches!(res, Ok(tweaks::TweakStatus::Applied))
        };

        Self {
            empty_standby_list: false,
            cs2_priority: cs2_running && is_applied(tweaks::Cs2PriorityTweak.status()),
            cs2_affinity: cs2_running && is_applied(tweaks::Cs2AffinityTweak.status()),
            disable_gpu_acceleration: tweaks::is_all_gpu_accel_disabled().unwrap_or(false),
            apply_dpi_optimization: is_applied(tweaks::Cs2DpiOptimizationTweak.status()),
            disable_memory_compression: is_applied(tweaks::MemoryCompressionTweak.status()),
            apply_timer_resolution: is_applied(tweaks::TimerResolutionTweak.status()),
            apply_mmcss: tweaks::is_mmcss_applied_for_hardware_specs(
                hardware.cpu.physical_cores,
                hardware.ram.total_gb,
            )
            .unwrap_or(false),
            disable_hpet: is_applied(tweaks::HpetTweak.status()),
            disable_game_bar: is_applied(tweaks::GameBarTweak.status()),
            enable_msi: tweaks::is_all_msi_enabled().unwrap_or(false),
            disable_power_saving: is_applied(tweaks::PcieUsbPowerSavingTweak.status()),
            apply_render_queue: is_applied(tweaks::RenderQueueTweak.status()),
        }
    }
}

struct EvaluationCollector {
    evaluations: Vec<TweakEvaluation>,
    blocked_tweaks: Vec<String>,
    already_optimized_tweaks: Vec<String>,
    pending_tweaks: Vec<String>,
}

impl EvaluationCollector {
    fn new() -> Self {
        Self {
            evaluations: Vec::with_capacity(13),
            blocked_tweaks: Vec::new(),
            already_optimized_tweaks: Vec::new(),
            pending_tweaks: Vec::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn push(
        &mut self,
        id: &str,
        name: &str,
        active: bool,
        supported: bool,
        optional_default_off: bool,
        already_applied: bool,
        reason: String,
    ) {
        let needs_optimization = active && !already_applied;
        if !supported {
            self.blocked_tweaks.push(reason.clone());
        } else if already_applied {
            self.already_optimized_tweaks.push(id.to_string());
        } else if needs_optimization {
            self.pending_tweaks.push(id.to_string());
        }

        self.evaluations.push(TweakEvaluation {
            id: id.to_string(),
            name: name.to_string(),
            active,
            supported,
            optional_default_off,
            already_applied,
            needs_optimization,
            reason,
        });
    }
}

fn format_status_suffix(
    active: bool,
    supported: bool,
    already_applied: bool,
    is_runtime: bool,
) -> &'static str {
    if !supported {
        return "Status SO: Não aplicável devido à restrição de hardware.";
    }
    if is_runtime {
        return if already_applied {
            "Status SO: Já aplicada no processo cs2.exe em execução."
        } else {
            "Status SO: Otimização de execução (aplicada ao iniciar o CS2)."
        };
    }
    if already_applied {
        "Status SO: Já otimizado no sistema (será preservado sem reaplicação)."
    } else if active {
        "Status SO: Pendente (será aplicado na otimização automática)."
    } else {
        "Status SO: Mantido desativado por padrão para este perfil."
    }
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

    /// Verifica o hardware atual e o status real das otimizações no sistema para gerar o `AutoDetect.json` e o relatório completo.
    pub fn generate_autodetect_config(
        &self,
    ) -> Result<(config::CustomOptimizationConfig, HardwareRestrictionsReport), CoreError> {
        let (_hw_path, hardware, category, _from_cache) = self.get_or_detect_hardware(None)?;
        let statuses = SystemTweakStatuses::query_for_hardware(&hardware);
        self.evaluate_autodetect_for_hardware_and_status(hardware, category, &statuses)
    }

    /// Gera a configuração recomendada avaliando apenas as capacidades e limites de hardware.
    pub fn generate_autodetect_config_for_hardware(
        &self,
        hardware: windows_api::HardwareInfo,
        category: PcCategory,
    ) -> Result<(config::CustomOptimizationConfig, HardwareRestrictionsReport), CoreError> {
        self.evaluate_autodetect_for_hardware_and_status(
            hardware,
            category,
            &SystemTweakStatuses::default(),
        )
    }

    /// Combina regras de hardware/categoria com o status atual das otimizações no sistema operacional.
    pub fn evaluate_autodetect_for_hardware_and_status(
        &self,
        hardware: windows_api::HardwareInfo,
        category: PcCategory,
        statuses: &SystemTweakStatuses,
    ) -> Result<(config::CustomOptimizationConfig, HardwareRestrictionsReport), CoreError> {
        let launch_args = get_default_launch_args_for_category(category);
        let mut collector = EvaluationCollector::new();

        let priority_reason = format!(
            "Desativada por padrão conforme diretriz do sistema. Mantida como otimização opcional para controle do usuário. | {}",
            if statuses.cs2_priority {
                "Status SO: Já aplicada no processo cs2.exe em execução."
            } else {
                "Status SO: Não aplicada."
            }
        );
        collector.push(
            "cs2_priority",
            "Prioridade em Tempo Real do CS2",
            false,
            true,
            true,
            statuses.cs2_priority,
            priority_reason,
        );

        let affinity_supported = hardware.cpu.physical_cores > 4;
        let affinity_base = if affinity_supported {
            format!(
                "Compatível: CPU com {} núcleos físicos permite isolar o Core 0 sem sobrecarregar as threads do CS2.",
                hardware.cpu.physical_cores
            )
        } else {
            format!(
                "Bloqueada por hardware: CPU com {} núcleos físicos (mínimo exigido: mais de 4 núcleos). Isolar o Core 0 causaria perda de desempenho no jogo.",
                hardware.cpu.physical_cores
            )
        };
        let affinity_reason = format!(
            "{affinity_base} | {}",
            format_status_suffix(
                affinity_supported,
                affinity_supported,
                statuses.cs2_affinity,
                true
            )
        );
        collector.push(
            "cs2_affinity",
            "Afinidade de Processador do CS2 (sem Core 0)",
            affinity_supported,
            affinity_supported,
            false,
            statuses.cs2_affinity,
            affinity_reason,
        );

        let mem_comp_supported = hardware.ram.total_gb > 16.0;
        let mem_comp_base = if mem_comp_supported {
            format!(
                "Compatível: Sistema possui {:.2} GB de RAM (> 16 GB). Desativar compactação reduz latência de CPU sem risco de exaustão de memória.",
                hardware.ram.total_gb
            )
        } else {
            format!(
                "Bloqueada por hardware: Sistema possui {:.2} GB de RAM (mínimo exigido: mais de 16 GB). A compactação é necessária para evitar esgotamento de RAM.",
                hardware.ram.total_gb
            )
        };
        let mem_comp_reason = format!(
            "{mem_comp_base} | {}",
            format_status_suffix(
                mem_comp_supported,
                mem_comp_supported,
                statuses.disable_memory_compression,
                false
            )
        );
        collector.push(
            "disable_memory_compression",
            "Desativação da Compactação de Memória",
            mem_comp_supported,
            mem_comp_supported,
            false,
            statuses.disable_memory_compression,
            mem_comp_reason,
        );

        let gpu_lower = hardware.gpu.model.to_lowercase();
        let is_nvidia = gpu_lower.contains("nvidia")
            || gpu_lower.contains("geforce")
            || gpu_lower.contains("rtx")
            || gpu_lower.contains("gtx")
            || (gpu_lower.is_empty() && tweaks::is_nvidia_smi_available());
        let is_amd = gpu_lower.contains("amd") || gpu_lower.contains("radeon");
        let render_queue_supported = is_nvidia || is_amd;
        let render_queue_already_applied = render_queue_supported && statuses.apply_render_queue;
        let render_queue_base = if is_nvidia {
            "Compatível: GPU NVIDIA detectada. Modo de persistência do driver suportado para baixa latência.".to_string()
        } else if is_amd {
            "Compatível: GPU AMD detectada. FlipQueueSize = 1 suportado no Registro.".to_string()
        } else {
            format!(
                "Bloqueada por hardware: GPU dedicada compatível (AMD/NVIDIA) não identificada (detectado: '{}').",
                hardware.gpu.model
            )
        };
        let render_queue_reason = format!(
            "{render_queue_base} | {}",
            format_status_suffix(
                render_queue_supported,
                render_queue_supported,
                render_queue_already_applied,
                false
            )
        );
        collector.push(
            "apply_render_queue",
            "Fila de Quadros Renderizados (Render Queue)",
            render_queue_supported,
            render_queue_supported,
            false,
            render_queue_already_applied,
            render_queue_reason,
        );

        let gpu_accel_active = category != PcCategory::Potente;
        let gpu_accel_base = if gpu_accel_active {
            format!(
                "Ativado: Recomendado para perfil {} para poupar VRAM dedicada e ciclos de decodificação.",
                category
            )
        } else {
            "Desativado: Perfil Potente com VRAM abundante; aceleração de hardware mantida nos aplicativos secundários.".to_string()
        };
        let gpu_accel_reason = format!(
            "{gpu_accel_base} | {}",
            format_status_suffix(
                gpu_accel_active,
                true,
                statuses.disable_gpu_acceleration,
                false
            )
        );
        collector.push(
            "disable_gpu_acceleration",
            "Desativar Aceleração de GPU em Apps",
            gpu_accel_active,
            true,
            category == PcCategory::Potente,
            statuses.disable_gpu_acceleration,
            gpu_accel_reason,
        );

        let universal_tweaks = [
            (
                "empty_standby_list",
                "Limpeza da Standby List (RAM)",
                "Compatível: Universalmente suportado. Libera memória cache antes da inicialização.",
                true,
                statuses.empty_standby_list,
            ),
            (
                "apply_dpi_optimization",
                "Otimização de DPI e Modo Janela",
                "Compatível: Universalmente suportado. Elimina latência de composição de janelas do DirectX.",
                false,
                statuses.apply_dpi_optimization,
            ),
            (
                "apply_timer_resolution",
                "Resolução do Temporizador do Sistema (0.5ms)",
                "Compatível: Universalmente suportado. Crava o timer do kernel em 0.5ms para menor input lag.",
                false,
                statuses.apply_timer_resolution,
            ),
            (
                "apply_mmcss",
                "Otimização do Agendador Multimídia (MMCSS)",
                "Compatível: Universalmente suportado com calibração automática conforme núcleos e RAM.",
                false,
                statuses.apply_mmcss,
            ),
            (
                "disable_hpet",
                "Desativação do HPET",
                "Compatível: Universalmente suportado para uso de contadores TSC de alta frequência da CPU.",
                false,
                statuses.disable_hpet,
            ),
            (
                "disable_game_bar",
                "Desativação do Xbox Game Bar e GameDVR",
                "Compatível: Universalmente suportado. Elimina overlays e gravação em segundo plano.",
                false,
                statuses.disable_game_bar,
            ),
            (
                "enable_msi",
                "Ativação do Modo MSI",
                "Compatível: Ativa interrupções baseadas em mensagem para barramentos PCI.",
                false,
                statuses.enable_msi,
            ),
            (
                "disable_power_saving",
                "Desativação de Economia de Energia PCIe e USB",
                "Compatível: Previne estados de suspensão em dispositivos PCIe e USB durante o jogo.",
                false,
                statuses.disable_power_saving,
            ),
        ];

        for (id, name, hardware_reason, is_runtime, already_applied) in universal_tweaks {
            let reason = format!(
                "{hardware_reason} | {}",
                format_status_suffix(true, true, already_applied, is_runtime)
            );
            collector.push(id, name, true, true, false, already_applied, reason);
        }

        let config = config::CustomOptimizationConfig {
            launch_args,
            empty_standby_list: true,
            cs2_priority: false,
            cs2_affinity: affinity_supported,
            disable_gpu_acceleration: gpu_accel_active,
            apply_dpi_optimization: true,
            disable_memory_compression: mem_comp_supported,
            apply_timer_resolution: true,
            apply_mmcss: true,
            disable_hpet: true,
            disable_game_bar: true,
            enable_msi: true,
            disable_power_saving: true,
            apply_render_queue: render_queue_supported,
        };

        let report = HardwareRestrictionsReport {
            hardware,
            category,
            evaluations: collector.evaluations,
            blocked_tweaks: collector.blocked_tweaks,
            already_optimized_tweaks: collector.already_optimized_tweaks,
            pending_tweaks: collector.pending_tweaks,
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

    pub fn optimize_running_cs2(
        &self,
        pid: Option<u32>,
        config_path: Option<&Path>,
    ) -> Result<Vec<TweakExecutionResult>, CoreError> {
        let (_resolved_path, mut cfg) = self.load_or_create_custom_config(config_path)?;
        let mut results = Vec::new();

        if let Ok((hw_info, _)) = self.get_hardware_profile() {
            let blocked = enforce_hardware_restrictions_for_info(&mut cfg, &hw_info);
            for b in blocked {
                results.push(TweakExecutionResult {
                    id: "hardware_restriction".to_string(),
                    name: b.clone(),
                    success: false,
                    message: format!("Tweak '{b}' bloqueado devido a restrições de hardware."),
                });
            }
        }

        if cfg.empty_standby_list {
            match self.empty_standby_list() {
                Ok(_) => results.push(TweakExecutionResult {
                    id: "empty_standby_list".into(),
                    name: "Limpeza da Standby List (RAM)".into(),
                    success: true,
                    message: "Memória cache liberada com sucesso.".into(),
                }),
                Err(err) => results.push(TweakExecutionResult {
                    id: "empty_standby_list".into(),
                    name: "Limpeza da Standby List (RAM)".into(),
                    success: false,
                    message: format!("Falha ao limpar Standby List: {err}"),
                }),
            }
        }

        if cfg.disable_gpu_acceleration {
            match tweaks::is_all_gpu_accel_disabled() {
                Ok(true) => results.push(TweakExecutionResult {
                    id: "gpu_accel".into(),
                    name: "Desativar Aceleração de GPU em Apps".into(),
                    success: true,
                    message: "Já desativada em navegadores e apps de suporte.".into(),
                }),
                _ => {
                    let res = self.disable_all_gpu_acceleration();
                    results.push(TweakExecutionResult {
                        id: "gpu_accel".into(),
                        name: "Desativar Aceleração de GPU em Apps".into(),
                        success: res.is_ok(),
                        message: if res.is_ok() {
                            "Aceleração de GPU desativada em apps secundários.".into()
                        } else {
                            "Não foi possível desativar aceleração de GPU em todos os apps.".into()
                        },
                    });
                }
            }
        }

        if cfg.apply_dpi_optimization {
            let res = self.apply_dpi_optimization();
            results.push(TweakExecutionResult {
                id: "dpi_optimization".into(),
                name: "Otimização de DPI e Modo Janela".into(),
                success: res.is_ok(),
                message: if res.is_ok() {
                    "Otimizações de tela cheia e DPI aplicadas.".into()
                } else {
                    "Falha ao aplicar otimização de DPI.".into()
                },
            });
        }

        if cfg.disable_memory_compression {
            let res = self.disable_memory_compression();
            results.push(TweakExecutionResult {
                id: "memory_compression".into(),
                name: "Desativação da Compactação de Memória".into(),
                success: res.is_ok(),
                message: if res.is_ok() {
                    "Compactação de memória desativada.".into()
                } else {
                    "Falha ou permissão insuficiente para compactação de memória.".into()
                },
            });
        }

        if cfg.apply_timer_resolution {
            let res = self.apply_timer_resolution();
            results.push(TweakExecutionResult {
                id: "timer_resolution".into(),
                name: "Resolução do Temporizador do Sistema (0.5ms)".into(),
                success: res.is_ok(),
                message: if res.is_ok() {
                    "Resolução do temporizador configurada para 0.5ms.".into()
                } else {
                    "Falha ao ajustar resolução do temporizador.".into()
                },
            });
        }

        if cfg.apply_mmcss {
            let res = self.apply_all_mmcss();
            results.push(TweakExecutionResult {
                id: "mmcss".into(),
                name: "Otimização do Agendador Multimídia (MMCSS)".into(),
                success: res.is_ok(),
                message: if res.is_ok() {
                    "Prioridade MMCSS configurada para jogos.".into()
                } else {
                    "Falha ao ajustar MMCSS.".into()
                },
            });
        }

        if cfg.disable_hpet {
            let res = self.disable_hpet();
            results.push(TweakExecutionResult {
                id: "hpet".into(),
                name: "Desativação do HPET".into(),
                success: res.is_ok(),
                message: if res.is_ok() {
                    "HPET desativado com sucesso.".into()
                } else {
                    "Falha ao desativar HPET.".into()
                },
            });
        }

        if cfg.disable_game_bar {
            let res = self.disable_all_game_bar_features();
            results.push(TweakExecutionResult {
                id: "game_bar".into(),
                name: "Desativação do Xbox Game Bar e GameDVR".into(),
                success: res.is_ok(),
                message: if res.is_ok() {
                    "Xbox Game Bar e GameDVR desativados.".into()
                } else {
                    "Falha ao desativar Xbox Game Bar.".into()
                },
            });
        }

        if cfg.enable_msi {
            let res = self.enable_all_msi();
            results.push(TweakExecutionResult {
                id: "msi".into(),
                name: "Ativação do Modo MSI".into(),
                success: res.is_ok(),
                message: if res.is_ok() {
                    "Modo MSI ativado para dispositivos compatíveis.".into()
                } else {
                    "Falha ao configurar Modo MSI.".into()
                },
            });
        }

        if cfg.disable_power_saving {
            let res = self.disable_all_power_saving();
            results.push(TweakExecutionResult {
                id: "power_saving".into(),
                name: "Desativação de Economia de Energia PCIe e USB".into(),
                success: res.is_ok(),
                message: if res.is_ok() {
                    "Economia de energia PCIe e USB desativada.".into()
                } else {
                    "Falha ao desativar economia de energia.".into()
                },
            });
        }

        if cfg.apply_render_queue {
            let res = self.apply_render_queue();
            results.push(TweakExecutionResult {
                id: "render_queue".into(),
                name: "Fila de Quadros Renderizados (Render Queue)".into(),
                success: res.is_ok(),
                message: if res.is_ok() {
                    "Fila de quadros otimizada para baixa latência.".into()
                } else {
                    "Falha ao configurar fila de renderização.".into()
                },
            });
        }

        let is_running = if let Some(p) = pid {
            windows_api::is_pid_alive(p)
        } else {
            windows_api::is_cs2_process_running_win32().unwrap_or(false)
        };

        if is_running {
            if cfg.cs2_priority {
                let res = tweaks::Cs2PriorityTweak.apply();
                results.push(TweakExecutionResult {
                    id: "cs2_priority".into(),
                    name: "Prioridade em Tempo Real do CS2".into(),
                    success: res.is_ok(),
                    message: if res.is_ok() {
                        "Prioridade do processo definida para Tempo Real.".into()
                    } else {
                        "Falha ao definir prioridade do processo.".into()
                    },
                });
            }

            if cfg.cs2_affinity {
                let res = tweaks::Cs2AffinityTweak.apply();
                results.push(TweakExecutionResult {
                    id: "cs2_affinity".into(),
                    name: "Afinidade de Processador do CS2".into(),
                    success: res.is_ok(),
                    message: if res.is_ok() {
                        "Afinidade configurada (Core 0 liberado para o SO).".into()
                    } else {
                        "Falha ao configurar afinidade de CPU.".into()
                    },
                });
            }
        }

        Ok(results)
    }
}

/// Reconcilia os flags de inicialização com o sistema e auto-otimização Steam.
///
/// Ligar a inicialização com o Windows não ativa a auto-otimização Steam automaticamente;
/// por outro lado, sem inicialização com o sistema ativa, a auto-otimização permanece desligada.
pub fn reconcile_startup_flags(
    startup_task_active: bool,
    optimize_on_steam_launch: bool,
) -> (bool, bool) {
    let effective_startup = startup_task_active;
    let effective_steam_auto = effective_startup && optimize_on_steam_launch;
    (effective_startup, effective_steam_auto)
}

/// Tenta adquirir o bloqueio de instância única do CounterRush ou notifica a instância ativa.
pub fn acquire_single_instance() -> Result<SingleInstanceOutcome, CoreError> {
    acquire_single_instance_with_id(DEFAULT_SINGLE_INSTANCE_ID)
}

pub fn acquire_single_instance_with_id(instance_id: &str) -> Result<SingleInstanceOutcome, CoreError> {
    windows_api::acquire_single_instance(instance_id).map_err(CoreError::from)
}

/// Notifica uma instância já em execução (se existir) para trazê-la ao primeiro plano sem reter posse.
pub fn try_notify_running_instance() -> bool {
    try_notify_running_instance_with_id(DEFAULT_SINGLE_INSTANCE_ID)
}

pub fn try_notify_running_instance_with_id(instance_id: &str) -> bool {
    windows_api::notify_existing_instance(instance_id).unwrap_or(false)
}

/// Seleciona o nome do perfil ativo priorizando `saved_active_preset` quando ainda existir.
pub fn resolve_selected_preset_name<'a>(
    saved_active_preset: Option<&str>,
    available_names: &[&'a str],
) -> Option<&'a str> {
    if available_names.is_empty() {
        return None;
    }
    saved_active_preset
        .and_then(|saved| available_names.iter().copied().find(|name| *name == saved))
        .or_else(|| available_names.first().copied())
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

fn current_exe_dir() -> Option<PathBuf> {
    std::env::current_exe()
        .ok()
        .and_then(|exe_path| exe_path.parent().map(|dir| dir.to_path_buf()))
}

fn is_workspace_cwd(cwd: &Path) -> bool {
    cwd.join("Cargo.toml").is_file()
}

fn join_dir(base: &Path, segments: &[&str]) -> PathBuf {
    let mut out = if base == Path::new(".") {
        PathBuf::new()
    } else {
        base.to_path_buf()
    };
    for segment in segments {
        out.push(segment);
    }
    out
}

fn resolve_optimizations_dir_with(exe_dir: Option<&Path>, cwd: &Path) -> PathBuf {
    if let Some(dir) = exe_dir {
        let candidate = join_dir(dir, &["config", "optimzations"]);
        if candidate.is_dir() {
            return candidate;
        }
    }

    if is_workspace_cwd(cwd) {
        let direct = join_dir(cwd, &["config", "optimzations"]);
        if direct.is_dir() {
            return direct;
        }
    }

    if let Some(dir) = exe_dir {
        return join_dir(dir, &["config", "optimzations"]);
    }

    join_dir(cwd, &["config", "optimzations"])
}

pub fn resolve_optimizations_dir() -> PathBuf {
    resolve_optimizations_dir_with(current_exe_dir().as_deref(), Path::new("."))
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

fn resolve_app_settings_path_with(exe_dir: Option<&Path>, cwd: &Path) -> PathBuf {
    if let Some(dir) = exe_dir {
        let candidate = join_dir(dir, &["config", "settings.json"]);
        if candidate.exists() {
            return candidate;
        }
    }

    if is_workspace_cwd(cwd) {
        let direct = join_dir(cwd, &["config", "settings.json"]);
        if direct.exists() {
            return direct;
        }
    }

    if let Some(dir) = exe_dir {
        return join_dir(dir, &["config", "settings.json"]);
    }

    join_dir(cwd, &["config", "settings.json"])
}

pub fn resolve_app_settings_path() -> PathBuf {
    resolve_app_settings_path_with(current_exe_dir().as_deref(), Path::new("."))
}

pub fn resolve_locale_dir() -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            let candidate = exe_dir.join("locale");
            if candidate.is_dir() {
                return candidate;
            }
        }
    }

    let direct = PathBuf::from("locale");
    if direct.is_dir() {
        return direct;
    }

    let rel_parent = PathBuf::from("../locale");
    if rel_parent.is_dir() {
        return rel_parent;
    }

    direct
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LocaleInfo {
    pub code: String,
    pub display_name: String,
}

pub fn list_available_locales() -> Vec<LocaleInfo> {
    list_available_locales_in_dir(resolve_locale_dir())
}

pub fn list_available_locales_in_dir<P: AsRef<Path>>(dir: P) -> Vec<LocaleInfo> {
    let mut locales = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir.as_ref()) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
                    let ui_po = path.join("counterrush-ui.po");
                    let ui_lc_po = path.join("LC_MESSAGES").join("counterrush-ui.po");
                    let generic_po = path.join("counterrush.po");
                    if ui_po.exists() || ui_lc_po.exists() || generic_po.exists() {
                        let display_name = match dir_name {
                            "pt_BR" => "Português (Brasil)",
                            "en" => "English",
                            "en_US" => "English (United States)",
                            "ru" => "Русский",
                            "ru_RU" => "Русский (Россия)",
                            other => other,
                        };
                        locales.push(LocaleInfo {
                            code: dir_name.to_string(),
                            display_name: display_name.to_string(),
                        });
                    }
                }
            }
        }
    }

    locales.sort_by(|a, b| {
        let order = |code: &str| match code {
            "pt_BR" => 0,
            "en" => 1,
            "en_US" => 2,
            "ru" => 3,
            "ru_RU" => 4,
            _ => 10,
        };
        let ord_a = order(&a.code);
        let ord_b = order(&b.code);
        if ord_a != ord_b {
            ord_a.cmp(&ord_b)
        } else {
            a.code.cmp(&b.code)
        }
    });

    locales
}

pub fn set_clipboard(text: &str) -> Result<(), CoreError> {
    windows_api::clipboard::set_clipboard_text(text)?;
    Ok(())
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

pub fn delete_optimization_profile_in_dir<P: AsRef<Path>>(dir: P, name: &str) -> Result<(), CoreError> {
    let clean_name = validate_profile_name(name)
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidInput, err))?;
    let path = dir.as_ref().join(format!("{clean_name}.json"));
    if !path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Perfil '{clean_name}' não foi encontrado para exclusão."),
        ).into());
    }
    std::fs::remove_file(path)?;
    Ok(())
}

pub fn delete_optimization_profile(name: &str) -> Result<(), CoreError> {
    delete_optimization_profile_in_dir(resolve_optimizations_dir(), name)
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
        if !p.as_os_str().is_empty() && !p.to_string_lossy().trim().is_empty() {
            if p.is_dir() {
                return p.join("AutoDetect.json");
            }
            let p_json = if p.extension().is_none() {
                p.with_extension("json")
            } else {
                p.to_path_buf()
            };
            if p_json.is_absolute() || p_json.exists() {
                return p_json;
            }
            if p_json.parent().map_or(false, |parent| !parent.as_os_str().is_empty()) {
                return p_json;
            }
            return opt_dir.join(p_json);
        }
    }

    opt_dir.join("AutoDetect.json")
}

fn resolve_hardware_config_path_with(
    custom: Option<&Path>,
    exe_dir: Option<&Path>,
    cwd: &Path,
) -> PathBuf {
    if let Some(p) = custom {
        if p.exists() {
            return p.to_path_buf();
        }
        if p.starts_with("config") || p.starts_with("../config") {
            return p.to_path_buf();
        }
        if let Some(dir) = exe_dir {
            let in_exe_hw = join_dir(dir, &["config", "hardware"]).join(p);
            if in_exe_hw.exists() {
                return in_exe_hw;
            }
        }
        if is_workspace_cwd(cwd) {
            let in_hardware = join_dir(cwd, &["config", "hardware"]).join(p);
            if in_hardware.exists() {
                return in_hardware;
            }
            let in_config = join_dir(cwd, &["config"]).join(p);
            if in_config.exists() {
                return in_config;
            }
        }
        if let Some(dir) = exe_dir {
            return join_dir(dir, &["config", "hardware"]).join(p);
        }
        return join_dir(cwd, &["config", "hardware"]).join(p);
    }

    if let Some(dir) = exe_dir {
        let candidate = join_dir(dir, &["config", "hardware", "Hardware.json"]);
        if candidate.exists() {
            return candidate;
        }
    }

    if is_workspace_cwd(cwd) {
        let direct = join_dir(cwd, &["config", "hardware", "Hardware.json"]);
        if direct.exists() || join_dir(cwd, &["config", "hardware"]).is_dir() {
            return direct;
        }
    }

    if let Some(dir) = exe_dir {
        return join_dir(dir, &["config", "hardware", "Hardware.json"]);
    }

    join_dir(cwd, &["config", "hardware", "Hardware.json"])
}

pub fn resolve_hardware_config_path(custom: Option<&Path>) -> PathBuf {
    resolve_hardware_config_path_with(custom, current_exe_dir().as_deref(), Path::new("."))
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

    #[test]
    fn test_autodetect_evaluation_respects_hardware_limits() {
        let core = OptimizerCore::new();

        // Cenário 1: Hardware modesto/antigo (CPU 4 núcleos, 8GB RAM, GPU Intel Integrada)
        let hw_modesto = windows_api::HardwareInfo {
            cpu: windows_api::CpuInfo {
                model: "Intel Core i5-4460".to_string(),
                physical_cores: 4,
                logical_cores: 4,
                frequency_mhz: 3200,
            },
            gpu: windows_api::GpuInfo {
                model: "Intel HD Graphics 4600".to_string(),
                vram_gb: 1.0,
            },
            ram: windows_api::RamInfo {
                total_gb: 8.0,
                speed_mhz: Some(1600),
            },
        };

        let (cfg_modesto, report_modesto) = core
            .generate_autodetect_config_for_hardware(hw_modesto, PcCategory::Economico)
            .expect("Falha ao gerar autodetect");

        assert!(!cfg_modesto.cs2_affinity, "CPU com 4 núcleos deve desativar cs2_affinity");
        assert!(!cfg_modesto.disable_memory_compression, "RAM <= 16GB deve manter compactação ativa");
        assert!(!cfg_modesto.apply_render_queue, "GPU integrada não deve ativar fila de renderização");
        assert!(cfg_modesto.disable_gpu_acceleration, "Perfil econômico deve desativar aceleração de GPU");
        assert!(!report_modesto.blocked_tweaks.is_empty(), "Deve listar tweaks bloqueados");

        // Cenário 2: Hardware potente (CPU 8 núcleos, 32GB RAM, GPU NVIDIA RTX 4070)
        let hw_potente = windows_api::HardwareInfo {
            cpu: windows_api::CpuInfo {
                model: "AMD Ryzen 7 7800X3D".to_string(),
                physical_cores: 8,
                logical_cores: 16,
                frequency_mhz: 4200,
            },
            gpu: windows_api::GpuInfo {
                model: "NVIDIA GeForce RTX 4070".to_string(),
                vram_gb: 12.0,
            },
            ram: windows_api::RamInfo {
                total_gb: 32.0,
                speed_mhz: Some(6000),
            },
        };

        let (cfg_potente, report_potente) = core
            .generate_autodetect_config_for_hardware(hw_potente, PcCategory::Potente)
            .expect("Falha ao gerar autodetect");

        assert!(cfg_potente.cs2_affinity, "CPU com 8 núcleos deve ativar cs2_affinity");
        assert!(cfg_potente.disable_memory_compression, "RAM > 16GB deve desativar compactação");
        assert!(cfg_potente.apply_render_queue, "GPU NVIDIA deve ativar fila de renderização");
        assert!(!cfg_potente.disable_gpu_acceleration, "Perfil potente mantém aceleração de GPU secundária");
        assert!(report_potente.blocked_tweaks.is_empty(), "Hardware potente não deve ter tweaks bloqueados");
    }

    #[test]
    fn test_evaluate_autodetect_combines_hardware_and_os_statuses() {
        let core = OptimizerCore::new();
        let hw = windows_api::HardwareInfo {
            cpu: windows_api::CpuInfo {
                model: "Intel Core i5-12400F".to_string(),
                physical_cores: 6,
                logical_cores: 12,
                frequency_mhz: 2500,
            },
            gpu: windows_api::GpuInfo {
                model: "NVIDIA GeForce RTX 3060".to_string(),
                vram_gb: 12.0,
            },
            ram: windows_api::RamInfo {
                total_gb: 24.0,
                speed_mhz: Some(3200),
            },
        };
        let statuses = SystemTweakStatuses {
            apply_dpi_optimization: true,
            apply_mmcss: true,
            disable_game_bar: true,
            enable_msi: true,
            ..Default::default()
        };

        let (cfg, report) = core
            .evaluate_autodetect_for_hardware_and_status(hw, PcCategory::Intermediario, &statuses)
            .expect("Falha ao avaliar hardware + status");

        // AutoDetect.json deve permanecer ativo para tudo que é recomendado para o hardware
        assert!(cfg.apply_dpi_optimization);
        assert!(cfg.apply_mmcss);
        assert!(cfg.disable_hpet);
        assert!(cfg.disable_gpu_acceleration);
        assert!(cfg.apply_render_queue);
        assert!(!cfg.cs2_priority);

        // Relatório distingue o que já estava otimizado do que ainda precisa ser aplicado
        assert!(report.is_already_applied("apply_dpi_optimization"));
        assert!(!report.needs_optimization("apply_dpi_optimization"));
        assert!(!report.is_already_applied("disable_hpet"));
        assert!(report.needs_optimization("disable_hpet"));
        assert!(report.already_optimized_tweaks.contains(&"apply_dpi_optimization".to_string()));
        assert!(report.already_optimized_tweaks.contains(&"apply_mmcss".to_string()));
        assert!(report.pending_tweaks.contains(&"disable_hpet".to_string()));
        assert!(report.pending_tweaks.contains(&"disable_gpu_acceleration".to_string()));
        assert!(!report.pending_tweaks.contains(&"apply_dpi_optimization".to_string()));
        assert!(!report.pending_tweaks.contains(&"cs2_priority".to_string()));

        // Hardware incompatível bloqueia mesmo se snapshot externo indicar flag ativa
        let hw_igpu = windows_api::HardwareInfo {
            cpu: windows_api::CpuInfo {
                model: "Intel Core i3-10100".to_string(),
                physical_cores: 4,
                logical_cores: 8,
                frequency_mhz: 3600,
            },
            gpu: windows_api::GpuInfo {
                model: "Intel(R) UHD Graphics 630".to_string(),
                vram_gb: 1.0,
            },
            ram: windows_api::RamInfo {
                total_gb: 8.0,
                speed_mhz: Some(2666),
            },
        };
        let statuses_igpu = SystemTweakStatuses {
            apply_render_queue: true,
            disable_game_bar: true,
            ..Default::default()
        };
        let (cfg_igpu, report_igpu) = core
            .evaluate_autodetect_for_hardware_and_status(
                hw_igpu,
                PcCategory::Economico,
                &statuses_igpu,
            )
            .expect("Falha ao avaliar hardware com GPU integrada");
        assert!(!cfg_igpu.apply_render_queue);
        assert!(!report_igpu.is_already_applied("apply_render_queue"));
        assert!(!report_igpu.needs_optimization("apply_render_queue"));
        assert!(!report_igpu.blocked_tweaks.is_empty());
        assert!(report_igpu.is_already_applied("disable_game_bar"));
    }

    #[test]
    fn test_autodetect_config_generation_and_no_custom_optimization() {
        let core = OptimizerCore::new();
        let temp_dir = tempfile::tempdir().expect("Falha ao criar diretório temporário");
        let autodetect_path = temp_dir.path().join("AutoDetect.json");
        let custom_opt_path = temp_dir.path().join("custom_optimization.json");

        let (path, config, report) = core
            .save_autodetect_config(Some(&autodetect_path))
            .expect("Falha ao executar save_autodetect_config");

        assert_eq!(path, autodetect_path);
        assert!(autodetect_path.exists(), "O arquivo AutoDetect.json deve ser criado");
        assert!(
            !custom_opt_path.exists(),
            "O arquivo custom_optimization.json NÃO deve ser criado sob nenhuma circunstância"
        );

        let created_files: Vec<String> = std::fs::read_dir(temp_dir.path())
            .expect("Falha ao ler diretório temporário de teste")
            .filter_map(|e| e.ok().map(|entry| entry.file_name().to_string_lossy().to_string()))
            .collect();
        assert_eq!(
            created_files,
            vec!["AutoDetect.json".to_string()],
            "A execução da função deve resultar exclusivamente na criação de AutoDetect.json"
        );

        let raw_json = std::fs::read_to_string(&autodetect_path).expect("Falha ao ler AutoDetect.json gerado");
        let parsed: serde_json::Value = serde_json::from_str(&raw_json).expect("Falha ao parsear AutoDetect.json");

        // Regras de hardware definem o AutoDetect.json
        let expected_affinity = report.hardware.cpu.physical_cores > 4;
        let expected_mem_comp = report.hardware.ram.total_gb > 16.0;
        let expected_gpu_accel_flag = report.category != PcCategory::Potente;
        let gpu_lower = report.hardware.gpu.model.to_lowercase();
        let expected_render_queue_flag = gpu_lower.contains("nvidia")
            || gpu_lower.contains("geforce")
            || gpu_lower.contains("rtx")
            || gpu_lower.contains("gtx")
            || gpu_lower.contains("amd")
            || gpu_lower.contains("radeon")
            || (gpu_lower.is_empty() && tweaks::is_nvidia_smi_available());

        assert_eq!(config.empty_standby_list, true);
        assert_eq!(parsed["empty_standby_list"], true);
        assert_eq!(config.cs2_priority, false);
        assert_eq!(parsed["cs2_priority"], false);
        assert_eq!(config.cs2_affinity, expected_affinity);
        assert_eq!(parsed["cs2_affinity"], expected_affinity);
        assert_eq!(config.disable_memory_compression, expected_mem_comp);
        assert_eq!(parsed["disable_memory_compression"], expected_mem_comp);
        assert_eq!(config.disable_gpu_acceleration, expected_gpu_accel_flag);
        assert_eq!(parsed["disable_gpu_acceleration"], expected_gpu_accel_flag);
        assert_eq!(config.apply_render_queue, expected_render_queue_flag);
        assert_eq!(parsed["apply_render_queue"], expected_render_queue_flag);
        assert_eq!(config.apply_timer_resolution, true);
        assert_eq!(config.apply_dpi_optimization, true);
        assert_eq!(config.apply_mmcss, true);
        assert_eq!(config.disable_hpet, true);
        assert_eq!(config.disable_game_bar, true);
        assert_eq!(config.enable_msi, true);
        assert_eq!(config.disable_power_saving, true);

        // Relatório também reflete exatamente os verificadores atuais do SO
        let cs2_running = windows_api::is_cs2_process_running_win32().unwrap_or(false);
        let expected_priority_applied = cs2_running
            && matches!(tweaks::Cs2PriorityTweak.status(), Ok(tweaks::TweakStatus::Applied));
        let expected_gpu_accel_applied = tweaks::is_all_gpu_accel_disabled().unwrap_or(false);
        let expected_dpi_applied =
            matches!(tweaks::Cs2DpiOptimizationTweak.status(), Ok(tweaks::TweakStatus::Applied));
        let expected_mem_comp_applied =
            matches!(tweaks::MemoryCompressionTweak.status(), Ok(tweaks::TweakStatus::Applied));
        let expected_timer_applied =
            matches!(tweaks::TimerResolutionTweak.status(), Ok(tweaks::TweakStatus::Applied));
        let expected_mmcss_applied = tweaks::is_mmcss_applied_for_hardware_specs(
            report.hardware.cpu.physical_cores,
            report.hardware.ram.total_gb,
        )
        .unwrap_or(false);
        let expected_hpet_applied =
            matches!(tweaks::HpetTweak.status(), Ok(tweaks::TweakStatus::Applied));
        let expected_game_bar_applied =
            matches!(tweaks::GameBarTweak.status(), Ok(tweaks::TweakStatus::Applied));
        let expected_msi_applied = tweaks::is_all_msi_enabled().unwrap_or(false);
        let expected_power_saving_applied =
            matches!(tweaks::PcieUsbPowerSavingTweak.status(), Ok(tweaks::TweakStatus::Applied));
        let expected_render_queue_applied = expected_render_queue_flag
            && matches!(tweaks::RenderQueueTweak.status(), Ok(tweaks::TweakStatus::Applied));

        assert_eq!(report.is_already_applied("cs2_priority"), expected_priority_applied);
        assert_eq!(report.is_already_applied("disable_gpu_acceleration"), expected_gpu_accel_applied);
        assert_eq!(report.is_already_applied("apply_dpi_optimization"), expected_dpi_applied);
        assert_eq!(report.is_already_applied("disable_memory_compression"), expected_mem_comp_applied);
        assert_eq!(report.is_already_applied("apply_timer_resolution"), expected_timer_applied);
        assert_eq!(report.is_already_applied("apply_mmcss"), expected_mmcss_applied);
        assert_eq!(report.is_already_applied("disable_hpet"), expected_hpet_applied);
        assert_eq!(report.is_already_applied("disable_game_bar"), expected_game_bar_applied);
        assert_eq!(report.is_already_applied("enable_msi"), expected_msi_applied);
        assert_eq!(report.is_already_applied("disable_power_saving"), expected_power_saving_applied);
        assert_eq!(report.is_already_applied("apply_render_queue"), expected_render_queue_applied);

        assert_eq!(report.evaluations.len(), 13);
        for eval in &report.evaluations {
            assert_eq!(
                eval.needs_optimization,
                eval.active && !eval.already_applied,
                "Invariante needs_optimization falhou para '{}'",
                eval.id
            );
        }
    }

    #[test]
    fn test_resolve_autodetect_config_path_edge_cases() {
        let temp_dir = tempfile::tempdir().expect("Falha ao criar diretório temporário");
        let temp_file = temp_dir.path().join("CustomAuto.json");

        // Caso 1: None -> aponta para AutoDetect.json no diretório padrão de otimizações
        let default_path = resolve_autodetect_config_path(None);
        assert!(default_path.ends_with("AutoDetect.json"));

        // Caso 2: Path vazio -> fallback para AutoDetect.json
        let empty_path = resolve_autodetect_config_path(Some(Path::new("")));
        assert!(empty_path.ends_with("AutoDetect.json"));

        // Caso 3: Nome relativo sem extensão .json -> deve assegurar extensão .json
        let no_ext_path = resolve_autodetect_config_path(Some(Path::new("AutoDetect")));
        assert!(no_ext_path.ends_with("AutoDetect.json"));

        // Caso 4: Caminho absoluto -> preservado integralmente
        let abs_path = resolve_autodetect_config_path(Some(&temp_file));
        assert_eq!(abs_path, temp_file);

        // Caso 5: Passagem de diretório existente -> aponta para AutoDetect.json dentro do diretório
        let dir_path = resolve_autodetect_config_path(Some(temp_dir.path()));
        assert_eq!(dir_path, temp_dir.path().join("AutoDetect.json"));

        // Caso 6: String somente de espaços -> fallback para AutoDetect.json
        let whitespace_path = resolve_autodetect_config_path(Some(Path::new("   ")));
        assert!(whitespace_path.ends_with("AutoDetect.json"));
    }

    #[test]
    fn test_delete_optimization_profile() {
        let temp_dir = tempfile::tempdir().expect("Falha ao criar diretório temporário");

        // Criação de perfil inicial para teste
        let profile_name = "test_del_profile";
        let target_file = temp_dir.path().join(format!("{profile_name}.json"));
        let mut cfg = config::CustomOptimizationConfig::all_disabled();
        cfg.cs2_priority = true;
        cfg.save_to_file(&target_file).expect("Falha ao criar perfil de teste");
        assert!(target_file.exists());

        // Confirma que o perfil é listado
        let configs = list_optimization_configs_in_dir(temp_dir.path());
        assert_eq!(configs.len(), 1);

        // Exclui com o nome sem extensão
        let res = delete_optimization_profile_in_dir(temp_dir.path(), profile_name);
        assert!(res.is_ok(), "Exclusão de perfil existente deve retornar Ok");
        assert!(!target_file.exists(), "Arquivo deve ter sido removido do disco");

        let configs_after = list_optimization_configs_in_dir(temp_dir.path());
        assert!(configs_after.is_empty(), "Lista deve estar vazia após exclusão");

        // Tentativa de excluir arquivo que não existe mais -> deve retornar erro
        let res_not_found = delete_optimization_profile_in_dir(temp_dir.path(), profile_name);
        assert!(res_not_found.is_err(), "Exclusão de perfil inexistente deve falhar");

        // Teste de exclusão com nome que possui extensão .json explicitamente
        let second_profile = "profile_with_ext";
        let second_file = temp_dir.path().join(format!("{second_profile}.json"));
        cfg.save_to_file(&second_file).expect("Falha ao criar segundo perfil");
        assert!(second_file.exists());

        let res_with_ext = delete_optimization_profile_in_dir(temp_dir.path(), "profile_with_ext.json");
        assert!(res_with_ext.is_ok(), "Exclusão passando .json explicitamente deve funcionar");
        assert!(!second_file.exists());

        // Teste de proteção contra path traversal e nomes inválidos
        let traversal_res = delete_optimization_profile_in_dir(temp_dir.path(), "../hack");
        assert!(traversal_res.is_err(), "Nomes com path traversal devem ser rejeitados");

        let empty_res = delete_optimization_profile_in_dir(temp_dir.path(), "   ");
        assert!(empty_res.is_err(), "Nome vazio deve ser rejeitado");
    }

    #[test]
    fn test_resolve_app_settings_path_structure() {
        let path = resolve_app_settings_path();
        assert!(path.to_string_lossy().ends_with("settings.json"));
    }

    #[test]
    fn test_list_available_locales_in_dir() {
        let temp_dir = tempfile::tempdir().expect("Falha ao criar diretório temporário");
        let root = temp_dir.path();

        // Cria estrutura de mock locales: pt_BR, en, ru
        let pt_dir = root.join("pt_BR");
        std::fs::create_dir_all(&pt_dir).unwrap();
        std::fs::write(pt_dir.join("counterrush-ui.po"), "msgid \"\"\nmsgstr \"\"\n").unwrap();

        let en_dir = root.join("en");
        std::fs::create_dir_all(&en_dir).unwrap();
        std::fs::write(en_dir.join("counterrush-ui.po"), "msgid \"\"\nmsgstr \"\"\n").unwrap();

        let ru_dir = root.join("ru");
        std::fs::create_dir_all(&ru_dir).unwrap();
        std::fs::write(ru_dir.join("counterrush-ui.po"), "msgid \"\"\nmsgstr \"\"\n").unwrap();

        // Pasta sem arquivo .po não deve ser listada
        let dummy_dir = root.join("dummy");
        std::fs::create_dir_all(&dummy_dir).unwrap();

        let locales = list_available_locales_in_dir(root);
        assert_eq!(locales.len(), 3);
        assert_eq!(locales[0].code, "pt_BR");
        assert_eq!(locales[0].display_name, "Português (Brasil)");
        assert_eq!(locales[1].code, "en");
        assert_eq!(locales[1].display_name, "English");
        assert_eq!(locales[2].code, "ru");
        assert_eq!(locales[2].display_name, "Русский");
    }

    #[test]
    fn test_core_set_clipboard() {
        let res = set_clipboard("CounterRush Core Test Clipboard");
        assert!(res.is_ok(), "set_clipboard em core deve retornar Ok: {:?}", res.err());
    }

    #[test]
    fn test_optimize_running_cs2_respects_explicit_config_path() {
        let temp_dir = tempfile::tempdir().expect("Falha ao criar diretório temporário");
        let custom_path = temp_dir.path().join("MeuPerfilCustomizado.json");
        let cfg = config::CustomOptimizationConfig::all_disabled();
        cfg.save_to_file(&custom_path)
            .expect("Falha ao salvar perfil customizado em disco");

        let core = OptimizerCore::default();
        let results = core
            .optimize_running_cs2(None, Some(&custom_path))
            .expect("optimize_running_cs2 deve carregar o arquivo customizado sem falhar");

        assert!(
            results.is_empty(),
            "Com todas as flags desativadas no perfil explicitamente passado, nenhum tweak deve rodar: {:?}",
            results
        );
    }

    #[test]
    fn test_reconcile_startup_flags_truth_table() {
        assert_eq!(reconcile_startup_flags(false, false), (false, false));
        assert_eq!(
            reconcile_startup_flags(true, false),
            (true, false),
            "Ativar inicialização com o sistema não deve ativar auto-otimização Steam"
        );
        assert_eq!(reconcile_startup_flags(true, true), (true, true));
        assert_eq!(
            reconcile_startup_flags(false, true),
            (false, false),
            "Sem inicialização com o sistema, auto-otimização Steam deve ficar desativada"
        );
    }

    #[test]
    fn test_resolve_selected_preset_name() {
        let presets = ["AutoDetect", "Competitivo", "Streaming"];
        assert_eq!(
            resolve_selected_preset_name(Some("Competitivo"), &presets),
            Some("Competitivo")
        );
        assert_eq!(
            resolve_selected_preset_name(Some("Inexistente"), &presets),
            Some("AutoDetect")
        );
        assert_eq!(resolve_selected_preset_name(None, &presets), Some("AutoDetect"));
        assert_eq!(resolve_selected_preset_name(Some("Competitivo"), &[]), None);
    }

    #[test]
    fn test_core_single_instance_facade_blocks_second_and_notifies() {
        let test_id = format!("CounterRush_Core_Test_{}", std::process::id());
        assert!(!try_notify_running_instance_with_id(&test_id));

        let mut guard = match acquire_single_instance_with_id(&test_id)
            .expect("Aquisição primária via core deve funcionar")
        {
            SingleInstanceOutcome::Primary(g) => g,
            SingleInstanceOutcome::SecondaryNotified => panic!("Não deveria existir instância prévia"),
        };

        let notified = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let notified_cb = std::sync::Arc::clone(&notified);
        guard
            .start_activation_listener(move || {
                notified_cb.store(true, std::sync::atomic::Ordering::SeqCst);
            })
            .expect("Listener deve iniciar");

        let outcome = acquire_single_instance_with_id(&test_id)
            .expect("Segunda chamada deve notificar instância existente");
        assert!(matches!(outcome, SingleInstanceOutcome::SecondaryNotified));

        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(2);
        while !notified.load(std::sync::atomic::Ordering::SeqCst)
            && std::time::Instant::now() < deadline
        {
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert!(
            notified.load(std::sync::atomic::Ordering::SeqCst),
            "Segunda chamada deve despertar o listener da instância primária"
        );
    }

    #[test]
    fn test_resolve_paths_ignore_non_workspace_cwd_and_start_empty() {
        let exe_temp = tempfile::tempdir().expect("Falha ao criar pasta exe temporária");
        let sys_cwd = tempfile::tempdir().expect("Falha ao criar pasta CWD tipo System32");

        // Simula diretório C:\Windows\System32\config existente sem Cargo.toml na raiz do CWD
        std::fs::create_dir_all(sys_cwd.path().join("config").join("optimzations")).unwrap();
        std::fs::write(
            sys_cwd.path().join("config").join("settings.json"),
            r#"{"active_preset":"Invasivo"}"#,
        )
        .unwrap();
        std::fs::write(
            sys_cwd.path().join("config").join("optimzations").join("AutoDetect.json"),
            "{}",
        )
        .unwrap();

        let opt_dir = resolve_optimizations_dir_with(Some(exe_temp.path()), sys_cwd.path());
        assert_eq!(opt_dir, exe_temp.path().join("config").join("optimzations"));
        assert!(
            list_optimization_configs_in_dir(&opt_dir).is_empty(),
            "Instalação limpa não deve listar perfis pré-existentes"
        );

        let settings = resolve_app_settings_path_with(Some(exe_temp.path()), sys_cwd.path());
        assert_eq!(settings, exe_temp.path().join("config").join("settings.json"));

        let hw = resolve_hardware_config_path_with(None, Some(exe_temp.path()), sys_cwd.path());
        assert_eq!(
            hw,
            exe_temp.path().join("config").join("hardware").join("Hardware.json")
        );
    }

    #[test]
    fn test_resolve_paths_use_workspace_when_cargo_toml_exists() {
        let exe_temp = tempfile::tempdir().expect("Falha ao criar pasta exe temporária");
        let ws_cwd = tempfile::tempdir().expect("Falha ao criar pasta workspace");

        std::fs::write(ws_cwd.path().join("Cargo.toml"), "[workspace]\n").unwrap();
        std::fs::create_dir_all(ws_cwd.path().join("config").join("optimzations")).unwrap();
        std::fs::create_dir_all(ws_cwd.path().join("config").join("hardware")).unwrap();
        std::fs::write(ws_cwd.path().join("config").join("settings.json"), "{}").unwrap();

        assert_eq!(
            resolve_optimizations_dir_with(Some(exe_temp.path()), ws_cwd.path()),
            ws_cwd.path().join("config").join("optimzations")
        );
        assert_eq!(
            resolve_app_settings_path_with(Some(exe_temp.path()), ws_cwd.path()),
            ws_cwd.path().join("config").join("settings.json")
        );
        assert_eq!(
            resolve_hardware_config_path_with(None, Some(exe_temp.path()), ws_cwd.path()),
            ws_cwd.path().join("config").join("hardware").join("Hardware.json")
        );
    }
}
