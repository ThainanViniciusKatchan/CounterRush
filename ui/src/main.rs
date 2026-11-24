#![windows_subsystem = "windows"]

slint::include_modules!();

mod binds;
mod binds_i18n;
mod i18n;
mod tray;

fn scan_cfg_files() -> Vec<slint::SharedString> {
    counterrush_core::list_cfg_files()
        .into_iter()
        .filter_map(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(slint::SharedString::from)
        })
        .collect()
}

fn preferred_cfg_from_config(cfg: &counterrush_core::config::CustomOptimizationConfig) -> String {
    counterrush_core::extract_exec_cfg_from_launch_args(&cfg.launch_args).unwrap_or_default()
}

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

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct InGameRuntimeState {
    cpu: bool,
    priority: bool,
    timer: bool,
    ram: bool,
    cs2_seen_running: bool,
    applied_at: Option<std::time::Instant>,
}

impl InGameRuntimeState {
    fn any_applied(&self) -> bool {
        self.cpu || self.priority || self.timer || self.ram
    }

    fn apply_to_ui(&self, ui: &MainWindow) {
        if ui.get_cpu_tweak_applied() != self.cpu {
            ui.set_cpu_tweak_applied(self.cpu);
        }
        if ui.get_priority_tweak_applied() != self.priority {
            ui.set_priority_tweak_applied(self.priority);
        }
        if ui.get_timer_tweak_applied() != self.timer {
            ui.set_timer_tweak_applied(self.timer);
        }
        if ui.get_ram_tweak_applied() != self.ram {
            ui.set_ram_tweak_applied(self.ram);
        }
    }

    fn clear(&mut self) {
        self.cpu = false;
        self.priority = false;
        self.timer = false;
        self.ram = false;
        self.cs2_seen_running = false;
        self.applied_at = None;
    }
}

type InGameAppliedCache = std::sync::Arc<std::sync::Mutex<InGameRuntimeState>>;

fn clear_in_game_applied(ui: &slint::Weak<MainWindow>, cache: &InGameAppliedCache) {
    if let Ok(mut guard) = cache.lock() {
        guard.clear();
    }
    post_ui(ui, |ui| {
        InGameRuntimeState::default().apply_to_ui(ui);
    });
}

fn update_in_game_applied<F>(
    ui: &slint::Weak<MainWindow>,
    cache: &InGameAppliedCache,
    updater: F,
) where
    F: FnOnce(&mut InGameRuntimeState),
{
    let snapshot = if let Ok(mut guard) = cache.lock() {
        updater(&mut guard);
        if guard.any_applied() && guard.applied_at.is_none() {
            guard.applied_at = Some(std::time::Instant::now());
        } else if !guard.any_applied() {
            guard.cs2_seen_running = false;
            guard.applied_at = None;
        }
        *guard
    } else {
        InGameRuntimeState::default()
    };
    post_ui(ui, move |ui| {
        snapshot.apply_to_ui(ui);
    });
}

fn enforce_neutral_no_profile_state(ui: &MainWindow, status_msg: &str) {
    Tweaks::default().apply_to_ui(ui);
    InGameRuntimeState::default().apply_to_ui(ui);
    ui.set_selected_preferred_cfg("".into());
    ui.set_cfg_dropdown_open(false);
    ui.set_status_message(status_msg.into());
}

fn reject_tweak_without_profile(ui: &slint::Weak<MainWindow>) {
    post_ui(ui, |ui| {
        let lang = i18n::UiLanguage::from_code(ui.get_active_language().as_str());
        enforce_neutral_no_profile_state(ui, i18n::profile_required_status_message(lang));
    });
}

/// Comandos enfileirados pela thread da janela e executados pela thread executora.
enum Command {
    TweakToggled { name: String, active: bool },
    BulkTweaksAction { action: String },
    PresetSelected { name: String },
    CreateProfile { name: String, launch_args: String, tweaks: Tweaks },
    SaveLaunchArgs { args: String },
    SelectPreferredCfg { cfg_file: String },
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

type AppliedSystemCache = std::sync::Arc<
    std::sync::RwLock<std::collections::HashSet<counterrush_core::PermanentTweakKind>>,
>;

fn permanent_kind_from_tweak_name(name: &str) -> Option<counterrush_core::PermanentTweakKind> {
    use counterrush_core::PermanentTweakKind;
    match name {
        "gpu_acceleration" => Some(PermanentTweakKind::DisableGpuAcceleration),
        "dpi_optimization" => Some(PermanentTweakKind::ApplyDpiOptimization),
        "memory_compression" => Some(PermanentTweakKind::DisableMemoryCompression),
        "mmcss" => Some(PermanentTweakKind::ApplyMmcss),
        "hpet" => Some(PermanentTweakKind::DisableHpet),
        "game_bar" => Some(PermanentTweakKind::DisableGameBar),
        "msi" => Some(PermanentTweakKind::EnableMsi),
        "power_saving" => Some(PermanentTweakKind::DisablePowerSaving),
        "render_queue" => Some(PermanentTweakKind::ApplyRenderQueue),
        _ => None,
    }
}

fn set_single_permanent_in_ui(
    ui: &MainWindow,
    kind: counterrush_core::PermanentTweakKind,
    active: bool,
) {
    use counterrush_core::PermanentTweakKind;
    match kind {
        PermanentTweakKind::DisableGpuAcceleration => ui.set_gpu_accel_tweak_active(active),
        PermanentTweakKind::ApplyDpiOptimization => ui.set_dpi_tweak_active(active),
        PermanentTweakKind::DisableMemoryCompression => ui.set_mem_compression_tweak_active(active),
        PermanentTweakKind::ApplyMmcss => ui.set_mmcss_tweak_active(active),
        PermanentTweakKind::DisableHpet => ui.set_hpet_tweak_active(active),
        PermanentTweakKind::DisableGameBar => ui.set_game_bar_tweak_active(active),
        PermanentTweakKind::EnableMsi => ui.set_msi_tweak_active(active),
        PermanentTweakKind::DisablePowerSaving => ui.set_power_saving_tweak_active(active),
        PermanentTweakKind::ApplyRenderQueue => ui.set_render_queue_tweak_active(active),
    }
}

fn merge_cache_into_config(
    config: &mut counterrush_core::config::CustomOptimizationConfig,
    cache: &AppliedSystemCache,
) -> bool {
    let mut changed = false;
    if let Ok(guard) = cache.read() {
        for &kind in guard.iter() {
            if !kind.is_enabled_in(config) {
                kind.set_enabled_in(config, true);
                changed = true;
            }
        }
    }
    changed
}

fn spawn_command_worker(
    ui: slint::Weak<MainWindow>,
    config_path: std::sync::Arc<std::sync::RwLock<std::path::PathBuf>>,
    applied_cache: AppliedSystemCache,
    in_game_cache: InGameAppliedCache,
    active_lang: std::sync::Arc<std::sync::atomic::AtomicU8>,
) -> std::sync::mpsc::Sender<Command> {
    let (sender, receiver) = std::sync::mpsc::channel::<Command>();
    std::thread::spawn(move || {
        while let Ok(command) = receiver.recv() {
            execute_command(
                command,
                &ui,
                &config_path,
                &applied_cache,
                &in_game_cache,
                &active_lang,
            );
        }
    });
    sender
}

fn execute_command(
    command: Command,
    ui: &slint::Weak<MainWindow>,
    config_path: &std::sync::Arc<std::sync::RwLock<std::path::PathBuf>>,
    applied_cache: &AppliedSystemCache,
    in_game_cache: &InGameAppliedCache,
    active_lang: &std::sync::Arc<std::sync::atomic::AtomicU8>,
) {
    let lang = i18n::UiLanguage::load_atomic(active_lang);
    match command {
        Command::TweakToggled { name, active } => {
            apply_tweak(ui, config_path, applied_cache, in_game_cache, name, active, lang)
        }
        Command::BulkTweaksAction { action } => {
            apply_bulk_action(ui, config_path, in_game_cache, action)
        }
        Command::PresetSelected { name } => select_preset(ui, config_path, applied_cache, name, lang),
        Command::CreateProfile {
            name,
            launch_args,
            tweaks,
        } => create_profile(ui, config_path, name, launch_args, tweaks, lang),
        Command::SaveLaunchArgs { args } => save_launch_args(ui, config_path, args, lang),
        Command::SelectPreferredCfg { cfg_file } => {
            select_preferred_cfg(ui, config_path, cfg_file, lang)
        }
        Command::DeleteProfile { name, active } => {
            delete_profile(ui, config_path, name, active, lang)
        }
    }
}

fn apply_tweak(
    ui: &slint::Weak<MainWindow>,
    config_path: &std::sync::Arc<std::sync::RwLock<std::path::PathBuf>>,
    applied_cache: &AppliedSystemCache,
    in_game_cache: &InGameAppliedCache,
    tweak_name: String,
    active: bool,
    lang: i18n::UiLanguage,
) {
    let target_path = config_path.read().map(|p| p.clone()).unwrap_or_default();
    if target_path.as_os_str().is_empty() {
        reject_tweak_without_profile(ui);
        return;
    }

    if !active {
        if matches!(
            tweak_name.as_str(),
            "cpu_core_0" | "process_priority" | "timer_resolution" | "empty_standby"
        ) {
            update_in_game_applied(ui, in_game_cache, |state| match tweak_name.as_str() {
                "cpu_core_0" => state.cpu = false,
                "process_priority" => state.priority = false,
                "timer_resolution" => state.timer = false,
                "empty_standby" => state.ram = false,
                _ => {}
            });
        }
        if let Some(kind) = permanent_kind_from_tweak_name(&tweak_name) {
            let core = counterrush_core::OptimizerCore::default();
            let was_applied = core.is_permanent_tweak_applied(kind).unwrap_or(false);
            if was_applied {
                if let Err(err) = core.rollback_permanent_tweak(kind) {
                    if let Ok(mut guard) = applied_cache.write() {
                        guard.insert(kind);
                    }
                    let err_msg = i18n::tweak_rollback_failed(lang, &tweak_name, &err);
                    post_ui(ui, move |ui| {
                        set_single_permanent_in_ui(ui, kind, true);
                        ui.set_status_message(err_msg.into());
                    });
                    return;
                }
            }
            if let Ok(mut guard) = applied_cache.write() {
                guard.remove(&kind);
            }
            let mut config =
                counterrush_core::config::CustomOptimizationConfig::load_from_file(&target_path)
                    .unwrap_or_default();
            kind.set_enabled_in(&mut config, false);
            let _ = config.save_to_file(&target_path);

            let msg = if was_applied {
                i18n::tweak_reverted_message(lang, &tweak_name)
            } else {
                i18n::tweak_toggled_message(lang, &tweak_name, false)
            };
            post_ui(ui, move |ui| ui.set_status_message(msg.into()));
            return;
        }
    }

    let msg = i18n::tweak_toggled_message(lang, &tweak_name, active);

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
    in_game_cache: &InGameAppliedCache,
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
        "deactivate_all" => {
            Tweaks::default().apply_to_config(&mut config);
            clear_in_game_applied(ui, in_game_cache);
        }
        "reset_defaults" => {
            config = Default::default();
            clear_in_game_applied(ui, in_game_cache);
        }
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

impl AppliedSessionTweak {
    fn permanent_kind(self) -> Option<counterrush_core::PermanentTweakKind> {
        use counterrush_core::PermanentTweakKind;
        match self {
            Self::Timer => None,
            Self::Mmcss => Some(PermanentTweakKind::ApplyMmcss),
            Self::Dpi => Some(PermanentTweakKind::ApplyDpiOptimization),
            Self::GpuAccel => Some(PermanentTweakKind::DisableGpuAcceleration),
            Self::MemCompression => Some(PermanentTweakKind::DisableMemoryCompression),
            Self::Hpet => Some(PermanentTweakKind::DisableHpet),
            Self::GameBar => Some(PermanentTweakKind::DisableGameBar),
            Self::Msi => Some(PermanentTweakKind::EnableMsi),
            Self::PowerSaving => Some(PermanentTweakKind::DisablePowerSaving),
            Self::RenderQueue => Some(PermanentTweakKind::ApplyRenderQueue),
        }
    }
}

fn rollback_applied_tweaks(
    core: &counterrush_core::OptimizerCore,
    applied: Vec<AppliedSessionTweak>,
    applied_cache: &AppliedSystemCache,
) {
    for tweak in applied.into_iter().rev() {
        let reverted_ok = match tweak {
            AppliedSessionTweak::Timer => core.rollback_timer_all().is_ok(),
            AppliedSessionTweak::Mmcss => core.rollback_all_mmcss().is_ok(),
            AppliedSessionTweak::Dpi => core.rollback_dpi_optimization().is_ok(),
            AppliedSessionTweak::GpuAccel => core.rollback_all_gpu_acceleration().is_ok(),
            AppliedSessionTweak::MemCompression => core.rollback_memory_compression().is_ok(),
            AppliedSessionTweak::Hpet => core.rollback_hpet().is_ok(),
            AppliedSessionTweak::GameBar => core.rollback_all_game_bar_features().is_ok(),
            AppliedSessionTweak::Msi => core.rollback_all_msi().is_ok(),
            AppliedSessionTweak::PowerSaving => core.rollback_all_power_saving().is_ok(),
            AppliedSessionTweak::RenderQueue => core.rollback_render_queue().is_ok(),
        };
        if reverted_ok {
            if let Some(kind) = tweak.permanent_kind() {
                if let Ok(mut guard) = applied_cache.write() {
                    guard.remove(&kind);
                }
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
    session_tweak: Option<AppliedSessionTweak>,
    apply_fn: fn(&counterrush_core::OptimizerCore) -> bool,
}

struct ConfiguredTweakStep {
    tweak_id: &'static str,
    enabled_in_config: bool,
    progress: f32,
    permanent_kind: Option<counterrush_core::PermanentTweakKind>,
    session_tweak: Option<AppliedSessionTweak>,
    apply_fn: fn(&counterrush_core::OptimizerCore) -> bool,
}

fn check_auto_optimize_cancelled(
    ui: &slint::Weak<MainWindow>,
    token: &std::sync::atomic::AtomicBool,
    core: &counterrush_core::OptimizerCore,
    applied_tweaks: &mut Vec<AppliedSessionTweak>,
    applied_cache: &AppliedSystemCache,
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
    rollback_applied_tweaks(core, to_revert, applied_cache);
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
    applied_cache: &AppliedSystemCache,
    token: std::sync::Arc<std::sync::atomic::AtomicBool>,
    active_lang: std::sync::Arc<std::sync::atomic::AtomicU8>,
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
    let lang = i18n::UiLanguage::load_atomic(&active_lang);
    update_auto_optimize_ui(ui, 0.08, i18n::auto_optimize_detecting_hw(lang));
    if check_auto_optimize_cancelled(
        ui,
        &token,
        &core,
        &mut applied_tweaks,
        applied_cache,
        Some(step_duration),
        i18n::auto_optimize_cancelled(i18n::UiLanguage::load_atomic(&active_lang)),
    ) {
        return;
    }

    // Etapa 1: Avaliação Combinada (Hardware + Status atual do SO)
    let lang = i18n::UiLanguage::load_atomic(&active_lang);
    update_auto_optimize_ui(ui, 0.16, i18n::auto_optimize_analyzing_hw(lang));
    let (mut config, report) = match core.generate_autodetect_config() {
        Ok(res) => res,
        Err(e) => {
            let err_msg = i18n::auto_optimize_err_generate(lang, &e);
            post_ui(ui, move |ui| {
                ui.set_show_auto_optimize_modal(false);
                ui.set_status_message(err_msg.into());
            });
            return;
        }
    };
    if check_auto_optimize_cancelled(
        ui,
        &token,
        &core,
        &mut applied_tweaks,
        applied_cache,
        Some(step_duration),
        i18n::auto_optimize_cancelled(i18n::UiLanguage::load_atomic(&active_lang)),
    ) {
        return;
    }

    let steps = [
        AutoTweakStep {
            progress: 0.23,
            tweak_id: "apply_timer_resolution",
            permanent_kind: None,
            enabled_in_config: config.apply_timer_resolution,
            session_tweak: Some(AppliedSessionTweak::Timer),
            apply_fn: |c| c.apply_timer_resolution().is_ok(),
        },
        AutoTweakStep {
            progress: 0.31,
            tweak_id: "apply_mmcss",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::ApplyMmcss),
            enabled_in_config: config.apply_mmcss,
            session_tweak: Some(AppliedSessionTweak::Mmcss),
            apply_fn: |c| c.apply_all_mmcss().is_ok(),
        },
        AutoTweakStep {
            progress: 0.39,
            tweak_id: "apply_dpi_optimization",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::ApplyDpiOptimization),
            enabled_in_config: config.apply_dpi_optimization,
            session_tweak: Some(AppliedSessionTweak::Dpi),
            apply_fn: |c| c.apply_dpi_optimization().is_ok(),
        },
        AutoTweakStep {
            progress: 0.47,
            tweak_id: "disable_gpu_acceleration",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::DisableGpuAcceleration),
            enabled_in_config: config.disable_gpu_acceleration,
            session_tweak: Some(AppliedSessionTweak::GpuAccel),
            apply_fn: |c| c.disable_all_gpu_acceleration().is_ok(),
        },
        AutoTweakStep {
            progress: 0.55,
            tweak_id: "disable_memory_compression",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::DisableMemoryCompression),
            enabled_in_config: config.disable_memory_compression,
            session_tweak: Some(AppliedSessionTweak::MemCompression),
            apply_fn: |c| c.disable_memory_compression().is_ok(),
        },
        AutoTweakStep {
            progress: 0.62,
            tweak_id: "empty_standby_list",
            permanent_kind: None,
            enabled_in_config: config.empty_standby_list,
            session_tweak: None,
            apply_fn: |c| c.empty_standby_list().is_ok(),
        },
        AutoTweakStep {
            progress: 0.69,
            tweak_id: "disable_hpet",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::DisableHpet),
            enabled_in_config: config.disable_hpet,
            session_tweak: Some(AppliedSessionTweak::Hpet),
            apply_fn: |c| c.disable_hpet().is_ok(),
        },
        AutoTweakStep {
            progress: 0.76,
            tweak_id: "disable_game_bar",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::DisableGameBar),
            enabled_in_config: config.disable_game_bar,
            session_tweak: Some(AppliedSessionTweak::GameBar),
            apply_fn: |c| c.disable_all_game_bar_features().is_ok(),
        },
        AutoTweakStep {
            progress: 0.83,
            tweak_id: "enable_msi",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::EnableMsi),
            enabled_in_config: config.enable_msi,
            session_tweak: Some(AppliedSessionTweak::Msi),
            apply_fn: |c| c.enable_all_msi().is_ok(),
        },
        AutoTweakStep {
            progress: 0.90,
            tweak_id: "disable_power_saving",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::DisablePowerSaving),
            enabled_in_config: config.disable_power_saving,
            session_tweak: Some(AppliedSessionTweak::PowerSaving),
            apply_fn: |c| c.disable_all_power_saving().is_ok(),
        },
        AutoTweakStep {
            progress: 0.97,
            tweak_id: "apply_render_queue",
            permanent_kind: Some(counterrush_core::PermanentTweakKind::ApplyRenderQueue),
            enabled_in_config: config.apply_render_queue,
            session_tweak: Some(AppliedSessionTweak::RenderQueue),
            apply_fn: |c| c.apply_render_queue().is_ok(),
        },
    ];

    for step in steps {
        let lang = i18n::UiLanguage::load_atomic(&active_lang);
        let cancel_reverted_msg = i18n::auto_optimize_cancelled_reverted(lang);
        if check_auto_optimize_cancelled(
            ui,
            &token,
            &core,
            &mut applied_tweaks,
            applied_cache,
            None,
            cancel_reverted_msg,
        ) {
            return;
        }

        if report.is_already_applied(step.tweak_id) {
            update_auto_optimize_ui(
                ui,
                step.progress,
                i18n::auto_optimize_step_label(
                    lang,
                    step.tweak_id,
                    i18n::AutoStepStatusKind::Already,
                ),
            );
            already_count += 1;
            if let Some(kind) = step.permanent_kind {
                kind.set_enabled_in(&mut config, true);
                if let Ok(mut guard) = applied_cache.write() {
                    guard.insert(kind);
                }
            }
        } else if !step.enabled_in_config {
            update_auto_optimize_ui(
                ui,
                step.progress,
                i18n::auto_optimize_step_label(
                    lang,
                    step.tweak_id,
                    i18n::AutoStepStatusKind::Skipped,
                ),
            );
        } else {
            update_auto_optimize_ui(
                ui,
                step.progress,
                i18n::auto_optimize_step_label(
                    lang,
                    step.tweak_id,
                    i18n::AutoStepStatusKind::Applying,
                ),
            );
            if (step.apply_fn)(&core) {
                applied_count += 1;
                if let Some(session_tweak) = step.session_tweak {
                    applied_tweaks.push(session_tweak);
                }
                if let Some(kind) = step.permanent_kind {
                    kind.set_enabled_in(&mut config, true);
                    if let Ok(mut guard) = applied_cache.write() {
                        guard.insert(kind);
                    }
                }
            } else if let Some(kind) = step.permanent_kind {
                kind.set_enabled_in(&mut config, false);
            }
        }

        let lang_after = i18n::UiLanguage::load_atomic(&active_lang);
        if check_auto_optimize_cancelled(
            ui,
            &token,
            &core,
            &mut applied_tweaks,
            applied_cache,
            Some(step_duration),
            i18n::auto_optimize_cancelled_reverted(lang_after),
        ) {
            return;
        }
    }

    // Etapa Final: Gravação de AutoDetect.json e atualização da UI
    let lang = i18n::UiLanguage::load_atomic(&active_lang);
    if check_auto_optimize_cancelled(
        ui,
        &token,
        &core,
        &mut applied_tweaks,
        applied_cache,
        None,
        i18n::auto_optimize_cancelled_reverted(lang),
    ) {
        return;
    }

    let autodetect_path = counterrush_core::resolve_autodetect_config_path(None);
    if let Err(e) = config.save_to_file(&autodetect_path) {
        let err_msg = i18n::auto_optimize_err_save(lang, &e);
        post_ui(ui, move |ui| {
            ui.set_show_auto_optimize_modal(false);
            ui.set_status_message(err_msg.into());
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
    let preferred_cfg = preferred_cfg_from_config(&config);
    let tweaks = Tweaks::from_config(&config);
    let msg = i18n::auto_optimize_completed_status(
        lang,
        report.category,
        &report.hardware.gpu.model,
        applied_count,
        already_count,
        report.blocked_tweaks.len(),
    );
    let completed_step = i18n::auto_optimize_completed_step(lang);

    post_ui(ui, move |ui| {
        ui.set_presets(slint::ModelRc::new(slint::VecModel::from(updated_presets)));
        ui.set_active_preset("AutoDetect".into());
        ui.set_launch_args_text(args_text.into());
        ui.set_selected_preferred_cfg(preferred_cfg.into());
        ui.set_is_editing_launch_args(false);
        tweaks.apply_to_ui(ui);
        ui.set_status_message(msg.into());
        ui.set_auto_optimize_progress(1.0);
        ui.set_auto_optimize_step_name(completed_step.into());
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

pub(crate) fn save_content_view_mode_setting(settings_path: &std::path::Path, is_grid_view: bool) {
    if let Ok(mut settings) = counterrush_core::config::AppSettings::load_or_create(settings_path) {
        settings.content_view_mode =
            counterrush_core::config::ContentViewMode::from_is_grid(is_grid_view);
        let _ = settings.save_to_file(settings_path);
    }
}

fn ensure_content_view_mode_key_migrated(
    settings_path: &std::path::Path,
    settings: &counterrush_core::config::AppSettings,
) {
    settings.ensure_content_view_mode_migrated(settings_path);
}

struct ConfiguredExecutionResult {
    applied_tweaks: Vec<AppliedSessionTweak>,
    config_dirty: bool,
    timer_applied: bool,
    ram_applied: bool,
}

fn execute_configured_tweaks_with_modal(
    ui: &slint::Weak<MainWindow>,
    core: &counterrush_core::OptimizerCore,
    config: &mut counterrush_core::config::CustomOptimizationConfig,
    applied_cache: &AppliedSystemCache,
    token: &std::sync::atomic::AtomicBool,
    step_duration: std::time::Duration,
    cancel_msg: &str,
    active_lang: &std::sync::Arc<std::sync::atomic::AtomicU8>,
) -> Option<ConfiguredExecutionResult> {
    let mut applied_tweaks = Vec::new();
    let mut config_dirty = false;
    let mut timer_applied = false;
    let mut ram_applied = false;
    let cancel_and_rollback =
        |applied: Vec<AppliedSessionTweak>| -> Option<ConfiguredExecutionResult> {
            rollback_applied_tweaks(core, applied, applied_cache);
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
    let lang = i18n::UiLanguage::load_atomic(active_lang);
    update_auto_optimize_ui(ui, 0.08, i18n::configured_checking_profile(lang));
    if !sleep_with_cancellation(step_duration, token) {
        return cancel_and_rollback(applied_tweaks);
    }

    let steps = [
        ConfiguredTweakStep {
            tweak_id: "empty_standby_list",
            enabled_in_config: config.empty_standby_list,
            progress: 0.16,
            permanent_kind: None,
            session_tweak: None,
            apply_fn: |c| c.empty_standby_list().is_ok(),
        },
        ConfiguredTweakStep {
            tweak_id: "apply_timer_resolution",
            enabled_in_config: config.apply_timer_resolution,
            progress: 0.24,
            permanent_kind: None,
            session_tweak: Some(AppliedSessionTweak::Timer),
            apply_fn: |c| c.apply_timer_resolution().is_ok(),
        },
        ConfiguredTweakStep {
            tweak_id: "apply_mmcss",
            enabled_in_config: config.apply_mmcss,
            progress: 0.32,
            permanent_kind: Some(counterrush_core::PermanentTweakKind::ApplyMmcss),
            session_tweak: Some(AppliedSessionTweak::Mmcss),
            apply_fn: |c| c.apply_all_mmcss().is_ok(),
        },
        ConfiguredTweakStep {
            tweak_id: "apply_dpi_optimization",
            enabled_in_config: config.apply_dpi_optimization,
            progress: 0.40,
            permanent_kind: Some(counterrush_core::PermanentTweakKind::ApplyDpiOptimization),
            session_tweak: Some(AppliedSessionTweak::Dpi),
            apply_fn: |c| c.apply_dpi_optimization().is_ok(),
        },
        ConfiguredTweakStep {
            tweak_id: "disable_gpu_acceleration",
            enabled_in_config: config.disable_gpu_acceleration,
            progress: 0.48,
            permanent_kind: Some(counterrush_core::PermanentTweakKind::DisableGpuAcceleration),
            session_tweak: Some(AppliedSessionTweak::GpuAccel),
            apply_fn: |c| c.disable_all_gpu_acceleration().is_ok(),
        },
        ConfiguredTweakStep {
            tweak_id: "disable_memory_compression",
            enabled_in_config: config.disable_memory_compression,
            progress: 0.56,
            permanent_kind: Some(counterrush_core::PermanentTweakKind::DisableMemoryCompression),
            session_tweak: Some(AppliedSessionTweak::MemCompression),
            apply_fn: |c| c.disable_memory_compression().is_ok(),
        },
        ConfiguredTweakStep {
            tweak_id: "disable_hpet",
            enabled_in_config: config.disable_hpet,
            progress: 0.64,
            permanent_kind: Some(counterrush_core::PermanentTweakKind::DisableHpet),
            session_tweak: Some(AppliedSessionTweak::Hpet),
            apply_fn: |c| c.disable_hpet().is_ok(),
        },
        ConfiguredTweakStep {
            tweak_id: "disable_game_bar",
            enabled_in_config: config.disable_game_bar,
            progress: 0.72,
            permanent_kind: Some(counterrush_core::PermanentTweakKind::DisableGameBar),
            session_tweak: Some(AppliedSessionTweak::GameBar),
            apply_fn: |c| c.disable_all_game_bar_features().is_ok(),
        },
        ConfiguredTweakStep {
            tweak_id: "enable_msi",
            enabled_in_config: config.enable_msi,
            progress: 0.80,
            permanent_kind: Some(counterrush_core::PermanentTweakKind::EnableMsi),
            session_tweak: Some(AppliedSessionTweak::Msi),
            apply_fn: |c| c.enable_all_msi().is_ok(),
        },
        ConfiguredTweakStep {
            tweak_id: "disable_power_saving",
            enabled_in_config: config.disable_power_saving,
            progress: 0.86,
            permanent_kind: Some(counterrush_core::PermanentTweakKind::DisablePowerSaving),
            session_tweak: Some(AppliedSessionTweak::PowerSaving),
            apply_fn: |c| c.disable_all_power_saving().is_ok(),
        },
        ConfiguredTweakStep {
            tweak_id: "apply_render_queue",
            enabled_in_config: config.apply_render_queue,
            progress: 0.92,
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

        let lang = i18n::UiLanguage::load_atomic(active_lang);
        let applying_label = i18n::configured_step_applying(lang, step.tweak_id);
        let already_label = i18n::configured_step_already(lang, step.tweak_id);

        if let Some(kind) = step.permanent_kind {
            let cached_applied = applied_cache
                .read()
                .map(|g| g.contains(&kind))
                .unwrap_or(false);
            if cached_applied {
                update_auto_optimize_ui(ui, step.progress, already_label);
            } else {
                match core.is_permanent_tweak_applied(kind) {
                    Ok(true) => {
                        update_auto_optimize_ui(ui, step.progress, already_label);
                        if let Ok(mut guard) = applied_cache.write() {
                            guard.insert(kind);
                        }
                    }
                    Ok(false) => {
                        update_auto_optimize_ui(ui, step.progress, applying_label);
                        if (step.apply_fn)(core) {
                            if let Some(t) = step.session_tweak {
                                applied_tweaks.push(t);
                            }
                            if let Ok(mut guard) = applied_cache.write() {
                                guard.insert(kind);
                            }
                        } else {
                            kind.set_enabled_in(config, false);
                            config_dirty = true;
                        }
                    }
                    Err(_) => {
                        update_auto_optimize_ui(ui, step.progress, applying_label);
                    }
                }
            }
        } else {
            update_auto_optimize_ui(ui, step.progress, applying_label);
            if (step.apply_fn)(core) {
                match step.tweak_id {
                    "empty_standby_list" => ram_applied = true,
                    "apply_timer_resolution" => timer_applied = true,
                    _ => {}
                }
                if let Some(t) = step.session_tweak {
                    applied_tweaks.push(t);
                }
            }
        }

        if !sleep_with_cancellation(step_duration, token) {
            return cancel_and_rollback(applied_tweaks);
        }
    }

    Some(ConfiguredExecutionResult {
        applied_tweaks,
        config_dirty,
        timer_applied,
        ram_applied,
    })
}

fn run_steam_auto_optimize_flow(
    ui: &slint::Weak<MainWindow>,
    config_path: &std::sync::Arc<std::sync::RwLock<std::path::PathBuf>>,
    applied_cache: &AppliedSystemCache,
    in_game_cache: &InGameAppliedCache,
    pid: u32,
    token: &std::sync::Arc<std::sync::atomic::AtomicBool>,
    monitor_weak: &std::sync::Weak<counterrush_core::GameMonitorService>,
    active_lang: &std::sync::Arc<std::sync::atomic::AtomicU8>,
) {
    token.store(false, std::sync::atomic::Ordering::SeqCst);

    post_ui(ui, move |ui| {
        let lang = i18n::UiLanguage::from_code(ui.get_active_language().as_str());
        let status_msg = i18n::steam_auto_detected_status(lang, pid);
        ui.set_is_launching(true);
        ui.set_auto_optimize_is_launch(true);
        ui.set_auto_optimize_is_restore(false);
        ui.set_auto_optimize_progress(0.0);
        ui.set_auto_optimize_step_name(i18n::steam_auto_detected_step(lang).into());
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
            let lang = i18n::UiLanguage::load_atomic(active_lang);
            let msg = i18n::steam_auto_no_profile(lang);
            post_ui(ui, move |ui| {
                ui.set_is_launching(false);
                ui.set_show_auto_optimize_modal(false);
                ui.set_show_no_profile_modal(true);
                ui.set_status_message(msg.into());
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
                let lang = i18n::UiLanguage::load_atomic(active_lang);
                let err_msg = i18n::launch_config_error(lang, &err);
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

    let cancel_msg = i18n::steam_auto_cancelled(i18n::UiLanguage::load_atomic(active_lang));
    let exec_res = match execute_configured_tweaks_with_modal(
        ui,
        &core,
        &mut config,
        applied_cache,
        token,
        std::time::Duration::from_millis(120),
        cancel_msg,
        active_lang,
    ) {
        Some(res) => res,
        None => {
            clear_in_game_applied(ui, in_game_cache);
            return;
        }
    };
    let steps_dirty = exec_res.config_dirty;

    if hw_dirty || steps_dirty {
        let _ = config.save_to_file(&resolved_cfg_path);
        let updated_tweaks = Tweaks::from_config(&config);
        post_ui(ui, move |ui| updated_tweaks.apply_to_ui(ui));
    }

    let is_running = counterrush_core::windows_api::is_pid_alive(pid)
        || counterrush_core::windows_api::is_cs2_process_running_win32().unwrap_or(false);
    let mut priority_applied = false;
    let mut cpu_applied = false;
    if config.cs2_priority || config.cs2_affinity {
        let lang = i18n::UiLanguage::load_atomic(active_lang);
        update_auto_optimize_ui(ui, 0.96, i18n::steam_auto_applying_process(lang));
        if is_running {
            use counterrush_core::tweaks::Tweak;
            if config.cs2_priority && counterrush_core::tweaks::Cs2PriorityTweak.apply().is_ok() {
                priority_applied = true;
            }
            if config.cs2_affinity && counterrush_core::tweaks::Cs2AffinityTweak.apply().is_ok() {
                cpu_applied = true;
            }
        }
    }

    update_in_game_applied(ui, in_game_cache, |state| {
        state.cpu = cpu_applied;
        state.priority = priority_applied;
        state.timer = exec_res.timer_applied;
        state.ram = exec_res.ram_applied;
        state.cs2_seen_running = is_running;
        state.applied_at = Some(std::time::Instant::now());
    });

    let profile_label = resolved_cfg_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("AutoDetect")
        .to_string();

    post_ui(ui, move |ui| {
        let lang = i18n::UiLanguage::from_code(ui.get_active_language().as_str());
        let msg = i18n::steam_auto_done_status(lang, &profile_label);
        ui.set_auto_optimize_stage_title("".into());
        ui.set_auto_optimize_stage_subtitle("".into());
        ui.set_auto_optimize_progress(1.0);
        ui.set_auto_optimize_step_name(i18n::launch_success(lang).into());
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
    applied_cache: &AppliedSystemCache,
    in_game_cache: &InGameAppliedCache,
    fallback_path: std::path::PathBuf,
    token: std::sync::Arc<std::sync::atomic::AtomicBool>,
    active_lang: std::sync::Arc<std::sync::atomic::AtomicU8>,
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
            let lang = i18n::UiLanguage::load_atomic(&active_lang);
            let err_msg = i18n::launch_config_error(lang, &err);
            post_ui(ui, move |ui| {
                ui.set_show_auto_optimize_modal(false);
                ui.set_is_launching(false);
                ui.set_status_message(err_msg.into());
            });
            return;
        }
    };

    let initial_step = i18n::launch_flow_initial_step(i18n::UiLanguage::load_atomic(&active_lang));
    post_ui(ui, move |ui| {
        ui.set_auto_optimize_is_launch(true);
        ui.set_auto_optimize_is_restore(false);
        ui.set_auto_optimize_progress(0.0);
        ui.set_auto_optimize_step_name(initial_step.into());
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

    let cancel_msg = i18n::launch_cancelled_reverted(i18n::UiLanguage::load_atomic(&active_lang));
    let exec_res = match execute_configured_tweaks_with_modal(
        ui,
        &core,
        &mut config,
        applied_cache,
        &token,
        std::time::Duration::from_millis(180),
        cancel_msg,
        &active_lang,
    ) {
        Some(res) => res,
        None => {
            clear_in_game_applied(ui, in_game_cache);
            return;
        }
    };
    let applied_tweaks = exec_res.applied_tweaks;
    let steps_dirty = exec_res.config_dirty;
    let timer_applied = exec_res.timer_applied;
    let ram_applied = exec_res.ram_applied;

    // Etapa 13: Verificação de Download/Atualização Ativa da Steam e Inicialização
    if token.load(std::sync::atomic::Ordering::SeqCst) {
        rollback_applied_tweaks(&core, applied_tweaks, applied_cache);
        clear_in_game_applied(ui, in_game_cache);
        let msg = i18n::launch_cancelled_reverted(i18n::UiLanguage::load_atomic(&active_lang));
        post_ui(ui, move |ui| {
            ui.set_show_auto_optimize_modal(false);
            ui.set_is_launching(false);
            ui.set_status_message(msg.into());
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
            rollback_applied_tweaks(&core, applied_tweaks, applied_cache);
            clear_in_game_applied(ui, in_game_cache);
            let msg = i18n::launch_cancelled_reverted(i18n::UiLanguage::load_atomic(&active_lang));
            post_ui(ui, move |ui| {
                ui.set_show_auto_optimize_modal(false);
                ui.set_is_launching(false);
                ui.set_auto_optimize_stage_title("".into());
                ui.set_auto_optimize_stage_subtitle("".into());
                ui.set_status_message(msg.into());
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
            let lang = i18n::UiLanguage::load_atomic(&active_lang);
            let dl_title = i18n::launch_download_title(lang);
            let dl_sub = i18n::launch_download_subtitle(lang);
            post_ui(ui, move |ui| {
                ui.set_auto_optimize_stage_title(dl_title.into());
                ui.set_auto_optimize_stage_subtitle(dl_sub.into());
                ui.set_auto_optimize_progress(0.95);
            });
        }

        if !sleep_with_cancellation(std::time::Duration::from_millis(500), &token) {
            rollback_applied_tweaks(&core, applied_tweaks, applied_cache);
            clear_in_game_applied(ui, in_game_cache);
            let msg = i18n::launch_cancelled_reverted(i18n::UiLanguage::load_atomic(&active_lang));
            post_ui(ui, move |ui| {
                ui.set_show_auto_optimize_modal(false);
                ui.set_is_launching(false);
                ui.set_auto_optimize_stage_title("".into());
                ui.set_auto_optimize_stage_subtitle("".into());
                ui.set_status_message(msg.into());
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

    let lang = i18n::UiLanguage::load_atomic(&active_lang);
    update_auto_optimize_ui(ui, 0.96, i18n::launch_starting_steam(lang));
    if let Err(e) = counterrush_core::windows_api::launch_cs2_with_args_slice(&config.launch_args) {
        clear_in_game_applied(ui, in_game_cache);
        let err_msg = i18n::launch_failed(lang, &e);
        post_ui(ui, move |ui| {
            ui.set_show_auto_optimize_modal(false);
            ui.set_is_launching(false);
            ui.set_status_message(err_msg.into());
        });
        return;
    }

    update_in_game_applied(ui, in_game_cache, |state| {
        state.cpu = false;
        state.priority = false;
        state.timer = timer_applied;
        state.ram = ram_applied;
        state.cs2_seen_running = false;
        state.applied_at = Some(std::time::Instant::now());
    });

    let _ = counterrush_core::windows_api::wait_for_process("cs2", std::time::Duration::from_millis(500));
    let mut priority_applied = false;
    let mut cpu_applied = false;
    {
        use counterrush_core::tweaks::Tweak;
        if config.cs2_priority && counterrush_core::tweaks::Cs2PriorityTweak.apply().is_ok() {
            priority_applied = true;
        }
        if config.cs2_affinity && counterrush_core::tweaks::Cs2AffinityTweak.apply().is_ok() {
            cpu_applied = true;
        }
    }
    if priority_applied || cpu_applied {
        update_in_game_applied(ui, in_game_cache, |state| {
            state.priority = priority_applied;
            state.cpu = cpu_applied;
            state.cs2_seen_running = true;
            state.applied_at = Some(std::time::Instant::now());
        });
    }

    // Estado Concluído a 100%
    let success_msg = i18n::launch_success(i18n::UiLanguage::load_atomic(&active_lang));
    post_ui(ui, move |ui| {
        ui.set_auto_optimize_stage_title("".into());
        ui.set_auto_optimize_stage_subtitle("".into());
        ui.set_auto_optimize_progress(1.0);
        ui.set_auto_optimize_step_name(success_msg.into());
        ui.set_auto_optimize_finished(true);
        ui.set_status_message(success_msg.into());
    });

    // Aguarda 5 segundos antes de fechar o modal automaticamente
    for _ in 0..50 {
        std::thread::sleep(std::time::Duration::from_millis(100));
        if token.load(std::sync::atomic::Ordering::SeqCst) {
            break;
        }
    }

    // Fecha o modal automaticamente e garante mensagem de status
    let final_msg = i18n::launch_success(i18n::UiLanguage::load_atomic(&active_lang));
    post_ui(ui, move |ui| {
        ui.set_show_auto_optimize_modal(false);
        ui.set_is_launching(false);
        ui.set_status_message(final_msg.into());
    });
}

fn select_preset(
    ui: &slint::Weak<MainWindow>,
    config_path: &std::sync::Arc<std::sync::RwLock<std::path::PathBuf>>,
    applied_cache: &AppliedSystemCache,
    preset_name: String,
    lang: i18n::UiLanguage,
) {
    let target_path = counterrush_core::resolve_optimization_file_path(&preset_name);

    if let Ok(mut current) = config_path.write() {
        *current = target_path.clone();
    }

    match counterrush_core::config::CustomOptimizationConfig::load_from_file(&target_path) {
        Ok(mut cfg) => {
            save_active_preset_setting(Some(preset_name.clone()));
            let args_text = cfg.launch_args.join(" ");
            let preferred_cfg = preferred_cfg_from_config(&cfg);
            let _ = merge_cache_into_config(&mut cfg, applied_cache);
            let tweaks = Tweaks::from_config(&cfg);
            let msg = i18n::preset_selected_ok(lang, &preset_name);

            post_ui(ui, move |ui| {
                ui.set_launch_args_text(args_text.into());
                ui.set_selected_preferred_cfg(preferred_cfg.into());
                ui.set_cfg_dropdown_open(false);
                ui.set_is_editing_launch_args(false);
                tweaks.apply_to_ui(ui);
                ui.set_status_message(msg.into());
            });
        }
        Err(err) => {
            let msg = i18n::preset_load_err(lang, &preset_name, &err);
            post_ui(ui, move |ui| ui.set_status_message(msg.into()));
        }
    }
}

fn create_profile(
    ui: &slint::Weak<MainWindow>,
    config_path: &std::sync::Arc<std::sync::RwLock<std::path::PathBuf>>,
    name: String,
    launch_args: String,
    tweaks: Tweaks,
    lang: i18n::UiLanguage,
) {
    let clean_name = match counterrush_core::validate_profile_name(&name) {
        Ok(name) => name,
        Err(err_msg) => {
            let localized_err = i18n::profile_validation_error(lang, &err_msg);
            post_ui(ui, move |ui| ui.set_create_modal_error(localized_err.into()));
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
            let preferred_cfg = preferred_cfg_from_config(&new_config);
            let ok_msg = i18n::profile_created_ok(lang, &clean_name);
            post_ui(ui, move |ui| {
                ui.set_presets(slint::ModelRc::new(slint::VecModel::from(updated_presets)));
                ui.set_active_preset(clean_name.as_str().into());
                ui.set_launch_args_text(args_text.into());
                ui.set_selected_preferred_cfg(preferred_cfg.into());
                ui.set_cfg_dropdown_open(false);
                ui.set_is_editing_launch_args(false);
                tweaks.apply_to_ui(ui);
                ui.set_create_modal_error("".into());
                ui.set_show_create_modal(false);
                ui.set_status_message(ok_msg.into());
            });
        }
        Err(e) => {
            let err_msg = i18n::profile_save_err(lang, &e);
            post_ui(ui, move |ui| ui.set_create_modal_error(err_msg.into()));
        }
    }
}

fn save_launch_args(
    ui: &slint::Weak<MainWindow>,
    config_path: &std::sync::Arc<std::sync::RwLock<std::path::PathBuf>>,
    args: String,
    lang: i18n::UiLanguage,
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
                .unwrap_or("profile")
                .to_string();
            let args_text = config.launch_args.join(" ");
            let preferred_cfg = preferred_cfg_from_config(&config);
            let msg = i18n::launch_args_saved(lang, &file_stem);
            post_ui(ui, move |ui| {
                ui.set_status_message(msg.into());
                ui.set_launch_args_text(args_text.into());
                ui.set_selected_preferred_cfg(preferred_cfg.into());
            });
        }
        Err(err) => {
            let msg = i18n::launch_args_save_err(lang, &err);
            post_ui(ui, move |ui| ui.set_status_message(msg.into()));
        }
    }
}

fn select_preferred_cfg(
    ui: &slint::Weak<MainWindow>,
    config_path: &std::sync::Arc<std::sync::RwLock<std::path::PathBuf>>,
    cfg_file: String,
    lang: i18n::UiLanguage,
) {
    let target_path = config_path.read().map(|p| p.clone()).unwrap_or_default();
    if target_path.as_os_str().is_empty() {
        reject_tweak_without_profile(ui);
        return;
    }

    let mut config = counterrush_core::config::CustomOptimizationConfig::load_from_file(&target_path)
        .unwrap_or_default();
    let trimmed = cfg_file.trim();
    let desired = if trimmed.is_empty() {
        None
    } else {
        Some(trimmed)
    };

    let updated_args = match counterrush_core::set_exec_cfg_in_launch_args(&config.launch_args, desired) {
        Ok(args) => args,
        Err(err) => {
            let actual_cfg = preferred_cfg_from_config(&config);
            let msg = i18n::preferred_cfg_save_err(lang, &err);
            post_ui(ui, move |ui| {
                ui.set_selected_preferred_cfg(actual_cfg.into());
                ui.set_status_message(msg.into());
            });
            return;
        }
    };
    config.launch_args = updated_args;

    let file_stem = target_path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("profile")
        .to_string();

    match config.save_to_file(&target_path) {
        Ok(_) => {
            let args_text = config.launch_args.join(" ");
            let final_cfg = preferred_cfg_from_config(&config);
            let msg = if final_cfg.is_empty() {
                i18n::preferred_cfg_cleared(lang, &file_stem)
            } else {
                i18n::preferred_cfg_selected(lang, &final_cfg, &file_stem)
            };
            post_ui(ui, move |ui| {
                ui.set_launch_args_text(args_text.into());
                ui.set_selected_preferred_cfg(final_cfg.into());
                ui.set_is_editing_launch_args(false);
                ui.set_status_message(msg.into());
            });
        }
        Err(err) => {
            let actual_cfg = counterrush_core::config::CustomOptimizationConfig::load_from_file(&target_path)
                .map(|c| preferred_cfg_from_config(&c))
                .unwrap_or_default();
            let msg = i18n::preferred_cfg_save_err(lang, &err);
            post_ui(ui, move |ui| {
                ui.set_selected_preferred_cfg(actual_cfg.into());
                ui.set_status_message(msg.into());
            });
        }
    }
}

fn delete_profile(
    ui: &slint::Weak<MainWindow>,
    config_path: &std::sync::Arc<std::sync::RwLock<std::path::PathBuf>>,
    name: String,
    active: String,
    lang: i18n::UiLanguage,
) {
    match counterrush_core::delete_optimization_profile(&name) {
        Ok(_) => {
            let updated_presets = scan_presets();
            if updated_presets.is_empty() {
                if let Ok(mut cur) = config_path.write() {
                    *cur = std::path::PathBuf::new();
                }
                save_active_preset_setting(None);
                let msg = i18n::profile_deleted_empty(lang, &name);
                post_ui(ui, move |ui| {
                    ui.set_presets(slint::ModelRc::new(slint::VecModel::from(updated_presets)));
                    ui.set_active_preset("".into());
                    ui.set_launch_args_text("".into());
                    ui.set_is_editing_launch_args(false);
                    enforce_neutral_no_profile_state(ui, &msg);
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
                let preferred_cfg = preferred_cfg_from_config(&cfg);
                let msg = i18n::profile_deleted_switched(lang, &name, &first_preset_name);
                post_ui(ui, move |ui| {
                    ui.set_presets(slint::ModelRc::new(slint::VecModel::from(updated_presets)));
                    ui.set_active_preset(first_preset_name.as_str().into());
                    ui.set_launch_args_text(args_text.into());
                    ui.set_selected_preferred_cfg(preferred_cfg.into());
                    ui.set_cfg_dropdown_open(false);
                    ui.set_is_editing_launch_args(false);
                    tweaks.apply_to_ui(ui);
                    ui.set_status_message(msg.into());
                });
            } else {
                let msg = i18n::profile_deleted_ok(lang, &name);
                post_ui(ui, move |ui| {
                    ui.set_presets(slint::ModelRc::new(slint::VecModel::from(updated_presets)));
                    ui.set_status_message(msg.into());
                });
            }
        }
        Err(err) => {
            let msg = i18n::profile_delete_err(lang, &name, &err);
            post_ui(ui, move |ui| ui.set_status_message(msg.into()));
        }
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
    ensure_content_view_mode_key_migrated(&settings_path, &app_settings);
    let initial_is_grid_view = app_settings.content_view_mode.is_grid();
    main_window.set_content_is_grid_view(initial_is_grid_view);

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

    let target_lang = i18n::UiLanguage::from_code(&app_settings.language).code();
    let active_lang = std::sync::Arc::new(std::sync::atomic::AtomicU8::new(
        i18n::UiLanguage::from_code(target_lang).to_u8(),
    ));

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
            display_name: "en-US".into(),
        },
    ];
    main_window.set_available_languages(slint::ModelRc::new(slint::VecModel::from(language_items)));
    main_window.set_active_language(target_lang.into());

    // Inicializa a lista de perfis de otimização e arquivos .cfg na interface a partir do disco
    let initial_cfg_files = scan_cfg_files();
    main_window.set_available_cfg_files(slint::ModelRc::new(slint::VecModel::from(initial_cfg_files.clone())));

    let initial_presets = scan_presets();
    main_window.set_presets(slint::ModelRc::new(slint::VecModel::from(initial_presets.clone())));

    // Se não existirem perfis salvos no disco, inicia no estado neutro desligado (R1)
    let initial_lang_enum = i18n::UiLanguage::from_code(target_lang);
    let (resolved_path, _config) = if initial_presets.is_empty() {
        main_window.set_active_preset("".into());
        main_window.set_launch_args_text("".into());
        main_window.set_is_editing_launch_args(false);
        enforce_neutral_no_profile_state(&main_window, i18n::neutral_status_message(initial_lang_enum));
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
        main_window.set_selected_preferred_cfg(preferred_cfg_from_config(&c).into());
        main_window.set_is_editing_launch_args(false);
        Tweaks::from_config(&c).apply_to_ui(&main_window);
        main_window.set_status_message(i18n::ready_status_message(initial_lang_enum).into());
        (p, c)
    };

    let active_config_path = std::sync::Arc::new(std::sync::RwLock::new(resolved_path.clone()));
    let applied_system_cache: AppliedSystemCache =
        std::sync::Arc::new(std::sync::RwLock::new(std::collections::HashSet::new()));
    let in_game_applied_cache: InGameAppliedCache =
        std::sync::Arc::new(std::sync::Mutex::new(InGameRuntimeState::default()));
    let auto_optimize_cancel_token =
        std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));

    // Consulta assíncrona do estado real no Windows para sincronizar otimizações já aplicadas
    let init_ui = main_window.as_weak();
    let init_cfg_path = active_config_path.clone();
    let init_cache = applied_system_cache.clone();
    std::thread::spawn(move || {
        let core = counterrush_core::OptimizerCore::default();
        let hardware = match core.get_or_detect_hardware(None) {
            Ok((_, hw, _, _)) => hw,
            Err(_) => return,
        };
        let statuses = counterrush_core::SystemTweakStatuses::query_for_hardware(&hardware);
        let mut applied_kinds = Vec::new();
        if let Ok(mut guard) = init_cache.write() {
            for kind in counterrush_core::PermanentTweakKind::ALL {
                if statuses.is_permanent_applied(kind) {
                    guard.insert(kind);
                    applied_kinds.push(kind);
                }
            }
        }
        if applied_kinds.is_empty() {
            return;
        }
        let target_path = init_cfg_path.read().map(|p| p.clone()).unwrap_or_default();
        if !target_path.as_os_str().is_empty() {
            if let Ok(mut cfg) =
                counterrush_core::config::CustomOptimizationConfig::load_from_file(&target_path)
            {
                if merge_cache_into_config(&mut cfg, &init_cache) {
                    let _ = cfg.save_to_file(&target_path);
                }
            }
        }
        post_ui(&init_ui, move |ui| {
            if ui.get_is_restoring_defaults() || ui.get_active_preset().is_empty() {
                return;
            }
            for kind in applied_kinds {
                set_single_permanent_in_ui(ui, kind, true);
            }
        });
    });

    let game_monitor = std::sync::Arc::new(counterrush_core::GameMonitorService::new());
    let monitor_ui = main_window.as_weak();
    let monitor_cfg_path = std::sync::Arc::clone(&active_config_path);
    let monitor_cache = std::sync::Arc::clone(&applied_system_cache);
    let monitor_in_game_cache = std::sync::Arc::clone(&in_game_applied_cache);
    let monitor_cancel_token = std::sync::Arc::clone(&auto_optimize_cancel_token);
    let monitor_weak = std::sync::Arc::downgrade(&game_monitor);
    let monitor_lang = std::sync::Arc::clone(&active_lang);
    game_monitor.set_on_launch(move |pid| {
        run_steam_auto_optimize_flow(
            &monitor_ui,
            &monitor_cfg_path,
            &monitor_cache,
            &monitor_in_game_cache,
            pid,
            &monitor_cancel_token,
            &monitor_weak,
            &monitor_lang,
        );
    });

    let monitor_ui_exit = main_window.as_weak();
    let monitor_exit_in_game = std::sync::Arc::clone(&in_game_applied_cache);
    game_monitor.set_on_exit(move |pid| {
        clear_in_game_applied(&monitor_ui_exit, &monitor_exit_in_game);
        let ui_weak = monitor_ui_exit.clone();
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(ui) = ui_weak.upgrade() {
                let lang = i18n::UiLanguage::from_code(ui.get_active_language().as_str());
                ui.set_status_message(i18n::steam_auto_closed_status(lang, pid).into());
            }
        });
    });

    if effective_steam_auto {
        game_monitor.start();
    }

    // Sincronização em tempo real (Arquivo JSON no disco -> UI)
    let sync_window_handle = main_window.as_weak();
    let sync_config_ref = active_config_path.clone();
    let sync_in_game_ref = std::sync::Arc::clone(&in_game_applied_cache);
    std::thread::spawn(move || {
        let mut last_modified = None;
        let mut tracked_path = std::path::PathBuf::new();
        let mut loop_count = 0usize;
        let mut last_preset_names: Vec<String> = initial_presets.iter().map(|p| p.name.to_string()).collect();
        let mut last_cfg_names: Vec<String> = initial_cfg_files.iter().map(|c| c.to_string()).collect();

        loop {
            std::thread::sleep(std::time::Duration::from_millis(500));

            // Monitora criação/remoção de arquivos nas pastas config/optimzations e config/cfg
            loop_count += 1;
            if loop_count % 3 == 0 {
                let runtime_snapshot = sync_in_game_ref.lock().map(|g| *g).ok();
                if let Some(state) = runtime_snapshot {
                    if state.any_applied() {
                        let cs2_running =
                            counterrush_core::windows_api::is_cs2_process_running_win32()
                                .unwrap_or(false);
                        if cs2_running {
                            if !state.cs2_seen_running {
                                if let Ok(mut guard) = sync_in_game_ref.lock() {
                                    guard.cs2_seen_running = true;
                                }
                            }
                        } else {
                            let launch_grace_elapsed = state
                                .applied_at
                                .map(|t| t.elapsed() >= std::time::Duration::from_secs(45))
                                .unwrap_or(true);
                            if state.cs2_seen_running || launch_grace_elapsed {
                                let expected_applied_at = state.applied_at;
                                let handle = sync_window_handle.clone();
                                let cache_ref = std::sync::Arc::clone(&sync_in_game_ref);
                                let _ = slint::invoke_from_event_loop(move || {
                                    if let Some(ui) = handle.upgrade() {
                                        if !ui.get_is_launching() {
                                            let cleared = if let Ok(mut guard) = cache_ref.lock() {
                                                if guard.applied_at == expected_applied_at {
                                                    guard.clear();
                                                    true
                                                } else {
                                                    false
                                                }
                                            } else {
                                                false
                                            };
                                            if cleared {
                                                InGameRuntimeState::default().apply_to_ui(&ui);
                                            }
                                        }
                                    }
                                });
                            }
                        }
                    }
                }

                let current_cfgs = scan_cfg_files();
                let current_cfg_names: Vec<String> = current_cfgs.iter().map(|c| c.to_string()).collect();
                if current_cfg_names != last_cfg_names {
                    last_cfg_names = current_cfg_names;
                    let _ = slint::invoke_from_event_loop({
                        let handle = sync_window_handle.clone();
                        move || {
                            if let Some(ui) = handle.upgrade() {
                                ui.set_available_cfg_files(slint::ModelRc::new(slint::VecModel::from(current_cfgs)));
                            }
                        }
                    });
                }

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
                                    let lang = i18n::UiLanguage::from_code(ui.get_active_language().as_str());
                                    enforce_neutral_no_profile_state(
                                        &ui,
                                        i18n::neutral_status_message(lang),
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
                                            ui.set_selected_preferred_cfg(preferred_cfg_from_config(&cfg).into());
                                        }
                                        ui.set_cfg_dropdown_open(false);
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
                                        if ui.get_is_restoring_defaults() {
                                            return;
                                        }
                                        if !ui.get_is_editing_launch_args() {
                                            let joined_args = cfg.launch_args.join(" ");
                                            if ui.get_launch_args_text() != joined_args.as_str() {
                                                ui.set_launch_args_text(joined_args.into());
                                            }
                                            let preferred_cfg = preferred_cfg_from_config(&cfg);
                                            if ui.get_selected_preferred_cfg() != preferred_cfg.as_str() {
                                                ui.set_selected_preferred_cfg(preferred_cfg.into());
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
    let command_sender = spawn_command_worker(
        main_window.as_weak(),
        active_config_path.clone(),
        applied_system_cache.clone(),
        in_game_applied_cache.clone(),
        active_lang.clone(),
    );

    // Callback para Lançamento do CS2 com Otimizações Personalizadas (test_custom_launch)
    let window_handle = main_window.as_weak();
    let launch_config_ref = active_config_path.clone();
    let launch_cache_ref = applied_system_cache.clone();
    let launch_in_game_ref = in_game_applied_cache.clone();
    let fallback_path = resolved_path.clone();
    let launch_cancel_token = auto_optimize_cancel_token.clone();
    let launch_lang = active_lang.clone();
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

        let lang = i18n::UiLanguage::from_code(ui.get_active_language().as_str());
        ui.set_is_launching(true);
        ui.set_auto_optimize_is_launch(true);
        ui.set_auto_optimize_is_restore(false);
        ui.set_auto_optimize_progress(0.0);
        ui.set_auto_optimize_step_name(i18n::launch_initial_step(lang).into());
        ui.set_auto_optimize_finished(false);
        ui.set_show_auto_optimize_modal(true);
        ui.set_status_message(i18n::launch_status_starting(lang).into());

        launch_cancel_token.store(false, std::sync::atomic::Ordering::SeqCst);
        let handle_for_thread = window_handle.clone();
        let target_cfg_path = launch_config_ref.clone();
        let cache_for_thread = launch_cache_ref.clone();
        let in_game_for_thread = launch_in_game_ref.clone();
        let fallback_cfg = fallback_path.clone();
        let token = launch_cancel_token.clone();
        let lang_ref = launch_lang.clone();
        std::thread::spawn(move || {
            run_launch_flow(
                &handle_for_thread,
                &target_cfg_path,
                &cache_for_thread,
                &in_game_for_thread,
                fallback_cfg,
                token,
                lang_ref,
            );
        });
    });

    // Inicializa controlador da view interna de gerenciamento de binds
    let binds_dirs = counterrush_core::BindDirectories::default();
    let binds_controller = binds::wire_binds_view(
        &main_window,
        binds_dirs.clone(),
        settings_path.clone(),
        initial_is_grid_view,
    );

    let window_handle = main_window.as_weak();
    let settings_path_for_view = settings_path.clone();
    let binds_ctrl_for_view = binds_controller.clone();
    main_window.on_content_view_mode_changed(move |grid_mode| {
        save_content_view_mode_setting(&settings_path_for_view, grid_mode);
        if let Some(ui) = window_handle.upgrade() {
            ui.set_content_is_grid_view(grid_mode);
            binds_ctrl_for_view.set_view_mode(&ui, grid_mode);
            let lang = i18n::UiLanguage::from_code(ui.get_active_language().as_str());
            ui.set_status_message(i18n::content_view_mode_changed_status(lang, grid_mode).into());
        }
    });

    // Callback para clique nos cards de ação
    let window_handle = main_window.as_weak();
    let binds_dirs_for_card = binds_dirs.clone();
    main_window.on_card_action_clicked(move |module_id| {
        if let Some(ui) = window_handle.upgrade() {
            if module_id.as_str() == "binds" {
                binds::refresh_existing_cfg_files(&ui, &binds_dirs_for_card);
                ui.set_show_binds_view(true);
            }
            let lang = i18n::UiLanguage::from_code(ui.get_active_language().as_str());
            ui.set_status_message(i18n::card_action_status(lang, module_id.as_str()).into());
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
            let lang = i18n::UiLanguage::from_code(ui.get_active_language().as_str());
            enforce_neutral_no_profile_state(
                &ui,
                i18n::profile_required_status_message(lang),
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

    // Callback para o botão "Restaurar Padrão" (1 clique: reverte otimizações aplicadas no Windows com modal de progresso)
    let restore_ui = main_window.as_weak();
    let restore_cfg_path = active_config_path.clone();
    let restore_cache = applied_system_cache.clone();
    let restore_in_game_cache = in_game_applied_cache.clone();
    let restore_lang = active_lang.clone();
    let restoring_guard = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    main_window.on_restore_defaults_clicked(move || {
        if restoring_guard.swap(true, std::sync::atomic::Ordering::SeqCst) {
            return;
        }
        let ui_weak = restore_ui.clone();
        let cfg_path_ref = restore_cfg_path.clone();
        let cache_ref = restore_cache.clone();
        let in_game_ref = restore_in_game_cache.clone();
        let lang_ref = restore_lang.clone();
        let guard_flag = restoring_guard.clone();

        let lang = i18n::UiLanguage::load_atomic(&lang_ref);
        let starting_msg = i18n::restore_defaults_starting(lang);
        let checking_step = i18n::restore_modal_checking(lang);
        post_ui(&ui_weak, move |ui| {
            ui.set_auto_optimize_is_launch(false);
            ui.set_auto_optimize_is_restore(true);
            ui.set_auto_optimize_progress(0.0);
            ui.set_auto_optimize_step_name(checking_step.into());
            ui.set_auto_optimize_stage_title("".into());
            ui.set_auto_optimize_stage_subtitle("".into());
            ui.set_auto_optimize_finished(false);
            ui.set_show_auto_optimize_modal(true);
            ui.set_is_restoring_defaults(true);
            ui.set_status_message(starting_msg.into());
        });

        std::thread::spawn(move || {
            let core = counterrush_core::OptimizerCore::default();
            let mut reverted_count = 0usize;
            let mut failed_kinds = Vec::new();
            let step_pause = std::time::Duration::from_millis(120);

            for (idx, &kind) in counterrush_core::PermanentTweakKind::ALL.iter().enumerate() {
                let progress = 0.10 + (idx as f32) * 0.09;
                let lang_step = i18n::UiLanguage::load_atomic(&lang_ref);
                update_auto_optimize_ui(
                    &ui_weak,
                    progress,
                    i18n::restore_step_label(
                        lang_step,
                        kind.id(),
                        i18n::RestoreStepStatusKind::Checking,
                    ),
                );
                if core.is_permanent_tweak_applied(kind).unwrap_or(false) {
                    update_auto_optimize_ui(
                        &ui_weak,
                        progress,
                        i18n::restore_step_label(
                            lang_step,
                            kind.id(),
                            i18n::RestoreStepStatusKind::Reverting,
                        ),
                    );
                    match core.rollback_permanent_tweak(kind) {
                        Ok(()) => {
                            reverted_count += 1;
                            if let Ok(mut guard) = cache_ref.write() {
                                guard.remove(&kind);
                            }
                        }
                        Err(_) => {
                            failed_kinds.push(kind);
                            if let Ok(mut guard) = cache_ref.write() {
                                guard.insert(kind);
                            }
                        }
                    }
                } else {
                    update_auto_optimize_ui(
                        &ui_weak,
                        progress,
                        i18n::restore_step_label(
                            lang_step,
                            kind.id(),
                            i18n::RestoreStepStatusKind::AlreadyDefault,
                        ),
                    );
                    if let Ok(mut guard) = cache_ref.write() {
                        guard.remove(&kind);
                    }
                }
                std::thread::sleep(step_pause);
            }

            let lang_timer = i18n::UiLanguage::load_atomic(&lang_ref);
            update_auto_optimize_ui(
                &ui_weak,
                0.90,
                i18n::restore_step_label(
                    lang_timer,
                    "apply_timer_resolution",
                    i18n::RestoreStepStatusKind::Checking,
                ),
            );
            if core.is_timer_registry_applied().unwrap_or(false) {
                update_auto_optimize_ui(
                    &ui_weak,
                    0.90,
                    i18n::restore_step_label(
                        lang_timer,
                        "apply_timer_resolution",
                        i18n::RestoreStepStatusKind::Reverting,
                    ),
                );
                if core.rollback_timer_all().is_ok() {
                    reverted_count += 1;
                }
            } else {
                update_auto_optimize_ui(
                    &ui_weak,
                    0.90,
                    i18n::restore_step_label(
                        lang_timer,
                        "apply_timer_resolution",
                        i18n::RestoreStepStatusKind::AlreadyDefault,
                    ),
                );
            }
            std::thread::sleep(step_pause);

            let lang_save = i18n::UiLanguage::load_atomic(&lang_ref);
            update_auto_optimize_ui(
                &ui_weak,
                0.96,
                i18n::restore_step_label(
                    lang_save,
                    "saving_profile",
                    i18n::RestoreStepStatusKind::Reverting,
                ),
            );

            let target_path = cfg_path_ref.read().map(|p| p.clone()).unwrap_or_default();
            let final_tweaks = if !target_path.as_os_str().is_empty() {
                let preserved_args =
                    counterrush_core::config::CustomOptimizationConfig::load_from_file(&target_path)
                        .map(|c| c.launch_args)
                        .unwrap_or_default();
                let mut final_cfg =
                    counterrush_core::config::CustomOptimizationConfig::all_disabled();
                final_cfg.launch_args = preserved_args;
                for &kind in &failed_kinds {
                    kind.set_enabled_in(&mut final_cfg, true);
                }
                let _ = final_cfg.save_to_file(&target_path);
                Tweaks::from_config(&final_cfg)
            } else {
                Tweaks::default()
            };
            std::thread::sleep(step_pause);

            let lang_done = i18n::UiLanguage::load_atomic(&lang_ref);
            let done_msg =
                i18n::restore_defaults_done(lang_done, reverted_count, failed_kinds.len());
            let modal_sub = done_msg.clone();
            let modal_step = done_msg.clone();

            if let Ok(mut guard) = in_game_ref.lock() {
                guard.clear();
            }

            post_ui(&ui_weak, move |ui| {
                final_tweaks.apply_to_ui(ui);
                InGameRuntimeState::default().apply_to_ui(ui);
                ui.set_auto_optimize_progress(1.0);
                ui.set_auto_optimize_stage_subtitle(modal_sub.into());
                ui.set_auto_optimize_step_name(modal_step.into());
                ui.set_auto_optimize_finished(true);
                ui.set_is_restoring_defaults(false);
                ui.set_status_message(done_msg.into());
            });

            guard_flag.store(false, std::sync::atomic::Ordering::SeqCst);
        });
    });

    // Callback para o botão "Otimizar Automaticamente" na aba Otimizações
    let auto_opt_ui = main_window.as_weak();
    let auto_opt_config_path = active_config_path.clone();
    let auto_opt_cache = applied_system_cache.clone();
    let auto_opt_cancel = auto_optimize_cancel_token.clone();
    let auto_opt_lang = active_lang.clone();
    main_window.on_auto_optimize_clicked(move || {
        if let Some(ui) = auto_opt_ui.upgrade() {
            let lang = i18n::UiLanguage::from_code(ui.get_active_language().as_str());
            ui.set_auto_optimize_is_launch(false);
            ui.set_auto_optimize_is_restore(false);
            ui.set_auto_optimize_stage_title("".into());
            ui.set_auto_optimize_stage_subtitle("".into());
            ui.set_auto_optimize_progress(0.0);
            ui.set_auto_optimize_step_name(i18n::auto_optimize_initial_step(lang).into());
            ui.set_auto_optimize_finished(false);
            ui.set_show_auto_optimize_modal(true);
        }
        auto_opt_cancel.store(false, std::sync::atomic::Ordering::SeqCst);
        let ui_weak = auto_opt_ui.clone();
        let cfg_path = auto_opt_config_path.clone();
        let cache = auto_opt_cache.clone();
        let token = auto_opt_cancel.clone();
        let lang_ref = auto_opt_lang.clone();
        std::thread::spawn(move || {
            run_auto_optimize_flow(&ui_weak, &cfg_path, &cache, token, lang_ref);
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

    // Callback para seleção de CFG preferida (+exec <arquivo.cfg>) na aba Otimizações
    let preferred_cfg_sender = command_sender.clone();
    main_window.on_preferred_cfg_selected(move |cfg_file| {
        let _ = preferred_cfg_sender.send(Command::SelectPreferredCfg {
            cfg_file: cfg_file.to_string(),
        });
    });

    let refresh_cfg_ui = main_window.as_weak();
    main_window.on_refresh_cfg_files_requested(move || {
        if let Some(ui) = refresh_cfg_ui.upgrade() {
            ui.set_available_cfg_files(slint::ModelRc::new(slint::VecModel::from(scan_cfg_files())));
        }
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
    let lang_state_for_select = active_lang.clone();
    let binds_ctrl_for_lang = binds_controller.clone();
    main_window.on_language_selected(move |code| {
        let ui = match window_handle.upgrade() {
            Some(ui) => ui,
            None => return,
        };
        let lang_enum = i18n::UiLanguage::from_code(code.as_str());
        let target_code = lang_enum.code();
        lang_enum.store_atomic(&lang_state_for_select);

        if let Err(e) = slint::select_bundled_translation(target_code) {
            eprintln!("Erro ao alternar idioma para '{target_code}': {e:?}");
        }
        ui.set_active_language(target_code.into());
        binds_ctrl_for_lang.refresh_for_language(&ui, lang_enum);

        if let Ok(mut settings) = counterrush_core::config::AppSettings::load_or_create(&settings_path_for_lang) {
            settings.language = target_code.to_string();
            let _ = settings.save_to_file(&settings_path_for_lang);
        }

        ui.set_status_message(i18n::language_changed_status(lang_enum, target_code).into());
    });

    // Callback para compartilhamento do projeto (cópia para o clipboard do Windows)
    let window_handle = main_window.as_weak();
    main_window.on_share_project_clicked(move || {
        let ui = match window_handle.upgrade() {
            Some(ui) => ui,
            None => return,
        };
        let lang = i18n::UiLanguage::from_code(ui.get_active_language().as_str());
        let share_text = i18n::share_project_text(lang);
        if let Err(e) = counterrush_core::set_clipboard(share_text) {
            eprintln!("Erro ao copiar link para o clipboard: {e}");
            let err_prefix = i18n::clipboard_err_prefix(lang);
            ui.set_status_message(format!("{err_prefix}: {e}").into());
        } else {
            ui.set_status_message(i18n::clipboard_copied_ok(lang).into());
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
                        let lang = i18n::UiLanguage::from_code(ui.get_active_language().as_str());
                        let err_prefix = i18n::open_url_err_prefix(lang);
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
                    let lang = i18n::UiLanguage::from_code(ui.get_active_language().as_str());
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
                            ui.set_status_message(
                                i18n::startup_toggled_status(lang, enabled, prev_steam_auto).into(),
                            );
                        }
                        Err(err) => {
                            ui.set_start_with_system(!enabled);
                            if !enabled && prev_steam_auto {
                                ui.set_optimize_on_steam_launch(true);
                                monitor.start();
                            }
                            let err_prefix = i18n::startup_toggled_err_prefix(lang);
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
                    let lang = i18n::UiLanguage::from_code(ui.get_active_language().as_str());
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
                            ui.set_status_message(
                                i18n::steam_auto_toggled_status(lang, enabled).into(),
                            );
                        }
                        Err(err) => {
                            ui.set_optimize_on_steam_launch(!enabled);
                            if enabled {
                                let actual_startup =
                                    counterrush_core::is_system_startup_enabled().unwrap_or(false);
                                ui.set_start_with_system(actual_startup);
                            }
                            let err_prefix = i18n::steam_auto_toggled_err_prefix(lang);
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

#[cfg(test)]
mod in_game_state_tests {
    use super::InGameRuntimeState;

    #[test]
    fn test_in_game_runtime_state_transitions() {
        let mut state = InGameRuntimeState::default();
        assert!(!state.any_applied());
        assert!(!state.cs2_seen_running);
        assert!(state.applied_at.is_none());

        state.timer = true;
        state.ram = true;
        assert!(state.any_applied());

        state.cpu = true;
        state.priority = true;
        state.cs2_seen_running = true;
        state.applied_at = Some(std::time::Instant::now());
        assert!(state.any_applied());

        state.clear();
        assert_eq!(state, InGameRuntimeState::default());
        assert!(!state.any_applied());
    }
}

#[cfg(test)]
mod content_view_mode_tests {
    use super::{ensure_content_view_mode_key_migrated, save_content_view_mode_setting};
    use counterrush_core::config::{AppSettings, ContentViewMode};

    #[test]
    fn test_ensure_content_view_mode_key_migrated_for_legacy_settings() {
        let temp = tempfile::tempdir().expect("Falha ao criar tempdir");
        let settings_path = temp.path().join("settings.json");

        std::fs::write(
            &settings_path,
            r#"{
  "language": "en_US",
  "start_with_system": true,
  "optimize_on_steam_launch": false,
  "initial_optimization_done": true,
  "minimize_to_tray": true,
  "active_preset": "AutoDetect"
}"#,
        )
        .unwrap();

        let loaded = AppSettings::load_or_create(&settings_path).unwrap();
        assert_eq!(loaded.content_view_mode, ContentViewMode::List);

        ensure_content_view_mode_key_migrated(&settings_path, &loaded);
        let raw = std::fs::read_to_string(&settings_path).unwrap();
        assert!(raw.contains("\"content_view_mode\": \"list\""));

        let reloaded = AppSettings::load_from_file(&settings_path).unwrap();
        assert_eq!(reloaded.language, "en_US");
        assert!(reloaded.start_with_system);
        assert!(reloaded.minimize_to_tray);
        assert_eq!(reloaded.active_preset.as_deref(), Some("AutoDetect"));
        assert_eq!(reloaded.content_view_mode, ContentViewMode::List);
    }

    #[test]
    fn test_save_content_view_mode_setting_updates_only_view_mode() {
        let temp = tempfile::tempdir().expect("Falha ao criar tempdir");
        let settings_path = temp.path().join("settings.json");

        let mut initial = AppSettings::default();
        initial.language = "ru_RU".to_string();
        initial.minimize_to_tray = true;
        initial.save_to_file(&settings_path).unwrap();

        save_content_view_mode_setting(&settings_path, true);
        let after_grid = AppSettings::load_from_file(&settings_path).unwrap();
        assert_eq!(after_grid.content_view_mode, ContentViewMode::Grid);
        assert_eq!(after_grid.language, "ru_RU");
        assert!(after_grid.minimize_to_tray);

        save_content_view_mode_setting(&settings_path, false);
        let after_list = AppSettings::load_from_file(&settings_path).unwrap();
        assert_eq!(after_list.content_view_mode, ContentViewMode::List);
        assert_eq!(after_list.language, "ru_RU");
    }
}
