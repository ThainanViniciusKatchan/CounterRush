use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use counterrush_core::{
    cs2_bind_catalog, determine_bind_save_mode, find_key_conflict_in_content,
    format_bind_commands, format_bind_line, list_cfg_files_in_dir, normalize_captured_key_to_cs2,
    save_cs2_bind_with_local_fallback, validate_bind_key, BindDirectories, BindFlag,
    BindOperationOutcome,
};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

slint::include_modules!();

fn build_catalog_rows() -> Vec<BindCatalogRow> {
    let catalog = cs2_bind_catalog();
    let mut rows = Vec::with_capacity(catalog.len() + 32);
    let mut last_category: Option<&str> = None;
    let mut last_subcategory: Option<&str> = None;

    for (idx, entry) in catalog.iter().enumerate() {
        if last_category != Some(entry.category) {
            rows.push(BindCatalogRow {
                row_kind: 0,
                command_index: -1,
                title: entry.category.into(),
                subtitle: entry.category_note.into(),
                command_value: SharedString::default(),
                checked: false,
            });
            last_category = Some(entry.category);
            last_subcategory = None;
        }

        if last_subcategory != Some(entry.subcategory) {
            rows.push(BindCatalogRow {
                row_kind: 1,
                command_index: -1,
                title: entry.subcategory.into(),
                subtitle: SharedString::default(),
                command_value: SharedString::default(),
                checked: false,
            });
            last_subcategory = Some(entry.subcategory);
        }

        rows.push(BindCatalogRow {
            row_kind: 2,
            command_index: idx as i32,
            title: entry.display_command.into(),
            subtitle: entry.description.into(),
            command_value: entry.cfg_command.into(),
            checked: false,
        });
    }

    rows
}

/// Função dedicada de captura e conversão de eventos de teclado Slint para códigos CS2 (Source 2).
fn capture_key_from_slint_text(raw_text: &str) -> Option<String> {
    if raw_text.is_empty() {
        return None;
    }

    let mut chars = raw_text.chars();
    if let (Some(ch), None) = (chars.next(), chars.next()) {
        use slint::platform::Key;
        let special = if ch == char::from(Key::Space) {
            Some("space")
        } else if ch == char::from(Key::Return) {
            Some("enter")
        } else if ch == char::from(Key::Escape) {
            Some("escape")
        } else if ch == char::from(Key::Tab) {
            Some("tab")
        } else if ch == char::from(Key::Backspace) {
            Some("backspace")
        } else if ch == char::from(Key::Delete) {
            Some("del")
        } else if ch == char::from(Key::Insert) {
            Some("ins")
        } else if ch == char::from(Key::Home) {
            Some("home")
        } else if ch == char::from(Key::End) {
            Some("end")
        } else if ch == char::from(Key::PageUp) {
            Some("pgup")
        } else if ch == char::from(Key::PageDown) {
            Some("pgdn")
        } else if ch == char::from(Key::UpArrow) {
            Some("uparrow")
        } else if ch == char::from(Key::DownArrow) {
            Some("downarrow")
        } else if ch == char::from(Key::LeftArrow) {
            Some("leftarrow")
        } else if ch == char::from(Key::RightArrow) {
            Some("rightarrow")
        } else if ch == char::from(Key::Shift) {
            Some("shift")
        } else if ch == char::from(Key::ShiftR) {
            Some("rshift")
        } else if ch == char::from(Key::Control) {
            Some("ctrl")
        } else if ch == char::from(Key::ControlR) {
            Some("rctrl")
        } else if ch == char::from(Key::Alt) {
            Some("alt")
        } else if ch == char::from(Key::AltGr) {
            Some("ralt")
        } else if ch == char::from(Key::CapsLock) {
            Some("capslock")
        } else if ch == char::from(Key::ScrollLock) {
            Some("scrolllock")
        } else if ch == char::from(Key::Pause) {
            Some("pause")
        } else if ch == char::from(Key::F1) {
            Some("f1")
        } else if ch == char::from(Key::F2) {
            Some("f2")
        } else if ch == char::from(Key::F3) {
            Some("f3")
        } else if ch == char::from(Key::F4) {
            Some("f4")
        } else if ch == char::from(Key::F5) {
            Some("f5")
        } else if ch == char::from(Key::F6) {
            Some("f6")
        } else if ch == char::from(Key::F7) {
            Some("f7")
        } else if ch == char::from(Key::F8) {
            Some("f8")
        } else if ch == char::from(Key::F9) {
            Some("f9")
        } else if ch == char::from(Key::F10) {
            Some("f10")
        } else if ch == char::from(Key::F11) {
            Some("f11")
        } else if ch == char::from(Key::F12) {
            Some("f12")
        } else if ch == char::from(Key::F13) {
            Some("f13")
        } else if ch == char::from(Key::F14) {
            Some("f14")
        } else if ch == char::from(Key::F15) {
            Some("f15")
        } else if ch == char::from(Key::F16) {
            Some("f16")
        } else if ch == char::from(Key::F17) {
            Some("f17")
        } else if ch == char::from(Key::F18) {
            Some("f18")
        } else if ch == char::from(Key::F19) {
            Some("f19")
        } else if ch == char::from(Key::F20) {
            Some("f20")
        } else if ch == char::from(Key::F21) {
            Some("f21")
        } else if ch == char::from(Key::F22) {
            Some("f22")
        } else if ch == char::from(Key::F23) {
            Some("f23")
        } else if ch == char::from(Key::F24) {
            Some("f24")
        } else {
            None
        };

        if let Some(code) = special {
            return Some(code.to_string());
        }
    }

    normalize_captured_key_to_cs2(raw_text)
}

fn collect_checked_commands(model: &ModelRc<BindCatalogRow>) -> Vec<String> {
    let mut selected = Vec::new();
    for idx in 0..model.row_count() {
        if let Some(row) = model.row_data(idx) {
            if row.row_kind == 2 && row.checked {
                selected.push(row.command_value.to_string());
            }
        }
    }
    selected
}

fn load_cfg_names_for_dir(dir: &Path) -> Vec<SharedString> {
    list_cfg_files_in_dir(dir)
        .into_iter()
        .filter_map(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(SharedString::from)
        })
        .collect()
}

fn sync_window_summary(ui: &BindsTestWindow) {
    let selected_existing = ui.get_selected_existing_file();
    let file_input = ui.get_file_name_input();

    if selected_existing.trim().is_empty() {
        ui.set_is_creating_new_file(true);
        ui.set_save_mode_badge("NOVO ARQUIVO".into());
    } else if file_input.trim().is_empty() {
        ui.set_is_creating_new_file(true);
        ui.set_save_mode_badge("DIGITE O NOME".into());
    } else {
        match determine_bind_save_mode(selected_existing.as_str(), file_input.as_str()) {
            Ok((_, BindFlag::Default)) => {
                ui.set_is_creating_new_file(false);
                ui.set_save_mode_badge("ATUALIZAR ATUAL".into());
            }
            Ok((_, BindFlag::NewConfig)) => {
                ui.set_is_creating_new_file(true);
                ui.set_save_mode_badge("CRIAR NOVO".into());
            }
            Err(_) | Ok((_, BindFlag::Restore)) => {
                ui.set_is_creating_new_file(true);
                ui.set_save_mode_badge("NOME INVÁLIDO".into());
            }
        }
    }

    let selected_cmds = collect_checked_commands(&ui.get_catalog_rows());
    ui.set_selected_commands_count(selected_cmds.len() as i32);

    let key = ui.get_bound_key();
    if !key.trim().is_empty() && !selected_cmds.is_empty() {
        if let (Ok(clean_key), Ok(unified)) = (
            validate_bind_key(key.as_str()),
            format_bind_commands(&selected_cmds),
        ) {
            ui.set_bind_preview_line(format_bind_line(&clean_key, &unified).into());
            return;
        }
    }
    ui.set_bind_preview_line("".into());
}

fn check_existing_key_conflict(
    dirs: &BindDirectories,
    file_name: &str,
    key: &str,
) -> Option<(String, String)> {
    for base in [&dirs.local_cfg_dir, &dirs.cs2_cfg_dir] {
        let candidate = base.join(file_name);
        if candidate.is_file() {
            if let Ok(content) = std::fs::read_to_string(&candidate) {
                if let Some(existing_cmd) = find_key_conflict_in_content(&content, key) {
                    return Some((candidate.display().to_string(), existing_cmd));
                }
            }
        }
    }
    None
}

fn execute_save_bind(
    ui: &BindsTestWindow,
    dirs: &BindDirectories,
    allow_conflict_overwrite: bool,
) {
    let file_input = ui.get_file_name_input().to_string();
    let selected_existing = ui.get_selected_existing_file().to_string();
    let raw_key = ui.get_bound_key().to_string();
    let selected_cmds = collect_checked_commands(&ui.get_catalog_rows());

    if file_input.trim().is_empty() {
        ui.set_status_kind("warning".into());
        ui.set_status_message(
            "Informe um nome para o arquivo .cfg ou selecione um arquivo existente.".into(),
        );
        return;
    }

    let clean_key = match validate_bind_key(&raw_key) {
        Ok(k) => k,
        Err(_) => {
            ui.set_status_kind("warning".into());
            ui.set_status_message(
                "Clique no campo de tecla e pressione a tecla desejada antes de salvar.".into(),
            );
            return;
        }
    };

    if selected_cmds.is_empty() {
        ui.set_status_kind("warning".into());
        ui.set_status_message("Marque ao menos um comando na listagem abaixo para salvar.".into());
        return;
    }

    let (target_file_name, flag) = match determine_bind_save_mode(&selected_existing, &file_input) {
        Ok(res) => res,
        Err(err) => {
            ui.set_status_kind("error".into());
            ui.set_status_message(format!("Nome de arquivo inválido: {err}").into());
            return;
        }
    };

    let unified_cmd = match format_bind_commands(&selected_cmds) {
        Ok(c) => c,
        Err(err) => {
            ui.set_status_kind("error".into());
            ui.set_status_message(format!("Erro nos comandos marcados: {err}").into());
            return;
        }
    };

    if flag == BindFlag::Default && !allow_conflict_overwrite {
        if let Some((_, existing_cmd)) =
            check_existing_key_conflict(dirs, &target_file_name, &clean_key)
        {
            ui.set_conflict_message(
                format!(
                    "A tecla '{}' já possui a bind \"{}\" em '{}'. Substituir por \"{}\"?",
                    clean_key, existing_cmd, target_file_name, unified_cmd
                )
                .into(),
            );
            ui.set_show_conflict_prompt(true);
            ui.set_status_kind("warning".into());
            ui.set_status_message("Confirme a substituição da tecla conflitante acima.".into());
            return;
        }
    }

    match save_cs2_bind_with_local_fallback(
        dirs,
        &target_file_name,
        &clean_key,
        &selected_cmds,
        flag,
        |_| Ok(allow_conflict_overwrite),
    ) {
        Ok(report) => {
            let refreshed = load_cfg_names_for_dir(&dirs.local_cfg_dir);
            ui.set_existing_cfg_files(ModelRc::new(VecModel::from(refreshed)));

            let saved_file = match &report.outcome {
                BindOperationOutcome::Created { file_name, .. }
                | BindOperationOutcome::Updated { file_name, .. } => file_name.clone(),
                _ => target_file_name.clone(),
            };

            ui.set_selected_existing_file(saved_file.clone().into());
            ui.set_file_name_input(saved_file.into());
            ui.set_show_conflict_prompt(false);
            sync_window_summary(ui);

            let msg = match (&report.outcome, &report.cs2_sync_warning) {
                (
                    BindOperationOutcome::Created {
                        file_name,
                        bind_line,
                        ..
                    },
                    None,
                ) => format!(
                    "Novo arquivo '{file_name}' criado com {bind_line} (local + CS2)."
                ),
                (
                    BindOperationOutcome::Created {
                        file_name,
                        bind_line,
                        ..
                    },
                    Some(warn),
                ) => format!("Novo arquivo '{file_name}' criado com {bind_line}. {warn}"),
                (
                    BindOperationOutcome::Updated {
                        file_name,
                        bind_line,
                        replaced_existing_key,
                        ..
                    },
                    None,
                ) => {
                    let verb = if flag == BindFlag::NewConfig {
                        "Novo arquivo salvo"
                    } else if *replaced_existing_key {
                        "Bind substituída"
                    } else {
                        "Arquivo atualizado"
                    };
                    format!("{verb} em '{file_name}': {bind_line} (local + CS2).")
                }
                (
                    BindOperationOutcome::Updated {
                        file_name,
                        bind_line,
                        replaced_existing_key,
                        ..
                    },
                    Some(warn),
                ) => {
                    let verb = if flag == BindFlag::NewConfig {
                        "Novo arquivo salvo"
                    } else if *replaced_existing_key {
                        "Bind substituída"
                    } else {
                        "Arquivo atualizado"
                    };
                    format!("{verb} em '{file_name}': {bind_line}. {warn}")
                }
                (BindOperationOutcome::CancelledByUser(conflict), _) => {
                    format!(
                        "Operação cancelada para tecla '{}' em '{}'.",
                        conflict.key,
                        conflict.file_path.display()
                    )
                }
                (BindOperationOutcome::Restored { file_name, .. }, _) => {
                    format!("Backup de '{file_name}' restaurado.")
                }
            };

            ui.set_status_kind(if report.synced_to_cs2_global {
                "success".into()
            } else {
                "warning".into()
            });
            ui.set_status_message(msg.into());
        }
        Err(err) => {
            ui.set_status_kind("error".into());
            ui.set_status_message(format!("Erro ao salvar bind: {err}").into());
        }
    }
}

fn wire_binds_window(ui: &BindsTestWindow, dirs: BindDirectories) {
    let rows_model = Rc::new(VecModel::from(build_catalog_rows()));
    ui.set_catalog_rows(ModelRc::from(rows_model.clone()));

    let cfg_names = load_cfg_names_for_dir(&dirs.local_cfg_dir);
    ui.set_existing_cfg_files(ModelRc::new(VecModel::from(cfg_names)));
    sync_window_summary(ui);

    ui.on_resolve_cs2_key(|raw_text| {
        capture_key_from_slint_text(raw_text.as_str())
            .map(SharedString::from)
            .unwrap_or_default()
    });

    let ui_weak = ui.as_weak();
    ui.on_existing_file_selected(move |selected| {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_show_conflict_prompt(false);
            sync_window_summary(&ui);
            ui.set_status_kind("info".into());
            if selected.trim().is_empty() {
                ui.set_status_message(
                    "Seleção limpa. Digite um nome no campo de arquivo para criar um novo .cfg."
                        .into(),
                );
            } else {
                ui.set_status_message(
                    format!(
                        "Arquivo '{}' carregado. Mantenha o nome para atualizar ou altere para criar um novo arquivo.",
                        selected
                    )
                    .into(),
                );
            }
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_file_name_edited(move |_| {
        if let Some(ui) = ui_weak.upgrade() {
            sync_window_summary(&ui);
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_key_captured(move |captured| {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_show_conflict_prompt(false);
            sync_window_summary(&ui);
            ui.set_status_kind("info".into());
            if captured.trim().is_empty() {
                ui.set_status_message("Tecla removida. Clique no campo para capturar outra.".into());
            } else {
                ui.set_status_message(
                    format!("Tecla '{}' associada. Marque os comandos e clique em Salvar Bind.", captured)
                        .into(),
                );
            }
        }
    });

    let ui_weak = ui.as_weak();
    let model_for_toggle = rows_model.clone();
    ui.on_command_toggled(move |row_idx, checked| {
        if let Some(mut row) = model_for_toggle.row_data(row_idx as usize) {
            row.checked = checked;
            model_for_toggle.set_row_data(row_idx as usize, row);
        }
        if let Some(ui) = ui_weak.upgrade() {
            sync_window_summary(&ui);
            ui.set_status_kind("info".into());
        }
    });

    let ui_weak = ui.as_weak();
    let model_for_clear = rows_model.clone();
    ui.on_clear_selected_commands(move || {
        for idx in 0..model_for_clear.row_count() {
            if let Some(mut row) = model_for_clear.row_data(idx) {
                if row.row_kind == 2 && row.checked {
                    row.checked = false;
                    model_for_clear.set_row_data(idx, row);
                }
            }
        }
        if let Some(ui) = ui_weak.upgrade() {
            sync_window_summary(&ui);
            ui.set_status_kind("info".into());
            ui.set_status_message("Seleção de comandos limpa.".into());
        }
    });

    let dirs_rc = Rc::new(RefCell::new(dirs));

    let ui_weak = ui.as_weak();
    let dirs_save = dirs_rc.clone();
    ui.on_save_bind_clicked(move || {
        if let Some(ui) = ui_weak.upgrade() {
            execute_save_bind(&ui, &dirs_save.borrow(), false);
        }
    });

    let ui_weak = ui.as_weak();
    let dirs_confirm = dirs_rc.clone();
    ui.on_confirm_conflict_overwrite(move || {
        if let Some(ui) = ui_weak.upgrade() {
            execute_save_bind(&ui, &dirs_confirm.borrow(), true);
        }
    });

    let ui_weak = ui.as_weak();
    ui.on_cancel_conflict_overwrite(move || {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_status_kind("info".into());
            ui.set_status_message(
                "Substituição cancelada. Nenhuma alteração foi gravada no arquivo.".into(),
            );
        }
    });
}

fn run_self_test() {
    println!("=== CounterRush — Self-Test Isolado da Janela de Binds ===");
    let temp = tempfile::tempdir().expect("Falha ao criar tempdir");
    let local_cfg = temp.path().join("local_cfg");
    let bkp_cfg = local_cfg.join("bkp");
    let cs2_cfg = temp.path().join("cs2_cfg");
    std::fs::create_dir_all(&local_cfg).unwrap();
    std::fs::create_dir_all(&cs2_cfg).unwrap();

    let original_autoexec = "bind \"f4\" \"buy ak47;buy vesthelm\"\n";
    std::fs::write(local_cfg.join("autoexec.cfg"), original_autoexec).unwrap();

    let dirs = BindDirectories::with_paths(&local_cfg, &bkp_cfg, &cs2_cfg);
    let ui = BindsTestWindow::new().expect("Falha ao instanciar BindsTestWindow");
    wire_binds_window(&ui, dirs);

    // 1. Validar catálogo completo e separação fiel
    let rows = ui.get_catalog_rows();
    let mut cmd_count = 0usize;
    let mut cat_titles = Vec::new();
    let mut noclip_row_idx = None;
    let mut rethrow_row_idx = None;
    let mut jump_row_idx = None;
    let mut smoke_row_idx = None;

    for i in 0..rows.row_count() {
        let row = rows.row_data(i).unwrap();
        match row.row_kind {
            0 => cat_titles.push(row.title.to_string()),
            2 => {
                cmd_count += 1;
                match row.command_value.as_str() {
                    "noclip" => noclip_row_idx = Some(i as i32),
                    "sv_rethrow_last_grenade" => rethrow_row_idx = Some(i as i32),
                    "+jump" => jump_row_idx = Some(i as i32),
                    "slot8" => smoke_row_idx = Some(i as i32),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    assert_eq!(cmd_count, 186, "Deve exibir todos os 186 comandos do guia");
    assert_eq!(cat_titles.len(), 7, "Deve exibir todas as 7 categorias");
    assert_eq!(cat_titles[0], "1. Movimentação");
    assert_eq!(cat_titles[6], "7. Outros (Áudio, Demos/Replays, HUD e Sistema)");

    // 2. Validar função dedicada de captura de tecla
    assert_eq!(ui.invoke_resolve_cs2_key(" ".into()), "space");
    assert_eq!(
        ui.invoke_resolve_cs2_key(char::from(slint::platform::Key::UpArrow).to_string().into()),
        "uparrow"
    );
    assert_eq!(
        ui.invoke_resolve_cs2_key(char::from(slint::platform::Key::F4).to_string().into()),
        "f4"
    );
    assert_eq!(ui.invoke_resolve_cs2_key(";".into()), "semicolon");
    assert_eq!(ui.invoke_resolve_cs2_key("\"".into()), "");

    // 3. Validar criação de NOVO arquivo sem selecionar existente
    ui.set_file_name_input("treino_smokes".into());
    ui.invoke_file_name_edited("treino_smokes".into());
    assert!(ui.get_is_creating_new_file());

    ui.set_bound_key("v".into());
    ui.invoke_key_captured("v".into());
    ui.invoke_command_toggled(noclip_row_idx.unwrap(), true);
    ui.invoke_command_toggled(rethrow_row_idx.unwrap(), true);
    assert_eq!(ui.get_selected_commands_count(), 2);
    assert_eq!(
        ui.get_bind_preview_line(),
        "bind \"v\" \"noclip;sv_rethrow_last_grenade\""
    );
    ui.invoke_save_bind_clicked();

    let treino_content = std::fs::read_to_string(local_cfg.join("treino_smokes.cfg")).unwrap();
    assert_eq!(
        treino_content,
        "bind \"v\" \"noclip;sv_rethrow_last_grenade\"\n"
    );

    // 4. Validar atualização do arquivo selecionado mantendo o nome
    ui.invoke_clear_selected_commands();
    ui.set_selected_existing_file("treino_smokes.cfg".into());
    ui.set_file_name_input("treino_smokes.cfg".into());
    ui.invoke_existing_file_selected("treino_smokes.cfg".into());
    assert!(!ui.get_is_creating_new_file());

    ui.set_bound_key("mwheeldown".into());
    ui.invoke_key_captured("mwheeldown".into());
    ui.invoke_command_toggled(jump_row_idx.unwrap(), true);
    ui.invoke_save_bind_clicked();

    let treino_updated = std::fs::read_to_string(local_cfg.join("treino_smokes.cfg")).unwrap();
    assert!(treino_updated.contains("bind \"v\" \"noclip;sv_rethrow_last_grenade\""));
    assert!(treino_updated.contains("bind \"mwheeldown\" \"+jump\""));

    // 5. Validar regra: selecionar arquivo existente e ALTERAR nome -> cria NOVO arquivo preservando o original!
    ui.invoke_clear_selected_commands();
    ui.set_selected_existing_file("autoexec.cfg".into());
    ui.set_file_name_input("autoexec.cfg".into());
    ui.invoke_existing_file_selected("autoexec.cfg".into());

    ui.set_file_name_input("autoexec_smokes.cfg".into());
    ui.invoke_file_name_edited("autoexec_smokes.cfg".into());
    assert!(ui.get_is_creating_new_file());

    ui.set_bound_key("c".into());
    ui.invoke_key_captured("c".into());
    ui.invoke_command_toggled(smoke_row_idx.unwrap(), true);
    ui.invoke_save_bind_clicked();

    assert_eq!(
        std::fs::read_to_string(local_cfg.join("autoexec.cfg")).unwrap(),
        original_autoexec,
        "Arquivo original autoexec.cfg deve permanecer 100% preservado"
    );
    assert_eq!(
        std::fs::read_to_string(local_cfg.join("autoexec_smokes.cfg")).unwrap(),
        "bind \"c\" \"slot8\"\n"
    );

    // 6. Validar detecção e confirmação de conflito ao editar arquivo selecionado
    ui.invoke_clear_selected_commands();
    ui.set_selected_existing_file("autoexec.cfg".into());
    ui.set_file_name_input("autoexec.cfg".into());
    ui.invoke_existing_file_selected("autoexec.cfg".into());
    ui.set_bound_key("f4".into());
    ui.invoke_key_captured("f4".into());
    ui.invoke_command_toggled(smoke_row_idx.unwrap(), true);
    ui.invoke_save_bind_clicked();

    assert!(
        ui.get_show_conflict_prompt(),
        "Conflito na tecla f4 em autoexec.cfg deve abrir confirmação"
    );
    assert_eq!(
        std::fs::read_to_string(local_cfg.join("autoexec.cfg")).unwrap(),
        original_autoexec
    );
    ui.invoke_confirm_conflict_overwrite();
    assert!(!ui.get_show_conflict_prompt());
    assert_eq!(
        std::fs::read_to_string(local_cfg.join("autoexec.cfg")).unwrap(),
        "bind \"f4\" \"slot8\"\n"
    );

    println!("[PASS] Todos os contratos da BindsTestWindow validados com sucesso!");
}

fn main() -> Result<(), slint::PlatformError> {
    if std::env::args().any(|a| a == "--self-test") {
        run_self_test();
        return Ok(());
    }

    let window = BindsTestWindow::new()?;
    wire_binds_window(&window, BindDirectories::default());
    window.show()?;
    slint::run_event_loop_until_quit()?;
    Ok(())
}
