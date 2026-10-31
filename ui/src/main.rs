#![windows_subsystem = "windows"]

slint::include_modules!();

fn scan_presets() -> Vec<PresetModel> {
    let files = counterrush_core::list_optimization_configs();
    let mut list = Vec::new();
    for p in files {
        if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
            let tag = match stem {
                "AutoDetect" => "HARDWARE ADAPTATIVO",
                "custom_optimization" => "CUSTOMIZADO / MANUAL",
                _ => "PERFIL DO USUÁRIO",
            };
            list.push(PresetModel {
                name: stem.into(),
                tag: tag.into(),
            });
        }
    }
    list
}

fn main() -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;

    // Inicializa a lista de perfis de otimização na interface a partir do disco
    let initial_presets = scan_presets();
    main_window.set_presets(slint::ModelRc::new(slint::VecModel::from(initial_presets.clone())));

    // Se não existirem perfis salvos no disco, inicia no estado neutro desligado (R1)
    let (resolved_path, _config) = if initial_presets.is_empty() {
        main_window.set_active_preset("".into());
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
        let core = counterrush_core::OptimizerCore::default();
        let (p, c) = match core.load_or_create_custom_config(None) {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Erro ao carregar configuração: {e}");
                (
                    counterrush_core::resolve_custom_config_path(None),
                    counterrush_core::config::CustomOptimizationConfig::default(),
                )
            }
        };

        if let Some(filename) = p.file_name().and_then(|s| s.to_str()) {
            let clean_name = filename.strip_suffix(".json").unwrap_or(filename);
            main_window.set_active_preset(clean_name.into());
        }

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
                    last_preset_names = current_names;
                    let _ = slint::invoke_from_event_loop({
                        let handle = sync_window_handle.clone();
                        move || {
                            if let Some(ui) = handle.upgrade() {
                                ui.set_presets(slint::ModelRc::new(slint::VecModel::from(current_presets)));
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

    // Callback para Lançamento do CS2 com Otimizações Personalizadas (test_custom_launch)
    let window_handle = main_window.as_weak();
    let launch_config_ref = active_config_path.clone();
    let fallback_path = resolved_path.clone();
    main_window.on_launch_cs2_clicked(move || {
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
        ) = match window_handle.upgrade() {
            Some(ui) => {
                ui.set_status_message("Otimizando sistema o CS2...".into());
                (
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
                )
            }
            None => return,
        };

        let handle_for_thread = window_handle.clone();
        let target_cfg_path = launch_config_ref.read().map(|p| p.clone()).unwrap_or_else(|_| fallback_path.clone());
        std::thread::spawn(move || {
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

    // Callback para alternância dos switches modulares (UI -> Arquivo JSON no disco)
    let window_handle = main_window.as_weak();
    let tweak_config_ref = active_config_path.clone();
    let fallback_path_tweak = resolved_path.clone();
    main_window.on_tweak_toggled(move |tweak_name, active| {
        if let Some(ui) = window_handle.upgrade() {
            let state = if active { "ativado" } else { "desativado" };
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
            let msg = format!("Módulo '{}' {}.", display_name, state);
            ui.set_status_message(msg.into());

            // Persiste a preferência atualizada no arquivo de configuração selecionado
            let target_path = tweak_config_ref.read().map(|p| p.clone()).unwrap_or_else(|_| fallback_path_tweak.clone());
            if target_path.as_os_str().is_empty() {
                ui.set_status_message("Crie ou selecione um perfil para salvar alterações.".into());
                return;
            }

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
        }
    });

    // Callback para ações em lote dos tweaks
    let bulk_config_ref = active_config_path.clone();
    let fallback_path_bulk = resolved_path.clone();
    main_window.on_bulk_tweaks_action(move |action| {
        let target_path = bulk_config_ref.read().map(|p| p.clone()).unwrap_or_else(|_| fallback_path_bulk.clone());
        if target_path.as_os_str().is_empty() {
            return;
        }

        let mut config = counterrush_core::config::CustomOptimizationConfig::load_from_file(&target_path)
            .unwrap_or_default();

        match action.as_str() {
            "activate_all" => {
                config.cs2_affinity = true;
                config.apply_timer_resolution = true;
                config.cs2_priority = true;
                config.empty_standby_list = true;
                config.disable_gpu_acceleration = true;
                config.apply_dpi_optimization = true;
                config.disable_memory_compression = true;
                config.apply_mmcss = true;
                config.disable_hpet = true;
                config.disable_game_bar = true;
                config.enable_msi = true;
                config.disable_power_saving = true;
                config.apply_render_queue = true;
            }
            "deactivate_all" => {
                config.cs2_affinity = false;
                config.apply_timer_resolution = false;
                config.cs2_priority = false;
                config.empty_standby_list = false;
                config.disable_gpu_acceleration = false;
                config.apply_dpi_optimization = false;
                config.disable_memory_compression = false;
                config.apply_mmcss = false;
                config.disable_hpet = false;
                config.disable_game_bar = false;
                config.enable_msi = false;
                config.disable_power_saving = false;
                config.apply_render_queue = false;
            }
            "reset_defaults" => {
                config = Default::default();
            }
            _ => {}
        }
        let _ = config.save_to_file(&target_path);
    });

    // Callback para o botão "Otimizar Automaticamente" na aba Otimizações
    let window_handle = main_window.as_weak();
    let auto_opt_ref = active_config_path.clone();
    main_window.on_auto_optimize_clicked(move || {
        if let Some(ui) = window_handle.upgrade() {
            ui.set_status_message("Detectando hardware e calibrando otimizações...".into());

            let core = counterrush_core::OptimizerCore::default();
            match core.save_autodetect_config(None) {
                Ok((path, config, report)) => {
                    if let Ok(mut current) = auto_opt_ref.write() {
                        *current = path.clone();
                    }

                    let updated_presets = scan_presets();
                    ui.set_presets(slint::ModelRc::new(slint::VecModel::from(updated_presets)));
                    ui.set_active_preset("AutoDetect".into());

                    ui.set_cpu_tweak_active(config.cs2_affinity);
                    ui.set_timer_tweak_active(config.apply_timer_resolution);
                    ui.set_priority_tweak_active(config.cs2_priority);
                    ui.set_ram_tweak_active(config.empty_standby_list);
                    ui.set_gpu_accel_tweak_active(config.disable_gpu_acceleration);
                    ui.set_dpi_tweak_active(config.apply_dpi_optimization);
                    ui.set_mem_compression_tweak_active(config.disable_memory_compression);
                    ui.set_mmcss_tweak_active(config.apply_mmcss);
                    ui.set_hpet_tweak_active(config.disable_hpet);
                    ui.set_game_bar_tweak_active(config.disable_game_bar);
                    ui.set_msi_tweak_active(config.enable_msi);
                    ui.set_power_saving_tweak_active(config.disable_power_saving);
                    ui.set_render_queue_tweak_active(config.apply_render_queue);

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
                    ui.set_status_message(msg.into());
                }
                Err(err) => {
                    ui.set_status_message(format!("Falha ao detectar hardware automaticamente: {err}").into());
                }
            }
        }
    });

    // Callback para seleção de perfis de otimização (seleção única na aba Otimizações)
    let window_handle = main_window.as_weak();
    let preset_config_ref = active_config_path.clone();
    main_window.on_preset_selected(move |preset_name| {
        if let Some(ui) = window_handle.upgrade() {
            let target_path = counterrush_core::resolve_optimization_file_path(preset_name.as_str());

            // Sincroniza o caminho ativo imediatamente
            if let Ok(mut current) = preset_config_ref.write() {
                *current = target_path.clone();
            }

            match counterrush_core::config::CustomOptimizationConfig::load_from_file(&target_path) {
                Ok(cfg) => {
                    ui.set_cpu_tweak_active(cfg.cs2_affinity);
                    ui.set_timer_tweak_active(cfg.apply_timer_resolution);
                    ui.set_priority_tweak_active(cfg.cs2_priority);
                    ui.set_ram_tweak_active(cfg.empty_standby_list);
                    ui.set_gpu_accel_tweak_active(cfg.disable_gpu_acceleration);
                    ui.set_dpi_tweak_active(cfg.apply_dpi_optimization);
                    ui.set_mem_compression_tweak_active(cfg.disable_memory_compression);
                    ui.set_mmcss_tweak_active(cfg.apply_mmcss);
                    ui.set_hpet_tweak_active(cfg.disable_hpet);
                    ui.set_game_bar_tweak_active(cfg.disable_game_bar);
                    ui.set_msi_tweak_active(cfg.enable_msi);
                    ui.set_power_saving_tweak_active(cfg.disable_power_saving);
                    ui.set_render_queue_tweak_active(cfg.apply_render_queue);

                    ui.set_status_message(format!("Perfil '{}' selecionado e carregado com sucesso.", preset_name.as_str()).into());
                }
                Err(err) => {
                    ui.set_status_message(format!("Erro ao carregar perfil '{}': {err}", preset_name.as_str()).into());
                }
            }
        }
    });

    // Callback para criação e ativação imediata de novo perfil de otimização (.json)
    let window_handle = main_window.as_weak();
    let create_config_ref = active_config_path.clone();
    main_window.on_create_profile_confirmed(move |data| {
        if let Some(ui) = window_handle.upgrade() {
            let clean_name = match counterrush_core::validate_profile_name(data.name.as_str()) {
                Ok(name) => name,
                Err(err_msg) => {
                    ui.set_create_modal_error(err_msg.into());
                    return;
                }
            };

            let launch_args: Vec<String> = data
                .launch_args
                .as_str()
                .split_whitespace()
                .map(|s| s.to_string())
                .collect();

            let new_config = counterrush_core::config::CustomOptimizationConfig {
                launch_args,
                empty_standby_list: data.empty_standby_list,
                cs2_priority: data.cs2_priority,
                cs2_affinity: data.cs2_affinity,
                disable_gpu_acceleration: data.disable_gpu_acceleration,
                apply_dpi_optimization: data.apply_dpi_optimization,
                disable_memory_compression: data.disable_memory_compression,
                apply_timer_resolution: data.apply_timer_resolution,
                apply_mmcss: data.apply_mmcss,
                disable_hpet: data.disable_hpet,
                disable_game_bar: data.disable_game_bar,
                enable_msi: data.enable_msi,
                disable_power_saving: data.disable_power_saving,
                apply_render_queue: data.apply_render_queue,
            };

            let target_path = counterrush_core::resolve_optimization_file_path(&clean_name);
            match new_config.save_to_file(&target_path) {
                Ok(_) => {
                    if let Ok(mut current) = create_config_ref.write() {
                        *current = target_path.clone();
                    }

                    let updated_presets = scan_presets();
                    ui.set_presets(slint::ModelRc::new(slint::VecModel::from(updated_presets)));
                    ui.set_active_preset(clean_name.as_str().into());

                    ui.set_cpu_tweak_active(new_config.cs2_affinity);
                    ui.set_timer_tweak_active(new_config.apply_timer_resolution);
                    ui.set_priority_tweak_active(new_config.cs2_priority);
                    ui.set_ram_tweak_active(new_config.empty_standby_list);
                    ui.set_gpu_accel_tweak_active(new_config.disable_gpu_acceleration);
                    ui.set_dpi_tweak_active(new_config.apply_dpi_optimization);
                    ui.set_mem_compression_tweak_active(new_config.disable_memory_compression);
                    ui.set_mmcss_tweak_active(new_config.apply_mmcss);
                    ui.set_hpet_tweak_active(new_config.disable_hpet);
                    ui.set_game_bar_tweak_active(new_config.disable_game_bar);
                    ui.set_msi_tweak_active(new_config.enable_msi);
                    ui.set_power_saving_tweak_active(new_config.disable_power_saving);
                    ui.set_render_queue_tweak_active(new_config.apply_render_queue);

                    ui.set_create_modal_error("".into());
                    ui.set_show_create_modal(false);
                    ui.set_status_message(format!("Perfil '{clean_name}' criado e selecionado com sucesso!").into());
                }
                Err(e) => {
                    ui.set_create_modal_error(format!("Falha ao salvar arquivo: {e}").into());
                }
            }
        }
    });

    main_window.run()
}
