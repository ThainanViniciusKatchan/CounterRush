use std::process::Command;
use crate::{Tweak, TweakError, TweakStatus};

pub const REG_GAME_BAR_KEY: &str = r#"HKCU\Software\Microsoft\GameBar"#;
pub const REG_GAMING_CONFIG_KEY: &str =
    r#"HKCU\Software\Microsoft\Windows\CurrentVersion\GamingConfiguration"#;
pub const REG_GAME_CONFIG_STORE_KEY: &str = r#"HKCU\System\GameConfigStore"#;
pub const REG_APP_CAPTURE_KEY: &str =
    r#"HKCU\Software\Microsoft\Windows\CurrentVersion\GameDVR"#;
pub const REG_POLICIES_GAMEDVR_KEY: &str =
    r#"HKLM\SOFTWARE\Policies\Microsoft\Windows\GameDVR"#;

pub const DEFAULT_XBOX_APP_PACKAGE: &str =
    "Microsoft.GamingApp_8wekyb3d8bbwe!Microsoft.Xbox.App";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameBarStatus {
    pub auto_game_mode: Option<u32>,
    pub use_nexus_game_bar: Option<u32>,
    pub gaming_home_app: Option<String>,
    pub gamedvr_enabled: Option<u32>,
    pub app_capture_enabled: Option<u32>,
    pub allow_gamedvr_policy: Option<u32>,
}

impl GameBarStatus {
    pub fn is_game_mode_disabled(&self) -> bool {
        self.auto_game_mode == Some(0)
    }

    pub fn is_game_bar_shortcuts_disabled(&self) -> bool {
        self.use_nexus_game_bar == Some(0)
    }

    pub fn is_xbox_mode_disabled(&self) -> bool {
        match &self.gaming_home_app {
            Some(s) => s.is_empty() || s == "\"\"" || s == "\"\"",
            None => true,
        }
    }

    pub fn is_captures_disabled(&self) -> bool {
        self.gamedvr_enabled == Some(0)
            && self.app_capture_enabled == Some(0)
            && self.allow_gamedvr_policy == Some(0)
    }

    pub fn is_all_disabled(&self) -> bool {
        self.is_game_mode_disabled()
            && self.is_game_bar_shortcuts_disabled()
            && self.is_xbox_mode_disabled()
            && self.is_captures_disabled()
    }
}

fn set_reg_dword(key: &str, value_name: &str, value: u32) -> Result<(), TweakError> {
    let output = Command::new("reg")
        .args([
            "add",
            key,
            "/v",
            value_name,
            "/f",
            "/t",
            "REG_DWORD",
            "/d",
            &value.to_string(),
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
            "/f",
            "/t",
            "REG_SZ",
            "/d",
            value,
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

fn delete_reg_value(key: &str, value_name: &str) -> Result<(), TweakError> {
    let output = Command::new("reg")
        .args(["delete", key, "/v", value_name, "/f"])
        .output()
        .map_err(|e| TweakError::WindowsApi(windows_api::WindowsApiError::Io(e)))?;

    if output.status.success() {
        return Ok(());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{} {}", stdout, stderr).to_lowercase();

    if combined.contains("não foi possível encontrar")
        || combined.contains("nao foi possivel encontrar")
        || combined.contains("unable to find")
        || combined.contains("not found")
    {
        return Ok(());
    }

    Err(TweakError::ValidationFailed(format!(
        "Falha ao remover '{}' de '{}': {}",
        value_name,
        key,
        stderr.trim()
    )))
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
            } else if parts.len() == 1 {
                return Ok(Some(String::new()));
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

pub fn get_game_bar_status() -> Result<GameBarStatus, TweakError> {
    let game_mode_raw = query_reg_value(REG_GAME_BAR_KEY, "AutoGameModeEnabled")?;
    let nexus_raw = query_reg_value(REG_GAME_BAR_KEY, "UseNexusForGameBarEnabled")?;
    let gaming_home_raw = query_reg_value(REG_GAMING_CONFIG_KEY, "GamingHomeApp")?;
    let gamedvr_raw = query_reg_value(REG_GAME_CONFIG_STORE_KEY, "GameDVR_Enabled")?;
    let app_cap_raw = query_reg_value(REG_APP_CAPTURE_KEY, "AppCaptureEnabled")?;
    let policy_raw = query_reg_value(REG_POLICIES_GAMEDVR_KEY, "AllowGameDVR")?;

    Ok(GameBarStatus {
        auto_game_mode: game_mode_raw.as_deref().and_then(parse_dword_str),
        use_nexus_game_bar: nexus_raw.as_deref().and_then(parse_dword_str),
        gaming_home_app: gaming_home_raw,
        gamedvr_enabled: gamedvr_raw.as_deref().and_then(parse_dword_str),
        app_capture_enabled: app_cap_raw.as_deref().and_then(parse_dword_str),
        allow_gamedvr_policy: policy_raw.as_deref().and_then(parse_dword_str),
    })
}

pub fn disable_game_mode() -> Result<(), TweakError> {
    set_reg_dword(REG_GAME_BAR_KEY, "AutoGameModeEnabled", 0)
}

pub fn enable_game_mode() -> Result<(), TweakError> {
    set_reg_dword(REG_GAME_BAR_KEY, "AutoGameModeEnabled", 1)
}

pub fn disable_game_bar_shortcuts() -> Result<(), TweakError> {
    set_reg_dword(REG_GAME_BAR_KEY, "UseNexusForGameBarEnabled", 0)
}

pub fn enable_game_bar_shortcuts() -> Result<(), TweakError> {
    set_reg_dword(REG_GAME_BAR_KEY, "UseNexusForGameBarEnabled", 1)
}

pub fn disable_xbox_mode() -> Result<(), TweakError> {
    set_reg_sz(REG_GAMING_CONFIG_KEY, "GamingHomeApp", "")
}

pub fn enable_xbox_mode() -> Result<(), TweakError> {
    set_reg_sz(REG_GAMING_CONFIG_KEY, "GamingHomeApp", DEFAULT_XBOX_APP_PACKAGE)
}

pub fn disable_gamedvr_captures() -> Result<(), TweakError> {
    set_reg_dword(REG_GAME_CONFIG_STORE_KEY, "GameDVR_Enabled", 0)?;
    set_reg_dword(REG_APP_CAPTURE_KEY, "AppCaptureEnabled", 0)?;
    set_reg_dword(REG_POLICIES_GAMEDVR_KEY, "AllowGameDVR", 0)?;
    Ok(())
}

pub fn enable_gamedvr_captures() -> Result<(), TweakError> {
    set_reg_dword(REG_GAME_CONFIG_STORE_KEY, "GameDVR_Enabled", 1)?;
    set_reg_dword(REG_APP_CAPTURE_KEY, "AppCaptureEnabled", 1)?;
    delete_reg_value(REG_POLICIES_GAMEDVR_KEY, "AllowGameDVR")?;
    Ok(())
}

pub fn disable_all_game_bar_features() -> Result<(), TweakError> {
    disable_game_mode()?;
    disable_game_bar_shortcuts()?;
    disable_xbox_mode()?;
    disable_gamedvr_captures()?;
    Ok(())
}

pub fn rollback_all_game_bar_features() -> Result<(), TweakError> {
    enable_game_mode()?;
    enable_game_bar_shortcuts()?;
    enable_xbox_mode()?;
    enable_gamedvr_captures()?;
    Ok(())
}

pub struct GameBarTweak;

impl Tweak for GameBarTweak {
    fn id(&self) -> &'static str {
        "game_bar"
    }

    fn name(&self) -> &'static str {
        "Desativação do Xbox Game Bar, Modo de Jogo e Capturas (GameDVR)"
    }

    fn description(&self) -> &'static str {
        "Desativa o Modo de Jogo, atalhos do Xbox Game Bar, Modo Xbox e gravação em segundo plano (GameDVR) no Registro."
    }

    fn requires_admin(&self) -> bool {
        true
    }

    fn status(&self) -> Result<TweakStatus, TweakError> {
        let status = get_game_bar_status()?;
        if status.is_all_disabled() {
            Ok(TweakStatus::Applied)
        } else {
            Ok(TweakStatus::Default)
        }
    }

    fn apply(&self) -> Result<(), TweakError> {
        disable_all_game_bar_features()
    }

    fn rollback(&self) -> Result<(), TweakError> {
        rollback_all_game_bar_features()
    }
}
