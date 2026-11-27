use std::cell::RefCell;
use std::path::Path;
use std::rc::Rc;

use counterrush_core::{
    cs2_bind_catalog, cs2_bind_categories, determine_bind_save_mode, find_key_conflict_in_content,
    format_bind_commands, format_bind_line, format_unbind_line, list_cfg_files_in_dir,
    normalize_captured_key_to_cs2, remove_cs2_bind_with_local_fallback,
    save_cs2_bind_with_local_fallback, validate_bind_key, BindCommandEntry, BindDirectories,
    BindFlag, BindOperationOutcome,
};
use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

use crate::binds_i18n::{
    localize_category, localize_category_header, localize_description, localize_featured_badge,
    localize_featured_subcategory_note, localize_subcategory, matches_localized_bind_search,
};
use crate::i18n::{
    self, BindSaveModeKind, BindUpdateVerb, UiLanguage,
};
use crate::{BindCatalogRow, BindCategoryTab, BindGridCell, BindsState, MainWindow};

struct BindsViewState {
    selected_flags: Vec<bool>,
    active_category: usize,
    is_grid_view: bool,
    search_query: String,
    lang: UiLanguage,
}

impl BindsViewState {
    fn new(lang: UiLanguage, is_grid_view: bool) -> Self {
        Self {
            selected_flags: vec![false; cs2_bind_catalog().len()],
            active_category: 0,
            is_grid_view,
            search_query: String::new(),
            lang,
        }
    }
}

#[derive(Clone)]
pub struct BindsController {
    state: Rc<RefCell<BindsViewState>>,
    rows_model: Rc<VecModel<BindCatalogRow>>,
    dirs: Rc<RefCell<BindDirectories>>,
}

impl BindsController {
    pub fn set_view_mode(&self, ui: &MainWindow, is_grid_view: bool) {
        {
            let mut st = self.state.borrow_mut();
            if st.is_grid_view == is_grid_view && ui.global::<BindsState>().get_is_grid_view() == is_grid_view {
                return;
            }
            st.is_grid_view = is_grid_view;
        }
        let binds = ui.global::<BindsState>();
        binds.set_tooltip_visible(false);
        refresh_catalog_view(&binds, &self.state.borrow(), &self.rows_model);
    }

    pub fn refresh_for_language(&self, ui: &MainWindow, lang: UiLanguage) {
        {
            let mut st = self.state.borrow_mut();
            st.lang = lang;
        }
        let binds = ui.global::<BindsState>();
        binds.set_tooltip_visible(false);
        binds.set_selector_dropdown_open(false);
        refresh_catalog_view(&binds, &self.state.borrow(), &self.rows_model);
        sync_unbind_preview(&binds, &self.dirs.borrow(), lang);

        let current_msg = binds.get_status_message().to_string();
        if current_msg == i18n::binds_default_status(UiLanguage::PtBr)
            || current_msg == i18n::binds_default_status(UiLanguage::EnUs)
            || current_msg == i18n::binds_default_status(UiLanguage::RuRu)
        {
            binds.set_status_message(i18n::binds_default_status(lang).into());
        } else if current_msg == i18n::binds_commands_cleared(UiLanguage::PtBr)
            || current_msg == i18n::binds_commands_cleared(UiLanguage::EnUs)
            || current_msg == i18n::binds_commands_cleared(UiLanguage::RuRu)
        {
            binds.set_status_message(i18n::binds_commands_cleared(lang).into());
        } else if current_msg == i18n::binds_selection_cleared(UiLanguage::PtBr)
            || current_msg == i18n::binds_selection_cleared(UiLanguage::EnUs)
            || current_msg == i18n::binds_selection_cleared(UiLanguage::RuRu)
        {
            binds.set_status_message(i18n::binds_selection_cleared(lang).into());
        } else if current_msg == i18n::binds_key_removed(UiLanguage::PtBr)
            || current_msg == i18n::binds_key_removed(UiLanguage::EnUs)
            || current_msg == i18n::binds_key_removed(UiLanguage::RuRu)
        {
            binds.set_status_message(i18n::binds_key_removed(lang).into());
        } else if current_msg == i18n::binds_overwrite_cancelled(UiLanguage::PtBr)
            || current_msg == i18n::binds_overwrite_cancelled(UiLanguage::EnUs)
            || current_msg == i18n::binds_overwrite_cancelled(UiLanguage::RuRu)
        {
            binds.set_status_message(i18n::binds_overwrite_cancelled(lang).into());
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
        featured: false,
    }
}

fn make_grid_cell(
    idx: usize,
    entry: &BindCommandEntry,
    checked: bool,
    lang: UiLanguage,
) -> BindGridCell {
    BindGridCell {
        present: true,
        command_index: idx as i32,
        title: entry.display_command.into(),
        info: localize_subcategory(entry.subcategory, lang).into(),
        description: localize_description(idx, entry, lang).into(),
        command_value: entry.cfg_command.into(),
        checked,
        featured: entry.featured,
    }
}

fn make_header_row(
    row_kind: i32,
    title: &str,
    subtitle: &str,
    badge: &str,
    featured: bool,
) -> BindCatalogRow {
    BindCatalogRow {
        row_kind,
        command_index: -1,
        title: title.into(),
        subtitle: subtitle.into(),
        command_value: badge.into(),
        checked: false,
        featured,
        cell_0: make_empty_grid_cell(),
        cell_1: make_empty_grid_cell(),
        cell_2: make_empty_grid_cell(),
        cell_3: make_empty_grid_cell(),
    }
}

fn make_separator_row() -> BindCatalogRow {
    make_header_row(4, "", "", "", false)
}

fn make_list_row(
    cmd_idx: usize,
    entry: &BindCommandEntry,
    checked: bool,
    lang: UiLanguage,
) -> BindCatalogRow {
    BindCatalogRow {
        row_kind: 2,
        command_index: cmd_idx as i32,
        title: entry.display_command.into(),
        subtitle: localize_description(cmd_idx, entry, lang).into(),
        command_value: entry.cfg_command.into(),
        checked,
        featured: entry.featured,
        cell_0: make_empty_grid_cell(),
        cell_1: make_empty_grid_cell(),
        cell_2: make_empty_grid_cell(),
        cell_3: make_empty_grid_cell(),
    }
}

fn make_grid_quad_row(
    chunk: &[(usize, &BindCommandEntry, bool)],
    lang: UiLanguage,
) -> BindCatalogRow {
    let cell_for = |slot: usize| -> BindGridCell {
        chunk
            .get(slot)
            .map(|&(idx, entry, checked)| make_grid_cell(idx, entry, checked, lang))
            .unwrap_or_else(make_empty_grid_cell)
    };

    BindCatalogRow {
        row_kind: 3,
        command_index: -1,
        title: SharedString::default(),
        subtitle: SharedString::default(),
        command_value: SharedString::default(),
        checked: false,
        featured: false,
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
    lang: UiLanguage,
) {
    if pending.is_empty() {
        return;
    }

    if is_grid_view {
        for chunk in pending.chunks(4) {
            out.push(make_grid_quad_row(chunk, lang));
        }
    } else {
        for &(idx, entry, checked) in pending.iter() {
            out.push(make_list_row(idx, entry, checked, lang));
        }
    }
    pending.clear();
}

fn build_category_tabs(state: &BindsViewState) -> Vec<BindCategoryTab> {
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
                    if is_searching
                        && matches_localized_bind_search(idx, entry, query, state.lang)
                    {
                        match_count += 1;
                    }
                }
            }

            let localized = localize_category(&cat, state.lang);
            BindCategoryTab {
                index: cat.index as i32,
                tab_label: localized.tab_label.into(),
                title: localized.title.into(),
                subtitle: localized.subtitle.into(),
                total_count: cat.command_count as i32,
                selected_count,
                match_count,
            }
        })
        .collect()
}

fn build_visible_catalog_rows(state: &BindsViewState) -> (Vec<BindCatalogRow>, usize) {
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
    let mut last_subcategory_featured = false;
    let mut total_matched = 0usize;

    for (idx, entry) in catalog.iter().enumerate() {
        let visible = if is_searching {
            matches_localized_bind_search(idx, entry, query, state.lang)
        } else {
            entry.category == active_cat_title
        };
        if !visible {
            continue;
        }

        total_matched += 1;

        if last_category != Some(entry.category) {
            flush_subcategory_rows(&mut rows, &mut pending, state.is_grid_view, state.lang);
            let (cat_title, cat_note) = localize_category_header(entry, state.lang);
            rows.push(make_header_row(0, cat_title, cat_note, "", false));
            last_category = Some(entry.category);
            last_subcategory = None;
            last_subcategory_featured = false;
        }

        if last_subcategory != Some(entry.subcategory) {
            flush_subcategory_rows(&mut rows, &mut pending, state.is_grid_view, state.lang);
            if last_subcategory_featured {
                rows.push(make_separator_row());
            }
            let (sub_note, sub_badge) = if entry.featured {
                (
                    localize_featured_subcategory_note(state.lang),
                    localize_featured_badge(state.lang),
                )
            } else {
                ("", "")
            };
            rows.push(make_header_row(
                1,
                localize_subcategory(entry.subcategory, state.lang),
                sub_note,
                sub_badge,
                entry.featured,
            ));
            last_subcategory = Some(entry.subcategory);
            last_subcategory_featured = entry.featured;
        }

        let checked = state.selected_flags.get(idx).copied().unwrap_or(false);
        pending.push((idx, entry, checked));
    }

    flush_subcategory_rows(&mut rows, &mut pending, state.is_grid_view, state.lang);
    (rows, total_matched)
}

/// Captura e converte eventos de teclado Slint para códigos CS2 (Source 2).
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

fn sync_binds_summary(binds: &BindsState<'_>, selected_flags: &[bool], lang: UiLanguage) {
    let selected_existing = binds.get_selected_existing_file();
    let file_input = binds.get_file_name_input();

    if selected_existing.trim().is_empty() {
        binds.set_is_creating_new_file(true);
        binds.set_save_mode_badge(i18n::binds_save_mode_badge(lang, BindSaveModeKind::NewFile).into());
    } else if file_input.trim().is_empty() {
        binds.set_is_creating_new_file(true);
        binds.set_save_mode_badge(i18n::binds_save_mode_badge(lang, BindSaveModeKind::EnterName).into());
    } else {
        match determine_bind_save_mode(selected_existing.as_str(), file_input.as_str()) {
            Ok((_, BindFlag::Default)) => {
                binds.set_is_creating_new_file(false);
                binds.set_save_mode_badge(
                    i18n::binds_save_mode_badge(lang, BindSaveModeKind::UpdateCurrent).into(),
                );
            }
            Ok((_, BindFlag::NewConfig)) => {
                binds.set_is_creating_new_file(true);
                binds.set_save_mode_badge(
                    i18n::binds_save_mode_badge(lang, BindSaveModeKind::CreateNew).into(),
                );
            }
            Err(_) | Ok((_, BindFlag::Restore)) => {
                binds.set_is_creating_new_file(true);
                binds.set_save_mode_badge(
                    i18n::binds_save_mode_badge(lang, BindSaveModeKind::InvalidName).into(),
                );
            }
        }
    }

    let selected_cmds = collect_checked_commands(selected_flags);
    binds.set_selected_commands_count(selected_cmds.len() as i32);

    let key = binds.get_bound_key();
    if !key.trim().is_empty() && !selected_cmds.is_empty() {
        if let (Ok(clean_key), Ok(unified)) = (
            validate_bind_key(key.as_str()),
            format_bind_commands(&selected_cmds),
        ) {
            binds.set_bind_preview_line(format_bind_line(&clean_key, &unified).into());
            return;
        }
    }
    binds.set_bind_preview_line("".into());
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
    binds: &BindsState<'_>,
    state: &BindsViewState,
    rows_model: &Rc<VecModel<BindCatalogRow>>,
) {
    let (visible_rows, matched_count) = build_visible_catalog_rows(state);
    rows_model.set_vec(visible_rows);
    binds.set_category_tabs(ModelRc::new(VecModel::from(build_category_tabs(state))));
    binds.set_filtered_commands_count(matched_count as i32);
    binds.set_active_category_index(state.active_category as i32);
    binds.set_is_grid_view(state.is_grid_view);
    sync_binds_summary(binds, &state.selected_flags, state.lang);
}

fn sync_unbind_preview(binds: &BindsState<'_>, dirs: &BindDirectories, lang: UiLanguage) {
    let file_input = binds.get_file_name_input().to_string();
    let selected_existing = binds.get_selected_existing_file().to_string();
    let raw_key = binds.get_bound_key().to_string();

    let valid_key = validate_bind_key(&raw_key).ok();
    let valid_file = if !file_input.trim().is_empty() {
        determine_bind_save_mode(&selected_existing, &file_input).ok()
    } else {
        None
    };

    if let Some(ref clean_key) = valid_key {
        binds.set_unbind_preview_line(format_unbind_line(clean_key).into());
    } else {
        binds.set_unbind_preview_line("".into());
    }

    let info = match (&valid_file, &valid_key) {
        (None, None) => i18n::binds_unbind_hint_both(lang).into(),
        (None, Some(clean_key)) => i18n::binds_unbind_hint_missing_file(lang, clean_key).into(),
        (Some((target_file, _)), None) => {
            i18n::binds_unbind_hint_missing_key(lang, target_file).into()
        }
        (Some((target_file, flag)), Some(clean_key)) => {
            if *flag == BindFlag::Default {
                if let Some((_, existing_cmd)) =
                    check_existing_key_conflict(dirs, target_file, clean_key)
                {
                    i18n::binds_unbind_hint_found_existing(
                        lang,
                        target_file,
                        &existing_cmd,
                        clean_key,
                    )
                    .into()
                } else {
                    i18n::binds_unbind_hint_ready(lang, target_file, clean_key).into()
                }
            } else {
                i18n::binds_unbind_hint_ready(lang, target_file, clean_key).into()
            }
        }
    };
    binds.set_unbind_existing_info(info);
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
    ui: &MainWindow,
    state: &BindsViewState,
    dirs: &BindDirectories,
    allow_conflict_overwrite: bool,
) {
    let lang = state.lang;
    let binds = ui.global::<BindsState>();
    let file_input = binds.get_file_name_input().to_string();
    let selected_existing = binds.get_selected_existing_file().to_string();
    let raw_key = binds.get_bound_key().to_string();
    let selected_cmds = collect_checked_commands(&state.selected_flags);

    if file_input.trim().is_empty() {
        binds.set_status_kind("warning".into());
        binds.set_status_message(i18n::binds_err_missing_file(lang).into());
        return;
    }

    let clean_key = match validate_bind_key(&raw_key) {
        Ok(k) => k,
        Err(_) => {
            binds.set_status_kind("warning".into());
            binds.set_status_message(i18n::binds_err_missing_key(lang).into());
            return;
        }
    };

    if selected_cmds.is_empty() {
        binds.set_status_kind("warning".into());
        binds.set_status_message(i18n::binds_err_no_commands(lang).into());
        return;
    }

    let (target_file_name, flag) = match determine_bind_save_mode(&selected_existing, &file_input) {
        Ok(res) => res,
        Err(err) => {
            binds.set_status_kind("error".into());
            binds.set_status_message(i18n::binds_err_invalid_file(lang, &err).into());
            return;
        }
    };

    let unified_cmd = match format_bind_commands(&selected_cmds) {
        Ok(c) => c,
        Err(err) => {
            binds.set_status_kind("error".into());
            binds.set_status_message(i18n::binds_err_invalid_commands(lang, &err).into());
            return;
        }
    };

    if flag == BindFlag::Default && !allow_conflict_overwrite {
        if let Some((_, existing_cmd)) =
            check_existing_key_conflict(dirs, &target_file_name, &clean_key)
        {
            binds.set_conflict_message(
                i18n::binds_conflict_message(
                    lang,
                    &clean_key,
                    &existing_cmd,
                    &target_file_name,
                    &unified_cmd,
                )
                .into(),
            );
            binds.set_show_conflict_prompt(true);
            binds.set_status_kind("warning".into());
            binds.set_status_message(i18n::binds_conflict_status(lang).into());
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
            ui.set_available_cfg_files(ModelRc::new(VecModel::from(refreshed.clone())));
            binds.set_existing_cfg_files(ModelRc::new(VecModel::from(refreshed)));

            let saved_file = match &report.outcome {
                BindOperationOutcome::Created { file_name, .. }
                | BindOperationOutcome::Updated { file_name, .. } => file_name.clone(),
                _ => target_file_name.clone(),
            };

            binds.set_selected_existing_file(saved_file.clone().into());
            binds.set_file_name_input(saved_file.into());
            binds.set_show_conflict_prompt(false);
            binds.set_unbind_panel_open(false);
            sync_binds_summary(&binds, &state.selected_flags, lang);
            sync_unbind_preview(&binds, dirs, lang);

            let msg = match (&report.outcome, &report.cs2_sync_warning) {
                (
                    BindOperationOutcome::Created {
                        file_name,
                        bind_line,
                        ..
                    },
                    warn,
                ) => i18n::binds_saved_created(lang, file_name, bind_line, warn.as_deref()),
                (
                    BindOperationOutcome::Updated {
                        file_name,
                        bind_line,
                        replaced_existing_key,
                        ..
                    },
                    warn,
                ) => {
                    let verb = if flag == BindFlag::NewConfig {
                        BindUpdateVerb::NewFileSaved
                    } else if *replaced_existing_key {
                        BindUpdateVerb::BindReplaced
                    } else {
                        BindUpdateVerb::FileUpdated
                    };
                    i18n::binds_saved_updated(lang, verb, file_name, bind_line, warn.as_deref())
                }
                (BindOperationOutcome::CancelledByUser(conflict), _) => {
                    i18n::binds_saved_cancelled(lang, &conflict.key, &conflict.file_path.display())
                }
                (BindOperationOutcome::Restored { file_name, .. }, _) => {
                    i18n::binds_saved_restored(lang, file_name)
                }
            };

            binds.set_status_kind(if report.synced_to_cs2_global {
                "success".into()
            } else {
                "warning".into()
            });
            binds.set_status_message(msg.into());
        }
        Err(err) => {
            binds.set_status_kind("error".into());
            binds.set_status_message(i18n::binds_save_err(lang, &err).into());
        }
    }
}

fn execute_remove_bind(ui: &MainWindow, state: &BindsViewState, dirs: &BindDirectories) {
    let lang = state.lang;
    let binds = ui.global::<BindsState>();
    let file_input = binds.get_file_name_input().to_string();
    let selected_existing = binds.get_selected_existing_file().to_string();
    let raw_key = binds.get_bound_key().to_string();

    if file_input.trim().is_empty() {
        binds.set_status_kind("warning".into());
        binds.set_status_message(i18n::binds_err_missing_file(lang).into());
        return;
    }

    let clean_key = match validate_bind_key(&raw_key) {
        Ok(k) => k,
        Err(_) => {
            binds.set_status_kind("warning".into());
            binds.set_status_message(i18n::binds_err_missing_key(lang).into());
            return;
        }
    };

    let (target_file_name, flag) = match determine_bind_save_mode(&selected_existing, &file_input) {
        Ok(res) => res,
        Err(err) => {
            binds.set_status_kind("error".into());
            binds.set_status_message(i18n::binds_err_invalid_file(lang, &err).into());
            return;
        }
    };

    match remove_cs2_bind_with_local_fallback(dirs, &target_file_name, &clean_key, flag) {
        Ok(report) => {
            let refreshed = load_cfg_names_for_dir(&dirs.local_cfg_dir);
            ui.set_available_cfg_files(ModelRc::new(VecModel::from(refreshed.clone())));
            binds.set_existing_cfg_files(ModelRc::new(VecModel::from(refreshed)));

            let saved_file = match &report.outcome {
                BindOperationOutcome::Created { file_name, .. }
                | BindOperationOutcome::Updated { file_name, .. } => file_name.clone(),
                _ => target_file_name.clone(),
            };

            binds.set_selected_existing_file(saved_file.clone().into());
            binds.set_file_name_input(saved_file.into());
            binds.set_show_conflict_prompt(false);
            binds.set_unbind_panel_open(false);
            sync_binds_summary(&binds, &state.selected_flags, lang);
            sync_unbind_preview(&binds, dirs, lang);

            let msg = match (&report.outcome, &report.cs2_sync_warning) {
                (
                    BindOperationOutcome::Created {
                        file_name,
                        bind_line,
                        ..
                    },
                    warn,
                ) => i18n::binds_unbind_saved_created(
                    lang,
                    file_name,
                    bind_line,
                    warn.as_deref(),
                ),
                (
                    BindOperationOutcome::Updated {
                        file_name,
                        bind_line,
                        replaced_existing_key,
                        ..
                    },
                    warn,
                ) => i18n::binds_unbind_saved_updated(
                    lang,
                    *replaced_existing_key,
                    file_name,
                    bind_line,
                    warn.as_deref(),
                ),
                _ => String::new(),
            };

            binds.set_status_kind(if report.synced_to_cs2_global {
                "success".into()
            } else {
                "warning".into()
            });
            binds.set_status_message(msg.into());
        }
        Err(err) => {
            binds.set_status_kind("error".into());
            binds.set_status_message(i18n::binds_unbind_err(lang, &err).into());
        }
    }
}

pub fn refresh_existing_cfg_files(ui: &MainWindow, dirs: &BindDirectories) {
    let binds = ui.global::<BindsState>();
    let cfg_names = load_cfg_names_for_dir(&dirs.local_cfg_dir);
    ui.set_available_cfg_files(ModelRc::new(VecModel::from(cfg_names.clone())));
    binds.set_existing_cfg_files(ModelRc::new(VecModel::from(cfg_names)));
}

pub fn wire_binds_view(
    ui: &MainWindow,
    dirs: BindDirectories,
    settings_path: std::path::PathBuf,
    initial_is_grid_view: bool,
) -> BindsController {
    let initial_lang = UiLanguage::from_code(ui.get_active_language().as_str());
    let state = Rc::new(RefCell::new(BindsViewState::new(initial_lang, initial_is_grid_view)));
    let rows_model = Rc::new(VecModel::default());
    let dirs_rc = Rc::new(RefCell::new(dirs));
    let binds = ui.global::<BindsState>();

    ui.set_content_is_grid_view(initial_is_grid_view);
    binds.set_backend_connected(true);
    binds.set_catalog_rows(ModelRc::from(rows_model.clone()));
    binds.set_status_message(i18n::binds_default_status(initial_lang).into());

    let cfg_names = load_cfg_names_for_dir(&dirs_rc.borrow().local_cfg_dir);
    binds.set_existing_cfg_files(ModelRc::new(VecModel::from(cfg_names)));
    refresh_catalog_view(&binds, &state.borrow(), &rows_model);
    sync_unbind_preview(&binds, &dirs_rc.borrow(), initial_lang);

    binds.on_resolve_cs2_key(|raw_text| {
        capture_key_from_slint_text(raw_text.as_str())
            .map(SharedString::from)
            .unwrap_or_default()
    });

    let ui_weak = ui.as_weak();
    let state_sel = state.clone();
    let dirs_sel = dirs_rc.clone();
    binds.on_existing_file_selected(move |selected| {
        if let Some(ui) = ui_weak.upgrade() {
            let binds = ui.global::<BindsState>();
            let st = state_sel.borrow();
            binds.set_show_conflict_prompt(false);
            sync_binds_summary(&binds, &st.selected_flags, st.lang);
            sync_unbind_preview(&binds, &dirs_sel.borrow(), st.lang);
            binds.set_status_kind("info".into());
            if selected.trim().is_empty() {
                binds.set_status_message(i18n::binds_selection_cleared(st.lang).into());
            } else {
                binds.set_status_message(
                    i18n::binds_file_loaded(st.lang, selected.as_str()).into(),
                );
            }
        }
    });

    let ui_weak = ui.as_weak();
    let state_edit = state.clone();
    let dirs_edit = dirs_rc.clone();
    binds.on_file_name_edited(move |_| {
        if let Some(ui) = ui_weak.upgrade() {
            let binds = ui.global::<BindsState>();
            let st = state_edit.borrow();
            sync_binds_summary(&binds, &st.selected_flags, st.lang);
            sync_unbind_preview(&binds, &dirs_edit.borrow(), st.lang);
        }
    });

    let ui_weak = ui.as_weak();
    let state_key = state.clone();
    let dirs_key = dirs_rc.clone();
    binds.on_key_captured(move |captured| {
        if let Some(ui) = ui_weak.upgrade() {
            let binds = ui.global::<BindsState>();
            let st = state_key.borrow();
            binds.set_show_conflict_prompt(false);
            sync_binds_summary(&binds, &st.selected_flags, st.lang);
            sync_unbind_preview(&binds, &dirs_key.borrow(), st.lang);
            binds.set_status_kind("info".into());
            if captured.trim().is_empty() {
                binds.set_status_message(i18n::binds_key_removed(st.lang).into());
            } else {
                binds.set_status_message(
                    i18n::binds_key_captured(st.lang, captured.as_str()).into(),
                );
            }
        }
    });

    let ui_weak = ui.as_weak();
    let state_tab = state.clone();
    let model_tab = rows_model.clone();
    binds.on_category_tab_selected(move |tab_idx| {
        let cat_count = cs2_bind_categories().len();
        {
            let mut st = state_tab.borrow_mut();
            st.active_category = (tab_idx.max(0) as usize).min(cat_count.saturating_sub(1));
            st.search_query.clear();
        }
        if let Some(ui) = ui_weak.upgrade() {
            let binds = ui.global::<BindsState>();
            if !binds.get_search_query().is_empty() {
                binds.set_search_query("".into());
            }
            refresh_catalog_view(&binds, &state_tab.borrow(), &model_tab);
        }
    });

    let ui_weak = ui.as_weak();
    let state_view = state.clone();
    let model_view = rows_model.clone();
    let settings_path_view = settings_path;
    binds.on_view_mode_changed(move |grid_mode| {
        {
            let mut st = state_view.borrow_mut();
            st.is_grid_view = grid_mode;
        }
        crate::save_content_view_mode_setting(&settings_path_view, grid_mode);
        if let Some(ui) = ui_weak.upgrade() {
            ui.set_content_is_grid_view(grid_mode);
            let binds = ui.global::<BindsState>();
            refresh_catalog_view(&binds, &state_view.borrow(), &model_view);
        }
    });

    let ui_weak = ui.as_weak();
    let state_search = state.clone();
    let model_search = rows_model.clone();
    binds.on_search_query_changed(move |query| {
        {
            let mut st = state_search.borrow_mut();
            st.search_query = query.to_string();
        }
        if let Some(ui) = ui_weak.upgrade() {
            let binds = ui.global::<BindsState>();
            let st = state_search.borrow();
            if binds.get_search_query().as_str() != st.search_query.as_str() {
                binds.set_search_query(st.search_query.clone().into());
            }
            refresh_catalog_view(&binds, &st, &model_search);
        }
    });

    let ui_weak = ui.as_weak();
    let state_toggle = state.clone();
    let model_for_toggle = rows_model.clone();
    binds.on_command_toggled(move |cmd_idx, checked| {
        if cmd_idx >= 0 {
            let idx_usize = cmd_idx as usize;
            let mut st = state_toggle.borrow_mut();
            if let Some(slot) = st.selected_flags.get_mut(idx_usize) {
                *slot = checked;
            }
        }
        sync_command_check_in_visible_rows(&model_for_toggle, cmd_idx, checked);
        if let Some(ui) = ui_weak.upgrade() {
            let binds = ui.global::<BindsState>();
            let st = state_toggle.borrow();
            binds.set_category_tabs(ModelRc::new(VecModel::from(build_category_tabs(&st))));
            sync_binds_summary(&binds, &st.selected_flags, st.lang);
            binds.set_status_kind("info".into());
        }
    });

    let ui_weak = ui.as_weak();
    let state_clear = state.clone();
    let model_for_clear = rows_model.clone();
    binds.on_clear_selected_commands(move || {
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
            let binds = ui.global::<BindsState>();
            let st = state_clear.borrow();
            binds.set_category_tabs(ModelRc::new(VecModel::from(build_category_tabs(&st))));
            sync_binds_summary(&binds, &st.selected_flags, st.lang);
            binds.set_status_kind("info".into());
            binds.set_status_message(i18n::binds_commands_cleared(st.lang).into());
        }
    });

    let ui_weak = ui.as_weak();
    let dirs_unbind_toggle = dirs_rc.clone();
    let state_unbind_toggle = state.clone();
    binds.on_unbind_panel_toggled(move |open| {
        if open {
            if let Some(ui) = ui_weak.upgrade() {
                let binds = ui.global::<BindsState>();
                let lang = state_unbind_toggle.borrow().lang;
                sync_unbind_preview(&binds, &dirs_unbind_toggle.borrow(), lang);
            }
        }
    });

    let ui_weak = ui.as_weak();
    let dirs_remove = dirs_rc.clone();
    let state_remove = state.clone();
    binds.on_remove_bind_clicked(move || {
        if let Some(ui) = ui_weak.upgrade() {
            execute_remove_bind(&ui, &state_remove.borrow(), &dirs_remove.borrow());
        }
    });

    let ui_weak = ui.as_weak();
    let dirs_save = dirs_rc.clone();
    let state_save = state.clone();
    binds.on_save_bind_clicked(move || {
        if let Some(ui) = ui_weak.upgrade() {
            execute_save_bind(&ui, &state_save.borrow(), &dirs_save.borrow(), false);
        }
    });

    let ui_weak = ui.as_weak();
    let dirs_confirm = dirs_rc.clone();
    let state_confirm = state.clone();
    binds.on_confirm_conflict_overwrite(move || {
        if let Some(ui) = ui_weak.upgrade() {
            execute_save_bind(&ui, &state_confirm.borrow(), &dirs_confirm.borrow(), true);
        }
    });

    let ui_weak = ui.as_weak();
    let state_cancel_conflict = state.clone();
    binds.on_cancel_conflict_overwrite(move || {
        if let Some(ui) = ui_weak.upgrade() {
            let binds = ui.global::<BindsState>();
            let lang = state_cancel_conflict.borrow().lang;
            binds.set_status_kind("info".into());
            binds.set_status_message(i18n::binds_overwrite_cancelled(lang).into());
        }
    });

    BindsController {
        state,
        rows_model,
        dirs: dirs_rc,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AppTray, MainWindow};

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

    #[test]
    fn test_binds_view_integrated_contracts_and_navigation() {
        let temp = tempfile::tempdir().expect("Falha ao criar tempdir");
        let local_cfg = temp.path().join("local_cfg");
        let bkp_cfg = local_cfg.join("bkp");
        let cs2_cfg = temp.path().join("cs2_cfg");
        std::fs::create_dir_all(&local_cfg).unwrap();
        std::fs::create_dir_all(&cs2_cfg).unwrap();

        let original_autoexec = "bind \"f4\" \"buy ak47;buy vesthelm\"\n";
        std::fs::write(local_cfg.join("autoexec.cfg"), original_autoexec).unwrap();

        let dirs = BindDirectories::with_paths(&local_cfg, &bkp_cfg, &cs2_cfg);
        let ui = MainWindow::new().expect("Falha ao instanciar MainWindow");
        let app_tray = AppTray::new().expect("Falha ao instanciar AppTray");
        let settings_path = temp.path().join("settings.json");

        assert!(!ui.get_show_binds_view(), "Painel deve iniciar sem view de binds aberta");

        crate::tray::setup_tray(&ui, &app_tray, settings_path.clone(), false);
        let controller = wire_binds_view(&ui, dirs.clone(), settings_path.clone(), false);

        let binds = ui.global::<BindsState>();

        // Simula ação de entrada na tela de Binds a partir do card no Painel
        let ui_weak = ui.as_weak();
        let dirs_card = dirs.clone();
        ui.on_card_action_clicked(move |module_id| {
            if let Some(ui) = ui_weak.upgrade() {
                if module_id.as_str() == "binds" {
                    refresh_existing_cfg_files(&ui, &dirs_card);
                    ui.set_show_binds_view(true);
                }
            }
        });
        ui.invoke_card_action_clicked("binds".into());
        assert!(ui.get_show_binds_view(), "Clique em Gerenciar Binds deve abrir BindsView");

        // 1. Validar 7 abas de categorias e soma total de 237 comandos
        let tabs = binds.get_category_tabs();
        assert_eq!(tabs.row_count(), 7);
        assert_eq!(tabs.row_data(0).unwrap().title, "1. Movimentação");
        assert_eq!(
            tabs.row_data(6).unwrap().title,
            "7. Outros (Áudio, Demos/Replays, HUD e Sistema)"
        );

        let mut total_across_tabs = 0usize;
        for tab_idx in 0..7 {
            binds.invoke_category_tab_selected(tab_idx);
            let tab_cmd_count = count_visible_commands_in_model(&binds.get_catalog_rows());
            assert_eq!(
                tab_cmd_count as i32,
                tabs.row_data(tab_idx as usize).unwrap().total_count
            );
            total_across_tabs += tab_cmd_count;
        }
        assert_eq!(total_across_tabs, 237);

        // 2. Alternância Lista <-> Grade mantendo contagem, destaque de sv_cheats e separador
        binds.invoke_category_tab_selected(1);
        let list_training_rows = binds.get_catalog_rows();
        assert_eq!(list_training_rows.row_data(1).unwrap().row_kind, 1);
        assert!(list_training_rows.row_data(1).unwrap().featured);
        assert_eq!(list_training_rows.row_data(2).unwrap().row_kind, 2);
        assert_eq!(list_training_rows.row_data(2).unwrap().command_value, "sv_cheats 0");
        assert!(list_training_rows.row_data(2).unwrap().featured);
        assert_eq!(list_training_rows.row_data(3).unwrap().row_kind, 2);
        assert_eq!(list_training_rows.row_data(3).unwrap().command_value, "sv_cheats 1");
        assert!(list_training_rows.row_data(3).unwrap().featured);
        assert_eq!(list_training_rows.row_data(4).unwrap().row_kind, 4);

        binds.invoke_view_mode_changed(true);
        assert!(ui.get_content_is_grid_view());
        assert_eq!(
            counterrush_core::config::AppSettings::load_from_file(&settings_path)
                .unwrap()
                .content_view_mode,
            counterrush_core::config::ContentViewMode::Grid
        );
        let grid_rows = binds.get_catalog_rows();
        assert_eq!(grid_rows.row_data(1).unwrap().row_kind, 1);
        assert!(grid_rows.row_data(1).unwrap().featured);
        assert_eq!(grid_rows.row_data(2).unwrap().row_kind, 3);
        assert_eq!(grid_rows.row_data(2).unwrap().cell_0.command_value, "sv_cheats 0");
        assert!(grid_rows.row_data(2).unwrap().cell_0.featured);
        assert_eq!(grid_rows.row_data(2).unwrap().cell_1.command_value, "sv_cheats 1");
        assert!(grid_rows.row_data(2).unwrap().cell_1.featured);
        assert!(!grid_rows.row_data(2).unwrap().cell_2.present);
        assert_eq!(grid_rows.row_data(3).unwrap().row_kind, 4);

        let mut has_grid_row = false;
        for i in 0..grid_rows.row_count() {
            let row = grid_rows.row_data(i).unwrap();
            assert_ne!(row.row_kind, 2);
            if row.row_kind == 3 {
                has_grid_row = true;
                for cell in [&row.cell_0, &row.cell_1, &row.cell_2, &row.cell_3] {
                    if cell.present {
                        assert!(!cell.description.is_empty());
                    }
                }
            }
        }
        assert!(has_grid_row);
        assert_eq!(count_visible_commands_in_model(&grid_rows), 37);

        binds.invoke_view_mode_changed(false);
        assert!(!ui.get_content_is_grid_view());
        assert_eq!(
            counterrush_core::config::AppSettings::load_from_file(&settings_path)
                .unwrap()
                .content_view_mode,
            counterrush_core::config::ContentViewMode::List
        );
        assert_eq!(count_visible_commands_in_model(&binds.get_catalog_rows()), 37);

        // 3. Pesquisa global por comando e descrição (sem acento)
        binds.invoke_search_query_changed("noclip".into());
        assert_eq!(binds.get_filtered_commands_count(), 1);
        binds.invoke_search_query_changed("fumaca".into());
        assert!(binds.get_filtered_commands_count() >= 1);
        binds.invoke_search_query_changed("termo_inexistente_xyz".into());
        assert_eq!(binds.get_filtered_commands_count(), 0);
        assert_eq!(binds.get_catalog_rows().row_count(), 0);

        binds.invoke_category_tab_selected(0);
        assert_eq!(binds.get_search_query(), "");

        // 4. Conversão de teclas
        assert_eq!(binds.invoke_resolve_cs2_key(" ".into()), "space");
        assert_eq!(
            binds.invoke_resolve_cs2_key(char::from(slint::platform::Key::UpArrow).to_string().into()),
            "uparrow"
        );
        assert_eq!(
            binds.invoke_resolve_cs2_key(char::from(slint::platform::Key::F4).to_string().into()),
            "f4"
        );
        assert_eq!(binds.invoke_resolve_cs2_key(";".into()), "semicolon");
        assert_eq!(binds.invoke_resolve_cs2_key("\"".into()), "");

        // 5. Preservação de estado entre abas + retorno para Painel + salvamento de novo arquivo
        let noclip_idx = find_catalog_command_index("noclip");
        let rethrow_idx = find_catalog_command_index("sv_rethrow_last_grenade");
        let jump_idx = find_catalog_command_index("+jump");
        let smoke_idx = find_catalog_command_index("slot8");

        binds.set_file_name_input("treino_smokes".into());
        binds.invoke_file_name_edited("treino_smokes".into());
        assert!(binds.get_is_creating_new_file());

        binds.set_bound_key("v".into());
        binds.invoke_key_captured("v".into());
        binds.invoke_category_tab_selected(1);
        binds.invoke_command_toggled(noclip_idx, true);
        binds.invoke_view_mode_changed(true);
        binds.invoke_command_toggled(rethrow_idx, true);

        // Voltar ao Painel ou trocar de aba no tray deve preservar estado no BindsState
        app_tray.invoke_open_tab(1);
        assert!(!ui.get_show_binds_view());
        app_tray.invoke_open_tab(0);
        ui.invoke_card_action_clicked("binds".into());
        assert!(ui.get_show_binds_view());
        assert_eq!(binds.get_selected_commands_count(), 2);
        assert_eq!(
            binds.get_bind_preview_line(),
            "bind \"v\" \"noclip;sv_rethrow_last_grenade\""
        );

        binds.invoke_save_bind_clicked();
        assert_eq!(
            std::fs::read_to_string(local_cfg.join("treino_smokes.cfg")).unwrap(),
            "bind \"v\" \"noclip;sv_rethrow_last_grenade\"\n"
        );

        // 6. Atualizar existente mantendo o nome
        binds.invoke_clear_selected_commands();
        binds.set_selected_existing_file("treino_smokes.cfg".into());
        binds.set_file_name_input("treino_smokes.cfg".into());
        binds.invoke_existing_file_selected("treino_smokes.cfg".into());
        assert!(!binds.get_is_creating_new_file());

        binds.set_bound_key("mwheeldown".into());
        binds.invoke_key_captured("mwheeldown".into());
        binds.invoke_command_toggled(jump_idx, true);
        binds.invoke_save_bind_clicked();

        let treino_updated = std::fs::read_to_string(local_cfg.join("treino_smokes.cfg")).unwrap();
        assert!(treino_updated.contains("bind \"v\" \"noclip;sv_rethrow_last_grenade\""));
        assert!(treino_updated.contains("bind \"mwheeldown\" \"+jump\""));

        // 7. Renomear arquivo selecionado cria NOVO arquivo preservando o original
        binds.invoke_clear_selected_commands();
        binds.set_selected_existing_file("autoexec.cfg".into());
        binds.set_file_name_input("autoexec.cfg".into());
        binds.invoke_existing_file_selected("autoexec.cfg".into());
        binds.set_file_name_input("autoexec_smokes.cfg".into());
        binds.invoke_file_name_edited("autoexec_smokes.cfg".into());
        assert!(binds.get_is_creating_new_file());

        binds.set_bound_key("c".into());
        binds.invoke_key_captured("c".into());
        binds.invoke_command_toggled(smoke_idx, true);
        binds.invoke_save_bind_clicked();

        assert_eq!(
            std::fs::read_to_string(local_cfg.join("autoexec.cfg")).unwrap(),
            original_autoexec
        );
        assert_eq!(
            std::fs::read_to_string(local_cfg.join("autoexec_smokes.cfg")).unwrap(),
            "bind \"c\" \"slot8\"\n"
        );

        // 8. Detecção e confirmação de conflito
        binds.invoke_clear_selected_commands();
        binds.set_selected_existing_file("autoexec.cfg".into());
        binds.set_file_name_input("autoexec.cfg".into());
        binds.invoke_existing_file_selected("autoexec.cfg".into());
        binds.set_bound_key("f4".into());
        binds.invoke_key_captured("f4".into());
        binds.invoke_command_toggled(smoke_idx, true);
        binds.invoke_save_bind_clicked();

        assert!(binds.get_show_conflict_prompt());
        binds.invoke_confirm_conflict_overwrite();
        assert!(!binds.get_show_conflict_prompt());
        assert_eq!(
            std::fs::read_to_string(local_cfg.join("autoexec.cfg")).unwrap(),
            "bind \"f4\" \"slot8\"\n"
        );

        assert_binds_view_full_localization_switching(&ui, &controller);
    }

    fn assert_binds_view_full_localization_switching(ui: &MainWindow, controller: &BindsController) {
        let binds = ui.global::<BindsState>();

        // Resetar estado da BindsView para validar troca limpa de idioma
        binds.invoke_clear_selected_commands();
        binds.set_selected_existing_file("".into());
        binds.set_file_name_input("".into());
        binds.invoke_file_name_edited("".into());
        binds.set_bound_key("".into());
        binds.invoke_key_captured("".into());
        binds.invoke_category_tab_selected(0);
        binds.invoke_view_mode_changed(false);
        binds.set_status_message(i18n::binds_default_status(UiLanguage::PtBr).into());

        // Padrão inicial PT-BR
        assert_eq!(binds.get_save_mode_badge(), "NOVO ARQUIVO");
        assert_eq!(
            binds.get_category_tabs().row_data(0).unwrap().tab_label,
            "Movimentação"
        );

        // Alternar para EN-US
        assert!(slint::select_bundled_translation("en_US").is_ok());
        controller.refresh_for_language(ui, UiLanguage::EnUs);
        assert_eq!(binds.get_save_mode_badge(), "NEW FILE");
        assert_eq!(
            binds.get_status_message(),
            i18n::binds_default_status(UiLanguage::EnUs)
        );
        let tabs_en = binds.get_category_tabs();
        assert_eq!(tabs_en.row_data(0).unwrap().tab_label, "Movement");
        assert_eq!(tabs_en.row_data(0).unwrap().title, "1. Movement");
        assert_eq!(tabs_en.row_data(1).unwrap().subtitle, "Requires sv_cheats 1");

        // Verificar linha de comando em modo Lista e modo Grade (EN-US)
        let rows_en = binds.get_catalog_rows();
        assert_eq!(rows_en.row_data(0).unwrap().title, "1. Movement");
        assert_eq!(rows_en.row_data(1).unwrap().title, "Core Commands");
        assert_eq!(rows_en.row_data(2).unwrap().subtitle, "Move forward (W).");

        binds.invoke_view_mode_changed(true);
        let grid_en = binds.get_catalog_rows();
        assert_eq!(
            grid_en.row_data(2).unwrap().cell_0.description,
            "Move forward (W)."
        );
        assert_eq!(grid_en.row_data(2).unwrap().cell_0.info, "Core Commands");

        binds.invoke_search_query_changed("walls".into());
        assert!(binds.get_filtered_commands_count() >= 1);
        binds.invoke_category_tab_selected(0);

        // Alternar para RU-RU e validar busca multilíngue em cirílico (incluindo ё -> е)
        assert!(slint::select_bundled_translation("ru_RU").is_ok());
        controller.refresh_for_language(ui, UiLanguage::RuRu);
        assert_eq!(binds.get_save_mode_badge(), "НОВЫЙ ФАЙЛ");
        assert_eq!(
            binds.get_status_message(),
            i18n::binds_default_status(UiLanguage::RuRu)
        );
        assert_eq!(
            binds.get_category_tabs().row_data(0).unwrap().tab_label,
            "Движение"
        );
        binds.invoke_view_mode_changed(false);
        assert_eq!(
            binds.get_catalog_rows().row_data(2).unwrap().subtitle,
            "Движение вперёд (W)."
        );

        binds.invoke_search_query_changed("сквозь стены".into());
        assert_eq!(binds.get_filtered_commands_count(), 2);
        binds.invoke_search_query_changed("свободный полет".into());
        assert_eq!(binds.get_filtered_commands_count(), 1);
        binds.invoke_category_tab_selected(0);

        // Validação com arquivo ausente em RU-RU
        binds.set_file_name_input("".into());
        binds.invoke_save_bind_clicked();
        assert_eq!(
            binds.get_status_message(),
            i18n::binds_err_missing_file(UiLanguage::RuRu)
        );

        // Restaurar para PT-BR
        assert!(slint::select_bundled_translation("pt_BR").is_ok());
        controller.refresh_for_language(ui, UiLanguage::PtBr);
        assert_eq!(binds.get_save_mode_badge(), "NOVO ARQUIVO");
        assert_eq!(
            binds.get_category_tabs().row_data(0).unwrap().tab_label,
            "Movimentação"
        );
    }

    #[test]
    fn test_binds_view_initial_grid_state_and_controller_sync() {
        let temp = tempfile::tempdir().expect("Falha ao criar tempdir");
        let local_cfg = temp.path().join("local_cfg");
        let bkp_cfg = local_cfg.join("bkp");
        let cs2_cfg = temp.path().join("cs2_cfg");
        std::fs::create_dir_all(&local_cfg).unwrap();
        std::fs::create_dir_all(&cs2_cfg).unwrap();

        let dirs = BindDirectories::with_paths(&local_cfg, &bkp_cfg, &cs2_cfg);
        let ui = MainWindow::new().expect("Falha ao instanciar MainWindow");
        let settings_path = temp.path().join("settings.json");

        let controller = wire_binds_view(&ui, dirs, settings_path, true);
        let binds = ui.global::<BindsState>();

        assert!(binds.get_is_grid_view());
        assert!(ui.get_content_is_grid_view());
        let rows_grid = binds.get_catalog_rows();
        assert!((0..rows_grid.row_count()).any(|i| rows_grid.row_data(i).unwrap().row_kind == 3));
        assert!(!(0..rows_grid.row_count()).any(|i| rows_grid.row_data(i).unwrap().row_kind == 2));

        controller.set_view_mode(&ui, false);
        assert!(!binds.get_is_grid_view());
        let rows_list = binds.get_catalog_rows();
        assert!((0..rows_list.row_count()).any(|i| rows_list.row_data(i).unwrap().row_kind == 2));
        assert!(!(0..rows_list.row_count()).any(|i| rows_list.row_data(i).unwrap().row_kind == 3));
    }

    #[test]
    fn test_binds_view_separated_unbind_flow_and_i18n() {
        let temp = tempfile::tempdir().expect("Falha ao criar tempdir");
        let local_cfg = temp.path().join("local_cfg");
        let bkp_cfg = local_cfg.join("bkp");
        let cs2_cfg = temp.path().join("cs2_cfg");
        std::fs::create_dir_all(&local_cfg).unwrap();
        std::fs::create_dir_all(&cs2_cfg).unwrap();

        let original_autoexec = "bind \"f4\" \"buy ak47;buy vesthelm\"\nbind \"v\" \"noclip\"\n";
        std::fs::write(local_cfg.join("autoexec.cfg"), original_autoexec).unwrap();

        let dirs = BindDirectories::with_paths(&local_cfg, &bkp_cfg, &cs2_cfg);
        let ui = MainWindow::new().expect("Falha ao instanciar MainWindow");
        let settings_path = temp.path().join("settings.json");

        let controller = wire_binds_view(&ui, dirs, settings_path, false);
        let binds = ui.global::<BindsState>();

        // 1. unbind não deve constar na pesquisa do catálogo comum
        binds.invoke_search_query_changed("unbind".into());
        assert_eq!(binds.get_filtered_commands_count(), 0);
        assert_eq!(binds.get_catalog_rows().row_count(), 0);
        binds.invoke_category_tab_selected(6);
        assert!((0..binds.get_catalog_rows().row_count()).all(|i| {
            let r = binds.get_catalog_rows().row_data(i).unwrap();
            !r.command_value.as_str().to_ascii_lowercase().starts_with("unbind")
        }));

        // 2. Selecionar autoexec.cfg e tecla f4 -> abrir painel e detectar bind existente
        binds.set_selected_existing_file("autoexec.cfg".into());
        binds.set_file_name_input("autoexec.cfg".into());
        binds.invoke_existing_file_selected("autoexec.cfg".into());
        binds.set_bound_key("f4".into());
        binds.invoke_key_captured("f4".into());
        binds.set_unbind_panel_open(true);
        binds.invoke_unbind_panel_toggled(true);

        assert_eq!(binds.get_unbind_preview_line(), "unbind \"f4\"");
        assert!(binds.get_unbind_existing_info().contains("buy ak47;buy vesthelm"));

        // 3. Troca de idioma atualiza a dica do painel de unbind
        assert!(slint::select_bundled_translation("en_US").is_ok());
        controller.refresh_for_language(&ui, UiLanguage::EnUs);
        assert!(binds.get_unbind_existing_info().contains("Active bind found in 'autoexec.cfg'"));

        assert!(slint::select_bundled_translation("ru_RU").is_ok());
        controller.refresh_for_language(&ui, UiLanguage::RuRu);
        assert!(binds.get_unbind_existing_info().contains("Найден активный бинд"));

        assert!(slint::select_bundled_translation("pt_BR").is_ok());
        controller.refresh_for_language(&ui, UiLanguage::PtBr);
        assert!(binds.get_unbind_existing_info().contains("Bind ativa encontrada"));

        // 4. Executar remoção -> grava unbind "f4", mantém bind "v", cria backup e fecha painel
        binds.invoke_remove_bind_clicked();
        assert!(!binds.get_unbind_panel_open());
        assert_eq!(binds.get_status_kind(), "success");
        assert_eq!(
            std::fs::read_to_string(bkp_cfg.join("autoexec.cfg")).unwrap(),
            original_autoexec
        );
        assert_eq!(
            std::fs::read_to_string(local_cfg.join("autoexec.cfg")).unwrap(),
            "unbind \"f4\"\nbind \"v\" \"noclip\"\n"
        );

        // 5. Criação de novo .cfg diretamente com unbind "v"
        binds.set_selected_existing_file("".into());
        binds.set_file_name_input("limpeza".into());
        binds.invoke_file_name_edited("limpeza".into());
        binds.set_bound_key("v".into());
        binds.invoke_key_captured("v".into());
        binds.invoke_remove_bind_clicked();
        assert_eq!(
            std::fs::read_to_string(local_cfg.join("limpeza.cfg")).unwrap(),
            "unbind \"v\"\n"
        );

        // 6. Re-vincular f4 em autoexec.cfg remove o unbind "f4" anterior
        let smoke_idx = find_catalog_command_index("slot8");
        binds.set_selected_existing_file("autoexec.cfg".into());
        binds.set_file_name_input("autoexec.cfg".into());
        binds.invoke_existing_file_selected("autoexec.cfg".into());
        binds.set_bound_key("f4".into());
        binds.invoke_key_captured("f4".into());
        binds.invoke_command_toggled(smoke_idx, true);
        binds.invoke_save_bind_clicked();
        assert_eq!(
            std::fs::read_to_string(local_cfg.join("autoexec.cfg")).unwrap(),
            "bind \"v\" \"noclip\"\nbind \"f4\" \"slot8\"\n"
        );
    }
}
