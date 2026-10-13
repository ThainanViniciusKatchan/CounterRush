use serde::{Deserialize, Serialize};
use sysinfo::System;
use crate::powershell::run_powershell;
use crate::WindowsApiError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model: String,
    pub physical_cores: usize,
    pub logical_cores: usize,
    pub frequency_mhz: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub model: String,
    pub vram_gb: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RamInfo {
    pub total_gb: f64,
    pub speed_mhz: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub cpu: CpuInfo,
    pub gpu: GpuInfo,
    pub ram: RamInfo,
}

pub fn detect_hardware() -> Result<HardwareInfo, WindowsApiError> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpu = detect_cpu(&sys)?;
    let ram = detect_ram(&sys)?;
    let gpu = detect_gpu()?;

    Ok(HardwareInfo { cpu, gpu, ram })
}

pub fn get_cpu_physical_cores() -> usize {
    let mut sys = System::new();
    sys.refresh_all();
    sys.physical_core_count().unwrap_or_else(|| sys.cpus().len())
}

fn detect_cpu(sys: &System) -> Result<CpuInfo, WindowsApiError> {
    let cpus = sys.cpus();
    if cpus.is_empty() {
        return Err(WindowsApiError::ExecutionFailed(
            "Nenhum processador foi identificado no sistema.".to_string(),
        ));
    }

    let first_cpu = &cpus[0];
    let model = first_cpu.brand().trim().to_string();
    let logical_cores = cpus.len();
    let physical_cores = sys.physical_core_count().unwrap_or(logical_cores);
    let mut frequency_mhz = first_cpu.frequency();

    if frequency_mhz == 0 {
        if let Ok(clock_output) = run_powershell(
            "(Get-CimInstance Win32_Processor -ErrorAction SilentlyContinue | Select-Object -First 1).MaxClockSpeed",
        ) {
            frequency_mhz = clock_output.parse::<u64>().unwrap_or(0);
        }
    }

    Ok(CpuInfo {
        model,
        physical_cores,
        logical_cores,
        frequency_mhz,
    })
}

fn detect_ram(sys: &System) -> Result<RamInfo, WindowsApiError> {
    let total_bytes = sys.total_memory();
    let total_gb = (total_bytes as f64) / (1024.0 * 1024.0 * 1024.0);

    let speed_mhz = run_powershell(
        "(Get-CimInstance Win32_PhysicalMemory -ErrorAction SilentlyContinue | Select-Object -First 1).Speed",
    )
    .ok()
    .and_then(|output| output.parse::<u32>().ok());

    Ok(RamInfo {
        total_gb: (total_gb * 100.0).round() / 100.0,
        speed_mhz,
    })
}

fn detect_gpu() -> Result<GpuInfo, WindowsApiError> {
    let script_registry = r#"
        $gpus = Get-ItemProperty -Path 'HKLM:\SYSTEM\CurrentControlSet\Control\Class\{4d36e968-e325-11ce-bfc1-08002be10318}\000*' -Name 'DriverDesc', 'HardwareInformation.qwMemorySize' -ErrorAction SilentlyContinue | Where-Object { $_.DriverDesc }
        if ($gpus) {
            $gpu = $gpus | Select-Object -First 1
            "$($gpu.DriverDesc)|$($gpu.'HardwareInformation.qwMemorySize')"
        }
    "#;

    if let Ok(output) = run_powershell(script_registry) {
        if !output.is_empty() && output.contains('|') {
            let parts: Vec<&str> = output.split('|').collect();
            if parts.len() >= 2 {
                let model = parts[0].trim().to_string();
                let bytes = parts[1].trim().parse::<u64>().unwrap_or(0);
                let vram_gb = (bytes as f64) / (1024.0 * 1024.0 * 1024.0);
                return Ok(GpuInfo {
                    model,
                    vram_gb: (vram_gb * 100.0).round() / 100.0,
                });
            }
        }
    }

    let script_fallback = r#"
        $gpu = Get-CimInstance Win32_VideoController -ErrorAction SilentlyContinue | Where-Object { $_.Name } | Select-Object -First 1
        if ($gpu) {
            "$($gpu.Name)|$($gpu.AdapterRAM)"
        }
    "#;

    let output = run_powershell(script_fallback)?;
    if !output.is_empty() && output.contains('|') {
        let parts: Vec<&str> = output.split('|').collect();
        if parts.len() >= 2 {
            let model = parts[0].trim().to_string();
            let bytes = parts[1].trim().parse::<u64>().unwrap_or(0);
            let vram_gb = (bytes as f64) / (1024.0 * 1024.0 * 1024.0);
            return Ok(GpuInfo {
                model,
                vram_gb: (vram_gb * 100.0).round() / 100.0,
            });
        }
    }

    Err(WindowsApiError::ExecutionFailed(
        "Nenhuma placa de vídeo compatível foi identificada no sistema.".to_string(),
    ))
}
