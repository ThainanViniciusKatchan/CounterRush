use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::path::Path;
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
