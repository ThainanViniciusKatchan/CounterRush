use crate::silent_command;
use crate::{Tweak, TweakError, TweakStatus};
use windows_api::WindowsApiError;

pub const DISPLAY_CLASS_GUID: &str = "{4d36e968-e325-11ce-bfc1-08002be10318}";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AmdGpuFlipQueueStatus {
    pub registry_key: String,
    pub driver_desc: String,
    pub flip_queue_size: Option<String>,
    pub raw_bytes: Option<String>,
    pub is_optimized: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NvidiaGpuPersistenceStatus {
    pub gpu_name: String,
    pub persistence_mode: Option<bool>,
    pub is_optimized: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderQueueStatus {
    pub amd_gpus: Vec<AmdGpuFlipQueueStatus>,
    pub nvidia_gpus: Vec<NvidiaGpuPersistenceStatus>,
}

impl RenderQueueStatus {
    pub fn is_amd_optimized(&self) -> bool {
        if self.amd_gpus.is_empty() {
            true
        } else {
            self.amd_gpus.iter().all(|g| g.is_optimized)
        }
    }

    pub fn is_nvidia_optimized(&self) -> bool {
        if self.nvidia_gpus.is_empty() {
            true
        } else {
            self.nvidia_gpus.iter().all(|g| g.is_optimized)
        }
    }

    pub fn is_all_optimized(&self) -> bool {
        let has_any_gpu = !self.amd_gpus.is_empty() || !self.nvidia_gpus.is_empty();
        has_any_gpu && self.is_amd_optimized() && self.is_nvidia_optimized()
    }
}

pub fn apply_amd_flip_queue() -> Result<(), TweakError> {
    let script = r#"
$ProgressPreference = 'SilentlyContinue'
$gpuKeys = Get-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\000*' -ErrorAction SilentlyContinue | Where-Object { $_.DriverDesc }
if (-not $gpuKeys) {
    throw "Nenhum driver de adaptador de vídeo ativo foi encontrado na classe {4d36e968-e325-11ce-bfc1-08002be10318}."
}
foreach ($gpu in $gpuKeys) {
    Set-ItemProperty -Path $gpu.PSPath -Name "FlipQueueSize" -Value ([byte[]](0x31, 0x00)) -Type Binary
}
"#;

    windows_api::run_powershell_script(script).map_err(|e| {
        match e {
            WindowsApiError::ExecutionFailed(msg) if msg.to_lowercase().contains("unauthorized") || msg.to_lowercase().contains("access") || msg.to_lowercase().contains("permissão") => {
                TweakError::WindowsApi(WindowsApiError::AccessDenied)
            }
            other => TweakError::WindowsApi(other),
        }
    })?;

    Ok(())
}

pub fn rollback_amd_flip_queue() -> Result<(), TweakError> {
    let script = r#"
$ProgressPreference = 'SilentlyContinue'
$gpuKeys = Get-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\000*' -ErrorAction SilentlyContinue | Where-Object { $_.DriverDesc }
foreach ($gpu in $gpuKeys) {
    Remove-ItemProperty -Path $gpu.PSPath -Name "FlipQueueSize" -ErrorAction SilentlyContinue
}
"#;

    windows_api::run_powershell_script(script).map_err(|e| {
        match e {
            WindowsApiError::ExecutionFailed(msg) if msg.to_lowercase().contains("unauthorized") || msg.to_lowercase().contains("access") || msg.to_lowercase().contains("permissão") => {
                TweakError::WindowsApi(WindowsApiError::AccessDenied)
            }
            other => TweakError::WindowsApi(other),
        }
    })?;

    Ok(())
}

pub fn get_amd_flip_queue_status() -> Result<Vec<AmdGpuFlipQueueStatus>, TweakError> {
    let script = r#"
$ProgressPreference = 'SilentlyContinue'
$gpuKeys = Get-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\000*' -ErrorAction SilentlyContinue | Where-Object { $_.DriverDesc }
foreach ($gpu in $gpuKeys) {
    $val = (Get-ItemProperty -Path $gpu.PSPath -Name "FlipQueueSize" -ErrorAction SilentlyContinue).FlipQueueSize
    $valStr = ""
    $valBytes = ""
    if ($val) {
        $valStr = [System.Text.Encoding]::ASCII.GetString($val).Trim([char]0)
        $valBytes = ($val -join ' ')
    }
    "$($gpu.PSChildName);$($gpu.DriverDesc);$valStr;$valBytes"
}
"#;

    let output = windows_api::run_powershell_script(script).map_err(TweakError::WindowsApi)?;
    let mut statuses = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.split(';').collect();
        if parts.len() >= 4 {
            let registry_key = parts[0].trim().to_string();
            let driver_desc = parts[1].trim().to_string();
            let val_str = parts[2].trim();
            let val_bytes = parts[3].trim();

            let flip_queue_size = if val_str.is_empty() {
                None
            } else {
                Some(val_str.to_string())
            };

            let raw_bytes = if val_bytes.is_empty() {
                None
            } else {
                Some(val_bytes.to_string())
            };

            let is_optimized = flip_queue_size.as_deref() == Some("1");

            statuses.push(AmdGpuFlipQueueStatus {
                registry_key,
                driver_desc,
                flip_queue_size,
                raw_bytes,
                is_optimized,
            });
        }
    }

    Ok(statuses)
}

pub fn is_nvidia_smi_available() -> bool {
    silent_command("nvidia-smi")
        .arg("-h")
        .output()
        .map(|out| out.status.success() || !out.stdout.is_empty() || !out.stderr.is_empty())
        .unwrap_or(false)
}

pub fn apply_nvidia_persistence_mode() -> Result<(), TweakError> {
    if !is_nvidia_smi_available() {
        return Err(TweakError::ValidationFailed(
            "O utilitário nvidia-smi não foi encontrado. Esta máquina não possui driver NVIDIA ou nvidia-smi no PATH.".to_string(),
        ));
    }

    let output = silent_command("nvidia-smi")
        .args(["-pm", "1"])
        .output()
        .map_err(|e| TweakError::WindowsApi(WindowsApiError::Io(e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{} {}", stdout, stderr).to_lowercase();

    if combined.contains("access denied") || combined.contains("admin") || combined.contains("permission") {
        return Err(TweakError::WindowsApi(WindowsApiError::AccessDenied));
    }

    if !output.status.success() {
        return Err(TweakError::WindowsApi(WindowsApiError::ExecutionFailed(format!(
            "Falha ao executar nvidia-smi -pm 1: {}",
            combined.trim()
        ))));
    }

    Ok(())
}

pub fn rollback_nvidia_persistence_mode() -> Result<(), TweakError> {
    if !is_nvidia_smi_available() {
        return Err(TweakError::ValidationFailed(
            "O utilitário nvidia-smi não foi encontrado. Esta máquina não possui driver NVIDIA ou nvidia-smi no PATH.".to_string(),
        ));
    }

    let output = silent_command("nvidia-smi")
        .args(["-pm", "0"])
        .output()
        .map_err(|e| TweakError::WindowsApi(WindowsApiError::Io(e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{} {}", stdout, stderr).to_lowercase();

    if combined.contains("access denied") || combined.contains("admin") || combined.contains("permission") {
        return Err(TweakError::WindowsApi(WindowsApiError::AccessDenied));
    }

    if !output.status.success() {
        return Err(TweakError::WindowsApi(WindowsApiError::ExecutionFailed(format!(
            "Falha ao executar nvidia-smi -pm 0: {}",
            combined.trim()
        ))));
    }

    Ok(())
}

pub fn get_nvidia_persistence_mode_status() -> Result<Vec<NvidiaGpuPersistenceStatus>, TweakError> {
    if !is_nvidia_smi_available() {
        return Ok(Vec::new());
    }

    let output = silent_command("nvidia-smi")
        .args(["--query-gpu=name,persistence_mode", "--format=csv,noheader"])
        .output()
        .map_err(|e| TweakError::WindowsApi(WindowsApiError::Io(e)))?;

    if !output.status.success() {
        return Ok(Vec::new());
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut statuses = Vec::new();

    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.split(',').collect();
        if parts.len() >= 2 {
            let gpu_name = parts[0].trim().to_string();
            let mode_str = parts[1].trim().to_lowercase();

            let persistence_mode = if mode_str == "enabled" || mode_str == "1" {
                Some(true)
            } else if mode_str == "disabled" || mode_str == "0" {
                Some(false)
            } else {
                None
            };

            let is_optimized = persistence_mode == Some(true);

            statuses.push(NvidiaGpuPersistenceStatus {
                gpu_name,
                persistence_mode,
                is_optimized,
            });
        }
    }

    Ok(statuses)
}

pub fn apply_render_queue() -> Result<(), TweakError> {
    let amd_statuses = get_amd_flip_queue_status().unwrap_or_default();
    let has_amd = !amd_statuses.is_empty();
    let has_nvidia = is_nvidia_smi_available();

    if !has_amd && !has_nvidia {
        return Err(TweakError::ValidationFailed(
            "Nenhuma GPU AMD ou NVIDIA compatível foi identificada no sistema.".to_string(),
        ));
    }

    let mut applied_any = false;

    if has_amd {
        apply_amd_flip_queue()?;
        applied_any = true;
    }

    if has_nvidia {
        apply_nvidia_persistence_mode()?;
        applied_any = true;
    }

    if !applied_any {
        return Err(TweakError::ValidationFailed(
            "Não foi possível aplicar otimização de fila de quadros.".to_string(),
        ));
    }

    Ok(())
}

pub fn rollback_render_queue() -> Result<(), TweakError> {
    let amd_statuses = get_amd_flip_queue_status().unwrap_or_default();
    let has_amd = !amd_statuses.is_empty();
    let has_nvidia = is_nvidia_smi_available();

    if has_amd {
        rollback_amd_flip_queue()?;
    }

    if has_nvidia {
        rollback_nvidia_persistence_mode()?;
    }

    Ok(())
}

pub fn get_render_queue_status() -> Result<RenderQueueStatus, TweakError> {
    let amd_gpus = get_amd_flip_queue_status()?;
    let nvidia_gpus = get_nvidia_persistence_mode_status()?;

    Ok(RenderQueueStatus {
        amd_gpus,
        nvidia_gpus,
    })
}

pub struct RenderQueueTweak;

impl Tweak for RenderQueueTweak {
    fn id(&self) -> &'static str {
        "render_queue"
    }

    fn name(&self) -> &'static str {
        "Fila de Quadros Renderizados (Render Queue / Flip Queue)"
    }

    fn description(&self) -> &'static str {
        "Configura FlipQueueSize=1 em placas AMD e Modo de Persistência em placas NVIDIA para resposta instantânea de mira e menor latência no CS2."
    }

    fn requires_admin(&self) -> bool {
        true
    }

    fn status(&self) -> Result<TweakStatus, TweakError> {
        let status = get_render_queue_status()?;
        if status.is_all_optimized() {
            Ok(TweakStatus::Applied)
        } else {
            Ok(TweakStatus::Default)
        }
    }

    fn apply(&self) -> Result<(), TweakError> {
        apply_render_queue()
    }

    fn rollback(&self) -> Result<(), TweakError> {
        rollback_render_queue()
    }
}
