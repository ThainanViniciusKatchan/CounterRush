use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use counterrush_core::{
    cs2_bind_catalog, cs2_bind_categories, determine_bind_save_mode, find_key_conflict_in_content,
    format_bind_commands, format_bind_line, list_cfg_files_in_dir, matches_bind_search,
    normalize_captured_key_to_cs2, save_cs2_bind_with_local_fallback, validate_bind_key,
    BindCommandEntry, BindDirectories, BindFlag, BindOperationOutcome,
};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

slint::include_modules!();

struct BindsWindowState {
    selected_flags: Vec<bool>,
    active_category: usize,
    is_grid_view: bool,
    search_query: String,
}

impl BindsWindowState {
    fn new() -> Self {
        Self {
            selected_flags: vec![false; cs2_bind_catalog().len()],
            active_category: 0,
            is_grid_view: false,
            search_query: String::new(),
        }
    }
}

fn make_empty_grid_cell() -> BindGridCell {
    BindGridCell {
        present: false,
        command_index: -1,
        title: SharedString::default(),
        info: SharedString::default(),
        description: SharedString::default(),
        command_value: SharedString::default(),
        checked: false,
    }
}

fn make_grid_cell(idx: usize, entry: &BindCommandEntry, checked: bool) -> BindGridCell {
    BindGridCell {
        present: true,
        command_index: idx as i32,
        title: entry.display_command.into(),
        info: entry.subcategory.into(),
        description: entry.description.into(),
        command_value: entry.cfg_command.into(),
        checked,
    }
}

fn make_header_row(row_kind: i32, title: &str, subtitle: &str) -> BindCatalogRow {
    BindCatalogRow {
        row_kind,
        command_index: -1,
        title: title.into(),
        subtitle: subtitle.into(),
        command_value: SharedString::default(),
        checked: false,
        cell_0: make_empty_grid_cell(),
        cell_1: make_empty_grid_cell(),
        cell_2: make_empty_grid_cell(),
        cell_3: make_empty_grid_cell(),
    }
}

fn make_list_row(cmd_idx: usize, entry: &BindCommandEntry, checked: bool) -> BindCatalogRow {
    BindCatalogRow {
        row_kind: 2,
        command_index: cmd_idx as i32,
        title: entry.display_command.into(),
        subtitle: entry.description.into(),
        command_value: entry.cfg_command.into(),
        checked,
        cell_0: make_empty_grid_cell(),
        cell_1: make_empty_grid_cell(),
        cell_2: make_empty_grid_cell(),
        cell_3: make_empty_grid_cell(),
    }
}

fn make_grid_quad_row(chunk: &[(usize, &BindCommandEntry, bool)]) -> BindCatalogRow {
    let cell_for = |slot: usize| -> BindGridCell {
        chunk
            .get(slot)
            .map(|&(idx, entry, checked)| make_grid_cell(idx, entry, checked))
            .unwrap_or_else(make_empty_grid_cell)
    };

    BindCatalogRow {
        row_kind: 3,
        command_index: -1,
        title: SharedString::default(),
        subtitle: SharedString::default(),
        command_value: SharedString::default(),
        checked: false,
        cell_0: cell_for(0),
        cell_1: cell_for(1),
        cell_2: cell_for(2),
        cell_3: cell_for(3),
    }
}

fn flush_subcategory_rows(
    out: &mut Vec<BindCatalogRow>,
    pending: &mut Vec<(usize, &'static BindCommandEntry, bool)>,
    is_grid_view: bool,
) {
    if pending.is_empty() {
        return;
    }

    if is_grid_view {
        for chunk in pending.chunks(4) {
            out.push(make_grid_quad_row(chunk));
        }
    } else {
        for &(idx, entry, checked) in pending.iter() {
            out.push(make_list_row(idx, entry, checked));
        }
    }
    pending.clear();
}

fn build_category_tabs(state: &BindsWindowState) -> Vec<BindCategoryTab> {
    let catalog = cs2_bind_catalog();
    let categories = cs2_bind_categories();
    let query = state.search_query.trim();
    let is_searching = !query.is_empty();

    categories
        .into_iter()
        .map(|cat| {
            let mut selected_count = 0i32;
            let mut match_count = 0i32;
            for (idx, entry) in catalog.iter().enumerate() {
                if entry.category == cat.title {
                    if state.selected_flags.get(idx).copied().unwrap_or(false) {
                        selected_count += 1;
                    }
                    if is_searching && matches_bind_search(entry, query) {
                        match_count += 1;
                    }
                }
            }

            BindCategoryTab {
                index: cat.index as i32,
                tab_label: cat.tab_label.into(),
                title: cat.title.into(),
                subtitle: cat.subtitle.into(),
                total_count: cat.command_count as i32,
                selected_count,
                match_count,
            }
        })
        .collect()
}

fn build_visible_catalog_rows(state: &BindsWindowState) -> (Vec<BindCatalogRow>, usize) {
    let catalog = cs2_bind_catalog();
    let categories = cs2_bind_categories();
    let query = state.search_query.trim();
    let is_searching = !query.is_empty();
    let active_cat_title = categories
        .get(state.active_category)
        .or_else(|| categories.first())
        .map(|c| c.title)
        .unwrap_or("");

    let mut rows = Vec::new();
    let mut pending: Vec<(usize, &'static BindCommandEntry, bool)> = Vec::new();
    let mut last_category: Option<&str> = None;
    let mut last_subcategory: Option<&str> = None;
    let mut total_matched = 0usize;

    for (idx, entry) in catalog.iter().enumerate() {
        let visible = if is_searching {
            matches_bind_search(entry, query)
        } else {
            entry.category == active_cat_title
        };
        if !visible {
            continue;
        }

        total_matched += 1;

        if last_category != Some(entry.category) {
            flush_subcategory_rows(&mut rows, &mut pending, state.is_grid_view);
            rows.push(make_header_row(0, entry.category, entry.category_note));
            last_category = Some(entry.category);
            last_subcategory = None;
        }

        if last_subcategory != Some(entry.subcategory) {
            flush_subcategory_rows(&mut rows, &mut pending, state.is_grid_view);
            rows.push(make_header_row(1, entry.subcategory, ""));
            last_subcategory = Some(entry.subcategory);
        }

        let checked = state.selected_flags.get(idx).copied().unwrap_or(false);
        pending.push((idx, entry, checked));
    }

    flush_subcategory_rows(&mut rows, &mut pending, state.is_grid_view);
    (rows, total_matched)
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

fn collect_checked_commands(selected_flags: &[bool]) -> Vec<String> {
    cs2_bind_catalog()
        .iter()
        .enumerate()
        .filter_map(|(idx, entry)| {
            if selected_flags.get(idx).copied().unwrap_or(false) {
                Some(entry.cfg_command.to_string())
            } else {
                None
            }
        })
        .collect()
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

fn sync_window_summary(ui: &BindsTestWindow, selected_flags: &[bool]) {
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

    let selected_cmds = collect_checked_commands(selected_flags);
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

fn sync_command_check_in_visible_rows(
    model: &VecModel<BindCatalogRow>,
    command_idx: i32,
    checked: bool,
) {
    for idx in 0..model.row_count() {
        if let Some(mut row) = model.row_data(idx) {
            let mut changed = false;
            if row.row_kind == 2 && row.command_index == command_idx && row.checked != checked {
                row.checked = checked;
                changed = true;
            }
            if row.row_kind == 3 {
                for cell in [&mut row.cell_0, &mut row.cell_1, &mut row.cell_2, &mut row.cell_3] {
                    if cell.present && cell.command_index == command_idx && cell.checked != checked {
                        cell.checked = checked;
                        changed = true;
                    }
                }
            }
            if changed {
                model.set_row_data(idx, row);
            }
        }
    }
}

fn refresh_catalog_view(
    ui: &BindsTestWindow,
    state: &BindsWindowState,
    rows_model: &Rc<VecModel<BindCatalogRow>>,
) {
    let (visible_rows, matched_count) = build_visible_catalog_rows(state);
    rows_model.set_vec(visible_rows);
    ui.set_category_tabs(ModelRc::new(VecModel::from(build_category_tabs(state))));
    ui.set_filtered_commands_count(matched_count as i32);
    ui.set_active_category_index(state.active_category as i32);
    ui.set_is_grid_view(state.is_grid_view);
    sync_window_summary(ui, &state.selected_flags);
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
    state: &BindsWindowState,
    dirs: &BindDirectories,
    allow_conflict_overwrite: bool,
) {
    let file_input = ui.get_file_name_input().to_string();
    let selected_existing = ui.get_selected_existing_file().to_string();
    let raw_key = ui.get_bound_key().to_string();
    let selected_cmds = collect_checked_commands(&state.selected_flags);

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
            sync_window_summary(ui, &state.selected_flags);

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
    let state = Rc::new(RefCell::new(BindsWindowState::new()));
    let rows_model = Rc::new(VecModel::default());
    ui.set_backend_connected(true);
    ui.set_catalog_rows(ModelRc::from(rows_model.clone()));

    let cfg_names = load_cfg_names_for_dir(&dirs.local_cfg_dir);
    ui.set_existing_cfg_files(ModelRc::new(VecModel::from(cfg_names)));
    refresh_catalog_view(ui, &state.borrow(), &rows_model);

    ui.on_resolve_cs2_key(|raw_text| {
        capture_key_from_slint_text(raw_text.as_str())
            .map(SharedString::from)
            .unwrap_or_default()
    });

    let ui_weak = ui.as_weak();
    let state_sel = state.clone();
    ui.on_existing_file_selected(move |selected| {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_show_conflict_prompt(false);
            sync_window_summary(&ui, &state_sel.borrow().selected_flags);
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
    let state_edit = state.clone();
    ui.on_file_name_edited(move |_| {
        if let Some(ui) = ui_weak.upgrade() {
            sync_window_summary(&ui, &state_edit.borrow().selected_flags);
        }
    });

    let ui_weak = ui.as_weak();
    let state_key = state.clone();
    ui.on_key_captured(move |captured| {
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_show_conflict_prompt(false);
            sync_window_summary(&ui, &state_key.borrow().selected_flags);
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
    let state_tab = state.clone();
    let model_tab = rows_model.clone();
    ui.on_category_tab_selected(move |tab_idx| {
        let cat_count = cs2_bind_categories().len();
        {
            let mut st = state_tab.borrow_mut();
            st.active_category = (tab_idx.max(0) as usize).min(cat_count.saturating_sub(1));
            st.search_query.clear();
        }
        if let Some(ui) = ui_weak.upgrade() {
            if !ui.get_search_query().is_empty() {
                ui.set_search_query("".into());
            }
            refresh_catalog_view(&ui, &state_tab.borrow(), &model_tab);
        }
    });

    let ui_weak = ui.as_weak();
    let state_view = state.clone();
    let model_view = rows_model.clone();
    ui.on_view_mode_changed(move |grid_mode| {
        {
            let mut st = state_view.borrow_mut();
            st.is_grid_view = grid_mode;
        }
        if let Some(ui) = ui_weak.upgrade() {
            refresh_catalog_view(&ui, &state_view.borrow(), &model_view);
        }
    });

    let ui_weak = ui.as_weak();
    let state_search = state.clone();
    let model_search = rows_model.clone();
    ui.on_search_query_changed(move |query| {
        {
            let mut st = state_search.borrow_mut();
            st.search_query = query.to_string();
        }
        if let Some(ui) = ui_weak.upgrade() {
            let st = state_search.borrow();
            if ui.get_search_query().as_str() != st.search_query.as_str() {
                ui.set_search_query(st.search_query.clone().into());
            }
            refresh_catalog_view(&ui, &st, &model_search);
        }
    });

    let ui_weak = ui.as_weak();
    let state_toggle = state.clone();
    let model_for_toggle = rows_model.clone();
    ui.on_command_toggled(move |cmd_idx, checked| {
        if cmd_idx >= 0 {
            let idx_usize = cmd_idx as usize;
            let mut st = state_toggle.borrow_mut();
            if let Some(slot) = st.selected_flags.get_mut(idx_usize) {
                *slot = checked;
            }
        }
        sync_command_check_in_visible_rows(&model_for_toggle, cmd_idx, checked);
        if let Some(ui) = ui_weak.upgrade() {
            let st = state_toggle.borrow();
            ui.set_category_tabs(ModelRc::new(VecModel::from(build_category_tabs(&st))));
            sync_window_summary(&ui, &st.selected_flags);
            ui.set_status_kind("info".into());
        }
    });

    let ui_weak = ui.as_weak();
    let state_clear = state.clone();
    let model_for_clear = rows_model.clone();
    ui.on_clear_selected_commands(move || {
        {
            let mut st = state_clear.borrow_mut();
            st.selected_flags.fill(false);
        }
        for idx in 0..model_for_clear.row_count() {
            if let Some(mut row) = model_for_clear.row_data(idx) {
                let mut changed = false;
                if row.checked {
                    row.checked = false;
                    changed = true;
                }
                for cell in [&mut row.cell_0, &mut row.cell_1, &mut row.cell_2, &mut row.cell_3] {
                    if cell.checked {
                        cell.checked = false;
                        changed = true;
                    }
                }
                if changed {
                    model_for_clear.set_row_data(idx, row);
                }
            }
        }
        if let Some(ui) = ui_weak.upgrade() {
            let st = state_clear.borrow();
            ui.set_category_tabs(ModelRc::new(VecModel::from(build_category_tabs(&st))));
            sync_window_summary(&ui, &st.selected_flags);
            ui.set_status_kind("info".into());
            ui.set_status_message("Seleção de comandos limpa.".into());
        }
    });

    let dirs_rc = Rc::new(RefCell::new(dirs));

    let ui_weak = ui.as_weak();
    let dirs_save = dirs_rc.clone();
    let state_save = state.clone();
    ui.on_save_bind_clicked(move || {
        if let Some(ui) = ui_weak.upgrade() {
            execute_save_bind(&ui, &state_save.borrow(), &dirs_save.borrow(), false);
        }
    });

    let ui_weak = ui.as_weak();
    let dirs_confirm = dirs_rc.clone();
    let state_confirm = state.clone();
    ui.on_confirm_conflict_overwrite(move || {
        if let Some(ui) = ui_weak.upgrade() {
            execute_save_bind(&ui, &state_confirm.borrow(), &dirs_confirm.borrow(), true);
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

fn find_catalog_command_index(cfg_command: &str) -> i32 {
    cs2_bind_catalog()
        .iter()
        .position(|e| e.cfg_command == cfg_command)
        .expect("Comando esperado deve existir no catálogo") as i32
}

fn count_visible_commands_in_model(rows: &ModelRc<BindCatalogRow>) -> usize {
    let mut count = 0usize;
    for i in 0..rows.row_count() {
        let row = rows.row_data(i).unwrap();
        match row.row_kind {
            2 => count += 1,
            3 => {
                for cell in [&row.cell_0, &row.cell_1, &row.cell_2, &row.cell_3] {
                    if cell.present {
                        count += 1;
                    }
                }
            }
            _ => {}
        }
    }
    count
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

    // 1. Validar 7 abas de categorias, separação por abas e soma total de 186 comandos
    let tabs = ui.get_category_tabs();
    assert_eq!(tabs.row_count(), 7, "Deve exibir exatamente 7 abas de categoria");
    assert_eq!(tabs.row_data(0).unwrap().title, "1. Movimentação");
    assert_eq!(
        tabs.row_data(6).unwrap().title,
        "7. Outros (Áudio, Demos/Replays, HUD e Sistema)"
    );

    let mut total_across_tabs = 0usize;
    for tab_idx in 0..7 {
        ui.invoke_category_tab_selected(tab_idx);
        let tab_rows = ui.get_catalog_rows();
        let tab_cmd_count = count_visible_commands_in_model(&tab_rows);
        assert_eq!(
            tab_cmd_count as i32,
            tabs.row_data(tab_idx as usize).unwrap().total_count,
            "Aba {} deve exibir somente os comandos da sua categoria",
            tab_idx
        );
        total_across_tabs += tab_cmd_count;
    }
    assert_eq!(total_across_tabs, 186, "Soma das 7 abas deve totalizar 186 comandos");

    // 2. Validar alternância Lista <-> Grade (4 colunas estilo Otimizações) mantendo contagem e descrições
    ui.invoke_category_tab_selected(1); // 2. Treinamento (23 comandos)
    ui.invoke_view_mode_changed(true);
    let grid_rows = ui.get_catalog_rows();
    let mut has_grid_row = false;
    for i in 0..grid_rows.row_count() {
        let row = grid_rows.row_data(i).unwrap();
        assert_ne!(row.row_kind, 2, "Modo grade não deve emitir row_kind == 2");
        if row.row_kind == 3 {
            has_grid_row = true;
            for cell in [&row.cell_0, &row.cell_1, &row.cell_2, &row.cell_3] {
                if cell.present {
                    assert!(
                        !cell.description.is_empty(),
                        "Célula da grade deve trazer descrição completa para o popup"
                    );
                }
            }
        }
    }
    assert!(has_grid_row, "Modo grade deve emitir linhas row_kind == 3");
    assert_eq!(count_visible_commands_in_model(&grid_rows), 23);

    ui.invoke_view_mode_changed(false);
    assert_eq!(count_visible_commands_in_model(&ui.get_catalog_rows()), 23);

    // 3. Validar pesquisa global por comando e descrição (inclusive sem acento)
    ui.invoke_search_query_changed("noclip".into());
    assert_eq!(ui.get_filtered_commands_count(), 1);
    assert_eq!(count_visible_commands_in_model(&ui.get_catalog_rows()), 1);

    ui.invoke_search_query_changed("fumaca".into());
    assert!(
        ui.get_filtered_commands_count() >= 1,
        "Pesquisa por descrição sem acento deve encontrar comandos relacionados a fumaça"
    );

    ui.invoke_search_query_changed("invencibilidade".into());
    assert_eq!(ui.get_filtered_commands_count(), 1);

    ui.invoke_search_query_changed("termo_inexistente_xyz".into());
    assert_eq!(ui.get_filtered_commands_count(), 0);
    assert_eq!(ui.get_catalog_rows().row_count(), 0);

    ui.invoke_category_tab_selected(0);
    assert_eq!(ui.get_search_query(), "");

    let noclip_idx = find_catalog_command_index("noclip");
    let rethrow_idx = find_catalog_command_index("sv_rethrow_last_grenade");
    let jump_idx = find_catalog_command_index("+jump");
    let smoke_idx = find_catalog_command_index("slot8");

    // 4. Validar função dedicada de captura de tecla
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

    // 5. Validar preservação de seleção entre abas e criação de NOVO arquivo sem selecionar existente
    ui.set_file_name_input("treino_smokes".into());
    ui.invoke_file_name_edited("treino_smokes".into());
    assert!(ui.get_is_creating_new_file());

    ui.set_bound_key("v".into());
    ui.invoke_key_captured("v".into());
    ui.invoke_category_tab_selected(1);
    ui.invoke_command_toggled(noclip_idx, true);
    ui.invoke_view_mode_changed(true);
    ui.invoke_command_toggled(rethrow_idx, true);
    ui.invoke_category_tab_selected(0); // Trocar de aba deve preservar seleções
    assert_eq!(ui.get_selected_commands_count(), 2);
    assert_eq!(
        ui.get_category_tabs().row_data(1).unwrap().selected_count,
        2
    );
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

    // 6. Validar atualização do arquivo selecionado mantendo o nome
    ui.invoke_clear_selected_commands();
    ui.set_selected_existing_file("treino_smokes.cfg".into());
    ui.set_file_name_input("treino_smokes.cfg".into());
    ui.invoke_existing_file_selected("treino_smokes.cfg".into());
    assert!(!ui.get_is_creating_new_file());

    ui.set_bound_key("mwheeldown".into());
    ui.invoke_key_captured("mwheeldown".into());
    ui.invoke_command_toggled(jump_idx, true);
    ui.invoke_save_bind_clicked();

    let treino_updated = std::fs::read_to_string(local_cfg.join("treino_smokes.cfg")).unwrap();
    assert!(treino_updated.contains("bind \"v\" \"noclip;sv_rethrow_last_grenade\""));
    assert!(treino_updated.contains("bind \"mwheeldown\" \"+jump\""));

    // 7. Validar regra: selecionar arquivo existente e ALTERAR nome -> cria NOVO arquivo preservando o original!
    ui.invoke_clear_selected_commands();
    ui.set_selected_existing_file("autoexec.cfg".into());
    ui.set_file_name_input("autoexec.cfg".into());
    ui.invoke_existing_file_selected("autoexec.cfg".into());

    ui.set_file_name_input("autoexec_smokes.cfg".into());
    ui.invoke_file_name_edited("autoexec_smokes.cfg".into());
    assert!(ui.get_is_creating_new_file());

    ui.set_bound_key("c".into());
    ui.invoke_key_captured("c".into());
    ui.invoke_command_toggled(smoke_idx, true);
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

    // 8. Validar detecção e confirmação de conflito ao editar arquivo selecionado
    ui.invoke_clear_selected_commands();
    ui.set_selected_existing_file("autoexec.cfg".into());
    ui.set_file_name_input("autoexec.cfg".into());
    ui.invoke_existing_file_selected("autoexec.cfg".into());
    ui.set_bound_key("f4".into());
    ui.invoke_key_captured("f4".into());
    ui.invoke_command_toggled(smoke_idx, true);
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
