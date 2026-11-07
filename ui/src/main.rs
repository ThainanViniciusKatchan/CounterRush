#![windows_subsystem = "windows"]

slint::include_modules!();

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

/// Comandos enfileirados pela thread da janela e executados pela thread executora.
enum Command {
    TweakToggled { name: String, active: bool },
    BulkTweaksAction { action: String },
    AutoOptimize,
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
        Command::AutoOptimize => run_auto_optimize(ui, config_path),
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
        post_ui(ui, |ui| {
            ui.set_status_message("Crie ou selecione um perfil para salvar alterações.".into())
        });
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
    _ui: &slint::Weak<MainWindow>,
    config_path: &std::sync::Arc<std::sync::RwLock<std::path::PathBuf>>,
    action: String,
) {
    let target_path = config_path.read().map(|p| p.clone()).unwrap_or_default();
    if target_path.as_os_str().is_empty() {
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

fn run_auto_optimize(
    ui: &slint::Weak<MainWindow>,
    config_path: &std::sync::Arc<std::sync::RwLock<std::path::PathBuf>>,
) {
    post_ui(ui, |ui| {
        ui.set_status_message("Detectando hardware e calibrando otimizações...".into())
    });

    let core = counterrush_core::OptimizerCore::default();
    match core.save_autodetect_config(None) {
        Ok((path, config, report)) => {
            if let Ok(mut current) = config_path.write() {
                *current = path.clone();
            }

            let updated_presets = scan_presets();
            let args_text = config.launch_args.join(" ");
            let tweaks = Tweaks::from_config(&config);
            let msg = if report.blocked_tweaks.is_empty() {
                format!(
                    "Hardware detectado ({:?} • {}). Perfil AutoDetect gerado e aplicado!",
                    report.category, report.hardware.gpu.model
                )
            } else {
                format!(
                    "Hardware detectado ({:?} • {}). Perfil AutoDetect gerado ({} ajustes calibrados para seu PC).",
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
            });
        }
        Err(err) => post_ui(ui, move |ui| {
            ui.set_status_message(
                format!("Falha ao detectar hardware automaticamente: {err}").into(),
            )
        }),
    }
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
        post_ui(ui, |ui| {
            ui.set_status_message("Crie ou selecione um perfil para salvar alterações.".into())
        });
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

    // Inicialização de AppSettings e seleção dinâmica de idiomas
    let settings_path = counterrush_core::resolve_app_settings_path();
    let app_settings = counterrush_core::config::AppSettings::load_or_create(&settings_path)
        .unwrap_or_default();

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
        main_window.set_cpu_tweak_active(false);
        main_window.set_timer_tweak_active(false);
        main_window.set_priority_tweak_active(false);
        main_window.set_ram_tweak_active(false);
        main_window.set_gpu_accel_tweak_active(false);
        main_window.set_dpi_tweak_active(false);
        main_window.set_mem_compression_tweak_active(false);
        main_window.set_mmcss_tweak_active(false);
        main_window.set_hpet_tweak_active(false);
        main_window.set_game_bar_tweak_active(false);
        main_window.set_msi_tweak_active(false);
        main_window.set_power_saving_tweak_active(false);
        main_window.set_render_queue_tweak_active(false);
        (std::path::PathBuf::new(), counterrush_core::config::CustomOptimizationConfig::all_disabled())
    } else {

        let first_preset_name = &initial_presets[0].name;
        let p = counterrush_core::resolve_optimization_file_path(first_preset_name.as_str());
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
        main_window.set_cpu_tweak_active(c.cs2_affinity);
        main_window.set_timer_tweak_active(c.apply_timer_resolution);
        main_window.set_priority_tweak_active(c.cs2_priority);
        main_window.set_ram_tweak_active(c.empty_standby_list);
        main_window.set_gpu_accel_tweak_active(c.disable_gpu_acceleration);
        main_window.set_dpi_tweak_active(c.apply_dpi_optimization);
        main_window.set_mem_compression_tweak_active(c.disable_memory_compression);
        main_window.set_mmcss_tweak_active(c.apply_mmcss);
        main_window.set_hpet_tweak_active(c.disable_hpet);
        main_window.set_game_bar_tweak_active(c.disable_game_bar);
        main_window.set_msi_tweak_active(c.enable_msi);
        main_window.set_power_saving_tweak_active(c.disable_power_saving);
        main_window.set_render_queue_tweak_active(c.apply_render_queue);
        (p, c)
    };

    let active_config_path = std::sync::Arc::new(std::sync::RwLock::new(resolved_path.clone()));

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
                                    if let Ok(mut cur) = sync_active_ref.write() {
                                        *cur = std::path::PathBuf::new();
                                    }
                                } else {
                                    let current_active = ui.get_active_preset().to_string();
                                    if !names_clone.contains(&current_active) {
                                        let first_preset = &names_clone[0];
                                        let target = counterrush_core::resolve_optimization_file_path(first_preset);
                                        if let Ok(mut cur) = sync_active_ref.write() {
                                            *cur = target.clone();
                                        }
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
    main_window.on_launch_cs2_clicked(move || {
        let ui = match window_handle.upgrade() {
            Some(ui) => ui,
            None => return,
        };

        ui.set_status_message("Otimizando sistema o CS2...".into());
        let (
            cpu_active,
            timer_active,
            priority_active,
            ram_active,
            gpu_active,
            dpi_active,
            mem_active,
            mmcss_active,
            hpet_active,
            game_bar_active,
            msi_active,
            power_active,
            render_queue_active,
        ) = (
            ui.get_cpu_tweak_active(),
            ui.get_timer_tweak_active(),
            ui.get_priority_tweak_active(),
            ui.get_ram_tweak_active(),
            ui.get_gpu_accel_tweak_active(),
            ui.get_dpi_tweak_active(),
            ui.get_mem_compression_tweak_active(),
            ui.get_mmcss_tweak_active(),
            ui.get_hpet_tweak_active(),
            ui.get_game_bar_tweak_active(),
            ui.get_msi_tweak_active(),
            ui.get_power_saving_tweak_active(),
            ui.get_render_queue_tweak_active(),
        );

        let handle_for_thread = window_handle.clone();
        let target_cfg_path = launch_config_ref.read().map(|p| p.clone()).unwrap_or_else(|_| fallback_path.clone());
        std::thread::spawn(move || {
            // Salvaguarda defensiva: Se não houver perfis, bloqueia o lançamento e exibe o modal
            if scan_presets().is_empty() {
                let _ = slint::invoke_from_event_loop({
                    let handle = handle_for_thread.clone();
                    move || {
                        if let Some(ui) = handle.upgrade() {
                            ui.set_is_launching(false);
                            ui.set_show_no_profile_modal(true);
                        }
                    }
                });
                return;
            }

            let core = counterrush_core::OptimizerCore::default();
            let (resolved, mut config) = match core.load_or_create_custom_config(Some(&target_cfg_path)) {
                Ok(res) => res,
                Err(err) => {
                    let err_msg = format!("Erro na configuração: {err}");
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = handle_for_thread.upgrade() {
                            ui.set_status_message(err_msg.into());
                            ui.set_is_launching(false);
                        }
                    });
                    return;
                }
            };

            // Sincroniza a configuração com as escolhas da interface
            config.cs2_affinity = cpu_active;
            config.apply_timer_resolution = timer_active;
            config.cs2_priority = priority_active;
            config.empty_standby_list = ram_active;
            config.disable_gpu_acceleration = gpu_active;
            config.apply_dpi_optimization = dpi_active;
            config.disable_memory_compression = mem_active;
            config.apply_mmcss = mmcss_active;
            config.disable_hpet = hpet_active;
            config.disable_game_bar = game_bar_active;
            config.enable_msi = msi_active;
            config.disable_power_saving = power_active;
            config.apply_render_queue = render_queue_active;
            let _ = config.save_to_file(&resolved);

            let _ = slint::invoke_from_event_loop({
                let handle = handle_for_thread.clone();
                move || {
                    if let Some(ui) = handle.upgrade() {
                        ui.set_status_message("Aplicando tweaks e inicializando CS2 via Steam...".into());
                    }
                }
            });

            match core.launch_custom(Some(&resolved), std::time::Duration::from_secs(90)) {
                Ok(report) => {
                    let count_applied = report.applied_tweaks.len();
                    let count_skipped = report.skipped_tweaks.len();
                    let msg = format!(
                        "Sistema otimizado com sucesso! ({} tweaks aplicados, {} pulados)",
                        count_applied, count_skipped
                    );
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = handle_for_thread.upgrade() {
                            ui.set_status_message(msg.into());
                            ui.set_is_launching(false);
                        }
                    });
                }
                Err(err) => {
                    let msg = format!("Falha ao iniciar CS2: {err}");
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = handle_for_thread.upgrade() {
                            ui.set_status_message(msg.into());
                            ui.set_is_launching(false);
                        }
                    });
                }
            }
        });
    });

    // Callback para clique nos cards de ação
    let window_handle = main_window.as_weak();
    main_window.on_card_action_clicked(move |module_id| {
        if let Some(ui) = window_handle.upgrade() {
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

    // Callback para ações em lote dos tweaks (UI -> fila de comandos)
    let bulk_sender = command_sender.clone();
    main_window.on_bulk_tweaks_action(move |action| {
        let _ = bulk_sender.send(Command::BulkTweaksAction {
            action: action.to_string(),
        });
    });

    // Callback para o botão "Otimizar Automaticamente" na aba Otimizações
    let auto_sender = command_sender.clone();
    main_window.on_auto_optimize_clicked(move || {
        let _ = auto_sender.send(Command::AutoOptimize);
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

    main_window.run()
}
