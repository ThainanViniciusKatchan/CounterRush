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
    if raw.chars().any(|c| matches!(c, '\0' | '\r' | '\n' | '"')) {
        return Err(WindowsApiError::ExecutionFailed(
            "Caminho inválido contendo caracteres de controle ou aspas.".to_string(),
        ));
    }
    Ok(raw)
}

fn write_powershell_script_utf8_bom(path: &Path, content: &str) -> std::io::Result<()> {
    let mut bytes = Vec::with_capacity(3 + content.len());
    bytes.extend_from_slice(&[0xEF, 0xBB, 0xBF]);
    bytes.extend_from_slice(content.as_bytes());
    std::fs::write(path, bytes)
}

#[cfg(windows)]
fn resolve_powershell_executable() -> PathBuf {
    if let Some(system_root) = std::env::var_os("SystemRoot").filter(|v| !v.is_empty()) {
        let candidate = strip_unc_prefix(
            &PathBuf::from(system_root)
                .join("System32")
                .join("WindowsPowerShell")
                .join("v1.0")
                .join("powershell.exe"),
        );
        if candidate.is_file() {
            return candidate;
        }
    }
    PathBuf::from("powershell.exe")
}

#[cfg(windows)]
fn spawn_hidden_powershell_file(script_path: &Path) -> std::io::Result<std::process::Child> {
    use std::os::windows::process::CommandExt;
    use std::process::Stdio;

    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;

    let script_clean = strip_unc_prefix(script_path);
    std::process::Command::new(resolve_powershell_executable())
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
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .creation_flags(CREATE_NO_WINDOW | CREATE_NEW_PROCESS_GROUP)
        .spawn()
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

    let log_dir = strip_unc_prefix(msi_path)
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(Path::to_path_buf)
        .unwrap_or_else(|| strip_unc_prefix(&std::env::temp_dir()));
    let update_log_raw = escape_ps_single_quoted(&validate_safe_path_str(&log_dir.join("update.log"))?);
    let msi_log_raw =
        escape_ps_single_quoted(&validate_safe_path_str(&log_dir.join("msi_install.log"))?);

    Ok(format!(
        r#"$ErrorActionPreference = 'Continue'
$msiPath = '{msi_raw}'
$fallbackExe = '{relaunch_raw}'
$updateLog = '{update_log_raw}'
$msiLog = '{msi_log_raw}'
$parentPid = {parent_pid}
$installSucceeded = $false

function Write-UpdateLog {{
    param([string]$Message)
    try {{
        $ts = Get-Date -Format 'yyyy-MM-dd HH:mm:ss.fff'
        Add-Content -LiteralPath $updateLog -Value "[$ts] $Message" -Encoding UTF8 -ErrorAction SilentlyContinue
    }} catch {{
    }}
}}

try {{
    Write-UpdateLog "Iniciando atualizacao MSI (parentPid=$parentPid, msi='$msiPath')."
    for ($i = 0; $i -lt 80; $i++) {{
        $running = Get-Process -Id $parentPid -ErrorAction SilentlyContinue
        if (-not $running) {{ break }}
        Start-Sleep -Milliseconds 250
    }}
    Start-Sleep -Milliseconds 500

    if (-not (Test-Path -LiteralPath $msiPath -PathType Leaf)) {{
        Write-UpdateLog "ERRO: Instalador MSI nao encontrado em '$msiPath'."
        return
    }}

    $msiexecPath = 'msiexec.exe'
    if ($env:SystemRoot) {{
        $sysMsiexec = Join-Path $env:SystemRoot 'System32\msiexec.exe'
        if (Test-Path -LiteralPath $sysMsiexec -PathType Leaf) {{
            $msiexecPath = $sysMsiexec
        }}
    }}

    $msiArgs = "/i `"$msiPath`" /qb /norestart /L*v `"$msiLog`""
    $exitCode = -1
    for ($attempt = 1; $attempt -le 3; $attempt++) {{
        Write-UpdateLog "Executando msiexec (tentativa $attempt/3): $msiexecPath $msiArgs"
        $proc = Start-Process -FilePath $msiexecPath -ArgumentList $msiArgs -WindowStyle Normal -PassThru -ErrorAction Stop
        $null = $proc.Handle
        $proc.WaitForExit()
        $exitCode = [int]$proc.ExitCode
        Write-UpdateLog "msiexec finalizado com codigo $exitCode."
        if ($exitCode -eq 1618 -and $attempt -lt 3) {{
            Write-UpdateLog 'Outra instalacao do Windows Installer em andamento (1618). Aguardando 3s...'
            Start-Sleep -Seconds 3
            continue
        }}
        break
    }}

    if ($exitCode -eq 0 -or $exitCode -eq 3010) {{
        $installSucceeded = $true
        $installedExe = $null
        if ($env:LOCALAPPDATA) {{
            $candidate = Join-Path $env:LOCALAPPDATA 'CounterRush\counterrush-ui.exe'
            if (Test-Path -LiteralPath $candidate -PathType Leaf) {{
                $installedExe = $candidate
            }}
        }}
        if (-not $installedExe) {{
            $installedExe = $fallbackExe
        }}
        if ($installedExe -and (Test-Path -LiteralPath $installedExe -PathType Leaf)) {{
            $workDir = Split-Path -Parent $installedExe
            Write-UpdateLog "Relancando CounterRush: '$installedExe'."
            Start-Process -FilePath $installedExe -WorkingDirectory $workDir -WindowStyle Normal -ErrorAction Stop | Out-Null
        }} else {{
            Write-UpdateLog "AVISO: Executavel para relancamento nao encontrado ('$installedExe')."
        }}
    }} else {{
        Write-UpdateLog "ERRO: Instalacao MSI falhou com codigo $exitCode. Verifique '$msiLog'."
    }}
}} catch {{
    Write-UpdateLog "EXCECAO: $($_.Exception.Message)"
}} finally {{
    if ($installSucceeded -and $msiPath -and (Test-Path -LiteralPath $msiPath)) {{
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
    std::fs::create_dir_all(&script_dir)?;
    let script_path = script_dir.join(format!("apply_update_{parent_pid}.ps1"));
    write_powershell_script_utf8_bom(&script_path, &script_content)?;

    spawn_hidden_powershell_file(&script_path).map_err(|e| {
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
        assert!(script.contains(
            r"$updateLog = 'C:\Users\O''Connor\AppData\Local\Temp\update.log'"
        ));
        assert!(script.contains(
            r"$msiLog = 'C:\Users\O''Connor\AppData\Local\Temp\msi_install.log'"
        ));
        assert!(script.contains("$parentPid = 4321"));
        assert!(script.contains(r#"$msiArgs = "/i `"$msiPath`" /qb /norestart /L*v `"$msiLog`"""#));
        assert!(script.contains("$null = $proc.Handle"));
    }

    #[test]
    fn test_build_msi_script_rejects_control_characters() {
        let bad_msi = Path::new("C:\\temp\\bad\npath.msi");
        let exe = Path::new("C:\\CounterRush\\counterrush-ui.exe");
        let err = build_msi_upgrade_powershell_script(bad_msi, 100, exe).unwrap_err();
        assert!(err.to_string().contains("caracteres de controle"));

        let quoted_msi = Path::new("C:\\temp\\bad\"path.msi");
        let err_quote = build_msi_upgrade_powershell_script(quoted_msi, 100, exe).unwrap_err();
        assert!(err_quote.to_string().contains("caracteres de controle"));

        let empty_msi = Path::new("");
        assert!(build_msi_upgrade_powershell_script(empty_msi, 100, exe).is_err());
    }

    #[test]
    fn test_write_powershell_script_utf8_bom_writes_bom_and_unicode() {
        let tmp = tempfile::tempdir().unwrap();
        let script_path = tmp.path().join("bom_check.ps1");
        let content = "$p = 'C:\\Users\\João\\Обновление\\atualização.msi'\n";
        write_powershell_script_utf8_bom(&script_path, content).unwrap();

        let bytes = std::fs::read(&script_path).unwrap();
        assert!(bytes.starts_with(&[0xEF, 0xBB, 0xBF]));
        assert_eq!(&bytes[3..], content.as_bytes());
    }

    #[cfg(windows)]
    #[test]
    fn test_spawn_hidden_powershell_executes_script_with_spaces_and_unicode() {
        let tmp = tempfile::tempdir().unwrap();
        let nested = tmp.path().join("João Update Test");
        std::fs::create_dir_all(&nested).unwrap();

        let script_path = nested.join("helper_test.ps1");
        let marker_path = nested.join("marker.txt");
        let escaped_marker = escape_ps_single_quoted(&validate_safe_path_str(&marker_path).unwrap());
        let script_body = format!(
            "$ErrorActionPreference = 'Stop'\n[System.IO.File]::WriteAllText('{escaped_marker}', $PSCommandPath, [System.Text.UTF8Encoding]::new($false))\n"
        );
        write_powershell_script_utf8_bom(&script_path, &script_body).unwrap();

        let mut child = spawn_hidden_powershell_file(&script_path)
            .expect("Processo PowerShell oculto deve iniciar sem falhar");
        let status = child.wait().expect("Deve aguardar término do PowerShell");
        assert!(
            status.success(),
            "PowerShell oculto retornou código de saída inesperado: {:?}",
            status.code()
        );
        assert!(
            marker_path.exists(),
            "O script PowerShell não foi executado (marker.txt ausente)"
        );
        let written = std::fs::read_to_string(&marker_path).unwrap();
        assert_eq!(written.trim(), strip_unc_prefix(&script_path).to_string_lossy());
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
