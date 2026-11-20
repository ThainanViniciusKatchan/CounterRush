#![windows_subsystem = "windows"]

slint::include_modules!();

mod binds;
mod tray;

fn scan_presets() -> Vec<PresetModel> {
    let files = counterrush_core::list_optimization_configs();
    let mut list = Vec::new();
    for p in files {
        if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
            // Identificador do tipo de perfil: o texto exibido no botão vem do
            // catálogo de traduções (@tr) em main.slint.
            let tag = match stem {
                "AutoDetect" => "auto",
                "custom_optimization" => "custom",
                _ => "user",
            };
            list.push(PresetModel {
                name: stem.into(),
                tag: tag.into(),
            });
        }
    }
    list
}

/// Seleção de tweaks independente da UI, usada para trafegar dados
/// entre a thread da janela e a thread executora de comandos.
#[derive(Clone, Copy, Default)]
struct Tweaks {
    cpu: bool,
    timer: bool,
    priority: bool,
    ram: bool,
    gpu: bool,
    dpi: bool,
    mem: bool,
    mmcss: bool,
    hpet: bool,
    game_bar: bool,
    msi: bool,
    power: bool,
    render_queue: bool,
}

impl Tweaks {
    fn from_config(config: &counterrush_core::config::CustomOptimizationConfig) -> Self {
        Self {
            cpu: config.cs2_affinity,
            timer: config.apply_timer_resolution,
            priority: config.cs2_priority,
            ram: config.empty_standby_list,
            gpu: config.disable_gpu_acceleration,
            dpi: config.apply_dpi_optimization,
            mem: config.disable_memory_compression,
            mmcss: config.apply_mmcss,
            hpet: config.disable_hpet,
            game_bar: config.disable_game_bar,
            msi: config.enable_msi,
            power: config.disable_power_saving,
            render_queue: config.apply_render_queue,
        }
    }

    fn apply_to_config(&self, config: &mut counterrush_core::config::CustomOptimizationConfig) {
        config.cs2_affinity = self.cpu;
        config.apply_timer_resolution = self.timer;
        config.cs2_priority = self.priority;
        config.empty_standby_list = self.ram;
        config.disable_gpu_acceleration = self.gpu;
        config.apply_dpi_optimization = self.dpi;
        config.disable_memory_compression = self.mem;
        config.apply_mmcss = self.mmcss;
        config.disable_hpet = self.hpet;
        config.disable_game_bar = self.game_bar;
        config.enable_msi = self.msi;
        config.disable_power_saving = self.power;
        config.apply_render_queue = self.render_queue;
    }

    fn apply_to_ui(&self, ui: &MainWindow) {
        ui.set_cpu_tweak_active(self.cpu);
        ui.set_timer_tweak_active(self.timer);
        ui.set_priority_tweak_active(self.priority);
        ui.set_ram_tweak_active(self.ram);
        ui.set_gpu_accel_tweak_active(self.gpu);
        ui.set_dpi_tweak_active(self.dpi);
        ui.set_mem_compression_tweak_active(self.mem);
        ui.set_mmcss_tweak_active(self.mmcss);
        ui.set_hpet_tweak_active(self.hpet);
        ui.set_game_bar_tweak_active(self.game_bar);
        ui.set_msi_tweak_active(self.msi);
        ui.set_power_saving_tweak_active(self.power);
        ui.set_render_queue_tweak_active(self.render_queue);
    }
}

fn neutral_status_message(lang: &str) -> &'static str {
    match lang {
        "en" | "en_US" | "en-US" | "en-EUA" => {
            "No active optimization profile. Create or detect a profile to begin."
        }
        "ru" | "ru_RU" | "ru-RU" => {
            "Нет активного профиля оптимизации. Создайте или определите профиль для начала."
        }
        _ => "Nenhum perfil de otimização ativo. Crie ou detecte um perfil para começar.",
    }
}

fn ready_status_message(lang: &str) -> &'static str {
    match lang {
        "en" | "en_US" | "en-US" | "en-EUA" => {
            "Ready to launch CS2 with 0.5ms polling rate and optimized CPU affinity."
        }
        "ru" | "ru_RU" | "ru-RU" => {
            "Готово к запуску CS2 с частотой опроса 0.5 мс и оптимизированной привязкой ЦП."
        }
        _ => "Pronto para iniciar o CS2 com taxa de polling de 0.5ms e afinidade de CPU otimizada.",
    }
}

fn profile_required_status_message(lang: &str) -> &'static str {
    match lang {
        "en" | "en_US" | "en-US" | "en-EUA" => {
            "Create or select a profile to enable optimizations."
        }
        "ru" | "ru_RU" | "ru-RU" => {
            "Создайте или выберите профиль для включения оптимизаций."
        }
        _ => "Crie ou selecione um perfil para ativar otimizações.",
    }
}

fn enforce_neutral_no_profile_state(ui: &MainWindow, status_msg: &str) {
    Tweaks::default().apply_to_ui(ui);
    ui.set_status_message(status_msg.into());
}

fn reject_tweak_without_profile(ui: &slint::Weak<MainWindow>) {
    post_ui(ui, |ui| {
        let lang = ui.get_active_language();
        enforce_neutral_no_profile_state(ui, profile_required_status_message(lang.as_str()));
    });
}

/// Comandos enfileirados pela thread da janela e executados pela thread executora.
enum Command {
    TweakToggled { name: String, active: bool },
    BulkTweaksAction { action: String },
    PresetSelected { name: String },
    CreateProfile { name: String, launch_args: String, tweaks: Tweaks },
    SaveLaunchArgs { args: String },
    DeleteProfile { name: String, active: String },
}

fn post_ui<F: FnOnce(&MainWindow) + Send + 'static>(ui: &slint::Weak<MainWindow>, update: F) {
    let handle = ui.clone();
    let _ = slint::invoke_from_event_loop(move || {
        if let Some(ui) = handle.upgrade() {
            update(&ui);
        }
    });
}

fn spawn_command_worker(
    ui: slint::Weak<MainWindow>,
    config_path: std::sync::Arc<std::sync::RwLock<std::path::PathBuf>>,
) -> std::sync::mpsc::Sender<Command> {
    let (sender, receiver) = std::sync::mpsc::channel::<Command>();
    std::thread::spawn(move || {
        while let Ok(command) = receiver.recv() {
            execute_command(command, &ui, &config_path);
        }
    });
    sender
}

fn execute_command(
    command: Command,
    ui: &slint::Weak<MainWindow>,
    config_path: &std::sync::Arc<std::sync::RwLock<std::path::PathBuf>>,
) {
    match command {
        Command::TweakToggled { name, active } => apply_tweak(ui, config_path, name, active),
        Command::BulkTweaksAction { action } => apply_bulk_action(ui, config_path, action),
        Command::PresetSelected { name } => select_preset(ui, config_path, name),
        Command::CreateProfile {
            name,
            launch_args,
            tweaks,
        } => create_profile(ui, config_path, name, launch_args, tweaks),
        Command::SaveLaunchArgs { args } => save_launch_args(ui, config_path, args),
        Command::DeleteProfile { name, active } => delete_profile(ui, config_path, name, active),
    }
}

fn apply_tweak(
    ui: &slint::Weak<MainWindow>,
    config_path: &std::sync::Arc<std::sync::RwLock<std::path::PathBuf>>,
    tweak_name: String,
    active: bool,
) {
    let target_path = config_path.read().map(|p| p.clone()).unwrap_or_default();
    if target_path.as_os_str().is_empty() {
        reject_tweak_without_profile(ui);
        return;
    }

    let display_name = match tweak_name.as_str() {
        "cpu_core_0" => "Afinidade de CPU (Núcleo 0)",
        "timer_resolution" => "Temporizador (0.5ms)",
        "process_priority" => "Prioridade de Processo",
        "empty_standby" => "Limpeza de RAM (Standby)",
        "gpu_acceleration" => "Aceleração de GPU em Apps",
        "dpi_optimization" => "Otimização de DPI e Janela",
        "memory_compression" => "Compactação de Memória",
        "mmcss" => "Agendador MMCSS",
        "hpet" => "Temporizador HPET",
        "game_bar" => "Xbox Game Bar e DVR",
        "msi" => "Modo de Interrupção MSI",
        "power_saving" => "Economia de Energia PCIe/USB",
        "render_queue" => "Fila de Renderização",
        _ => tweak_name.as_str(),
    };
    let state = if active { "ativado" } else { "desativado" };
    let msg = format!("Módulo '{}' {}.", display_name, state);

    let mut config = counterrush_core::config::CustomOptimizationConfig::load_from_file(&target_path)
        .unwrap_or_default();

    match tweak_name.as_str() {
        "cpu_core_0" => config.cs2_affinity = active,
        "timer_resolution" => config.apply_timer_resolution = active,
        "process_priority" => config.cs2_priority = active,
        "empty_standby" => config.empty_standby_list = active,
        "gpu_acceleration" => config.disable_gpu_acceleration = active,
        "dpi_optimization" => config.apply_dpi_optimization = active,
        "memory_compression" => config.disable_memory_compression = active,
        "mmcss" => config.apply_mmcss = active,
        "hpet" => config.disable_hpet = active,
        "game_bar" => config.disable_game_bar = active,
        "msi" => config.enable_msi = active,
        "power_saving" => config.disable_power_saving = active,
        "render_queue" => config.apply_render_queue = active,
        _ => {}
    }
    let _ = config.save_to_file(&target_path);

    post_ui(ui, move |ui| ui.set_status_message(msg.into()));
}

fn apply_bulk_action(
    ui: &slint::Weak<MainWindow>,
    config_path: &std::sync::Arc<std::sync::RwLock<std::path::PathBuf>>,
    action: String,
) {
    let target_path = config_path.read().map(|p| p.clone()).unwrap_or_default();
    if target_path.as_os_str().is_empty() {
        reject_tweak_without_profile(ui);
        return;
    }

    let mut config = counterrush_core::config::CustomOptimizationConfig::load_from_file(&target_path)
        .unwrap_or_default();

    match action.as_str() {
        "activate_all" => Tweaks {
            cpu: true,
            timer: true,
            priority: true,
            ram: true,
            gpu: true,
            dpi: true,
            mem: true,
            mmcss: true,
            hpet: true,
            game_bar: true,
            msi: true,
            power: true,
            render_queue: true,
        }
        .apply_to_config(&mut config),
        "deactivate_all" => Tweaks::default().apply_to_config(&mut config),
        "reset_defaults" => config = Default::default(),
        _ => {}
    }
    let _ = config.save_to_file(&target_path);
}

#[derive(Debug, Clone, Copy)]
enum AppliedSessionTweak {
    Timer,
    Mmcss,
    Dpi,
    GpuAccel,
    MemCompression,
    Hpet,
    GameBar,
    Msi,
    PowerSaving,
    RenderQueue,
}

fn rollback_applied_tweaks(
    core: &counterrush_core::OptimizerCore,
    applied: Vec<AppliedSessionTweak>,
) {
    for tweak in applied.into_iter().rev() {
        match tweak {
            AppliedSessionTweak::Timer => {
                let _ = core.rollback_timer_all();
            }
            AppliedSessionTweak::Mmcss => {
                let _ = core.rollback_all_mmcss();
            }
            AppliedSessionTweak::Dpi => {
                let _ = core.rollback_dpi_optimization();
            }
            AppliedSessionTweak::GpuAccel => {
                let _ = core.rollback_all_gpu_acceleration();
            }
            AppliedSessionTweak::MemCompression => {
                let _ = core.rollback_memory_compression();
            }
            AppliedSessionTweak::Hpet => {
                let _ = core.rollback_hpet();
            }
            AppliedSessionTweak::GameBar => {
                let _ = core.rollback_all_game_bar_features();
            }
            AppliedSessionTweak::Msi => {
                let _ = core.rollback_all_msi();
            }
            AppliedSessionTweak::PowerSaving => {
                let _ = core.rollback_all_power_saving();
            }
            AppliedSessionTweak::RenderQueue => {
                let _ = core.rollback_render_queue();
            }
        }
    }
}

fn sleep_with_cancellation(
    duration: std::time::Duration,
    token: &std::sync::atomic::AtomicBool,
) -> bool {
    let chunk = std::time::Duration::from_millis(25);
    let mut elapsed = std::time::Duration::ZERO;
    while elapsed < duration {
        if token.load(std::sync::atomic::Ordering::SeqCst) {
            return false;
        }
        let to_sleep = chunk.min(duration - elapsed);
        std::thread::sleep(to_sleep);
        elapsed += to_sleep;
    }
    !token.load(std::sync::atomic::Ordering::SeqCst)
}

fn update_auto_optimize_ui(
    ui: &slint::Weak<MainWindow>,
    progress: f32,
    step_name: &str,
) {
    let name_string = step_name.to_string();
    post_ui(ui, move |ui| {
        ui.set_auto_optimize_progress(progress);
        ui.set_auto_optimize_step_name(name_string.into());
    });
}

struct AutoTweakStep {
    progress: f32,
    tweak_id: &'static str,
    permanent_kind: Option<counterrush_core::PermanentTweakKind>,
    enabled_in_config: bool,
    applying_label: &'static str,
    already_label: &'static str,
    skipped_label: &'static str,
    session_tweak: Option<AppliedSessionTweak>,
    apply_fn: fn(&counterrush_core::OptimizerCore) -> bool,
}

struct ConfiguredTweakStep {
    enabled_in_config: bool,
    progress: f32,
    applying_label: &'static str,
    already_label: &'static str,
    permanent_kind: Option<counterrush_core::PermanentTweakKind>,
    session_tweak: Option<AppliedSessionTweak>,
    apply_fn: fn(&counterrush_core::OptimizerCore) -> bool,
}

fn check_auto_optimize_cancelled(
    ui: &slint::Weak<MainWindow>,
    token: &std::sync::atomic::AtomicBool,
    core: &counterrush_core::OptimizerCore,
    applied_tweaks: &mut Vec<AppliedSessionTweak>,
    sleep_duration: Option<std::time::Duration>,
    reverted_msg: &str,
) -> bool {
    let cancelled = match sleep_duration {
        Some(dur) => !sleep_with_cancellation(dur, token),
        None => token.load(std::sync::atomic::Ordering::SeqCst),
    };
    if !cancelled {
        return false;
    }

    let to_revert = std::mem::take(applied_tweaks);
    rollback_applied_tweaks(core, to_revert);
    let msg = reverted_msg.to_string();
    post_ui(ui, move |ui| {
        ui.set_show_auto_optimize_modal(false);
        ui.set_status_message(msg.into());
    });
    true
}

fn run_auto_optimize_flow(
    ui: &slint::Weak<MainWindow>,
    config_path: &std::sync::Arc<std::sync::RwLock<std::path::PathBuf>>,
    token: std::sync::Arc<std::sync::atomic::AtomicBool>,
) {
    let step_duration = std::time::Duration::from_millis(200);
    let core = counterrush_core::OptimizerCore::default();
    let mut applied_tweaks = Vec::new();
    let mut applied_count = 0usize;
    let mut already_count = 0usize;

    // Etapa 0: Detecção de Hardware
    if token.load(std::sync::atomic::Ordering::SeqCst) {
        return;
    }
    update_auto_optimize_ui(ui, 0.08, "Detectando hardware e configurações do sistema...");
    if check_auto_optimize_cancelled(
        ui,
        &token,
        &core,
        &mut applied_tweaks,
        Some(step_duration),
        "Calibração automática cancelada pelo usuário.",
    ) {
        return;
    }

    // Etapa 1: Avaliação Combinada (Hardware + Status atual do SO)
    update_auto_optimize_ui(ui, 0.16, "Analisando hardware e otimizações existentes...");
    let (mut config, report) = match core.generate_autodetect_config() {
        Ok(res) => res,
        Err(e) => {
            post_ui(ui, move |ui| {
                ui.set_show_auto_optimize_modal(false);
                ui.set_status_message(format!("Erro ao gerar perfil de otimização: {e}").into());
            });
            return;
        }
    };
    if check_auto_optimize_cancelled(
        ui,
        &token,
        &core,
        &mut applied_tweaks,
        Some(step_duration),
        "Calibração automática cancelada pelo usuário.",
    ) {
        return;
    }

    let steps = [
        AutoTweakStep {
            progress: 0.23,
            tweak_id: "apply_timer_resolution",
            permanent_kind: None,
            enabled_in_config: config.apply_timer_resolution,
            applying_label: "Otimizando resolução do temporizador (0.5ms)...",
            already_label: "Resolução do temporizador (0.5ms) já configurada — preservada.",
            skipped_label: "Resolução do temporizador mantida padrão.",
            session_tweak: Some(AppliedSessionTweak::Timer),
            apply_fn: |c| c.apply_timer_resolution().is_ok(),
        },
        AutoTweakStep {
            progress: 0.31,
            tweak_id: "apply_mmcss",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::ApplyMmcss),
            enabled_in_config: config.apply_mmcss,
            applying_label: "Configurando agendador multimídia e rede (MMCSS)...",
            already_label: "Agendador multimídia (MMCSS) já otimizado — preservado.",
            skipped_label: "Agendador multimídia (MMCSS) ignorado.",
            session_tweak: Some(AppliedSessionTweak::Mmcss),
            apply_fn: |c| c.apply_all_mmcss().is_ok(),
        },
        AutoTweakStep {
            progress: 0.39,
            tweak_id: "apply_dpi_optimization",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::ApplyDpiOptimization),
            enabled_in_config: config.apply_dpi_optimization,
            applying_label: "Ajustando DPI e compatibilidade de tela inteira CS2...",
            already_label: "Otimização de DPI já configurada — preservada.",
            skipped_label: "Otimização de DPI ignorada.",
            session_tweak: Some(AppliedSessionTweak::Dpi),
            apply_fn: |c| c.apply_dpi_optimization().is_ok(),
        },
        AutoTweakStep {
            progress: 0.47,
            tweak_id: "disable_gpu_acceleration",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::DisableGpuAcceleration),
            enabled_in_config: config.disable_gpu_acceleration,
            applying_label: "Desativando aceleração de GPU em aplicativos secundários...",
            already_label: "Aceleração de GPU em apps já desativada — preservada.",
            skipped_label: "Aceleração de GPU em apps mantida para perfil Potente.",
            session_tweak: Some(AppliedSessionTweak::GpuAccel),
            apply_fn: |c| c.disable_all_gpu_acceleration().is_ok(),
        },
        AutoTweakStep {
            progress: 0.55,
            tweak_id: "disable_memory_compression",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::DisableMemoryCompression),
            enabled_in_config: config.disable_memory_compression,
            applying_label: "Desativando compactação de memória do Windows...",
            already_label: "Compactação de memória já desativada — preservada.",
            skipped_label: "Compactação de memória mantida (RAM <= 16 GB).",
            session_tweak: Some(AppliedSessionTweak::MemCompression),
            apply_fn: |c| c.disable_memory_compression().is_ok(),
        },
        AutoTweakStep {
            progress: 0.62,
            tweak_id: "empty_standby_list",
            permanent_kind: None,
            enabled_in_config: config.empty_standby_list,
            applying_label: "Liberando cache de memória RAM em Standby...",
            already_label: "Standby List já verificada.",
            skipped_label: "Limpeza da Standby List ignorada.",
            session_tweak: None,
            apply_fn: |c| c.empty_standby_list().is_ok(),
        },
        AutoTweakStep {
            progress: 0.69,
            tweak_id: "disable_hpet",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::DisableHpet),
            enabled_in_config: config.disable_hpet,
            applying_label: "Desativando HPET no BCD do Windows...",
            already_label: "HPET já desativado no sistema — preservado.",
            skipped_label: "Configuração HPET mantida.",
            session_tweak: Some(AppliedSessionTweak::Hpet),
            apply_fn: |c| c.disable_hpet().is_ok(),
        },
        AutoTweakStep {
            progress: 0.76,
            tweak_id: "disable_game_bar",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::DisableGameBar),
            enabled_in_config: config.disable_game_bar,
            applying_label: "Desativando Xbox Game Bar e capturas GameDVR...",
            already_label: "Xbox Game Bar / GameDVR já desativados — preservados.",
            skipped_label: "Xbox Game Bar mantido.",
            session_tweak: Some(AppliedSessionTweak::GameBar),
            apply_fn: |c| c.disable_all_game_bar_features().is_ok(),
        },
        AutoTweakStep {
            progress: 0.83,
            tweak_id: "enable_msi",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::EnableMsi),
            enabled_in_config: config.enable_msi,
            applying_label: "Ativando Modo MSI para GPU e periféricos...",
            already_label: "Modo MSI já habilitado nos dispositivos — preservado.",
            skipped_label: "Modo MSI ignorado.",
            session_tweak: Some(AppliedSessionTweak::Msi),
            apply_fn: |c| c.enable_all_msi().is_ok(),
        },
        AutoTweakStep {
            progress: 0.90,
            tweak_id: "disable_power_saving",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::DisablePowerSaving),
            enabled_in_config: config.disable_power_saving,
            applying_label: "Desativando suspensão de energia PCIe e USB...",
            already_label: "Economia de energia PCIe/USB já desativada — preservada.",
            skipped_label: "Economia de energia PCIe/USB mantida.",
            session_tweak: Some(AppliedSessionTweak::PowerSaving),
            apply_fn: |c| c.disable_all_power_saving().is_ok(),
        },
        AutoTweakStep {
            progress: 0.97,
            tweak_id: "apply_render_queue",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::ApplyRenderQueue),
            enabled_in_config: config.apply_render_queue,
            applying_label: "Configurando fila de renderização (Render Queue)...",
            already_label: "Fila de renderização já configurada — preservada.",
            skipped_label: "Fila de renderização não aplicável ao adaptador atual.",
            session_tweak: Some(AppliedSessionTweak::RenderQueue),
            apply_fn: |c| c.apply_render_queue().is_ok(),
        },
    ];

    let cancel_reverted_msg = "Calibração automática cancelada pelo usuário. Alterações revertidas.";
    for step in steps {
        if check_auto_optimize_cancelled(
            ui,
            &token,
            &core,
            &mut applied_tweaks,
            None,
            cancel_reverted_msg,
        ) {
            return;
        }

        if report.is_already_applied(step.tweak_id) {
            update_auto_optimize_ui(ui, step.progress, step.already_label);
            already_count += 1;
        } else if !step.enabled_in_config {
            update_auto_optimize_ui(ui, step.progress, step.skipped_label);
        } else {
            update_auto_optimize_ui(ui, step.progress, step.applying_label);
            if (step.apply_fn)(&core) {
                applied_count += 1;
                if let Some(session_tweak) = step.session_tweak {
                    applied_tweaks.push(session_tweak);
                }
                if let Some(kind) = step.permanent_kind {
                    kind.set_enabled_in(&mut config, false);
                }
            }
        }

        if check_auto_optimize_cancelled(
            ui,
            &token,
            &core,
            &mut applied_tweaks,
            Some(step_duration),
            cancel_reverted_msg,
        ) {
            return;
        }
    }

    // Etapa Final: Gravação de AutoDetect.json e atualização da UI
    if check_auto_optimize_cancelled(
        ui,
        &token,
        &core,
        &mut applied_tweaks,
        None,
        cancel_reverted_msg,
    ) {
        return;
    }

    let autodetect_path = counterrush_core::resolve_autodetect_config_path(None);
    if let Err(e) = config.save_to_file(&autodetect_path) {
        post_ui(ui, move |ui| {
            ui.set_show_auto_optimize_modal(false);
            ui.set_status_message(format!("Erro ao salvar AutoDetect.json: {e}").into());
        });
        return;
    }

    if let Ok(mut current) = config_path.write() {
        *current = autodetect_path.clone();
    }

    let settings_path = counterrush_core::resolve_app_settings_path();
    if let Ok(mut settings) = counterrush_core::config::AppSettings::load_or_create(&settings_path) {
        settings.initial_optimization_done = true;
        settings.active_preset = Some("AutoDetect".to_string());
        let _ = settings.save_to_file(&settings_path);
    }

    let updated_presets = scan_presets();
    let args_text = config.launch_args.join(" ");
    let tweaks = Tweaks::from_config(&config);
    let msg = if report.blocked_tweaks.is_empty() {
        format!(
            "Hardware detectado ({:?} • {}). AutoDetect salvo ({applied_count} aplicadas agora, {already_count} já estavam otimizadas).",
            report.category, report.hardware.gpu.model
        )
    } else {
        format!(
            "Hardware detectado ({:?} • {}). AutoDetect salvo ({applied_count} aplicadas agora, {already_count} já otimizadas, {} não aplicáveis ao hardware).",
            report.category,
            report.hardware.gpu.model,
            report.blocked_tweaks.len()
        )
    };

    post_ui(ui, move |ui| {
        ui.set_presets(slint::ModelRc::new(slint::VecModel::from(updated_presets)));
        ui.set_active_preset("AutoDetect".into());
        ui.set_launch_args_text(args_text.into());
        ui.set_is_editing_launch_args(false);
        tweaks.apply_to_ui(ui);
        ui.set_status_message(msg.into());
        ui.set_auto_optimize_progress(1.0);
        ui.set_auto_optimize_step_name("Sistema otimizado com sucesso!".into());
        ui.set_auto_optimize_finished(true);
    });
}

fn save_active_preset_setting(preset_name: Option<String>) {
    let settings_path = counterrush_core::resolve_app_settings_path();
    if let Ok(mut settings) = counterrush_core::config::AppSettings::load_or_create(&settings_path) {
        settings.active_preset = preset_name;
        let _ = settings.save_to_file(&settings_path);
    }
}

fn execute_configured_tweaks_with_modal(
    ui: &slint::Weak<MainWindow>,
    core: &counterrush_core::OptimizerCore,
    config: &mut counterrush_core::config::CustomOptimizationConfig,
    token: &std::sync::atomic::AtomicBool,
    step_duration: std::time::Duration,
    cancel_msg: &str,
) -> Option<(Vec<AppliedSessionTweak>, bool)> {
    let mut applied_tweaks = Vec::new();
    let mut config_dirty = false;
    let cancel_and_rollback =
        |applied: Vec<AppliedSessionTweak>| -> Option<(Vec<AppliedSessionTweak>, bool)> {
            rollback_applied_tweaks(core, applied);
            let msg = cancel_msg.to_string();
            post_ui(ui, move |ui| {
                ui.set_show_auto_optimize_modal(false);
                ui.set_is_launching(false);
                ui.set_status_message(msg.into());
            });
            None
        };

    if token.load(std::sync::atomic::Ordering::SeqCst) {
        return cancel_and_rollback(applied_tweaks);
    }
    update_auto_optimize_ui(ui, 0.08, "Verificando perfil e compatibilidade do hardware...");
    if !sleep_with_cancellation(step_duration, token) {
        return cancel_and_rollback(applied_tweaks);
    }

    let steps = [
        ConfiguredTweakStep {
            enabled_in_config: config.empty_standby_list,
            progress: 0.16,
            applying_label: "Liberando cache de memória RAM em Standby...",
            already_label: "",
            permanent_kind: None,
            session_tweak: None,
            apply_fn: |c| c.empty_standby_list().is_ok(),
        },
        ConfiguredTweakStep {
            enabled_in_config: config.apply_timer_resolution,
            progress: 0.24,
            applying_label: "Otimizando resolução do temporizador (0.5ms)...",
            already_label: "",
            permanent_kind: None,
            session_tweak: Some(AppliedSessionTweak::Timer),
            apply_fn: |c| c.apply_timer_resolution().is_ok(),
        },
        ConfiguredTweakStep {
            enabled_in_config: config.apply_mmcss,
            progress: 0.32,
            applying_label: "Configurando agendador multimídia e rede (MMCSS)...",
            already_label: "Agendador multimídia (MMCSS) já otimizado — preservado.",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::ApplyMmcss),
            session_tweak: Some(AppliedSessionTweak::Mmcss),
            apply_fn: |c| c.apply_all_mmcss().is_ok(),
        },
        ConfiguredTweakStep {
            enabled_in_config: config.apply_dpi_optimization,
            progress: 0.40,
            applying_label: "Ajustando DPI e compatibilidade de tela inteira CS2...",
            already_label: "Otimização de DPI já configurada — preservada.",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::ApplyDpiOptimization),
            session_tweak: Some(AppliedSessionTweak::Dpi),
            apply_fn: |c| c.apply_dpi_optimization().is_ok(),
        },
        ConfiguredTweakStep {
            enabled_in_config: config.disable_gpu_acceleration,
            progress: 0.48,
            applying_label: "Desativando aceleração de GPU em aplicativos secundários...",
            already_label: "Aceleração de GPU em apps já desativada — preservada.",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::DisableGpuAcceleration),
            session_tweak: Some(AppliedSessionTweak::GpuAccel),
            apply_fn: |c| c.disable_all_gpu_acceleration().is_ok(),
        },
        ConfiguredTweakStep {
            enabled_in_config: config.disable_memory_compression,
            progress: 0.56,
            applying_label: "Calibrando compactação de memória do Windows...",
            already_label: "Compactação de memória já desativada — preservada.",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::DisableMemoryCompression),
            session_tweak: Some(AppliedSessionTweak::MemCompression),
            apply_fn: |c| c.disable_memory_compression().is_ok(),
        },
        ConfiguredTweakStep {
            enabled_in_config: config.disable_hpet,
            progress: 0.64,
            applying_label: "Desativando HPET (temporizador de alta precisão)...",
            already_label: "HPET já desativado no sistema — preservado.",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::DisableHpet),
            session_tweak: Some(AppliedSessionTweak::Hpet),
            apply_fn: |c| c.disable_hpet().is_ok(),
        },
        ConfiguredTweakStep {
            enabled_in_config: config.disable_game_bar,
            progress: 0.72,
            applying_label: "Desativando Xbox Game Bar e capturas GameDVR...",
            already_label: "Xbox Game Bar / GameDVR já desativados — preservados.",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::DisableGameBar),
            session_tweak: Some(AppliedSessionTweak::GameBar),
            apply_fn: |c| c.disable_all_game_bar_features().is_ok(),
        },
        ConfiguredTweakStep {
            enabled_in_config: config.enable_msi,
            progress: 0.80,
            applying_label: "Ativando Modo MSI para GPU e periféricos...",
            already_label: "Modo MSI já habilitado nos dispositivos — preservado.",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::EnableMsi),
            session_tweak: Some(AppliedSessionTweak::Msi),
            apply_fn: |c| c.enable_all_msi().is_ok(),
        },
        ConfiguredTweakStep {
            enabled_in_config: config.disable_power_saving,
            progress: 0.86,
            applying_label: "Desativando suspensão de energia PCIe e USB...",
            already_label: "Economia de energia PCIe/USB já desativada — preservada.",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::DisablePowerSaving),
            session_tweak: Some(AppliedSessionTweak::PowerSaving),
            apply_fn: |c| c.disable_all_power_saving().is_ok(),
        },
        ConfiguredTweakStep {
            enabled_in_config: config.apply_render_queue,
            progress: 0.92,
            applying_label: "Configurando fila de renderização (Render Queue)...",
            already_label: "Fila de renderização já configurada — preservada.",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::ApplyRenderQueue),
            session_tweak: Some(AppliedSessionTweak::RenderQueue),
            apply_fn: |c| c.apply_render_queue().is_ok(),
        },
    ];

    for step in steps {
        if token.load(std::sync::atomic::Ordering::SeqCst) {
            return cancel_and_rollback(applied_tweaks);
        }
        if !step.enabled_in_config {
            continue;
        }

        if let Some(kind) = step.permanent_kind {
            match core.is_permanent_tweak_applied(kind) {
                Ok(true) => {
                    update_auto_optimize_ui(ui, step.progress, step.already_label);
                    kind.set_enabled_in(config, false);
                    config_dirty = true;
                }
                Ok(false) => {
                    update_auto_optimize_ui(ui, step.progress, step.applying_label);
                    if (step.apply_fn)(core) {
                        if let Some(t) = step.session_tweak {
                            applied_tweaks.push(t);
                        }
                        kind.set_enabled_in(config, false);
                        config_dirty = true;
                    }
                }
                Err(_) => {
                    update_auto_optimize_ui(ui, step.progress, step.applying_label);
                }
            }
        } else {
            update_auto_optimize_ui(ui, step.progress, step.applying_label);
            if (step.apply_fn)(core) {
                if let Some(t) = step.session_tweak {
                    applied_tweaks.push(t);
                }
            }
        }

        if !sleep_with_cancellation(step_duration, token) {
            return cancel_and_rollback(applied_tweaks);
        }
    }

    Some((applied_tweaks, config_dirty))
}

fn run_steam_auto_optimize_flow(
    ui: &slint::Weak<MainWindow>,
    config_path: &std::sync::Arc<std::sync::RwLock<std::path::PathBuf>>,
    pid: u32,
    token: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    monitor_weak: &std::sync::Weak<counterrush_core::GameMonitorService>,
) {
    token.store(false, std::sync::atomic::Ordering::SeqCst);

    post_ui(ui, move |ui| {
        let lang = ui.get_active_language();
        let status_msg = match lang.as_str() {
            "en" | "en_US" | "en-US" | "en-EUA" => {
                format!("CS2 detected via Steam (PID: {pid}). Applying optimizations...")
            }
            "ru" | "ru_RU" | "ru-RU" => {
                format!("CS2 обнаружен через Steam (PID: {pid}). Применение оптимизаций...")
            }
            _ => format!("CS2 detectado via Steam (PID: {pid}). Aplicando otimizações..."),
        };
        ui.set_is_launching(true);
        ui.set_auto_optimize_is_launch(true);
        ui.set_auto_optimize_progress(0.0);
        ui.set_auto_optimize_step_name("CS2 detectado via Steam. Iniciando otimizações...".into());
        ui.set_auto_optimize_stage_title("".into());
        ui.set_auto_optimize_stage_subtitle("".into());
        ui.set_auto_optimize_finished(false);
        ui.set_show_auto_optimize_modal(true);
        ui.set_status_message(status_msg.into());
        tray::restore_and_focus_window(&ui);
    });

    let target_cfg_path = config_path.read().map(|p| p.clone()).unwrap_or_default();
    let effective_path = if target_cfg_path.as_os_str().is_empty() {
        let presets = scan_presets();
        if let Some(first) = presets.first() {
            let resolved = counterrush_core::resolve_optimization_file_path(first.name.as_str());
            if let Ok(mut guard) = config_path.write() {
                *guard = resolved.clone();
            }
            resolved
        } else {
            post_ui(ui, |ui| {
                ui.set_is_launching(false);
                ui.set_show_auto_optimize_modal(false);
                ui.set_show_no_profile_modal(true);
                ui.set_status_message(
                    "Nenhum perfil de otimização selecionado para auto-otimização Steam.".into(),
                );
            });
            return;
        }
    } else {
        target_cfg_path
    };

    let core = counterrush_core::OptimizerCore::default();
    let (resolved_cfg_path, mut config) =
        match core.load_or_create_custom_config(Some(&effective_path)) {
            Ok(res) => res,
            Err(err) => {
                let err_msg = format!("Erro na configuração: {err}");
                post_ui(ui, move |ui| {
                    ui.set_show_auto_optimize_modal(false);
                    ui.set_is_launching(false);
                    ui.set_status_message(err_msg.into());
                });
                return;
            }
        };

    let mut hw_dirty = false;
    if let Ok((hw_info, _)) = core.get_hardware_profile() {
        let before = config.clone();
        let _ = counterrush_core::enforce_hardware_restrictions_for_info(&mut config, &hw_info);
        hw_dirty = config != before;
    }

    let steps_dirty = match execute_configured_tweaks_with_modal(
        ui,
        &core,
        &mut config,
        token,
        std::time::Duration::from_millis(120),
        "Auto-otimização Steam cancelada pelo usuário. Alterações revertidas.",
    ) {
        Some((_, dirty)) => dirty,
        None => return,
    };

    if hw_dirty || steps_dirty {
        let _ = config.save_to_file(&resolved_cfg_path);
        let updated_tweaks = Tweaks::from_config(&config);
        post_ui(ui, move |ui| updated_tweaks.apply_to_ui(ui));
    }

    if config.cs2_priority || config.cs2_affinity {
        update_auto_optimize_ui(ui, 0.96, "Aplicando prioridade e afinidade no processo CS2...");
        let is_running = counterrush_core::windows_api::is_pid_alive(pid)
            || counterrush_core::windows_api::is_cs2_process_running_win32().unwrap_or(false);
        if is_running {
            use counterrush_core::tweaks::Tweak;
            if config.cs2_priority {
                let _ = counterrush_core::tweaks::Cs2PriorityTweak.apply();
            }
            if config.cs2_affinity {
                let _ = counterrush_core::tweaks::Cs2AffinityTweak.apply();
            }
        }
    }

    let profile_label = resolved_cfg_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("selecionado")
        .to_string();

    post_ui(ui, move |ui| {
        let lang = ui.get_active_language();
        let msg = match lang.as_str() {
            "en" | "en_US" | "en-US" | "en-EUA" => {
                format!("CS2 optimized with profile '{profile_label}'! Waiting for game focus...")
            }
            "ru" | "ru_RU" | "ru-RU" => {
                format!("CS2 оптимизирован с профилем '{profile_label}'! Ожидание фокуса игры...")
            }
            _ => format!(
                "CS2 otimizado com o perfil '{profile_label}'! Aguardando foco do jogo..."
            ),
        };
        ui.set_auto_optimize_stage_title("".into());
        ui.set_auto_optimize_stage_subtitle("".into());
        ui.set_auto_optimize_progress(1.0);
        ui.set_auto_optimize_step_name("CS2 OTIMIZADO COM SUCESSO".into());
        ui.set_auto_optimize_finished(true);
        ui.set_is_launching(false);
        ui.set_status_message(msg.into());
    });

    let start = std::time::Instant::now();
    let max_wait = std::time::Duration::from_secs(45);
    while start.elapsed() < max_wait {
        if let Some(m) = monitor_weak.upgrade() {
            if !m.is_running() {
                break;
            }
        }
        if !counterrush_core::windows_api::is_pid_alive(pid) {
            break;
        }
        if counterrush_core::windows_api::get_foreground_process_id() == Some(pid) {
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(250));
    }

    post_ui(ui, |ui| {
        ui.set_show_auto_optimize_modal(false);
        ui.set_is_launching(false);
        let _ = ui.hide();
    });
}

fn run_launch_flow(
    ui: &slint::Weak<MainWindow>,
    config_path: &std::sync::Arc<std::sync::RwLock<std::path::PathBuf>>,
    fallback_path: std::path::PathBuf,
    token: std::sync::Arc<std::sync::atomic::AtomicBool>,
) {
    if scan_presets().is_empty() {
        post_ui(ui, |ui| {
            ui.set_is_launching(false);
            ui.set_show_auto_optimize_modal(false);
            ui.set_show_no_profile_modal(true);
        });
        return;
    }

    let target_cfg_path = config_path.read().map(|p| p.clone()).unwrap_or(fallback_path);
    let core = counterrush_core::OptimizerCore::default();
    let (resolved_cfg_path, mut config) = match core.load_or_create_custom_config(Some(&target_cfg_path)) {
        Ok(res) => res,
        Err(err) => {
            let err_msg = format!("Erro na configuração: {err}");
            post_ui(ui, move |ui| {
                ui.set_show_auto_optimize_modal(false);
                ui.set_is_launching(false);
                ui.set_status_message(err_msg.into());
            });
            return;
        }
    };

    post_ui(ui, |ui| {
        ui.set_auto_optimize_is_launch(true);
        ui.set_auto_optimize_progress(0.0);
        ui.set_auto_optimize_step_name("Iniciando calibração e aplicação de otimizações...".into());
        ui.set_auto_optimize_stage_title("".into());
        ui.set_auto_optimize_stage_subtitle("".into());
        ui.set_auto_optimize_finished(false);
        ui.set_show_auto_optimize_modal(true);
        ui.set_is_launching(true);
    });

    let mut hw_dirty = false;
    if let Ok((hw_info, _)) = core.get_hardware_profile() {
        let before = config.clone();
        let _ = counterrush_core::enforce_hardware_restrictions_for_info(&mut config, &hw_info);
        hw_dirty = config != before;
    }

    let (applied_tweaks, steps_dirty) = match execute_configured_tweaks_with_modal(
        ui,
        &core,
        &mut config,
        &token,
        std::time::Duration::from_millis(180),
        "Inicialização cancelada pelo usuário. Alterações revertidas.",
    ) {
        Some(res) => res,
        None => return,
    };

    // Etapa 13: Verificação de Download/Atualização Ativa da Steam e Inicialização
    if token.load(std::sync::atomic::Ordering::SeqCst) {
        rollback_applied_tweaks(&core, applied_tweaks);
        post_ui(ui, |ui| {
            ui.set_show_auto_optimize_modal(false);
            ui.set_is_launching(false);
            ui.set_status_message("Inicialização cancelada pelo usuário. Alterações revertidas.".into());
        });
        return;
    }

    let mut was_downloading = false;
    let mut last_logged_detail = String::new();
    loop {
        let validation_status =
            counterrush_core::windows_api::get_cs2_download_validation_status();
        if !validation_status.is_active() {
            break;
        }

        if token.load(std::sync::atomic::Ordering::SeqCst) {
            rollback_applied_tweaks(&core, applied_tweaks);
            post_ui(ui, |ui| {
                ui.set_show_auto_optimize_modal(false);
                ui.set_is_launching(false);
                ui.set_auto_optimize_stage_title("".into());
                ui.set_auto_optimize_stage_subtitle("".into());
                ui.set_status_message("Inicialização cancelada pelo usuário. Alterações revertidas.".into());
            });
            return;
        }

        let current_detail = format!(
            "[VALIDAÇÃO] Camada '{}' detectou estado {:?} ({})",
            validation_status.source, validation_status.state, validation_status.detail
        );
        if current_detail != last_logged_detail {
            println!("   {}", current_detail);
            last_logged_detail = current_detail;
        }

        if !was_downloading {
            was_downloading = true;
            post_ui(ui, |ui| {
                ui.set_auto_optimize_stage_title("JOGO EM PROCESSO DE DOWNLOAD OU ATUALIZAÇÃO".into());
                ui.set_auto_optimize_stage_subtitle("Aguardando finalização para continuar".into());
                ui.set_auto_optimize_progress(0.95);
            });
        }

        if !sleep_with_cancellation(std::time::Duration::from_millis(500), &token) {
            rollback_applied_tweaks(&core, applied_tweaks);
            post_ui(ui, |ui| {
                ui.set_show_auto_optimize_modal(false);
                ui.set_is_launching(false);
                ui.set_auto_optimize_stage_title("".into());
                ui.set_auto_optimize_stage_subtitle("".into());
                ui.set_status_message("Inicialização cancelada pelo usuário. Alterações revertidas.".into());
            });
            return;
        }
    }

    if was_downloading {
        post_ui(ui, |ui| {
            ui.set_auto_optimize_stage_title("".into());
            ui.set_auto_optimize_stage_subtitle("".into());
        });
    }

    if hw_dirty || steps_dirty {
        let _ = config.save_to_file(&resolved_cfg_path);
        let updated_tweaks = Tweaks::from_config(&config);
        post_ui(ui, move |ui| updated_tweaks.apply_to_ui(ui));
    }

    update_auto_optimize_ui(ui, 0.96, "Inicializando Counter-Strike 2 pela Steam...");
    if let Err(e) = counterrush_core::windows_api::launch_cs2_with_args_slice(&config.launch_args) {
        let err_msg = format!("Falha ao iniciar CS2: {e}");
        post_ui(ui, move |ui| {
            ui.set_show_auto_optimize_modal(false);
            ui.set_is_launching(false);
            ui.set_status_message(err_msg.into());
        });
        return;
    }

    let _ = counterrush_core::windows_api::wait_for_process("cs2", std::time::Duration::from_millis(500));
    {
        use counterrush_core::tweaks::Tweak;
        if config.cs2_priority {
            let _ = counterrush_core::tweaks::Cs2PriorityTweak.apply();
        }
        if config.cs2_affinity {
            let _ = counterrush_core::tweaks::Cs2AffinityTweak.apply();
        }
    }

    // Estado Concluído a 100%
    post_ui(ui, |ui| {
        ui.set_auto_optimize_stage_title("".into());
        ui.set_auto_optimize_stage_subtitle("".into());
        ui.set_auto_optimize_progress(1.0);
        ui.set_auto_optimize_step_name("CS2 OTIMIZADO COM SUCESSO".into());
        ui.set_auto_optimize_finished(true);
        ui.set_status_message("CS2 OTIMIZADO COM SUCESSO".into());
    });

    // Aguarda 5 segundos antes de fechar o modal automaticamente
    for _ in 0..50 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if token.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }
    }

    // Fecha o modal automaticamente e garante mensagem de status
    post_ui(ui, |ui| {
        ui.set_show_auto_optimize_modal(false);
        ui.set_is_launching(false);
        ui.set_status_message("CS2 OTIMIZADO COM SUCESSO".into());
    });
}

fn select_preset(
    ui: &slint::Weak<MainWindow>,
    config_path: &std::sync::Arc<std::sync::RwLock<std::path::PathBuf>>,
    preset_name: String,
) {
    let target_path = counterrush_core::resolve_optimization_file_path(&preset_name);

    if let Ok(mut current) = config_path.write() {
        *current = target_path.clone();
    }

    match counterrush_core::config::CustomOptimizationConfig::load_from_file(&target_path) {
        Ok(cfg) => {
            save_active_preset_setting(Some(preset_name.clone()));
            let tweaks = Tweaks::from_config(&cfg);
            let args_text = cfg.launch_args.join(" ");
            let msg = format!(
                "Perfil '{}' selecionado e carregado com sucesso.",
                preset_name
            );

            post_ui(ui, move |ui| {
                ui.set_launch_args_text(args_text.into());
                ui.set_is_editing_launch_args(false);
                tweaks.apply_to_ui(ui);
                ui.set_status_message(msg.into());
            });
        }
        Err(err) => post_ui(ui, move |ui| {
            ui.set_status_message(
                format!("Erro ao carregar perfil '{}': {err}", preset_name).into(),
            )
        }),
    }
}

fn create_profile(
    ui: &slint::Weak<MainWindow>,
    config_path: &std::sync::Arc<std::sync::RwLock<std::path::PathBuf>>,
    name: String,
    launch_args: String,
    tweaks: Tweaks,
) {
    let clean_name = match counterrush_core::validate_profile_name(&name) {
        Ok(name) => name,
        Err(err_msg) => {
            post_ui(ui, move |ui| ui.set_create_modal_error(err_msg.into()));
            return;
        }
    };

    let mut new_config = counterrush_core::config::CustomOptimizationConfig {
        launch_args: launch_args.split_whitespace().map(|s| s.to_string()).collect(),
        ..Default::default()
    };
    tweaks.apply_to_config(&mut new_config);

    let target_path = counterrush_core::resolve_optimization_file_path(&clean_name);
    match new_config.save_to_file(&target_path) {
        Ok(_) => {
            if let Ok(mut current) = config_path.write() {
                *current = target_path.clone();
            }
            save_active_preset_setting(Some(clean_name.clone()));

            let updated_presets = scan_presets();
            let args_text = new_config.launch_args.join(" ");
            post_ui(ui, move |ui| {
                ui.set_presets(slint::ModelRc::new(slint::VecModel::from(updated_presets)));
                ui.set_active_preset(clean_name.as_str().into());
                ui.set_launch_args_text(args_text.into());
                ui.set_is_editing_launch_args(false);
                tweaks.apply_to_ui(ui);
                ui.set_create_modal_error("".into());
                ui.set_show_create_modal(false);
                ui.set_status_message(
                    format!("Perfil '{clean_name}' criado e selecionado com sucesso!").into(),
                );
            });
        }
        Err(e) => post_ui(ui, move |ui| {
            ui.set_create_modal_error(format!("Falha ao salvar arquivo: {e}").into())
        }),
    }
}

fn save_launch_args(
    ui: &slint::Weak<MainWindow>,
    config_path: &std::sync::Arc<std::sync::RwLock<std::path::PathBuf>>,
    args: String,
) {
    let target_path = config_path.read().map(|p| p.clone()).unwrap_or_default();
    if target_path.as_os_str().is_empty() {
        reject_tweak_without_profile(ui);
        return;
    }

    let mut config = counterrush_core::config::CustomOptimizationConfig::load_from_file(&target_path)
        .unwrap_or_default();
    config.launch_args = args.split_whitespace().map(|s| s.to_string()).collect();

    match config.save_to_file(&target_path) {
        Ok(_) => {
            let file_stem = target_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("perfil")
                .to_string();
            let args_text = config.launch_args.join(" ");
            post_ui(ui, move |ui| {
                ui.set_status_message(
                    format!("Parâmetros de inicialização salvos no perfil '{file_stem}'.").into(),
                );
                ui.set_launch_args_text(args_text.into());
            });
        }
        Err(err) => post_ui(ui, move |ui| {
            ui.set_status_message(format!("Falha ao salvar parâmetros: {err}").into())
        }),
    }
}

fn delete_profile(
    ui: &slint::Weak<MainWindow>,
    config_path: &std::sync::Arc<std::sync::RwLock<std::path::PathBuf>>,
    name: String,
    active: String,
) {
    match counterrush_core::delete_optimization_profile(&name) {
        Ok(_) => {
            let updated_presets = scan_presets();
            if updated_presets.is_empty() {
                if let Ok(mut cur) = config_path.write() {
                    *cur = std::path::PathBuf::new();
                }
                save_active_preset_setting(None);
                post_ui(ui, move |ui| {
                    ui.set_presets(slint::ModelRc::new(slint::VecModel::from(updated_presets)));
                    ui.set_active_preset("".into());
                    ui.set_launch_args_text("".into());
                    ui.set_is_editing_launch_args(false);
                    Tweaks::default().apply_to_ui(ui);
                    ui.set_status_message(
                        format!("Perfil '{name}' excluído com sucesso. Nenhum perfil restante.")
                            .into(),
                    );
                });
            } else if active == name {
                let first_preset_name = updated_presets[0].name.to_string();
                let target = counterrush_core::resolve_optimization_file_path(&first_preset_name);
                if let Ok(mut cur) = config_path.write() {
                    *cur = target.clone();
                }
                save_active_preset_setting(Some(first_preset_name.clone()));
                let cfg = counterrush_core::config::CustomOptimizationConfig::load_from_file(&target)
                    .unwrap_or_default();
                let tweaks = Tweaks::from_config(&cfg);
                let args_text = cfg.launch_args.join(" ");
                post_ui(ui, move |ui| {
                    ui.set_presets(slint::ModelRc::new(slint::VecModel::from(updated_presets)));
                    ui.set_active_preset(first_preset_name.as_str().into());
                    ui.set_launch_args_text(args_text.into());
                    ui.set_is_editing_launch_args(false);
                    tweaks.apply_to_ui(ui);
                    ui.set_status_message(
                        format!(
                            "Perfil '{name}' excluído. Perfil ativo alternado para '{}'.",
                            first_preset_name
                        )
                        .into(),
                    );
                });
            } else {
                post_ui(ui, move |ui| {
                    ui.set_presets(slint::ModelRc::new(slint::VecModel::from(updated_presets)));
                    ui.set_status_message(format!("Perfil '{name}' excluído com sucesso.").into());
                });
            }
        }
        Err(err) => post_ui(ui, move |ui| {
            ui.set_status_message(format!("Erro ao excluir perfil '{name}': {err}").into())
        }),
    }
}

fn main() -> Result<(), slint::PlatformError> {
    #[cfg(windows)]
    if !counterrush_core::windows_api::is_elevated() {
        if counterrush_core::try_notify_running_instance() {
            return Ok(());
        }
        if let Ok(()) = counterrush_core::windows_api::relaunch_as_admin() {
            return Ok(());
        }
        eprintln!("Aviso: Não foi possível obter privilégios de Administrador automaticamente.");
    }

    let mut single_instance_guard = match counterrush_core::acquire_single_instance() {
        Ok(counterrush_core::SingleInstanceOutcome::Primary(guard)) => Some(guard),
        Ok(counterrush_core::SingleInstanceOutcome::SecondaryNotified) => return Ok(()),
        Err(err) => {
            eprintln!("Aviso: falha ao inicializar controle de instância única: {err}");
            None
        }
    };

    if std::env::var("LANG").is_err() && std::env::var("LC_ALL").is_err() {
        std::env::set_var("LANG", "pt_BR.UTF-8");
        std::env::set_var("LANGUAGE", "pt_BR");
    }

    let locale_dir = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.join("locale")))
        .filter(|p| p.exists())
        .unwrap_or_else(|| std::path::PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/../locale")));

    slint::init_translations!(locale_dir);

    let main_window = MainWindow::new()?;
    main_window.set_app_version(env!("CARGO_PKG_VERSION").into());
    let app_tray = AppTray::new()?;

    if let Some(guard) = single_instance_guard.as_mut() {
        let single_instance_ui = main_window.as_weak();
        if let Err(err) = guard.start_activation_listener(move || {
            let ui_weak = single_instance_ui.clone();
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    tray::restore_and_focus_window(&ui);
                }
            });
        }) {
            eprintln!("Aviso: falha ao iniciar listener de instância única: {err}");
        }
    }

    // Inicialização de AppSettings e seleção dinâmica de idiomas
    let settings_path = counterrush_core::resolve_app_settings_path();
    let app_settings = counterrush_core::config::AppSettings::load_or_create(&settings_path)
        .unwrap_or_default();

    // Reconciliação do estado de inicialização e auto-otimização Steam
    let startup_task_active =
        counterrush_core::is_system_startup_enabled().unwrap_or(app_settings.start_with_system);
    let (effective_startup, effective_steam_auto) = counterrush_core::reconcile_startup_flags(
        startup_task_active,
        app_settings.optimize_on_steam_launch,
    );
    if app_settings.start_with_system != effective_startup
        || app_settings.optimize_on_steam_launch != effective_steam_auto
    {
        let mut reconciled = app_settings.clone();
        reconciled.start_with_system = effective_startup;
        reconciled.optimize_on_steam_launch = effective_steam_auto;
        let _ = reconciled.save_to_file(&settings_path);
    }
    main_window.set_start_with_system(effective_startup);
    main_window.set_optimize_on_steam_launch(effective_steam_auto);

    main_window.set_minimize_to_tray(app_settings.minimize_to_tray);
    let _minimize_to_tray_flag = tray::setup_tray(
        &main_window,
        &app_tray,
        settings_path.clone(),
        app_settings.minimize_to_tray,
    );

    let target_lang = match app_settings.language.as_str() {
        "en" | "en_US" | "en-US" | "en-EUA" => "en_US",
        "ru" | "ru_RU" | "ru-RU" => "ru_RU",
        "pt" | "pt_BR" | "pt-BR" => "pt_BR",
        _ => "pt_BR",
    };

    if let Err(e) = slint::select_bundled_translation(target_lang) {
        eprintln!("Aviso: falha ao carregar tradução embutida para '{}': {:?}", target_lang, e);
    }

    let language_items = vec![
        LanguageItem {
            code: "pt_BR".into(),
            display_name: "pt-BR".into(),
        },
        LanguageItem {
            code: "ru_RU".into(),
            display_name: "ru-RU".into(),
        },
        LanguageItem {
            code: "en_US".into(),
            display_name: "en-EUA".into(),
        },
    ];
    main_window.set_available_languages(slint::ModelRc::new(slint::VecModel::from(language_items)));
    main_window.set_active_language(target_lang.into());

    // Inicializa a lista de perfis de otimização na interface a partir do disco
    let initial_presets = scan_presets();
    main_window.set_presets(slint::ModelRc::new(slint::VecModel::from(initial_presets.clone())));

    // Se não existirem perfis salvos no disco, inicia no estado neutro desligado (R1)
    let (resolved_path, _config) = if initial_presets.is_empty() {
        main_window.set_active_preset("".into());
        main_window.set_launch_args_text("".into());
        main_window.set_is_editing_launch_args(false);
        enforce_neutral_no_profile_state(&main_window, neutral_status_message(target_lang));
        (std::path::PathBuf::new(), counterrush_core::config::CustomOptimizationConfig::all_disabled())
    } else {
        let preset_names: Vec<&str> = initial_presets.iter().map(|p| p.name.as_str()).collect();
        let selected_preset_name = counterrush_core::resolve_selected_preset_name(
            app_settings.active_preset.as_deref(),
            &preset_names,
        )
        .unwrap_or_else(|| initial_presets[0].name.as_str())
        .to_string();
        let p = counterrush_core::resolve_optimization_file_path(&selected_preset_name);
        let c = match counterrush_core::config::CustomOptimizationConfig::load_from_file(&p) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Erro ao carregar configuração: {e}");
                counterrush_core::config::CustomOptimizationConfig::default()
            }
        };

        if let Some(filename) = p.file_name().and_then(|s| s.to_str()) {
            let clean_name = filename.strip_suffix(".json").unwrap_or(filename);
            main_window.set_active_preset(clean_name.into());
        }

        main_window.set_launch_args_text(c.launch_args.join(" ").into());
        main_window.set_is_editing_launch_args(false);
        Tweaks::from_config(&c).apply_to_ui(&main_window);
        main_window.set_status_message(ready_status_message(target_lang).into());
        (p, c)
    };

    let active_config_path = std::sync::Arc::new(std::sync::RwLock::new(resolved_path.clone()));
    let auto_optimize_cancel_token =
        std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    let game_monitor = std::sync::Arc::new(counterrush_core::GameMonitorService::new());
    let monitor_ui = main_window.as_weak();
    let monitor_cfg_path = std::sync::Arc::clone(&active_config_path);
    let monitor_cancel_token = std::sync::Arc::clone(&auto_optimize_cancel_token);
    let monitor_weak = std::sync::Arc::downgrade(&game_monitor);
    game_monitor.set_on_launch(move |pid| {
        run_steam_auto_optimize_flow(
            &monitor_ui,
            &monitor_cfg_path,
            pid,
            &monitor_cancel_token,
            &monitor_weak,
        );
    });

    let monitor_ui_exit = main_window.as_weak();
    game_monitor.set_on_exit(move |pid| {
        let ui_weak = monitor_ui_exit.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let lang = ui.get_active_language();
                let msg = match lang.as_str() {
                    "en" | "en_US" | "en-US" | "en-EUA" => format!("CS2 closed (PID: {pid}). Monitoring active."),
                    "ru" | "ru_RU" | "ru-RU" => format!("CS2 закрыт (PID: {pid}). Мониторинг активен."),
                    _ => format!("CS2 encerrado (PID: {pid}). Monitoramento ativo."),
                };
                ui.set_status_message(msg.into());
            }
        });
    });

    if effective_steam_auto {
        game_monitor.start();
    }

    // Sincronização em tempo real (Arquivo JSON no disco -> UI)
    let sync_window_handle = main_window.as_weak();
    let sync_config_ref = active_config_path.clone();
    std::thread::spawn(move || {
        let mut last_modified = None;
        let mut tracked_path = std::path::PathBuf::new();
        let mut loop_count = 0usize;
        let mut last_preset_names: Vec<String> = initial_presets.iter().map(|p| p.name.to_string()).collect();

        loop {
            std::thread::sleep(std::time::Duration::from_millis(500));

            // Monitora criação/remoção de arquivos na pasta config/optimzations
            loop_count += 1;
            if loop_count % 3 == 0 {
                let current_presets = scan_presets();
                let current_names: Vec<String> = current_presets.iter().map(|p| p.name.to_string()).collect();
                if current_names != last_preset_names {
                    last_preset_names = current_names.clone();
                    let _ = slint::invoke_from_event_loop({
                        let handle = sync_window_handle.clone();
                        let sync_active_ref = sync_config_ref.clone();
                        let names_clone = current_names.clone();
                        move || {
                            if let Some(ui) = handle.upgrade() {
                                ui.set_presets(slint::ModelRc::new(slint::VecModel::from(current_presets)));
                                if names_clone.is_empty() {
                                    ui.set_active_preset("".into());
                                    ui.set_launch_args_text("".into());
                                    ui.set_is_editing_launch_args(false);
                                    let lang = ui.get_active_language();
                                    enforce_neutral_no_profile_state(
                                        &ui,
                                        neutral_status_message(lang.as_str()),
                                    );
                                    if let Ok(mut cur) = sync_active_ref.write() {
                                        *cur = std::path::PathBuf::new();
                                    }
                                    std::thread::spawn(|| save_active_preset_setting(None));
                                } else {
                                    let current_active = ui.get_active_preset().to_string();
                                    if !names_clone.contains(&current_active) {
                                        let first_preset = &names_clone[0];
                                        let target = counterrush_core::resolve_optimization_file_path(first_preset);
                                        if let Ok(mut cur) = sync_active_ref.write() {
                                            *cur = target.clone();
                                        }
                                        let saved_name = first_preset.clone();
                                        std::thread::spawn(move || {
                                            save_active_preset_setting(Some(saved_name));
                                        });
                                        ui.set_active_preset(first_preset.as_str().into());
                                        if let Ok(cfg) = counterrush_core::config::CustomOptimizationConfig::load_from_file(&target) {
                                            ui.set_launch_args_text(cfg.launch_args.join(" ").into());
                                        }
                                        ui.set_is_editing_launch_args(false);
                                    }
                                }
                            }
                        }
                    });
                }
            }

            let current_path = if let Ok(guard) = sync_config_ref.read() {
                guard.clone()
            } else {
                continue;
            };

            if current_path.as_os_str().is_empty() {
                continue;
            }

            if current_path != tracked_path {
                tracked_path = current_path.clone();
                last_modified = std::fs::metadata(&tracked_path).and_then(|m| m.modified()).ok();
            }

            if let Ok(metadata) = std::fs::metadata(&tracked_path) {
                if let Ok(modified) = metadata.modified() {
                    let should_check = match last_modified {
                        Some(prev) => modified > prev,
                        None => true,
                    };
                    if should_check {
                        last_modified = Some(modified);
                        if let Ok(cfg) = counterrush_core::config::CustomOptimizationConfig::load_from_file(&tracked_path) {
                            let _ = slint::invoke_from_event_loop({
                                let handle = sync_window_handle.clone();
                                move || {
                                    if let Some(ui) = handle.upgrade() {
                                        if !ui.get_is_editing_launch_args() {
                                            let joined_args = cfg.launch_args.join(" ");
                                            if ui.get_launch_args_text() != joined_args.as_str() {
                                                ui.set_launch_args_text(joined_args.into());
                                            }
                                        }
                                        if ui.get_cpu_tweak_active() != cfg.cs2_affinity {
                                            ui.set_cpu_tweak_active(cfg.cs2_affinity);
                                        }
                                        if ui.get_timer_tweak_active() != cfg.apply_timer_resolution {
                                            ui.set_timer_tweak_active(cfg.apply_timer_resolution);
                                        }
                                        if ui.get_priority_tweak_active() != cfg.cs2_priority {
                                            ui.set_priority_tweak_active(cfg.cs2_priority);
                                        }
                                        if ui.get_ram_tweak_active() != cfg.empty_standby_list {
                                            ui.set_ram_tweak_active(cfg.empty_standby_list);
                                        }
                                        if ui.get_gpu_accel_tweak_active() != cfg.disable_gpu_acceleration {
                                            ui.set_gpu_accel_tweak_active(cfg.disable_gpu_acceleration);
                                        }
                                        if ui.get_dpi_tweak_active() != cfg.apply_dpi_optimization {
                                            ui.set_dpi_tweak_active(cfg.apply_dpi_optimization);
                                        }
                                        if ui.get_mem_compression_tweak_active() != cfg.disable_memory_compression {
                                            ui.set_mem_compression_tweak_active(cfg.disable_memory_compression);
                                        }
                                        if ui.get_mmcss_tweak_active() != cfg.apply_mmcss {
                                            ui.set_mmcss_tweak_active(cfg.apply_mmcss);
                                        }
                                        if ui.get_hpet_tweak_active() != cfg.disable_hpet {
                                            ui.set_hpet_tweak_active(cfg.disable_hpet);
                                        }
                                        if ui.get_game_bar_tweak_active() != cfg.disable_game_bar {
                                            ui.set_game_bar_tweak_active(cfg.disable_game_bar);
                                        }
                                        if ui.get_msi_tweak_active() != cfg.enable_msi {
                                            ui.set_msi_tweak_active(cfg.enable_msi);
                                        }
                                        if ui.get_power_saving_tweak_active() != cfg.disable_power_saving {
                                            ui.set_power_saving_tweak_active(cfg.disable_power_saving);
                                        }
                                        if ui.get_render_queue_tweak_active() != cfg.apply_render_queue {
                                            ui.set_render_queue_tweak_active(cfg.apply_render_queue);
                                        }
                                    }
                                }
                            });
                        }
                    }
                }
            }
        }
    });

    // Thread executora única: leitura/gravação de perfis é serializada nela,
    // deixando a thread da janela apenas no loop de eventos.
    let command_sender = spawn_command_worker(main_window.as_weak(), active_config_path.clone());

    // Callback para Lançamento do CS2 com Otimizações Personalizadas (test_custom_launch)
    let window_handle = main_window.as_weak();
    let launch_config_ref = active_config_path.clone();
    let fallback_path = resolved_path.clone();
    let launch_cancel_token = auto_optimize_cancel_token.clone();
    main_window.on_launch_cs2_clicked(move || {
        let ui = match window_handle.upgrade() {
            Some(ui) => ui,
            None => return,
        };

        if scan_presets().is_empty() {
            ui.set_is_launching(false);
            ui.set_show_no_profile_modal(true);
            return;
        }

        ui.set_is_launching(true);
        ui.set_auto_optimize_is_launch(true);
        ui.set_auto_optimize_progress(0.0);
        ui.set_auto_optimize_step_name("Iniciando otimizações para o CS2...".into());
        ui.set_auto_optimize_finished(false);
        ui.set_show_auto_optimize_modal(true);
        ui.set_status_message("Otimizando sistema e iniciando CS2...".into());

        launch_cancel_token.store(false, std::sync::atomic::Ordering::SeqCst);
        let handle_for_thread = window_handle.clone();
        let target_cfg_path = launch_config_ref.clone();
        let fallback_cfg = fallback_path.clone();
        let token = launch_cancel_token.clone();
        std::thread::spawn(move || {
            run_launch_flow(&handle_for_thread, &target_cfg_path, fallback_cfg, token);
        });
    });

    // Inicializa controlador da view interna de gerenciamento de binds
    let binds_dirs = counterrush_core::BindDirectories::default();
    binds::wire_binds_view(&main_window, binds_dirs.clone());

    // Callback para clique nos cards de ação
    let window_handle = main_window.as_weak();
    let binds_dirs_for_card = binds_dirs.clone();
    main_window.on_card_action_clicked(move |module_id| {
        if let Some(ui) = window_handle.upgrade() {
            if module_id.as_str() == "binds" {
                binds::refresh_existing_cfg_files(&ui, &binds_dirs_for_card);
                ui.set_show_binds_view(true);
            }
            let msg = match module_id.as_str() {
                "custom_optimization" => "Acessando módulo: Otimização Personalizada",
                "pro_cfg" => "Acessando módulo: Catálogo de .cfg Profissionais",
                "binds" => "Acessando módulo: Criador e Gerenciador de Binds",
                "workshop_maps" => "Acessando módulo: Mapas da Oficina CS2",
                "benchmark" => "Acessando módulo: Benchmark e Análise de Desempenho",
                "about" => "Acessando módulo: Sobre o CounterRush",
                _ => "Módulo selecionado",
            };
            ui.set_status_message(msg.into());
        }
    });

    // Callback para alternância dos switches modulares (UI -> fila de comandos)
    let tweak_sender = command_sender.clone();
    main_window.on_tweak_toggled(move |tweak_name, active| {
        let _ = tweak_sender.send(Command::TweakToggled {
            name: tweak_name.to_string(),
            active,
        });
    });

    let blocked_tweak_ui = main_window.as_weak();
    main_window.on_tweak_requires_profile(move || {
        if let Some(ui) = blocked_tweak_ui.upgrade() {
            let lang = ui.get_active_language();
            enforce_neutral_no_profile_state(
                &ui,
                profile_required_status_message(lang.as_str()),
            );
        }
    });

    // Callback para ações em lote dos tweaks (UI -> fila de comandos)
    let bulk_sender = command_sender.clone();
    main_window.on_bulk_tweaks_action(move |action| {
        let _ = bulk_sender.send(Command::BulkTweaksAction {
            action: action.to_string(),
        });
    });

    // Callback para o botão "Otimizar Automaticamente" na aba Otimizações
    let auto_opt_ui = main_window.as_weak();
    let auto_opt_config_path = active_config_path.clone();
    let auto_opt_cancel = auto_optimize_cancel_token.clone();
    main_window.on_auto_optimize_clicked(move || {
        if let Some(ui) = auto_opt_ui.upgrade() {
            ui.set_auto_optimize_is_launch(false);
            ui.set_auto_optimize_progress(0.0);
            ui.set_auto_optimize_step_name("Iniciando calibração...".into());
            ui.set_auto_optimize_finished(false);
            ui.set_show_auto_optimize_modal(true);
        }
        auto_opt_cancel.store(false, std::sync::atomic::Ordering::SeqCst);
        let ui_weak = auto_opt_ui.clone();
        let cfg_path = auto_opt_config_path.clone();
        let token = auto_opt_cancel.clone();
        std::thread::spawn(move || {
            run_auto_optimize_flow(&ui_weak, &cfg_path, token);
        });
    });

    // Callback para cancelamento da otimização automática pelo modal
    let cancel_token_clone = auto_optimize_cancel_token.clone();
    main_window.on_cancel_auto_optimize_clicked(move || {
        cancel_token_clone.store(true, std::sync::atomic::Ordering::SeqCst);
    });

    // Callback para seleção de perfis de otimização (seleção única na aba Otimizações)
    let preset_sender = command_sender.clone();
    main_window.on_preset_selected(move |preset_name| {
        let _ = preset_sender.send(Command::PresetSelected {
            name: preset_name.to_string(),
        });
    });

    // Callback para criação e ativação imediata de novo perfil de otimização (.json)
    let create_sender = command_sender.clone();
    main_window.on_create_profile_confirmed(move |data| {
        let _ = create_sender.send(Command::CreateProfile {
            name: data.name.to_string(),
            launch_args: data.launch_args.to_string(),
            tweaks: Tweaks {
                cpu: data.cs2_affinity,
                timer: data.apply_timer_resolution,
                priority: data.cs2_priority,
                ram: data.empty_standby_list,
                gpu: data.disable_gpu_acceleration,
                dpi: data.apply_dpi_optimization,
                mem: data.disable_memory_compression,
                mmcss: data.apply_mmcss,
                hpet: data.disable_hpet,
                game_bar: data.disable_game_bar,
                msi: data.enable_msi,
                power: data.disable_power_saving,
                render_queue: data.apply_render_queue,
            },
        });
    });

    // Callback para salvar parâmetros de inicialização editados no arquivo de perfil selecionado
    let save_args_sender = command_sender.clone();
    main_window.on_save_launch_args_clicked(move |args_str| {
        let _ = save_args_sender.send(Command::SaveLaunchArgs {
            args: args_str.to_string(),
        });
    });

    // Callback para exclusão de perfil de otimização (.json)
    let delete_sender = command_sender.clone();
    let delete_ui_ref = main_window.as_weak();
    main_window.on_delete_profile_confirmed(move |target_name| {
        let active = delete_ui_ref
            .upgrade()
            .map(|ui| ui.get_active_preset().to_string())
            .unwrap_or_default();
        let _ = delete_sender.send(Command::DeleteProfile {
            name: target_name.to_string(),
            active,
        });
    });

    // Callback para seleção dinâmica de idioma na aba de Configurações
    let window_handle = main_window.as_weak();
    let settings_path_for_lang = settings_path.clone();
    main_window.on_language_selected(move |code| {
        let ui = match window_handle.upgrade() {
            Some(ui) => ui,
            None => return,
        };
        let target_code = match code.as_str() {
            "en" | "en_US" | "en-US" | "en-EUA" => "en_US",
            "ru" | "ru_RU" | "ru-RU" => "ru_RU",
            "pt" | "pt_BR" | "pt-BR" => "pt_BR",
            other => other,
        };
        if let Err(e) = slint::select_bundled_translation(target_code) {
            eprintln!("Erro ao alternar idioma para '{target_code}': {e:?}");
        }
        ui.set_active_language(target_code.into());

        if let Ok(mut settings) = counterrush_core::config::AppSettings::load_or_create(&settings_path_for_lang) {
            settings.language = target_code.to_string();
            let _ = settings.save_to_file(&settings_path_for_lang);
        }

        let msg = match target_code {
            "en" | "en_US" => format!("Language changed to '{}'.", target_code),
            "ru" | "ru_RU" => format!("Язык изменен на '{}'.", target_code),
            _ => format!("Idioma alterado para '{}'.", target_code),
        };
        ui.set_status_message(msg.into());
    });

    // Callback para compartilhamento do projeto (cópia para o clipboard do Windows)
    let window_handle = main_window.as_weak();
    main_window.on_share_project_clicked(move || {
        let ui = match window_handle.upgrade() {
            Some(ui) => ui,
            None => return,
        };
        let share_text = "CounterRush - Otimizador de Desempenho e Latência para Counter-Strike 2: https://github.com/ThainanViniciusKatchan/CounterRush";
        let lang = ui.get_active_language();
        if let Err(e) = counterrush_core::set_clipboard(share_text) {
            eprintln!("Erro ao copiar link para o clipboard: {e}");
            let err_prefix = match lang.as_str() {
                "en" | "en_US" => "Error accessing clipboard",
                "ru" | "ru_RU" => "Ошибка доступа к буферу обмена",
                _ => "Erro ao acessar clipboard",
            };
            ui.set_status_message(format!("{err_prefix}: {e}").into());
        } else {
            let msg = match lang.as_str() {
                "en" | "en_US" => "Official CounterRush link copied to clipboard!",
                "ru" | "ru_RU" => "Официальная ссылка CounterRush скопирована в буфер обмена!",
                _ => "Link oficial do CounterRush copiado para a área de transferência!",
            };
            ui.set_status_message(msg.into());
        }
    });

    // Callback para abertura de links externos no navegador padrão
    let window_handle = main_window.as_weak();
    main_window.on_open_external_url(move |url| {
        let url_str = url.to_string();
        let window_handle_clone = window_handle.clone();
        std::thread::spawn(move || {
            if let Err(e) = counterrush_core::windows_api::launcher::shell_execute(&url_str, None) {
                eprintln!("Erro ao abrir URL '{url_str}': {e}");
                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(ui) = window_handle_clone.upgrade() {
                        let lang = ui.get_active_language();
                        let err_prefix = match lang.as_str() {
                            "en" | "en_US" => "Error opening link",
                            "ru" | "ru_RU" => "Ошибка при открытии ссылки",
                            _ => "Erro ao abrir link",
                        };
                        ui.set_status_message(format!("{err_prefix}: {e}").into());
                    }
                });
            }
        });
    });

    // Callback para alternância de Inicialização com o Windows (schtasks ONLOGON /RL HIGHEST)
    let window_handle = main_window.as_weak();
    let startup_settings_path = settings_path.clone();
    let monitor_ref1 = std::sync::Arc::clone(&game_monitor);
    main_window.on_start_with_system_toggled(move |enabled| {
        let ui_weak = window_handle.clone();
        let target_path = startup_settings_path.clone();
        let monitor = std::sync::Arc::clone(&monitor_ref1);
        std::thread::spawn(move || {
            let prev_steam_auto = counterrush_core::config::AppSettings::load_or_create(&target_path)
                .map(|s| s.optimize_on_steam_launch)
                .unwrap_or(false);
            let result = if enabled {
                counterrush_core::enable_system_startup()
            } else {
                monitor.stop();
                counterrush_core::disable_system_startup()
            };

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let lang = ui.get_active_language();
                    match result {
                        Ok(()) => {
                            if let Ok(mut settings) =
                                counterrush_core::config::AppSettings::load_or_create(&target_path)
                            {
                                settings.start_with_system = enabled;
                                if !enabled {
                                    settings.optimize_on_steam_launch = false;
                                }
                                let _ = settings.save_to_file(&target_path);
                            }
                            ui.set_start_with_system(enabled);
                            if !enabled {
                                ui.set_optimize_on_steam_launch(false);
                            }
                            let msg = match (lang.as_str(), enabled, prev_steam_auto) {
                                ("en" | "en_US" | "en-US" | "en-EUA", true, _) => {
                                    "Start with Windows enabled (Highest privileges)."
                                }
                                ("en" | "en_US" | "en-US" | "en-EUA", false, true) => {
                                    "Start with Windows and Steam Auto-Optimization disabled."
                                }
                                ("en" | "en_US" | "en-US" | "en-EUA", false, false) => {
                                    "Start with Windows disabled."
                                }
                                ("ru" | "ru_RU" | "ru-RU", true, _) => {
                                    "Автозапуск с Windows включен (Наивысшие права)."
                                }
                                ("ru" | "ru_RU" | "ru-RU", false, true) => {
                                    "Автозапуск с Windows и автооптимизация Steam отключены."
                                }
                                ("ru" | "ru_RU" | "ru-RU", false, false) => {
                                    "Автозапуск с Windows отключен."
                                }
                                (_, true, _) => {
                                    "Inicialização com o Windows ativada com privilégios máximos."
                                }
                                (_, false, true) => {
                                    "Inicialização com o Windows e Auto-Otimização Steam desativadas."
                                }
                                (_, false, false) => "Inicialização com o Windows desativada.",
                            };
                            ui.set_status_message(msg.into());
                        }
                        Err(err) => {
                            ui.set_start_with_system(!enabled);
                            if !enabled && prev_steam_auto {
                                ui.set_optimize_on_steam_launch(true);
                                monitor.start();
                            }
                            let err_prefix = match lang.as_str() {
                                "en" | "en_US" | "en-US" | "en-EUA" => {
                                    "Failed to configure Windows startup"
                                }
                                "ru" | "ru_RU" | "ru-RU" => "Ошибка настройки автозапуска Windows",
                                _ => "Falha ao configurar inicialização com o Windows",
                            };
                            ui.set_status_message(format!("{err_prefix}: {err}").into());
                        }
                    }
                }
            });
        });
    });

    // Callback para alternância de Auto-Otimização Steam
    let window_handle_steam = main_window.as_weak();
    let steam_settings_path = settings_path.clone();
    let monitor_ref2 = std::sync::Arc::clone(&game_monitor);
    main_window.on_optimize_on_steam_launch_toggled(move |enabled| {
        let ui_weak = window_handle_steam.clone();
        let target_path = steam_settings_path.clone();
        let monitor = std::sync::Arc::clone(&monitor_ref2);
        std::thread::spawn(move || {
            let result = if enabled {
                let startup_res = counterrush_core::enable_system_startup();
                if startup_res.is_ok() {
                    monitor.start();
                }
                startup_res
            } else {
                monitor.stop();
                Ok(())
            };

            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_weak.upgrade() {
                    let lang = ui.get_active_language();
                    match result {
                        Ok(()) => {
                            if let Ok(mut settings) =
                                counterrush_core::config::AppSettings::load_or_create(&target_path)
                            {
                                settings.optimize_on_steam_launch = enabled;
                                if enabled {
                                    settings.start_with_system = true;
                                }
                                let _ = settings.save_to_file(&target_path);
                            }
                            ui.set_optimize_on_steam_launch(enabled);
                            if enabled {
                                ui.set_start_with_system(true);
                            }
                            let msg = match (lang.as_str(), enabled) {
                                ("en" | "en_US" | "en-US" | "en-EUA", true) => {
                                    "Steam Auto-Optimization and Startup enabled."
                                }
                                ("en" | "en_US" | "en-US" | "en-EUA", false) => {
                                    "Steam Auto-Optimization disabled."
                                }
                                ("ru" | "ru_RU" | "ru-RU", true) => {
                                    "Автооптимизация Steam и автозапуск включены."
                                }
                                ("ru" | "ru_RU" | "ru-RU", false) => {
                                    "Автооптимизация Steam отключена."
                                }
                                (_, true) => {
                                    "Auto-Otimização Steam e Inicialização com o Windows ativadas."
                                }
                                (_, false) => "Auto-Otimização Steam desativada.",
                            };
                            ui.set_status_message(msg.into());
                        }
                        Err(err) => {
                            ui.set_optimize_on_steam_launch(!enabled);
                            if enabled {
                                let actual_startup =
                                    counterrush_core::is_system_startup_enabled().unwrap_or(false);
                                ui.set_start_with_system(actual_startup);
                            }
                            let err_prefix = match lang.as_str() {
                                "en" | "en_US" | "en-US" | "en-EUA" => {
                                    "Failed to configure Steam Auto-Optimization"
                                }
                                "ru" | "ru_RU" | "ru-RU" => {
                                    "Ошибка настройки автооптимизации Steam"
                                }
                                _ => "Falha ao configurar Auto-Otimização Steam",
                            };
                            ui.set_status_message(format!("{err_prefix}: {err}").into());
                        }
                    }
                }
            });
        });
    });

    main_window.show()?;
    slint::run_event_loop_until_quit()?;
    Ok(())
}
