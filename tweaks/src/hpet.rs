use std::process::Command;
use crate::{Tweak, TweakError, TweakStatus};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HpetStatus {
    Disabled,
    Enabled,
    Default,
}

impl std::fmt::Display for HpetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Disabled => write!(f, "Desativado (useplatformclock No - Relógio TSC da CPU ativo)"),
            Self::Enabled => write!(f, "Ativado (useplatformclock Yes - HPET forçado)"),
            Self::Default => write!(f, "Padrão do Windows (useplatformclock não definido no BCD)"),
        }
    }
}

pub fn get_hpet_status() -> Result<HpetStatus, TweakError> {
    let output = Command::new("bcdedit")
        .args(["/enum", "{current}"])
        .output()
        .map_err(|e| TweakError::WindowsApi(windows_api::WindowsApiError::Io(e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{} {}", stdout, stderr).to_lowercase();

    if combined.contains("acesso negado") || combined.contains("access is denied") {
        return Err(TweakError::WindowsApi(windows_api::WindowsApiError::AccessDenied));
    }

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("useplatformclock") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                let val = parts[1].to_lowercase();
                if val == "no" || val == "false" || val == "não" || val == "nao" {
                    return Ok(HpetStatus::Disabled);
                } else if val == "yes" || val == "true" || val == "sim" {
                    return Ok(HpetStatus::Enabled);
                }
            }
        }
    }

    Ok(HpetStatus::Default)
}

pub fn disable_hpet() -> Result<(), TweakError> {
    let output = Command::new("bcdedit")
        .args(["/set", "useplatformclock", "false"])
        .output()
        .map_err(|e| TweakError::WindowsApi(windows_api::WindowsApiError::Io(e)))?;

    if output.status.success() {
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{} {}", stdout, stderr).to_lowercase();

    if combined.contains("acesso negado") || combined.contains("access is denied") {
        return Err(TweakError::WindowsApi(windows_api::WindowsApiError::AccessDenied));
    }

    Err(TweakError::ValidationFailed(format!(
        "Falha ao executar 'bcdedit /set useplatformclock false': {}",
        stderr.trim()
    )))
}

pub fn rollback_hpet() -> Result<(), TweakError> {
    let output = Command::new("bcdedit")
        .args(["/deletevalue", "useplatformclock"])
        .output()
        .map_err(|e| TweakError::WindowsApi(windows_api::WindowsApiError::Io(e)))?;

    if output.status.success() {
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{} {}", stdout, stderr).to_lowercase();

    if combined.contains("acesso negado") || combined.contains("access is denied") {
        return Err(TweakError::WindowsApi(windows_api::WindowsApiError::AccessDenied));
    }

    if combined.contains("não foi encontrado")
        || combined.contains("nao foi encontrado")
        || combined.contains("element not found")
        || combined.contains("not found")
    {
        return Ok(());
    }

    Err(TweakError::ValidationFailed(format!(
        "Falha ao executar 'bcdedit /deletevalue useplatformclock': {}",
        stderr.trim()
    )))
}

pub struct HpetTweak;

impl Tweak for HpetTweak {
    fn id(&self) -> &'static str {
        "hpet"
    }

    fn name(&self) -> &'static str {
        "Desativação do Temporizador de Alta Precisão (HPET)"
    }

    fn description(&self) -> &'static str {
        "Desativa useplatformclock no BCD para que o Windows utilize os contadores de tempo rápidos da CPU (TSC), reduzindo input lag e stutters no CS2."
    }

    fn requires_admin(&self) -> bool {
        true
    }

    fn status(&self) -> Result<TweakStatus, TweakError> {
        let status = get_hpet_status()?;
        match status {
            HpetStatus::Disabled => Ok(TweakStatus::Applied),
            _ => Ok(TweakStatus::Default),
        }
    }

    fn apply(&self) -> Result<(), TweakError> {
        disable_hpet()
    }

    fn rollback(&self) -> Result<(), TweakError> {
        rollback_hpet()
    }
}
