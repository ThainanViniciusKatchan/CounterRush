#![windows_subsystem = "windows"]

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let main_window = MainWindow::new()?;

    // Callback para Lançamento do CS2
    let window_handle = main_window.as_weak();
    main_window.on_launch_cs2_clicked(move || {
        if let Some(ui) = window_handle.upgrade() {
            ui.set_status_message("Comando enviado: CS2 iniciando com alta performance...".into());
        }
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
