#![windows_subsystem = "windows"]

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;

    // Callback para Lançamento do CS2 com Otimizações Personalizadas (test_custom_launch)
    let window_handle = main_window.as_weak();
    main_window.on_launch_cs2_clicked(move || {
        let (cpu_active, timer_active, priority_active, ram_active) = match window_handle.upgrade() {
            Some(ui) => {
                ui.set_status_message("Iniciando CS2 com otimizações personalizadas...".into());
                (
                    ui.get_cpu_tweak_active(),
                    ui.get_timer_tweak_active(),
                    ui.get_priority_tweak_active(),
                    ui.get_ram_tweak_active(),
                )
            }
            None => return,
        };

        let handle_for_thread = window_handle.clone();
        std::thread::spawn(move || {
            let core = counterrush_core::OptimizerCore::default();
            let (resolved, mut config) = match core.load_or_create_custom_config(None) {
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
                        "CS2 iniciado com sucesso! ({} tweaks ativos, {} pulados)",
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

    // Callback para alternância dos switches modulares
    let window_handle = main_window.as_weak();
    main_window.on_tweak_toggled(move |tweak_name, active| {
        if let Some(ui) = window_handle.upgrade() {
            let state = if active { "ativado" } else { "desativado" };
            let msg = format!("Ajuste rápido '{}' {}.", tweak_name, state);
            ui.set_status_message(msg.into());
        }
    });

    main_window.run()
}
