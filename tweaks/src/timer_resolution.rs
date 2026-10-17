use std::process::Command;
use crate::{Tweak, TweakError, TweakStatus};
use windows_api::run_powershell_script;

pub const REG_KERNEL_KEY: &str = r#"HKLM\SYSTEM\CurrentControlSet\Control\Session Manager\kernel"#;
pub const REG_TIMER_VALUE: &str = "GlobalTimerResolutionRequests";

pub const PS_CMD_FIRST_WITH_TEST: &str = r#"
if (-not ([System.Management.Automation.PSTypeName]'Timer05').Type) {
    Add-Type -TypeDefinition 'using System; using System.Runtime.InteropServices; public class Timer05 { [DllImport("ntdll.dll")] public static extern int NtSetTimerResolution(uint DesiredResolution, bool SetResolution, out uint CurrentResolution); }';
}
if (-not ([System.Management.Automation.PSTypeName]'KernelTimer').Type) {
    Add-Type -TypeDefinition 'using System; using System.Runtime.InteropServices; public class KernelTimer { [DllImport("ntdll.dll")] public static extern int NtSetTimerResolution(uint Desired, bool Set, out uint Current); [DllImport("ntdll.dll")] public static extern int NtDelayExecution(bool Alertable, ref long DelayInterval); }';
}
$cur = 0;
[Timer05]::NtSetTimerResolution(5000, $true, [ref]$cur) | Out-Null;
Write-Output "Timer Cravado: $($cur / 10000) ms";
$delay = -5000;
1..10 | ForEach-Object {
    $sw = [Diagnostics.Stopwatch]::StartNew();
    [KernelTimer]::NtDelayExecution($false, [ref]$delay) | Out-Null;
    $sw.Stop();
    Write-Output "Pausa $_ : $([math]::Round($sw.Elapsed.TotalMilliseconds, 4)) ms";
}
"#;

pub const PS_CMD_SECOND_WITH_TEST: &str = r#"
if (-not ([System.Management.Automation.PSTypeName]'KernelTimer').Type) {
    Add-Type -TypeDefinition 'using System; using System.Runtime.InteropServices; public class KernelTimer { [DllImport("ntdll.dll")] public static extern int NtSetTimerResolution(uint Desired, bool Set, out uint Current); [DllImport("ntdll.dll")] public static extern int NtDelayExecution(bool Alertable, ref long DelayInterval); }';
}
$cur = 0;
[KernelTimer]::NtSetTimerResolution(5000, $true, [ref]$cur) | Out-Null;
Write-Output "Timer Cravado: $($cur / 10000) ms`n---";
$delay = -5000;
1..10 | ForEach-Object {
    $sw = [Diagnostics.Stopwatch]::StartNew();
    [KernelTimer]::NtDelayExecution($false, [ref]$delay) | Out-Null;
    $sw.Stop();
    Write-Output "Pausa $_ : $([math]::Round($sw.Elapsed.TotalMilliseconds, 4)) ms";
}
"#;

pub const PS_CMD_TEST_ONLY: &str = r#"
if (-not ([System.Management.Automation.PSTypeName]'KernelTimer').Type) {
    Add-Type -TypeDefinition 'using System; using System.Runtime.InteropServices; public class KernelTimer { [DllImport("ntdll.dll")] public static extern int NtSetTimerResolution(uint Desired, bool Set, out uint Current); [DllImport("ntdll.dll")] public static extern int NtDelayExecution(bool Alertable, ref long DelayInterval); }';
}
$delay = -5000;
1..10 | ForEach-Object {
    $sw = [Diagnostics.Stopwatch]::StartNew();
    [KernelTimer]::NtDelayExecution($false, [ref]$delay) | Out-Null;
    $sw.Stop();
    Write-Output "Pausa $_ : $([math]::Round($sw.Elapsed.TotalMilliseconds, 4)) ms";
}
"#;

pub const PS_CMD_ROLLBACK: &str = r#"
if (-not ([System.Management.Automation.PSTypeName]'Timer05').Type) {
    Add-Type -TypeDefinition 'using System; using System.Runtime.InteropServices; public class Timer05 { [DllImport("ntdll.dll")] public static extern int NtSetTimerResolution(uint DesiredResolution, bool SetResolution, out uint CurrentResolution); }';
}
$cur = 0;
[Timer05]::NtSetTimerResolution(5000, $false, [ref]$cur) | Out-Null;
Write-Output "Rollback feito! Timer voltou para: $($cur / 10000) ms";
"#;

#[derive(Debug, Clone)]
pub struct TimerApplyResult {
    pub command_used: u8,
    pub pauses_ms: Vec<f64>,
    pub registry_applied: bool,
}

pub fn parse_pause_times(output: &str) -> Vec<f64> {
    let mut times = Vec::new();
    for line in output.lines() {
        if line.contains("Pausa") && line.contains("ms") {
            if let Some(colon_pos) = line.find(':') {
                let rest = &line[colon_pos + 1..];
                let num_str = rest.trim().trim_end_matches("ms").trim();
                let normalized = num_str.replace(',', ".");
                if let Ok(val) = normalized.parse::<f64>() {
                    times.push(val);
                }
            }
        }
    }
    times
}

pub fn is_precision_reduced(pauses: &[f64]) -> bool {
    if pauses.is_empty() {
        return false;
    }
    let relevant: &[f64] = if pauses.len() > 2 {
        &pauses[2..]
    } else {
        pauses
    };
    let avg = relevant.iter().sum::<f64>() / relevant.len() as f64;
    avg < 2.5
}

pub fn apply_timer_registry() -> Result<(), TweakError> {
    let output = Command::new("reg")
        .args([
            "add",
            REG_KERNEL_KEY,
            "/v",
            REG_TIMER_VALUE,
            "/t",
            "REG_DWORD",
            "/d",
            "1",
            "/f",
        ])
        .output()
        .map_err(|e| TweakError::WindowsApi(windows_api::WindowsApiError::Io(e)))?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(TweakError::ValidationFailed(format!(
            "Falha ao adicionar GlobalTimerResolutionRequests no Registro: {}",
            stderr.trim()
        )))
    }
}

pub fn rollback_timer_registry() -> Result<(), TweakError> {
    let output = Command::new("reg")
        .args(["delete", REG_KERNEL_KEY, "/v", REG_TIMER_VALUE, "/f"])
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
    {
        return Ok(());
    }

    Err(TweakError::ValidationFailed(format!(
        "Falha ao remover GlobalTimerResolutionRequests do Registro: {}",
        stderr.trim()
    )))
}

pub fn is_timer_registry_applied() -> Result<bool, TweakError> {
    let output = Command::new("reg")
        .args(["query", REG_KERNEL_KEY, "/v", REG_TIMER_VALUE])
        .output()
        .map_err(|e| TweakError::WindowsApi(windows_api::WindowsApiError::Io(e)))?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(stdout.contains("0x1"))
    } else {
        Ok(false)
    }
}

pub fn test_timer_precision() -> Result<Vec<f64>, TweakError> {
    let output = run_powershell_script(PS_CMD_TEST_ONLY)?;
    Ok(parse_pause_times(&output))
}

pub fn apply_timer_resolution() -> Result<TimerApplyResult, TweakError> {
    let reg_res = apply_timer_registry();
    let registry_applied = reg_res.is_ok();

    let first_output = run_powershell_script(PS_CMD_FIRST_WITH_TEST);
    if let Ok(ref out) = first_output {
        let pauses = parse_pause_times(out);
        if is_precision_reduced(&pauses) {
            return Ok(TimerApplyResult {
                command_used: 1,
                pauses_ms: pauses,
                registry_applied,
            });
        }
    }

    let second_output = run_powershell_script(PS_CMD_SECOND_WITH_TEST)?;
    let pauses = parse_pause_times(&second_output);
    let reduced = is_precision_reduced(&pauses);

    if !reduced && pauses.is_empty() {
        return Err(TweakError::ValidationFailed(
            "Ambos os comandos de timer falharam ao reduzir a latência de pausas.".to_string(),
        ));
    }

    Ok(TimerApplyResult {
        command_used: 2,
        pauses_ms: pauses,
        registry_applied,
    })
}

pub fn rollback_timer_resolution() -> Result<f64, TweakError> {
    let output = run_powershell_script(PS_CMD_ROLLBACK)?;
    for line in output.lines() {
        if line.contains("Timer voltou para:") {
            if let Some(pos) = line.find(':') {
                let rest = &line[pos + 1..];
                let num_str = rest.trim().trim_end_matches("ms").trim();
                let normalized = num_str.replace(',', ".");
                if let Ok(val) = normalized.parse::<f64>() {
                    return Ok(val);
                }
            }
        }
    }
    Ok(0.0)
}

pub fn rollback_timer_all() -> Result<(), TweakError> {
    let _ = rollback_timer_resolution();
    rollback_timer_registry()?;
    Ok(())
}

pub struct TimerResolutionTweak;

impl Tweak for TimerResolutionTweak {
    fn id(&self) -> &'static str {
        "timer_resolution"
    }

    fn name(&self) -> &'static str {
        "Resolução de Temporizador do Sistema (0.5ms)"
    }

    fn description(&self) -> &'static str {
        "Crava a resolução do timer do Windows em 0.5ms e ativa solicitações globais no registro para reduzir input lag no CS2."
    }

    fn requires_admin(&self) -> bool {
        true
    }

    fn status(&self) -> Result<TweakStatus, TweakError> {
        let reg_applied = is_timer_registry_applied()?;
        if reg_applied {
            Ok(TweakStatus::Applied)
        } else {
            Ok(TweakStatus::Default)
        }
    }

    fn apply(&self) -> Result<(), TweakError> {
        apply_timer_resolution()?;
        Ok(())
    }

    fn rollback(&self) -> Result<(), TweakError> {
        rollback_timer_all()
    }
}
