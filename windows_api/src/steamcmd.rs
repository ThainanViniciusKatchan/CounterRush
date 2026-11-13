use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, SystemTime};
use sysinfo::{ProcessRefreshKind, System, UpdateKind};
use crate::WindowsApiError;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub const DEFAULT_STEAMCMD_DIR: &str = r"C:\steamcmd";
pub const CS2_APP_ID: u32 = 730;
const LOG_TAIL_MAX_BYTES: u64 = 65_536;
const LOG_TAIL_MAX_LINES: usize = 250;
const ORPHAN_LOG_MAX_AGE: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadActivityState {
    Idle,
    Downloading,
    Validating,
    Updating,
    BootstrapUpdating,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SteamCmdLogStatus {
    pub state: DownloadActivityState,
    pub progress_percent: Option<f32>,
    pub source: String,
    pub detail: String,
}

impl SteamCmdLogStatus {
    pub fn idle() -> Self {
        Self {
            state: DownloadActivityState::Idle,
            progress_percent: None,
            source: String::new(),
            detail: String::new(),
        }
    }

    pub fn is_active(&self) -> bool {
        !matches!(self.state, DownloadActivityState::Idle)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SteamCmdStdoutEvent {
    Progress {
        state: DownloadActivityState,
        percent: Option<f32>,
        raw_line: String,
    },
    Completed {
        app_id: u32,
        raw_line: String,
    },
    Error {
        message: String,
        raw_line: String,
    },
    Ignored,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SteamCmdProcessInfo {
    pub pid: u32,
    pub command_line: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SteamCmdUpdateSummary {
    pub exit_code: Option<i32>,
    pub completed: bool,
    pub final_state: DownloadActivityState,
    pub last_progress_percent: Option<f32>,
    pub error_message: Option<String>,
}

pub fn read_last_lines<P: AsRef<Path>>(path: P, max_lines: usize) -> std::io::Result<Vec<String>> {
    let path_ref = path.as_ref();
    if !path_ref.exists() || !path_ref.is_file() || max_lines == 0 {
        return Ok(Vec::new());
    }

    let mut file = File::open(path_ref)?;
    let total_len = file.metadata()?.len();
    if total_len == 0 {
        return Ok(Vec::new());
    }

    let start_pos = total_len.saturating_sub(LOG_TAIL_MAX_BYTES);
    file.seek(SeekFrom::Start(start_pos))?;

    let mut buffer = Vec::with_capacity((total_len - start_pos) as usize);
    file.read_to_end(&mut buffer)?;

    let text = String::from_utf8_lossy(&buffer);
    let slice = if start_pos > 0 {
        match text.find('\n') {
            Some(idx) => &text[idx + 1..],
            None => text.as_ref(),
        }
    } else {
        text.as_ref()
    };

    let mut lines: Vec<String> = slice
        .lines()
        .map(|l| l.trim_end_matches('\r').to_string())
        .collect();

    if lines.len() > max_lines {
        let keep_from = lines.len() - max_lines;
        lines.drain(..keep_from);
    }

    Ok(lines)
}

fn is_file_modified_within<P: AsRef<Path>>(path: P, max_age: Duration) -> bool {
    let Ok(meta) = std::fs::metadata(path.as_ref()) else {
        return false;
    };
    let Ok(modified) = meta.modified() else {
        return false;
    };
    match SystemTime::now().duration_since(modified) {
        Ok(elapsed) => elapsed <= max_age,
        Err(_) => true,
    }
}

fn contains_numeric_token(text_lower: &str, token: &str) -> bool {
    let bytes = text_lower.as_bytes();
    let mut offset = 0;
    while let Some(pos) = text_lower[offset..].find(token) {
        let abs_start = offset + pos;
        let abs_end = abs_start + token.len();

        let prev_ok = abs_start == 0 || !bytes[abs_start - 1].is_ascii_digit();
        let next_ok = abs_end >= bytes.len() || !bytes[abs_end].is_ascii_digit();

        if prev_ok && next_ok {
            return true;
        }
        offset = abs_end;
    }
    false
}

fn line_matches_app_id(lower_line: &str, app_id: u32) -> bool {
    let id_str = app_id.to_string();
    let appid_pattern = format!("appid {id_str}");
    let quoted_app_pattern = format!("app '{id_str}'");
    let app_update_pattern = format!("app_update {id_str}");

    contains_numeric_token(lower_line, &appid_pattern)
        || lower_line.contains(&quoted_app_pattern)
        || contains_numeric_token(lower_line, &app_update_pattern)
}

fn extract_another_app_id_active(lower_line: &str, target_app_id: u32) -> bool {
    if !lower_line.contains("appid ") {
        return false;
    }
    !line_matches_app_id(lower_line, target_app_id)
}

fn classify_update_flags(flags_lower: &str) -> Option<DownloadActivityState> {
    let trimmed = flags_lower.trim();
    if trimmed == "none" || trimmed.is_empty() {
        return None;
    }
    if trimmed.contains("verifying") || trimmed.contains("validating") {
        return Some(DownloadActivityState::Validating);
    }
    if trimmed.contains("downloading") {
        return Some(DownloadActivityState::Downloading);
    }
    if trimmed.contains("running update")
        || trimmed.contains("reconfiguring")
        || trimmed.contains("preallocating")
        || trimmed.contains("staging")
        || trimmed.contains("committing")
    {
        return Some(DownloadActivityState::Updating);
    }
    None
}

fn extract_progress_percent(lower_line: &str) -> Option<f32> {
    let marker = "progress:";
    let idx = lower_line.find(marker)?;
    let after = lower_line[idx + marker.len()..].trim_start();
    let num_str: String = after
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    if num_str.is_empty() {
        return None;
    }
    let val = num_str.parse::<f32>().ok()?;
    if (0.0..=100.0).contains(&val) {
        Some(val)
    } else {
        None
    }
}

fn extract_kb_progress_percent(lower_line: &str) -> Option<f32> {
    let start_paren = lower_line.find('(')?;
    let after_paren = &lower_line[start_paren + 1..];
    let of_idx = after_paren.find(" of ")?;
    let current_raw = &after_paren[..of_idx];
    let rest = &after_paren[of_idx + 4..];
    let kb_idx = rest.find(" kb")?;
    let total_raw = &rest[..kb_idx];

    let parse_num = |s: &str| -> Option<f64> {
        let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
        digits.parse::<f64>().ok()
    };

    let current = parse_num(current_raw)?;
    let total = parse_num(total_raw)?;
    if total > 0.0 {
        Some(((current / total) * 100.0).clamp(0.0, 100.0) as f32)
    } else {
        None
    }
}

pub fn parse_steamcmd_stdout_line(line: &str, app_id: u32) -> SteamCmdStdoutEvent {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return SteamCmdStdoutEvent::Ignored;
    }

    let lower = trimmed.to_ascii_lowercase();
    let success_installed = format!("success! app '{app_id}'");
    let up_to_date = format!("app '{app_id}' already up to date");
    if lower.contains(&success_installed) || lower.contains(&up_to_date) {
        return SteamCmdStdoutEvent::Completed {
            app_id,
            raw_line: trimmed.to_string(),
        };
    }

    let app_error_prefix = format!("app '{app_id}'");
    if (lower.contains("error!") && (lower.contains(&app_error_prefix) || lower.contains("failed to install app") || lower.contains("missing") || lower.contains("timed out")))
        || lower.starts_with("error! app ")
    {
        return SteamCmdStdoutEvent::Error {
            message: trimmed.to_string(),
            raw_line: trimmed.to_string(),
        };
    }

    if lower.contains("update state (") {
        let percent = extract_progress_percent(&lower);
        let state = if lower.contains("downloading") {
            DownloadActivityState::Downloading
        } else if lower.contains("verifying") || lower.contains("validating") {
            DownloadActivityState::Validating
        } else {
            DownloadActivityState::Updating
        };
        return SteamCmdStdoutEvent::Progress {
            state,
            percent,
            raw_line: trimmed.to_string(),
        };
    }

    SteamCmdStdoutEvent::Ignored
}

pub fn parse_content_log_lines(lines: &[String], app_id: u32) -> SteamCmdLogStatus {
    let start_idx = lines
        .iter()
        .rposition(|l| l.to_ascii_lowercase().contains("client version:"))
        .unwrap_or(0);

    let mut app_update_state: Option<(DownloadActivityState, String)> = None;
    let mut workshop_update_state: Option<(DownloadActivityState, String)> = None;
    let mut state_running = false;
    let mut last_percent: Option<f32> = None;
    let mut tracking_target_app = false;

    for line in &lines[start_idx..] {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let lower = trimmed.to_ascii_lowercase();

        if lower.contains("client version:") {
            app_update_state = None;
            workshop_update_state = None;
            state_running = false;
            last_percent = None;
            tracking_target_app = false;
            continue;
        }

        if line_matches_app_id(&lower, app_id) {
            tracking_target_app = true;

            if lower.contains("start validating appid") {
                app_update_state = Some((DownloadActivityState::Validating, trimmed.to_string()));
                continue;
            }

            if let Some((_, after_colon)) = lower.split_once("app update changed :") {
                app_update_state = classify_update_flags(after_colon).map(|st| (st, trimmed.to_string()));
                continue;
            }

            if let Some((_, after_colon)) = lower.split_once("workshop update changed :") {
                workshop_update_state = classify_update_flags(after_colon).map(|st| (st, trimmed.to_string()));
                continue;
            }

            if let Some((_, after_colon)) = lower.split_once("state changed :") {
                state_running = after_colon.contains("update running") || after_colon.contains("update started");
                continue;
            }

            if lower.contains("scheduler finished") || lower.contains("update canceled") {
                app_update_state = None;
                workshop_update_state = None;
                state_running = false;
                last_percent = None;
                continue;
            }
        } else if extract_another_app_id_active(&lower, app_id)
            && (lower.contains("app update changed :") || lower.contains("start validating appid"))
        {
            tracking_target_app = false;
        }

        if tracking_target_app || line_matches_app_id(&lower, app_id) {
            match parse_steamcmd_stdout_line(trimmed, app_id) {
                SteamCmdStdoutEvent::Progress { state, percent, raw_line } => {
                    if let Some(p) = percent {
                        last_percent = Some(p);
                    }
                    app_update_state = Some((state, raw_line));
                }
                SteamCmdStdoutEvent::Completed { .. } | SteamCmdStdoutEvent::Error { .. } => {
                    app_update_state = None;
                    workshop_update_state = None;
                    state_running = false;
                    last_percent = None;
                }
                SteamCmdStdoutEvent::Ignored => {}
            }
        }
    }

    let candidates = [app_update_state.as_ref(), workshop_update_state.as_ref()];
    for priority in [
        DownloadActivityState::Downloading,
        DownloadActivityState::Validating,
        DownloadActivityState::Updating,
    ] {
        for (st, detail) in candidates.into_iter().flatten() {
            if *st == priority {
                return SteamCmdLogStatus {
                    state: priority,
                    progress_percent: last_percent,
                    source: "logs/content_log.txt".to_string(),
                    detail: detail.clone(),
                };
            }
        }
    }

    if state_running {
        return SteamCmdLogStatus {
            state: DownloadActivityState::Updating,
            progress_percent: last_percent,
            source: "logs/content_log.txt".to_string(),
            detail: format!("AppID {app_id} Update Running"),
        };
    }

    SteamCmdLogStatus::idle()
}

pub fn check_content_log_file<P: AsRef<Path>>(path: P, app_id: u32) -> SteamCmdLogStatus {
    match read_last_lines(path, LOG_TAIL_MAX_LINES) {
        Ok(lines) if !lines.is_empty() => parse_content_log_lines(&lines, app_id),
        _ => SteamCmdLogStatus::idle(),
    }
}

pub fn parse_bootstrap_log_lines(lines: &[String]) -> SteamCmdLogStatus {
    let start_idx = lines
        .iter()
        .rposition(|l| l.to_ascii_lowercase().contains("startup -"))
        .unwrap_or(0);

    let mut current_state = DownloadActivityState::Idle;
    let mut last_detail = String::new();
    let mut last_percent: Option<f32> = None;

    for line in &lines[start_idx..] {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let lower = trimmed.to_ascii_lowercase();

        if lower.contains("startup -") {
            current_state = DownloadActivityState::Idle;
            last_detail.clear();
            last_percent = None;
            continue;
        }

        if lower.contains("shutdown")
            || lower.contains("verification complete")
            || lower.contains("update complete")
            || lower.contains("nothing to do")
        {
            current_state = DownloadActivityState::Idle;
            last_detail = trimmed.to_string();
            last_percent = None;
            continue;
        }

        if lower.contains("verifying all executable checksums")
            || lower.contains("verificando a instalação")
            || lower.contains("verifying")
        {
            current_state = DownloadActivityState::Validating;
            last_detail = trimmed.to_string();
            continue;
        }

        if lower.contains("downloading update") {
            current_state = DownloadActivityState::Downloading;
            last_detail = trimmed.to_string();
            if let Some(pct) = extract_kb_progress_percent(&lower) {
                last_percent = Some(pct);
            }
            continue;
        }

        if lower.contains("checking for update on startup")
            || lower.contains("checking for available update")
            || lower.contains("verificando se há atualizações")
            || lower.contains("downloading manifest:")
            || lower.contains("manifest download:")
            || lower.contains("applying update")
            || lower.contains("extracting package")
            || lower.contains("installing update")
            || lower.contains("found pending update")
        {
            current_state = DownloadActivityState::BootstrapUpdating;
            last_detail = trimmed.to_string();
        }
    }

    if matches!(current_state, DownloadActivityState::Idle) {
        return SteamCmdLogStatus::idle();
    }

    SteamCmdLogStatus {
        state: current_state,
        progress_percent: last_percent,
        source: "logs/bootstrap_log.txt".to_string(),
        detail: last_detail,
    }
}

pub fn check_bootstrap_log_file<P: AsRef<Path>>(path: P) -> SteamCmdLogStatus {
    match read_last_lines(path, LOG_TAIL_MAX_LINES) {
        Ok(lines) if !lines.is_empty() => parse_bootstrap_log_lines(&lines),
        _ => SteamCmdLogStatus::idle(),
    }
}

pub fn check_steamcmd_logs_in_dir<P: AsRef<Path>>(steamcmd_dir: P, app_id: u32) -> SteamCmdLogStatus {
    let dir = steamcmd_dir.as_ref();
    let content_status = check_content_log_file(dir.join("logs").join("content_log.txt"), app_id);
    if content_status.is_active() {
        return content_status;
    }

    let bootstrap_status = check_bootstrap_log_file(dir.join("logs").join("bootstrap_log.txt"));
    if bootstrap_status.is_active() {
        return bootstrap_status;
    }

    SteamCmdLogStatus::idle()
}

pub fn is_command_line_updating_app(cmd_line: &str, app_id: u32) -> bool {
    let lower = cmd_line.to_ascii_lowercase();
    let app_str = app_id.to_string();

    contains_numeric_token(&lower, &format!("app_update {app_str}"))
        || contains_numeric_token(&lower, &format!("workshop_download_item {app_str}"))
        || (contains_numeric_token(&lower, &app_str)
            && (lower.contains("app_update")
                || lower.contains("validate")
                || lower.contains("steamcmd")))
}

pub fn get_running_steamcmd_processes() -> Vec<SteamCmdProcessInfo> {
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessRefreshKind::new().with_cmd(UpdateKind::Always),
    );

    sys.processes()
        .iter()
        .filter_map(|(pid, process)| {
            let name = process.name().to_ascii_lowercase();
            if name == "steamcmd.exe" || name == "steamcmd" {
                let cmd_joined = process.cmd().join(" ");
                Some(SteamCmdProcessInfo {
                    pid: pid.as_u32(),
                    command_line: cmd_joined,
                })
            } else {
                None
            }
        })
        .collect()
}

pub fn monitor_child_stdout<F>(
    child: &mut Child,
    app_id: u32,
    cancel_token: Option<&AtomicBool>,
    mut on_event: F,
) -> Result<SteamCmdUpdateSummary, WindowsApiError>
where
    F: FnMut(&SteamCmdStdoutEvent),
{
    let mut final_state = DownloadActivityState::Updating;
    let mut last_progress: Option<f32> = None;
    let mut completed = false;
    let mut error_message: Option<String> = None;

    if let Some(stdout) = child.stdout.take() {
        let mut reader = BufReader::new(stdout);
        let mut raw_buf = [0u8; 4096];
        let mut line_buf = Vec::new();

        loop {
            if let Some(token) = cancel_token {
                if token.load(Ordering::SeqCst) {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(WindowsApiError::ExecutionFailed(
                        "Execução do SteamCMD cancelada pelo usuário.".to_string(),
                    ));
                }
            }

            let read_bytes = reader.read(&mut raw_buf).map_err(WindowsApiError::Io)?;
            if read_bytes == 0 {
                if !line_buf.is_empty() {
                    let line = String::from_utf8_lossy(&line_buf).to_string();
                    let event = parse_steamcmd_stdout_line(&line, app_id);
                    apply_stdout_event(
                        &event,
                        &mut final_state,
                        &mut last_progress,
                        &mut completed,
                        &mut error_message,
                    );
                    on_event(&event);
                    line_buf.clear();
                }
                break;
            }

            for &byte in &raw_buf[..read_bytes] {
                if byte == b'\n' || byte == b'\r' {
                    if !line_buf.is_empty() {
                        let line = String::from_utf8_lossy(&line_buf).to_string();
                        let event = parse_steamcmd_stdout_line(&line, app_id);
                        apply_stdout_event(
                            &event,
                            &mut final_state,
                            &mut last_progress,
                            &mut completed,
                            &mut error_message,
                        );
                        on_event(&event);
                        line_buf.clear();
                    }
                } else {
                    line_buf.push(byte);
                }
            }
        }
    }

    let status = child.wait().map_err(WindowsApiError::Io)?;
    if completed || (status.success() && error_message.is_none()) {
        final_state = DownloadActivityState::Idle;
    }

    Ok(SteamCmdUpdateSummary {
        exit_code: status.code(),
        completed: completed || (status.success() && error_message.is_none()),
        final_state,
        last_progress_percent: last_progress,
        error_message,
    })
}

fn apply_stdout_event(
    event: &SteamCmdStdoutEvent,
    final_state: &mut DownloadActivityState,
    last_progress: &mut Option<f32>,
    completed: &mut bool,
    error_message: &mut Option<String>,
) {
    match event {
        SteamCmdStdoutEvent::Progress { state, percent, .. } => {
            *final_state = *state;
            if let Some(p) = percent {
                *last_progress = Some(*p);
            }
        }
        SteamCmdStdoutEvent::Completed { .. } => {
            *completed = true;
            *final_state = DownloadActivityState::Idle;
            *last_progress = Some(100.0);
        }
        SteamCmdStdoutEvent::Error { message, .. } => {
            *error_message = Some(message.clone());
        }
        SteamCmdStdoutEvent::Ignored => {}
    }
}

pub struct SteamCmdClient {
    steamcmd_dir: PathBuf,
}

impl Default for SteamCmdClient {
    fn default() -> Self {
        Self::new(DEFAULT_STEAMCMD_DIR)
    }
}

impl SteamCmdClient {
    pub fn new<P: Into<PathBuf>>(steamcmd_dir: P) -> Self {
        Self {
            steamcmd_dir: steamcmd_dir.into(),
        }
    }

    pub fn base_dir(&self) -> &Path {
        &self.steamcmd_dir
    }

    pub fn exe_path(&self) -> PathBuf {
        self.steamcmd_dir.join("steamcmd.exe")
    }

    pub fn content_log_path(&self) -> PathBuf {
        self.steamcmd_dir.join("logs").join("content_log.txt")
    }

    pub fn bootstrap_log_path(&self) -> PathBuf {
        self.steamcmd_dir.join("logs").join("bootstrap_log.txt")
    }

    pub fn downloading_dir(&self) -> PathBuf {
        self.steamcmd_dir.join("steamapps").join("downloading")
    }

    pub fn check_logs_status(&self, app_id: u32) -> SteamCmdLogStatus {
        check_steamcmd_logs_in_dir(&self.steamcmd_dir, app_id)
    }

    pub fn check_validation_status(&self, app_id: u32) -> SteamCmdLogStatus {
        let log_status = self.check_logs_status(app_id);
        let running_procs = get_running_steamcmd_processes();

        if !running_procs.is_empty() {
            if log_status.is_active() {
                return log_status;
            }
            for proc_info in &running_procs {
                if is_command_line_updating_app(&proc_info.command_line, app_id) {
                    return SteamCmdLogStatus {
                        state: DownloadActivityState::Updating,
                        progress_percent: None,
                        source: "steamcmd.exe (process)".to_string(),
                        detail: format!(
                            "PID {} executando atualização/validação: {}",
                            proc_info.pid, proc_info.command_line
                        ),
                    };
                }
            }
        } else if log_status.is_active() {
            let log_file = match log_status.source.as_str() {
                "logs/bootstrap_log.txt" => self.bootstrap_log_path(),
                _ => self.content_log_path(),
            };
            if is_file_modified_within(log_file, ORPHAN_LOG_MAX_AGE) {
                return log_status;
            }
        }

        SteamCmdLogStatus::idle()
    }

    pub fn run_app_update<F>(
        &self,
        app_id: u32,
        validate: bool,
        cancel_token: Option<&AtomicBool>,
        on_event: F,
    ) -> Result<SteamCmdUpdateSummary, WindowsApiError>
    where
        F: FnMut(&SteamCmdStdoutEvent),
    {
        let exe = self.exe_path();
        if !exe.exists() {
            return Err(WindowsApiError::ExecutionFailed(format!(
                "SteamCMD não encontrado em '{}'",
                exe.display()
            )));
        }

        let app_id_arg = app_id.to_string();
        let mut args = vec!["+login", "anonymous", "+app_update", &app_id_arg];
        if validate {
            args.push("validate");
        }
        args.push("+quit");

        let mut cmd = Command::new(&exe);
        cmd.current_dir(&self.steamcmd_dir)
            .args(&args)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());

        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let mut child = cmd.spawn().map_err(WindowsApiError::Io)?;
        let mut summary = monitor_child_stdout(&mut child, app_id, cancel_token, on_event)?;

        let post_log_status = self.check_logs_status(app_id);
        if post_log_status.is_active() {
            summary.final_state = post_log_status.state;
            summary.completed = false;
        }

        Ok(summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_content_log_detects_active_download_and_validation_and_completion() {
        let downloading_lines = vec![
            "[2026-11-11 21:04:22] Client version: 1788292693".to_string(),
            "[2026-11-11 21:04:22] AppID 730 App update changed : Running Update,".to_string(),
            "[2026-11-11 21:04:22] AppID 730 state changed : Update Required,Fully Installed,Update Queued,Update Running,Update Started,".to_string(),
            "[2026-11-11 21:04:22] AppID 730 App update changed : Running Update,Downloading,Staging,".to_string(),
        ];
        let status = parse_content_log_lines(&downloading_lines, 730);
        assert_eq!(status.state, DownloadActivityState::Downloading);
        assert!(status.is_active());

        let validating_lines = vec![
            "[2026-11-11 22:02:50] Client version: 1788292693".to_string(),
            "[2026-11-11 22:02:50] Start validating appID 730 on BuildID 26598758".to_string(),
            "[2026-11-11 22:02:50] AppID 730 state changed : Fully Installed,Update Queued,Update Running,".to_string(),
            "[2026-11-11 22:02:50] AppID 730 App update changed : Running Update,Verifying Installed,".to_string(),
        ];
        let val_status = parse_content_log_lines(&validating_lines, 730);
        assert_eq!(val_status.state, DownloadActivityState::Validating);
        assert!(val_status.is_active());

        let mut finished_lines = validating_lines;
        finished_lines.extend([
            "[2026-11-11 22:06:11] AppID 730 App update changed : Running Update,".to_string(),
            "[2026-11-11 22:06:11] AppID 730 App update changed : None".to_string(),
            "[2026-11-11 22:06:11] AppID 730 state changed : Fully Installed,Update Queued,".to_string(),
            "[2026-11-11 22:06:11] AppID 730 scheduler finished : removed from schedule (result No Error, state 0xc)".to_string(),
        ]);
        let done_status = parse_content_log_lines(&finished_lines, 730);
        assert_eq!(done_status.state, DownloadActivityState::Idle);
        assert!(!done_status.is_active());
    }

    #[test]
    fn test_content_log_ignores_other_app_ids_and_handles_workshop_only() {
        let lines = vec![
            "[2026-11-11 21:00:00] Client version: 1788292693".to_string(),
            "[2026-11-11 21:00:01] AppID 570 App update changed : Running Update,Downloading,".to_string(),
            "[2026-11-11 21:00:02] AppID 7301 App update changed : Running Update,Downloading,".to_string(),
        ];
        let status = parse_content_log_lines(&lines, 730);
        assert!(!status.is_active(), "AppID 570 ou 7301 não deve ativar bloqueio para 730");

        let ws_lines = vec![
            "[2026-10-10 20:11:06] Client version: 1788292693".to_string(),
            "[2026-10-10 20:11:07] AppID 730 Workshop update changed : Running Update,".to_string(),
            "[2026-10-10 20:11:08] AppID 730 Workshop update changed : Running Update,Verifying Installed,".to_string(),
        ];
        let ws_active = parse_content_log_lines(&ws_lines, 730);
        assert_eq!(ws_active.state, DownloadActivityState::Validating);
    }

    #[test]
    fn test_bootstrap_log_detects_active_update_validation_and_shutdown() {
        let downloading = vec![
            "[2026-10-10 19:49:09] Startup - updater built Dec  2 2013 18:43:44".to_string(),
            "[2026-10-10 19:49:11] Downloading update (21,736 of 43,472 KB)...".to_string(),
        ];
        let st = parse_bootstrap_log_lines(&downloading);
        assert_eq!(st.state, DownloadActivityState::Downloading);
        let pct = st.progress_percent.expect("Deve extrair porcentagem de KB");
        assert!((pct - 50.0).abs() < 0.1);

        let verifying = vec![
            "[2026-10-10 20:11:02] Startup - updater built Sep  1 2026 12:04:15".to_string(),
            "[2026-10-10 20:11:02] Verificando a instalação...".to_string(),
            "[2026-10-10 20:11:02] Verifying all executable checksums".to_string(),
        ];
        let val_st = parse_bootstrap_log_lines(&verifying);
        assert_eq!(val_st.state, DownloadActivityState::Validating);

        let mut finished = verifying;
        finished.push("[2026-10-10 20:11:03] Verification complete".to_string());
        finished.push("[2026-10-10 20:11:08] Shutdown".to_string());
        assert!(!parse_bootstrap_log_lines(&finished).is_active());
    }

    #[test]
    fn test_parse_steamcmd_stdout_line_patterns() {
        let dl = parse_steamcmd_stdout_line(
            " Update state (0x61) downloading, progress: 42.75 (123456 / 288785)",
            730,
        );
        assert_eq!(
            dl,
            SteamCmdStdoutEvent::Progress {
                state: DownloadActivityState::Downloading,
                percent: Some(42.75),
                raw_line: "Update state (0x61) downloading, progress: 42.75 (123456 / 288785)".to_string(),
            }
        );

        let val = parse_steamcmd_stdout_line(
            " Update state (0x5) verifying install, progress: 89.10 (100 / 112)",
            730,
        );
        assert!(matches!(
            val,
            SteamCmdStdoutEvent::Progress {
                state: DownloadActivityState::Validating,
                percent: Some(p),
                ..
            } if (p - 89.10).abs() < 0.01
        ));

        let ok = parse_steamcmd_stdout_line("Success! App '730' fully installed.", 730);
        assert!(matches!(ok, SteamCmdStdoutEvent::Completed { app_id: 730, .. }));

        let err = parse_steamcmd_stdout_line("Error! App '730' state is 0x202 after update job.", 730);
        assert!(matches!(err, SteamCmdStdoutEvent::Error { .. }));
    }

    #[test]
    fn test_check_steamcmd_logs_in_dir_and_read_tail() {
        let tmp = tempdir().unwrap();
        let logs_dir = tmp.path().join("logs");
        std::fs::create_dir_all(&logs_dir).unwrap();

        let content_path = logs_dir.join("content_log.txt");
        let bootstrap_path = logs_dir.join("bootstrap_log.txt");

        let mut bulk = String::new();
        for i in 0..300 {
            bulk.push_str(&format!("[2026-10-10 10:00:00] Dummy line {i}\n"));
        }
        bulk.push_str("[2026-10-10 10:05:00] Client version: 1788292693\n");
        bulk.push_str("[2026-10-10 10:05:01] AppID 730 App update changed : Running Update,Downloading,\n");
        std::fs::write(&content_path, bulk).unwrap();
        std::fs::write(&bootstrap_path, "[2026-10-10 10:05:00] Startup - updater\n[2026-10-10 10:05:01] Verification complete\n").unwrap();

        let status = check_steamcmd_logs_in_dir(tmp.path(), 730);
        assert_eq!(status.state, DownloadActivityState::Downloading);
        assert_eq!(status.source, "logs/content_log.txt");

        let client = SteamCmdClient::new(tmp.path());
        let val_status = client.check_validation_status(730);
        assert_eq!(val_status.state, DownloadActivityState::Downloading);
    }

    #[test]
    fn test_is_command_line_updating_app_detection() {
        assert!(is_command_line_updating_app(
            r#""C:\steamcmd\steamcmd.exe" +login anonymous +app_update 730 validate +quit"#,
            730
        ));
        assert!(is_command_line_updating_app(
            r#"C:\steamcmd\steamcmd.exe +workshop_download_item 730 3240880604 +quit"#,
            730
        ));
        assert!(!is_command_line_updating_app(
            r#"C:\steamcmd\steamcmd.exe +login anonymous +app_update 570 validate +quit"#,
            730
        ));
        assert!(!is_command_line_updating_app(
            r#"C:\steamcmd\steamcmd.exe +login anonymous"#,
            730
        ));
    }

    #[test]
    fn test_monitor_child_stdout_streams_progress_until_exit() {
        let mut cmd = Command::new("powershell");
        #[cfg(windows)]
        cmd.creation_flags(CREATE_NO_WINDOW);
        cmd.args([
            "-NoProfile",
            "-Command",
            "Write-Output 'Update state (0x61) downloading, progress: 25.50 (100 / 400)'; Write-Output 'Update state (0x5) verifying install, progress: 90.00 (360 / 400)'; Write-Output \"Success! App '730' fully installed.\"",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::null());

        let mut child = cmd.spawn().unwrap();
        let mut seen_events = Vec::new();
        let summary = monitor_child_stdout(&mut child, 730, None, |ev| {
            if !matches!(ev, SteamCmdStdoutEvent::Ignored) {
                seen_events.push(ev.clone());
            }
        })
        .unwrap();

        assert!(summary.completed);
        assert_eq!(summary.final_state, DownloadActivityState::Idle);
        assert_eq!(seen_events.len(), 3);
        assert!(matches!(
            seen_events[0],
            SteamCmdStdoutEvent::Progress {
                state: DownloadActivityState::Downloading,
                percent: Some(p),
                ..
            } if (p - 25.5).abs() < 0.01
        ));
        assert!(matches!(
            seen_events[1],
            SteamCmdStdoutEvent::Progress {
                state: DownloadActivityState::Validating,
                percent: Some(p),
                ..
            } if (p - 90.0).abs() < 0.01
        ));
        assert!(matches!(
            seen_events[2],
            SteamCmdStdoutEvent::Completed { app_id: 730, .. }
        ));
    }
}
