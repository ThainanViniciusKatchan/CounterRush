use std::process::Command;
use crate::{Tweak, TweakError, TweakStatus};

pub const REG_SYSTEM_PROFILE_KEY: &str =
    r#"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile"#;
pub const REG_GAMES_PROFILE_KEY: &str =
    r#"HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion\Multimedia\SystemProfile\Tasks\Games"#;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MmcssHardwareTier {
    HighEnd,
    EntryLevel,
}

impl std::fmt::Display for MmcssHardwareTier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::HighEnd => write!(f, "PC Forte (6+ núcleos / 16GB+ RAM)"),
            Self::EntryLevel => write!(f, "PC Fraco / Quad-Core / 8GB RAM"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MmcssStatus {
    pub network_throttling_index: Option<u32>,
    pub system_responsiveness: Option<u32>,
    pub gpu_priority: Option<u32>,
    pub priority: Option<u32>,
    pub scheduling_category: Option<String>,
    pub sfio_priority: Option<String>,
    pub tier_detected: MmcssHardwareTier,
}

pub fn detect_mmcss_hardware_tier() -> Result<MmcssHardwareTier, TweakError> {
    let info = windows_api::detect_hardware()?;
    let is_high_end = info.cpu.physical_cores >= 6 && info.ram.total_gb >= 15.0;
    if is_high_end {
        Ok(MmcssHardwareTier::HighEnd)
    } else {
        Ok(MmcssHardwareTier::EntryLevel)
    }
}

fn set_reg_dword(key: &str, value_name: &str, value: u32) -> Result<(), TweakError> {
    let output = Command::new("reg")
        .args([
            "add",
            key,
            "/v",
            value_name,
            "/t",
            "REG_DWORD",
            "/d",
            &value.to_string(),
            "/f",
        ])
        .output()
        .map_err(|e| TweakError::WindowsApi(windows_api::WindowsApiError::Io(e)))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(TweakError::ValidationFailed(format!(
            "Falha ao gravar REG_DWORD '{}' em '{}': {}",
            value_name,
            key,
            stderr.trim()
        )))
    }
}

fn set_reg_sz(key: &str, value_name: &str, value: &str) -> Result<(), TweakError> {
    let output = Command::new("reg")
        .args([
            "add",
            key,
            "/v",
            value_name,
            "/t",
            "REG_SZ",
            "/d",
            value,
            "/f",
        ])
        .output()
        .map_err(|e| TweakError::WindowsApi(windows_api::WindowsApiError::Io(e)))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(TweakError::ValidationFailed(format!(
            "Falha ao gravar REG_SZ '{}' em '{}': {}",
            value_name,
            key,
            stderr.trim()
        )))
    }
}

fn query_reg_value(key: &str, value_name: &str) -> Result<Option<String>, TweakError> {
    let output = Command::new("reg")
        .args(["query", key, "/v", value_name])
        .output()
        .map_err(|e| TweakError::WindowsApi(windows_api::WindowsApiError::Io(e)))?;

    if !output.status.success() {
        return Ok(None);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with(value_name) {
            let after_name = trimmed[value_name.len()..].trim();
            let parts: Vec<&str> = after_name.split_whitespace().collect();
            if parts.len() >= 2 {
                let val = parts[1..].join(" ");
                return Ok(Some(val));
            }
        }
    }

    Ok(None)
}

fn parse_dword_str(val: &str) -> Option<u32> {
    let trimmed = val.trim();
    if trimmed.starts_with("0x") || trimmed.starts_with("0X") {
        u32::from_str_radix(&trimmed[2..], 16).ok()
    } else {
        trimmed.parse::<u32>().ok()
    }
}

pub fn get_mmcss_status() -> Result<MmcssStatus, TweakError> {
    let tier_detected = detect_mmcss_hardware_tier()?;

    let net_throttling_raw = query_reg_value(REG_SYSTEM_PROFILE_KEY, "NetworkThrottlingIndex")?;
    let sys_resp_raw = query_reg_value(REG_SYSTEM_PROFILE_KEY, "SystemResponsiveness")?;
    let gpu_prio_raw = query_reg_value(REG_GAMES_PROFILE_KEY, "GPU Priority")?;
    let prio_raw = query_reg_value(REG_GAMES_PROFILE_KEY, "Priority")?;
    let sched_raw = query_reg_value(REG_GAMES_PROFILE_KEY, "Scheduling Category")?;
    let sfio_raw = query_reg_value(REG_GAMES_PROFILE_KEY, "SFIO Priority")?;

    Ok(MmcssStatus {
        network_throttling_index: net_throttling_raw.as_deref().and_then(parse_dword_str),
        system_responsiveness: sys_resp_raw.as_deref().and_then(parse_dword_str),
        gpu_priority: gpu_prio_raw.as_deref().and_then(parse_dword_str),
        priority: prio_raw.as_deref().and_then(parse_dword_str),
        scheduling_category: sched_raw,
        sfio_priority: sfio_raw,
        tier_detected,
    })
}

pub fn apply_mmcss_with_tier(tier: MmcssHardwareTier) -> Result<(), TweakError> {
    match tier {
        MmcssHardwareTier::HighEnd => {
            set_reg_dword(REG_SYSTEM_PROFILE_KEY, "SystemResponsiveness", 10)?;
            set_reg_dword(REG_SYSTEM_PROFILE_KEY, "NetworkThrottlingIndex", 4294967295)?;
            set_reg_dword(REG_GAMES_PROFILE_KEY, "GPU Priority", 8)?;
            set_reg_dword(REG_GAMES_PROFILE_KEY, "Priority", 6)?;
            set_reg_sz(REG_GAMES_PROFILE_KEY, "Scheduling Category", "High")?;
            set_reg_sz(REG_GAMES_PROFILE_KEY, "SFIO Priority", "High")?;
        }
        MmcssHardwareTier::EntryLevel => {
            set_reg_dword(REG_SYSTEM_PROFILE_KEY, "SystemResponsiveness", 20)?;
            set_reg_dword(REG_SYSTEM_PROFILE_KEY, "NetworkThrottlingIndex", 10)?;
            set_reg_dword(REG_GAMES_PROFILE_KEY, "GPU Priority", 8)?;
            set_reg_dword(REG_GAMES_PROFILE_KEY, "Priority", 6)?;
            set_reg_sz(REG_GAMES_PROFILE_KEY, "Scheduling Category", "High")?;
            set_reg_sz(REG_GAMES_PROFILE_KEY, "SFIO Priority", "Normal")?;
        }
    }

    Ok(())
}

pub fn apply_all_mmcss() -> Result<MmcssHardwareTier, TweakError> {
    let tier = detect_mmcss_hardware_tier()?;
    apply_mmcss_with_tier(tier)?;
    Ok(tier)
}

pub fn rollback_all_mmcss() -> Result<(), TweakError> {
    set_reg_dword(REG_SYSTEM_PROFILE_KEY, "NetworkThrottlingIndex", 10)?;
    set_reg_dword(REG_SYSTEM_PROFILE_KEY, "SystemResponsiveness", 20)?;
    set_reg_dword(REG_GAMES_PROFILE_KEY, "GPU Priority", 8)?;
    set_reg_dword(REG_GAMES_PROFILE_KEY, "Priority", 2)?;
    set_reg_sz(REG_GAMES_PROFILE_KEY, "Scheduling Category", "Medium")?;
    set_reg_sz(REG_GAMES_PROFILE_KEY, "SFIO Priority", "Normal")?;
    Ok(())
}

pub fn is_mmcss_applied_for_tier(status: &MmcssStatus, tier: MmcssHardwareTier) -> bool {
    let (expected_resp, expected_net, expected_sfio) = match tier {
        MmcssHardwareTier::HighEnd => (10, 4294967295, "High"),
        MmcssHardwareTier::EntryLevel => (20, 10, "Normal"),
    };

    let resp_ok = status.system_responsiveness == Some(expected_resp);
    let net_ok = status.network_throttling_index == Some(expected_net);
    let gpu_ok = status.gpu_priority == Some(8);
    let prio_ok = status.priority == Some(6);
    let sched_ok = status
        .scheduling_category
        .as_deref()
        .map(|s| s.eq_ignore_ascii_case("High"))
        .unwrap_or(false);
    let sfio_ok = status
        .sfio_priority
        .as_deref()
        .map(|s| s.eq_ignore_ascii_case(expected_sfio))
        .unwrap_or(false);

    resp_ok && net_ok && gpu_ok && prio_ok && sched_ok && sfio_ok
}

pub fn is_mmcss_applied_for_current_hardware() -> Result<bool, TweakError> {
    let status = get_mmcss_status()?;
    Ok(is_mmcss_applied_for_tier(&status, status.tier_detected))
}

pub struct MmcssTweak;

impl Tweak for MmcssTweak {
    fn id(&self) -> &'static str {
        "mmcss"
    }

    fn name(&self) -> &'static str {
        "Otimização MMCSS (Prioridade de Rede, CPU, GPU e I/O por Perfil de Hardware)"
    }

    fn description(&self) -> &'static str {
        "Configura o Multimedia Class Scheduler Service (MMCSS) no Registro adaptando parâmetros para PC Forte (6+ núcleos / 16GB+ RAM) ou PC Fraco (Quad-Core / 8GB RAM)."
    }

    fn requires_admin(&self) -> bool {
        true
    }

    fn status(&self) -> Result<TweakStatus, TweakError> {
        let applied = is_mmcss_applied_for_current_hardware()?;
        if applied {
            Ok(TweakStatus::Applied)
        } else {
            Ok(TweakStatus::Default)
        }
    }

    fn apply(&self) -> Result<(), TweakError> {
        apply_all_mmcss()?;
        Ok(())
    }

    fn rollback(&self) -> Result<(), TweakError> {
        rollback_all_mmcss()
    }
}
