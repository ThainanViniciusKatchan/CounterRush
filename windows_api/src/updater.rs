use std::path::{Path, PathBuf};

use crate::WindowsApiError;

fn strip_unc_prefix(path: &Path) -> PathBuf {
    let raw = path.to_string_lossy();
    match raw.strip_prefix(r"\\?\") {
        Some(clean) => PathBuf::from(clean),
        None => path.to_path_buf(),
    }
}

fn validate_safe_path_str(path: &Path) -> Result<String, WindowsApiError> {
    let clean = strip_unc_prefix(path);
    let raw = clean.to_string_lossy().into_owned();
    if raw.trim().is_empty() {
        return Err(WindowsApiError::ExecutionFailed(
            "Caminho vazio informado para atualização MSI.".to_string(),
        ));
    }
    if raw.chars().any(|c| matches!(c, '\0' | '\r' | '\n')) {
        return Err(WindowsApiError::ExecutionFailed(
            "Caminho inválido contendo caracteres de controle.".to_string(),
        ));
    }
    Ok(raw)
}

fn escape_ps_single_quoted(value: &str) -> String {
    value.replace('\'', "''")
}

fn msi_install_dir_from_localappdata(local_appdata: Option<&std::ffi::OsStr>) -> Option<PathBuf> {
    local_appdata
        .filter(|v| !v.is_empty())
        .map(|base| strip_unc_prefix(&PathBuf::from(base).join("CounterRush")))
}

fn is_exe_inside_dir(exe: &Path, install_dir: &Path) -> bool {
    let exe_norm = strip_unc_prefix(exe)
        .to_string_lossy()
        .replace('/', "\\")
        .to_lowercase();
    let dir_norm = strip_unc_prefix(install_dir)
        .to_string_lossy()
        .replace('/', "\\")
        .trim_end_matches('\\')
        .to_lowercase();
    if dir_norm.is_empty() {
        return false;
    }
    exe_norm.starts_with(&format!("{dir_norm}\\"))
}

pub fn is_installed_via_msi() -> bool {
    #[cfg(windows)]
    {
        let Ok(current_exe) = std::env::current_exe() else {
            return false;
        };
        let Some(install_dir) =
            msi_install_dir_from_localappdata(std::env::var_os("LOCALAPPDATA").as_deref())
        else {
            return false;
        };
        is_exe_inside_dir(&current_exe, &install_dir)
    }
    #[cfg(not(windows))]
    {
        false
    }
}

pub fn resolve_post_msi_relaunch_exe(current_exe: &Path) -> PathBuf {
    resolve_post_msi_relaunch_exe_with_base(
        current_exe,
        std::env::var_os("LOCALAPPDATA").as_deref(),
    )
}

fn resolve_post_msi_relaunch_exe_with_base(
    current_exe: &Path,
    local_appdata: Option<&std::ffi::OsStr>,
) -> PathBuf {
    if let Some(install_dir) = msi_install_dir_from_localappdata(local_appdata) {
        let msi_exe = install_dir.join("counterrush-ui.exe");
        if msi_exe.exists() || is_exe_inside_dir(current_exe, &install_dir) {
            return msi_exe;
        }
    }
    strip_unc_prefix(current_exe)
}

pub fn build_msi_upgrade_powershell_script(
    msi_path: &Path,
    parent_pid: u32,
    relaunch_exe: &Path,
) -> Result<String, WindowsApiError> {
    let msi_raw = escape_ps_single_quoted(&validate_safe_path_str(msi_path)?);
    let relaunch_raw = escape_ps_single_quoted(&validate_safe_path_str(relaunch_exe)?);

    Ok(format!(
        r#"$ErrorActionPreference = 'SilentlyContinue'
$msiPath = '{msi_raw}'
$fallbackExe = '{relaunch_raw}'
$parentPid = {parent_pid}

try {{
    for ($i = 0; $i -lt 80; $i++) {{
        $running = Get-Process -Id $parentPid -ErrorAction SilentlyContinue
        if (-not $running) {{ break }}
        Start-Sleep -Milliseconds 250
    }}

    $proc = Start-Process -FilePath 'msiexec.exe' -ArgumentList @('/i', $msiPath, '/passive', '/norestart') -Wait -PassThru
    if ($proc -and ($proc.ExitCode -eq 0 -or $proc.ExitCode -eq 3010)) {{
        $installedExe = $null
        if ($env:LOCALAPPDATA) {{
            $candidate = Join-Path $env:LOCALAPPDATA 'CounterRush\counterrush-ui.exe'
            if (Test-Path -LiteralPath $candidate) {{
                $installedExe = $candidate
            }}
        }}
        if (-not $installedExe) {{
            $installedExe = $fallbackExe
        }}
        if ($installedExe -and (Test-Path -LiteralPath $installedExe)) {{
            $workDir = Split-Path -Parent $installedExe
            Start-Process -FilePath $installedExe -WorkingDirectory $workDir | Out-Null
        }}
    }}
}} finally {{
    if ($msiPath -and (Test-Path -LiteralPath $msiPath)) {{
        Remove-Item -LiteralPath $msiPath -Force -ErrorAction SilentlyContinue
    }}
    if ($PSCommandPath -and (Test-Path -LiteralPath $PSCommandPath)) {{
        Remove-Item -LiteralPath $PSCommandPath -Force -ErrorAction SilentlyContinue
    }}
}}
"#
    ))
}

#[cfg(windows)]
pub fn spawn_msi_upgrade_and_relaunch(
    msi_path: &Path,
    parent_pid: u32,
    relaunch_exe: &Path,
) -> Result<(), WindowsApiError> {
    use std::os::windows::process::CommandExt;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    const DETACHED_PROCESS: u32 = 0x0000_0008;

    if !msi_path.exists() {
        return Err(WindowsApiError::ExecutionFailed(format!(
            "Instalador MSI não encontrado em '{}'.",
            msi_path.display()
        )));
    }

    let script_content = build_msi_upgrade_powershell_script(msi_path, parent_pid, relaunch_exe)?;
    let script_dir = msi_path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(std::env::temp_dir);
    let script_path = script_dir.join(format!("apply_update_{parent_pid}.ps1"));
    std::fs::write(&script_path, script_content)?;

    let script_clean = strip_unc_prefix(&script_path);
    std::process::Command::new("powershell.exe")
        .args([
            "-NoLogo",
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-WindowStyle",
            "Hidden",
            "-File",
        ])
        .arg(&script_clean)
        .creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS)
        .spawn()
        .map_err(|e| {
            let _ = std::fs::remove_file(&script_path);
            WindowsApiError::ExecutionFailed(format!(
                "Falha ao iniciar processo auxiliar de atualização MSI: {e}"
            ))
        })?;

    Ok(())
}

#[cfg(not(windows))]
pub fn spawn_msi_upgrade_and_relaunch(
    _msi_path: &Path,
    _parent_pid: u32,
    _relaunch_exe: &Path,
) -> Result<(), WindowsApiError> {
    Err(WindowsApiError::ExecutionFailed(
        "Atualização via MSI é suportada apenas no Windows.".to_string(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn test_build_msi_script_escapes_quotes_and_preserves_paths() {
        let msi = Path::new(r"\\?\C:\Users\O'Connor\AppData\Local\Temp\counterrush-ui-0.24.0-x86_64.msi");
        let exe = Path::new(r"C:\Users\O'Connor\AppData\Local\CounterRush\counterrush-ui.exe");
        let script = build_msi_upgrade_powershell_script(msi, 4321, exe)
            .expect("Script deve ser gerado com escape seguro");

        assert!(script.contains(
            r"$msiPath = 'C:\Users\O''Connor\AppData\Local\Temp\counterrush-ui-0.24.0-x86_64.msi'"
        ));
        assert!(script.contains(
            r"$fallbackExe = 'C:\Users\O''Connor\AppData\Local\CounterRush\counterrush-ui.exe'"
        ));
        assert!(script.contains("$parentPid = 4321"));
        assert!(script.contains("'/passive', '/norestart'"));
    }

    #[test]
    fn test_build_msi_script_rejects_control_characters() {
        let bad_msi = Path::new("C:\\temp\\bad\npath.msi");
        let exe = Path::new("C:\\CounterRush\\counterrush-ui.exe");
        let err = build_msi_upgrade_powershell_script(bad_msi, 100, exe).unwrap_err();
        assert!(err.to_string().contains("caracteres de controle"));

        let empty_msi = Path::new("");
        assert!(build_msi_upgrade_powershell_script(empty_msi, 100, exe).is_err());
    }

    #[test]
    fn test_resolve_post_msi_relaunch_exe_prefers_installed_location() {
        let tmp = tempfile::tempdir().unwrap();
        let install_dir = tmp.path().join("CounterRush");
        std::fs::create_dir_all(&install_dir).unwrap();
        let installed_exe = install_dir.join("counterrush-ui.exe");
        std::fs::write(&installed_exe, b"bin").unwrap();

        let dev_exe = Path::new(r"D:\CounterRush\target\release\counterrush-ui.exe");
        let resolved = resolve_post_msi_relaunch_exe_with_base(
            dev_exe,
            Some(OsStr::new(tmp.path().to_str().unwrap())),
        );
        assert_eq!(resolved, installed_exe);
    }

    #[test]
    fn test_resolve_post_msi_relaunch_exe_falls_back_to_current_when_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let dev_exe = Path::new(r"\\?\D:\CounterRush\target\release\counterrush-ui.exe");
        let resolved = resolve_post_msi_relaunch_exe_with_base(
            dev_exe,
            Some(OsStr::new(tmp.path().to_str().unwrap())),
        );
        assert_eq!(
            resolved,
            PathBuf::from(r"D:\CounterRush\target\release\counterrush-ui.exe")
        );
    }
}
