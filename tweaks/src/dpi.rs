use crate::silent_command;
use crate::{Tweak, TweakError, TweakStatus};

pub const CS2_REG_KEY: &str = r#"HKCU\Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers"#;
pub const CS2_EXE_PATH: &str = r#"C:\Program Files (x86)\Steam\steamapps\common\Counter-Strike Global Offensive\game\bin\win64\cs2.exe"#;
pub const CS2_DPI_FLAGS: &str = "~ DISABLEDXMAXIMIZEDWINDOWEDMODE PERPROCESSSYSTEMDPIFORCEOFF HIGHDPIAWARE";

pub const CS2_DPI_APPLY_CMD: &str = r#"reg add "HKCU\Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers" /v "C:\Program Files (x86)\Steam\steamapps\common\Counter-Strike Global Offensive\game\bin\win64\cs2.exe" /t REG_SZ /d "~ DISABLEDXMAXIMIZEDWINDOWEDMODE PERPROCESSSYSTEMDPIFORCEOFF HIGHDPIAWARE" /f"#;
pub const CS2_DPI_ROLLBACK_CMD: &str = r#"reg delete "HKCU\Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers" /v "C:\Program Files (x86)\Steam\steamapps\common\Counter-Strike Global Offensive\game\bin\win64\cs2.exe" /f"#;
pub const CS2_DPI_QUERY_CMD: &str = r#"reg query "HKCU\Software\Microsoft\Windows NT\CurrentVersion\AppCompatFlags\Layers" /v "C:\Program Files (x86)\Steam\steamapps\common\Counter-Strike Global Offensive\game\bin\win64\cs2.exe""#;

pub fn apply_cs2_dpi_tweak() -> Result<(), TweakError> {
    let output = silent_command("reg")
        .args([
            "add",
            CS2_REG_KEY,
            "/v",
            CS2_EXE_PATH,
            "/t",
            "REG_SZ",
            "/d",
            CS2_DPI_FLAGS,
            "/f",
        ])
        .output()
        .map_err(|e| TweakError::WindowsApi(windows_api::WindowsApiError::Io(e)))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(TweakError::ValidationFailed(format!(
            "Falha ao executar reg add para otimização de DPI do CS2: {}",
            stderr.trim()
        )))
    }
}

pub fn rollback_cs2_dpi_tweak() -> Result<(), TweakError> {
    let output = silent_command("reg")
        .args([
            "delete",
            CS2_REG_KEY,
            "/v",
            CS2_EXE_PATH,
            "/f",
        ])
        .output()
        .map_err(|e| TweakError::WindowsApi(windows_api::WindowsApiError::Io(e)))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let combined = format!("{} {}", stdout, stderr).to_lowercase();

    if combined.contains("não foi possível encontrar")
        || combined.contains("unable to find")
        || combined.contains("not find")
    {
        Ok(())
    } else {
        Err(TweakError::ValidationFailed(format!(
            "Falha ao executar reg delete de rollback da otimização de DPI do CS2: {}",
            stderr.trim()
        )))
    }
}

pub fn launch_cs2_with_dpi_tweak() -> Result<(), TweakError> {
    apply_cs2_dpi_tweak()?;
    windows_api::launch_cs2()?;
    Ok(())
}

pub struct Cs2DpiOptimizationTweak;

impl Tweak for Cs2DpiOptimizationTweak {
    fn id(&self) -> &'static str {
        "cs2_dpi_optimization"
    }

    fn name(&self) -> &'static str {
        "Otimização de DPI e Modo Janela do CS2"
    }

    fn description(&self) -> &'static str {
        "Aplica flags de compatibilidade (DISABLEDXMAXIMIZEDWINDOWEDMODE, PERPROCESSSYSTEMDPIFORCEOFF, HIGHDPIAWARE) para mitigar input lag e problemas de escala no CS2."
    }

    fn requires_admin(&self) -> bool {
        false
    }

    fn status(&self) -> Result<TweakStatus, TweakError> {
        let output = silent_command("reg")
            .args([
                "query",
                CS2_REG_KEY,
                "/v",
                CS2_EXE_PATH,
            ])
            .output()
            .map_err(|e| TweakError::WindowsApi(windows_api::WindowsApiError::Io(e)))?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("DISABLEDXMAXIMIZEDWINDOWEDMODE") {
                return Ok(TweakStatus::Applied);
            }
        }
        Ok(TweakStatus::Default)
    }

    fn apply(&self) -> Result<(), TweakError> {
        apply_cs2_dpi_tweak()
    }

    fn rollback(&self) -> Result<(), TweakError> {
        rollback_cs2_dpi_tweak()
    }
}
