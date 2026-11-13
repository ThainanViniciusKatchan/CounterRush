use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::{Path, PathBuf};
use std::time::Duration;
use crate::WindowsApiError;

#[link(name = "shell32")]
extern "system" {
    fn ShellExecuteW(
        hwnd: *mut std::ffi::c_void,
        lpOperation: *const u16,
        lpFile: *const u16,
        lpParameters: *const u16,
        lpDirectory: *const u16,
        nShowCmd: i32,
    ) -> isize;
}

const SW_SHOWNORMAL: i32 = 1;

fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

pub fn shell_execute(file: &str, params: Option<&str>) -> Result<(), WindowsApiError> {
    let operation = to_wide("open");
    let file_wide = to_wide(file);
    let params_wide = params.map(to_wide);

    let params_ptr = match &params_wide {
        Some(w) => w.as_ptr(),
        None => std::ptr::null(),
    };

    let res = unsafe {
        ShellExecuteW(
            std::ptr::null_mut(),
            operation.as_ptr(),
            file_wide.as_ptr(),
            params_ptr,
            std::ptr::null(),
            SW_SHOWNORMAL,
        )
    };

    if res > 32 {
        Ok(())
    } else {
        Err(WindowsApiError::ExecutionFailed(format!(
            "ShellExecuteW falhou com código de retorno: {res}"
        )))
    }
}

fn resolve_steam_paths() -> (String, String) {
    let default_dir = r"C:\Program Files (x86)\Steam".to_string();
    let default_exe = r"C:\Program Files (x86)\Steam\steam.exe".to_string();

    let script = r#"
$reg = Get-ItemProperty -Path 'HKCU:\Software\Valve\Steam' -ErrorAction SilentlyContinue
if ($reg -and $reg.SteamExe -and $reg.SteamPath) {
    $exe = $reg.SteamExe -replace '/', '\'
    $dir = $reg.SteamPath -replace '/', '\'
    Write-Output "$exe|$dir"
}
"#;
    if let Ok(out) = crate::run_powershell_script(script) {
        let trimmed = out.trim();
        if let Some((exe, dir)) = trimmed.split_once('|') {
            if Path::new(exe).exists() {
                return (exe.to_string(), dir.to_string());
            }
        }
    }

    (default_exe, default_dir)
}

pub fn launch_on_interactive_desktop(
    exe_path: &str,
    args: &str,
    working_dir: &str,
) -> Result<(), WindowsApiError> {
    let escaped_exe = exe_path.replace('\'', "''");
    let escaped_args = args.replace('\'', "''");
    let escaped_dir = working_dir.replace('\'', "''");

    let script = format!(
        r#"
$ErrorActionPreference = 'Stop'
$service = New-Object -ComObject 'Schedule.Service'
$service.Connect()
$folder = $service.GetFolder('\')
$taskDef = $service.NewTask(0)
$taskDef.Settings.Enabled = $true
$taskDef.Settings.AllowDemandStart = $true
$taskDef.Settings.StartWhenAvailable = $false
$taskDef.Settings.DisallowStartIfOnBatteries = $false
$taskDef.Settings.StopIfGoingOnBatteries = $false
$taskDef.Principal.LogonType = 3
$taskDef.Principal.RunLevel = 0
$action = $taskDef.Actions.Create(0)
$action.Path = '{escaped_exe}'
$action.Arguments = '{escaped_args}'
$action.WorkingDirectory = '{escaped_dir}'
$taskName = 'CounterRush_DesktopLaunch_' + [guid]::NewGuid().ToString('N')
$regTask = $folder.RegisterTaskDefinition($taskName, $taskDef, 6, $null, $null, 3)
$null = $regTask.Run($null)
Start-Sleep -Milliseconds 350
$folder.DeleteTask($taskName, 0)
"#
    );

    match crate::run_powershell_script(&script) {
        Ok(_) => Ok(()),
        Err(_) => {
            let params = if args.is_empty() { None } else { Some(args) };
            shell_execute(exe_path, params)
        }
    }
}

fn is_steam_healthy_on_desktop(steam_dir: &str) -> bool {
    let log_path = format!(r"{}\logs\console_log.txt", steam_dir).replace('\'', "''");
    let script = format!(
        r#"
$code = @"
using System;
using System.Runtime.InteropServices;
using System.Diagnostics;
public class IntegrityCheck {{
    [DllImport("advapi32.dll", SetLastError=true)]
    public static extern bool OpenProcessToken(IntPtr ProcessHandle, uint DesiredAccess, out IntPtr TokenHandle);
    [DllImport("advapi32.dll", SetLastError=true)]
    public static extern bool GetTokenInformation(IntPtr TokenHandle, int TokenInformationClass, IntPtr TokenInformation, int TokenInformationLength, out int ReturnLength);
    [DllImport("kernel32.dll")]
    public static extern bool CloseHandle(IntPtr hObject);
    public static int GetIntegrity(int pid) {{
        try {{
            Process p = Process.GetProcessById(pid);
            IntPtr hToken;
            if (!OpenProcessToken(p.Handle, 0x0008, out hToken)) return -1;
            int len = 0;
            GetTokenInformation(hToken, 25, IntPtr.Zero, 0, out len);
            IntPtr buf = Marshal.AllocHGlobal(len);
            GetTokenInformation(hToken, 25, buf, len, out len);
            IntPtr pSid = Marshal.ReadIntPtr(buf);
            IntPtr pCount = new IntPtr(pSid.ToInt64() + 1);
            byte count = Marshal.ReadByte(pCount);
            IntPtr pRid = new IntPtr(pSid.ToInt64() + 8 + (count - 1) * 4);
            int rid = Marshal.ReadInt32(pRid);
            Marshal.FreeHGlobal(buf);
            CloseHandle(hToken);
            return rid;
        }} catch {{ return -1; }}
    }}
}}
"@
Add-Type -TypeDefinition $code -ErrorAction SilentlyContinue
$p = Get-Process -Name 'steam' -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $p) {{ Write-Output 'NOT_RUNNING'; exit 0 }}
$rid = [IntegrityCheck]::GetIntegrity($p.Id)
if ($rid -ge 12288) {{ Write-Output 'ELEVATED'; exit 0 }}
if (Test-Path '{log_path}') {{
    $tail = Get-Content '{log_path}' -Tail 80 -ErrorAction SilentlyContinue
    $joined = $tail -join "`n"
    $lastStart = $joined.LastIndexOf('startup - updater built')
    $segment = if ($lastStart -ge 0) {{ $joined.Substring($lastStart) }} else {{ $joined }}
    if ($segment -match 'SetNotifyIcon\(-2147467259\) failed') {{
        Write-Output 'HIDDEN_DESKTOP'
        exit 0
    }}
}}
Write-Output 'HEALTHY'
"#
    );

    if let Ok(out) = crate::run_powershell_script(&script) {
        let status = out.trim();
        status == "HEALTHY"
    } else {
        true
    }
}

pub fn ensure_steam_running() -> Result<(), WindowsApiError> {
    let (steam_exe, steam_dir) = resolve_steam_paths();

    if crate::is_process_running("steam")? {
        let cs2_running = crate::is_process_running("cs2").unwrap_or(false);
        if !cs2_running && !is_steam_healthy_on_desktop(&steam_dir) {
            println!("   [STEAM] Instância da Steam em background invisível/elevado detectada. Reiniciando no desktop interativo...");
            let _ = crate::run_powershell_script(
                "Stop-Process -Name 'steam', 'steamwebhelper' -Force -ErrorAction SilentlyContinue; Start-Sleep -Seconds 2",
            );
        }
    }

    if !crate::is_process_running("steam")? {
        println!("   [STEAM] Steam fechada detectada. Disparando inicialização do cliente no desktop...");
        launch_on_interactive_desktop(&steam_exe, "steam://open/main", &steam_dir)?;

        crate::wait_for_process("steam", Duration::from_secs(30))?;
        println!("   [STEAM] Processo da Steam instanciado. Aguardando 10s para conclusão do login e serviços...");
        std::thread::sleep(Duration::from_secs(10));
        println!("   [STEAM] Steam pronta para receber ordens de lançamento!");
    }
    Ok(())
}

pub fn launch_cs2() -> Result<(), WindowsApiError> {
    ensure_steam_running()?;
    let (steam_exe, steam_dir) = resolve_steam_paths();
    launch_on_interactive_desktop(&steam_exe, "-applaunch 730", &steam_dir)
}

pub fn launch_cs2_with_args(extra_args: &[&str]) -> Result<(), WindowsApiError> {
    launch_cs2_with_args_slice(extra_args)
}

pub fn launch_cs2_with_args_slice<T: AsRef<str>>(extra_args: &[T]) -> Result<(), WindowsApiError> {
    ensure_steam_running()?;
    let (steam_exe, steam_dir) = resolve_steam_paths();

    let joined = extra_args
        .iter()
        .map(|a| a.as_ref())
        .collect::<Vec<_>>()
        .join(" ");

    let params = if joined.is_empty() {
        "-applaunch 730".to_string()
    } else {
        format!("-applaunch 730 {}", joined)
    };

    launch_on_interactive_desktop(&steam_exe, &params, &steam_dir)
}

pub fn launch_cs2_economico() -> Result<(), WindowsApiError> {
    launch_cs2_with_args(&["-console", "-nojoy", "+fps_max", "120", "-vulkan"])
}

pub fn launch_cs2_intermediario() -> Result<(), WindowsApiError> {
    launch_cs2_with_args(&[
        "-fullscreen",
        "-console",
        "-nojoy",
        "+engine_low_latency_sleep_after_client_tick",
        "true",
        "+fps_max",
        "240",
        "+mat_queue_mode",
    ])
}

pub fn launch_cs2_potente() -> Result<(), WindowsApiError> {
    launch_cs2_with_args(&[
        "-console",
        "-nojoy",
        "-refresh",
        "240",
        "+fps_max",
        "400",
        "+engine_low_latency_sleep_after_client_tick",
        "true",
    ])
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadingDirCheckResult {
    None,
    Empty730Folder(PathBuf),
    Active730Content(PathBuf),
}

fn dir_contains_any_file(dir: &Path, depth: u8) -> bool {
    if depth > 32 {
        return true;
    }
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    for entry in entries.flatten() {
        let Ok(ft) = entry.file_type() else {
            continue;
        };
        if ft.is_file() {
            return true;
        }
        if ft.is_dir() && dir_contains_any_file(&entry.path(), depth + 1) {
            return true;
        }
    }
    false
}

pub fn inspect_downloading_dir_730<P: AsRef<Path>>(dir: P) -> DownloadingDirCheckResult {
    let path = dir.as_ref();
    if !path.exists() || !path.is_dir() {
        return DownloadingDirCheckResult::None;
    }

    let mut found_empty_folder: Option<PathBuf> = None;
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let name_lossy = file_name.to_string_lossy();
            if name_lossy == "730" || name_lossy.contains("730") {
                let entry_path = entry.path();
                if entry_path.is_file()
                    || (entry_path.is_dir() && dir_contains_any_file(&entry_path, 0))
                {
                    return DownloadingDirCheckResult::Active730Content(entry_path);
                }
                if entry_path.is_dir() && found_empty_folder.is_none() {
                    found_empty_folder = Some(entry_path);
                }
            }
        }
    }

    match found_empty_folder {
        Some(empty_path) => DownloadingDirCheckResult::Empty730Folder(empty_path),
        None => DownloadingDirCheckResult::None,
    }
}

pub fn check_downloading_dir_has_730<P: AsRef<Path>>(dir: P) -> bool {
    !matches!(inspect_downloading_dir_730(dir), DownloadingDirCheckResult::None)
}

pub fn get_steam_downloading_dirs_from_base<P: AsRef<Path>>(base_steam_dir: P) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let base = base_steam_dir.as_ref();
    let primary_downloading = base.join("steamapps").join("downloading");
    dirs.push(primary_downloading);

    let vdf_path = base.join("steamapps").join("libraryfolders.vdf");
    if vdf_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&vdf_path) {
            for line in content.lines() {
                let trimmed = line.trim();
                if trimmed.starts_with("\"path\"") {
                    let parts: Vec<&str> = trimmed.split('"').filter(|s| !s.trim().is_empty()).collect();
                    if parts.len() >= 2 {
                        let raw_path = parts[1].replace("\\\\", "\\");
                        let lib_downloading = PathBuf::from(raw_path).join("steamapps").join("downloading");
                        if !dirs.iter().any(|d| d == &lib_downloading) {
                            dirs.push(lib_downloading);
                        }
                    }
                }
            }
        }
    }

    dirs
}

pub fn get_steam_downloading_dirs() -> Vec<PathBuf> {
    let (_steam_exe, steam_dir) = resolve_steam_paths();
    let mut dirs = get_steam_downloading_dirs_from_base(&steam_dir);

    let steamcmd_dir = Path::new(crate::steamcmd::DEFAULT_STEAMCMD_DIR);
    if steamcmd_dir.exists() {
        for cmd_dl_dir in get_steam_downloading_dirs_from_base(steamcmd_dir) {
            if !dirs.iter().any(|d| d == &cmd_dl_dir) {
                dirs.push(cmd_dl_dir);
            }
        }
    }

    dirs
}

pub fn get_cs2_download_validation_status_for_paths<P1: AsRef<Path>, P2: AsRef<Path>>(
    steam_base_dir: P1,
    steamcmd_base_dir: P2,
    require_live_steam_process: bool,
) -> crate::steamcmd::SteamCmdLogStatus {
    let mut dirs = get_steam_downloading_dirs_from_base(steam_base_dir.as_ref());
    if steamcmd_base_dir.as_ref().exists() {
        for cmd_dir in get_steam_downloading_dirs_from_base(steamcmd_base_dir.as_ref()) {
            if !dirs.iter().any(|d| d == &cmd_dir) {
                dirs.push(cmd_dir);
            }
        }
    }

    for dir in dirs {
        if let DownloadingDirCheckResult::Active730Content(matched_path) =
            inspect_downloading_dir_730(&dir)
        {
            return crate::steamcmd::SteamCmdLogStatus {
                state: crate::steamcmd::DownloadActivityState::Downloading,
                progress_percent: None,
                source: "steamapps/downloading/".to_string(),
                detail: format!(
                    "Arquivos ativos de download 730 detectados em {}",
                    matched_path.display()
                ),
            };
        }
    }

    if steamcmd_base_dir.as_ref().exists() {
        let steamcmd_client = crate::steamcmd::SteamCmdClient::new(steamcmd_base_dir.as_ref());
        let cmd_status = steamcmd_client.check_validation_status(crate::steamcmd::CS2_APP_ID);
        if cmd_status.is_active() {
            return cmd_status;
        }
    }

    let steam_content_log = steam_base_dir.as_ref().join("logs").join("content_log.txt");
    let steam_log_status =
        crate::steamcmd::check_content_log_file(&steam_content_log, crate::steamcmd::CS2_APP_ID);
    if steam_log_status.is_active() {
        let process_ok = if require_live_steam_process {
            crate::is_process_running("steam").unwrap_or(false)
        } else {
            true
        };
        if process_ok {
            return steam_log_status;
        }
    }

    crate::steamcmd::SteamCmdLogStatus::idle()
}

pub fn get_cs2_download_validation_status() -> crate::steamcmd::SteamCmdLogStatus {
    let (_steam_exe, steam_dir) = resolve_steam_paths();
    get_cs2_download_validation_status_for_paths(
        steam_dir,
        crate::steamcmd::DEFAULT_STEAMCMD_DIR,
        true,
    )
}

pub fn is_cs2_downloading_or_updating() -> bool {
    get_cs2_download_validation_status().is_active()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_check_downloading_dir_has_730_folder() {
        let tmp = tempdir().unwrap();
        let downloading = tmp.path().join("steamapps").join("downloading");
        std::fs::create_dir_all(&downloading).unwrap();

        assert!(!check_downloading_dir_has_730(&downloading));

        let folder_730 = downloading.join("730");
        std::fs::create_dir(&folder_730).unwrap();
        assert!(check_downloading_dir_has_730(&downloading));

        std::fs::remove_dir(&folder_730).unwrap();
        assert!(!check_downloading_dir_has_730(&downloading));
    }

    #[test]
    fn test_check_downloading_dir_has_730_file() {
        let tmp = tempdir().unwrap();
        let downloading = tmp.path().join("steamapps").join("downloading");
        std::fs::create_dir_all(&downloading).unwrap();

        let patch_file = downloading.join("state_730_23456.patch");
        std::fs::write(&patch_file, b"test").unwrap();
        assert!(check_downloading_dir_has_730(&downloading));

        std::fs::remove_file(&patch_file).unwrap();
        assert!(!check_downloading_dir_has_730(&downloading));
    }

    #[test]
    fn test_check_downloading_dir_other_game_not_flagged() {
        let tmp = tempdir().unwrap();
        let downloading = tmp.path().join("steamapps").join("downloading");
        std::fs::create_dir_all(&downloading).unwrap();

        let dota_folder = downloading.join("570");
        std::fs::create_dir(&dota_folder).unwrap();
        let dota_file = downloading.join("state_570_123.patch");
        std::fs::write(&dota_file, b"test").unwrap();

        assert!(!check_downloading_dir_has_730(&downloading));
    }

    #[test]
    fn test_check_downloading_dir_non_existent() {
        let tmp = tempdir().unwrap();
        let non_existent = tmp.path().join("does_not_exist");
        assert!(!check_downloading_dir_has_730(&non_existent));
    }

    #[test]
    fn test_get_steam_downloading_dirs_from_base_with_vdf() {
        let tmp = tempdir().unwrap();
        let steamapps = tmp.path().join("steamapps");
        std::fs::create_dir_all(&steamapps).unwrap();

        let extra_lib = tempdir().unwrap();
        let extra_path_str = extra_lib.path().to_str().unwrap().replace('\\', "\\\\");

        let vdf_content = format!(
            r#""libraryfolders"
{{
	"0"
	{{
		"path"		"{}"
	}}
	"1"
	{{
		"path"		"{}"
	}}
}}
"#,
            tmp.path().to_str().unwrap().replace('\\', "\\\\"),
            extra_path_str
        );

        std::fs::write(steamapps.join("libraryfolders.vdf"), vdf_content).unwrap();

        let dirs = get_steam_downloading_dirs_from_base(tmp.path());
        assert_eq!(dirs.len(), 2);
        assert_eq!(dirs[0], tmp.path().join("steamapps").join("downloading"));
        assert_eq!(dirs[1], extra_lib.path().join("steamapps").join("downloading"));
    }

    #[test]
    fn test_multi_layer_validation_combines_dir_steamcmd_and_steam_logs() {
        let steam_tmp = tempdir().unwrap();
        let steamcmd_tmp = tempdir().unwrap();

        std::fs::create_dir_all(steam_tmp.path().join("logs")).unwrap();
        std::fs::create_dir_all(steam_tmp.path().join("steamapps").join("downloading")).unwrap();
        std::fs::create_dir_all(steamcmd_tmp.path().join("logs")).unwrap();

        let idle_status = get_cs2_download_validation_status_for_paths(
            steam_tmp.path(),
            steamcmd_tmp.path(),
            false,
        );
        assert!(!idle_status.is_active());

        // Camada SteamCMD content_log indicando validação em andamento
        let cmd_content_log = steamcmd_tmp.path().join("logs").join("content_log.txt");
        std::fs::write(
            &cmd_content_log,
            "[2026-11-13 14:00:00] Client version: 1788292693\n[2026-11-13 14:00:01] AppID 730 App update changed : Running Update,Verifying Installed,\n",
        )
        .unwrap();

        let cmd_val_status = get_cs2_download_validation_status_for_paths(
            steam_tmp.path(),
            steamcmd_tmp.path(),
            false,
        );
        assert_eq!(
            cmd_val_status.state,
            crate::steamcmd::DownloadActivityState::Validating
        );

        // Conclui log do SteamCMD e ativa validação na Steam Desktop
        std::fs::write(
            &cmd_content_log,
            "[2026-11-13 14:00:00] Client version: 1788292693\n[2026-11-13 14:00:01] AppID 730 App update changed : None\n[2026-11-13 14:00:02] AppID 730 scheduler finished : removed from schedule\n",
        )
        .unwrap();

        let steam_content_log = steam_tmp.path().join("logs").join("content_log.txt");
        std::fs::write(
            &steam_content_log,
            "[2026-11-13 14:05:00] Start validating appID 730 on BuildID 26598758\n[2026-11-13 14:05:00] AppID 730 App update changed : Running Update,Verifying Installed,\n",
        )
        .unwrap();

        let steam_val_status = get_cs2_download_validation_status_for_paths(
            steam_tmp.path(),
            steamcmd_tmp.path(),
            false,
        );
        assert_eq!(
            steam_val_status.state,
            crate::steamcmd::DownloadActivityState::Validating
        );

        // Quando Steam também finaliza, volta para Idle
        std::fs::write(
            &steam_content_log,
            "[2026-11-13 14:05:00] AppID 730 App update changed : None\n[2026-11-13 14:06:00] AppID 730 scheduler finished : removed from schedule\n",
        )
        .unwrap();

        let final_status = get_cs2_download_validation_status_for_paths(
            steam_tmp.path(),
            steamcmd_tmp.path(),
            false,
        );
        assert!(!final_status.is_active());
    }

    #[test]
    fn test_empty_730_downloading_folder_continues_to_check_logs() {
        let steam_tmp = tempdir().unwrap();
        let steamcmd_tmp = tempdir().unwrap();

        let downloading_730 = steam_tmp
            .path()
            .join("steamapps")
            .join("downloading")
            .join("730");
        std::fs::create_dir_all(&downloading_730).unwrap();
        std::fs::create_dir_all(steam_tmp.path().join("logs")).unwrap();
        std::fs::create_dir_all(steamcmd_tmp.path().join("logs")).unwrap();

        assert_eq!(
            inspect_downloading_dir_730(downloading_730.parent().unwrap()),
            DownloadingDirCheckResult::Empty730Folder(downloading_730.clone())
        );

        // Caso 1: Pasta 730 existe e está vazia + logs inativos => libera (Idle)
        let steam_log = steam_tmp.path().join("logs").join("content_log.txt");
        std::fs::write(
            &steam_log,
            "[2026-11-13 15:00:00] AppID 730 App update changed : None\n[2026-11-13 15:00:01] AppID 730 scheduler finished : removed from schedule\n",
        )
        .unwrap();

        let status_idle = get_cs2_download_validation_status_for_paths(
            steam_tmp.path(),
            steamcmd_tmp.path(),
            false,
        );
        assert!(
            !status_idle.is_active(),
            "Pasta 730 vazia com logs finalizados deve continuar aos logs e retornar Idle"
        );

        // Subpasta vazia dentro de 730 também deve continuar aos logs
        let empty_sub = downloading_730.join("game");
        std::fs::create_dir_all(&empty_sub).unwrap();
        let status_empty_sub = get_cs2_download_validation_status_for_paths(
            steam_tmp.path(),
            steamcmd_tmp.path(),
            false,
        );
        assert!(!status_empty_sub.is_active());

        // Caso 2: Pasta 730 existe e está vazia, mas logs mostram validação/download ativo => bloqueia pelos logs
        std::fs::write(
            &steam_log,
            "[2026-11-13 15:02:00] AppID 730 App update changed : Running Update,Verifying Installed,\n",
        )
        .unwrap();

        let status_active_from_logs = get_cs2_download_validation_status_for_paths(
            steam_tmp.path(),
            steamcmd_tmp.path(),
            false,
        );
        assert_eq!(
            status_active_from_logs.state,
            crate::steamcmd::DownloadActivityState::Validating
        );
        assert_eq!(status_active_from_logs.source, "logs/content_log.txt");

        // Caso 3: Logs finalizados, mas pasta 730 contém arquivo ativo => detecta download
        std::fs::write(
            &steam_log,
            "[2026-11-13 15:03:00] AppID 730 App update changed : None\n",
        )
        .unwrap();
        std::fs::write(empty_sub.join("chunk.tmp"), b"payload").unwrap();

        let status_with_file = get_cs2_download_validation_status_for_paths(
            steam_tmp.path(),
            steamcmd_tmp.path(),
            false,
        );
        assert_eq!(
            status_with_file.state,
            crate::steamcmd::DownloadActivityState::Downloading
        );
        assert_eq!(status_with_file.source, "steamapps/downloading/");
    }
}
