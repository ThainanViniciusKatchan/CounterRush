use crate::{Tweak, TweakError, TweakStatus};
use windows_api::run_powershell_script;

pub const PS_SCRIPT_ENABLE_GPU_MSI: &str = r#"
$ProgressPreference = 'SilentlyContinue'
$gpus = Get-PnpDevice -Class Display | Where-Object { $_.InstanceId -like "PCI*" -and $_.Status -eq "OK" }
foreach ($gpu in $gpus) {
    $basePath = "HKLM:\SYSTEM\CurrentControlSet\Enum\$($gpu.InstanceId)\Device Parameters\Interrupt Management"
    New-Item -Path "$basePath\MessageSignaledInterruptProperties" -Force | Out-Null
    Set-ItemProperty -Path "$basePath\MessageSignaledInterruptProperties" -Name "MSISupported" -Value 1 -Type DWord
    
    New-Item -Path "$basePath\Affinity Policy" -Force | Out-Null
    Set-ItemProperty -Path "$basePath\Affinity Policy" -Name "DevicePriority" -Value 3 -Type DWord
}
"#;

pub const PS_SCRIPT_ENABLE_NETWORK_MSI: &str = r#"
$ProgressPreference = 'SilentlyContinue'
$nics = Get-PnpDevice -Class Net | Where-Object { $_.InstanceId -like "PCI*" -and $_.Status -eq "OK" }
foreach ($nic in $nics) {
    $basePath = "HKLM:\SYSTEM\CurrentControlSet\Enum\$($nic.InstanceId)\Device Parameters\Interrupt Management"
    New-Item -Path "$basePath\MessageSignaledInterruptProperties" -Force | Out-Null
    Set-ItemProperty -Path "$basePath\MessageSignaledInterruptProperties" -Name "MSISupported" -Value 1 -Type DWord
}
"#;

pub const PS_SCRIPT_ENABLE_USB_MSI: &str = r#"
$ProgressPreference = 'SilentlyContinue'
$usbControllers = Get-PnpDevice | Where-Object { $_.InstanceId -like "PCI*" -and $_.FriendlyName -like "*Host Controller*" -and $_.Status -eq "OK" }
foreach ($usb in $usbControllers) {
    $basePath = "HKLM:\SYSTEM\CurrentControlSet\Enum\$($usb.InstanceId)\Device Parameters\Interrupt Management"
    New-Item -Path "$basePath\MessageSignaledInterruptProperties" -Force | Out-Null
    Set-ItemProperty -Path "$basePath\MessageSignaledInterruptProperties" -Name "MSISupported" -Value 1 -Type DWord
}
"#;

pub const PS_SCRIPT_ROLLBACK_GPU_MSI: &str = r#"
$ProgressPreference = 'SilentlyContinue'
$gpus = Get-PnpDevice -Class Display | Where-Object { $_.InstanceId -like "PCI*" -and $_.Status -eq "OK" }
foreach ($gpu in $gpus) {
    $basePath = "HKLM:\SYSTEM\CurrentControlSet\Enum\$($gpu.InstanceId)\Device Parameters\Interrupt Management"
    Remove-Item -Path "$basePath\MessageSignaledInterruptProperties" -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item -Path "$basePath\Affinity Policy" -Recurse -Force -ErrorAction SilentlyContinue
}
"#;

pub const PS_SCRIPT_ROLLBACK_NETWORK_MSI: &str = r#"
$ProgressPreference = 'SilentlyContinue'
$nics = Get-PnpDevice -Class Net | Where-Object { $_.InstanceId -like "PCI*" -and $_.Status -eq "OK" }
foreach ($nic in $nics) {
    $basePath = "HKLM:\SYSTEM\CurrentControlSet\Enum\$($nic.InstanceId)\Device Parameters\Interrupt Management"
    Remove-Item -Path "$basePath\MessageSignaledInterruptProperties" -Recurse -Force -ErrorAction SilentlyContinue
}
"#;

pub const PS_SCRIPT_ROLLBACK_USB_MSI: &str = r#"
$ProgressPreference = 'SilentlyContinue'
$usbControllers = Get-PnpDevice | Where-Object { $_.InstanceId -like "PCI*" -and $_.FriendlyName -like "*Host Controller*" -and $_.Status -eq "OK" }
foreach ($usb in $usbControllers) {
    $basePath = "HKLM:\SYSTEM\CurrentControlSet\Enum\$($usb.InstanceId)\Device Parameters\Interrupt Management"
    Remove-Item -Path "$basePath\MessageSignaledInterruptProperties" -Recurse -Force -ErrorAction SilentlyContinue
}
"#;

pub const PS_SCRIPT_QUERY_MSI_STATUS: &str = r#"
$ProgressPreference = 'SilentlyContinue'
Get-PnpDevice -Class Display | Where-Object { $_.InstanceId -like "PCI*" -and $_.Status -eq "OK" } | ForEach-Object {
    $base = "HKLM:\SYSTEM\CurrentControlSet\Enum\$($_.InstanceId)\Device Parameters\Interrupt Management"
    $msi = (Get-ItemProperty -Path "$base\MessageSignaledInterruptProperties" -ErrorAction SilentlyContinue).MSISupported
    $prio = (Get-ItemProperty -Path "$base\Affinity Policy" -ErrorAction SilentlyContinue).DevicePriority
    Write-Output "GPU;$($_.FriendlyName);$($_.InstanceId);$msi;$prio"
}
Get-PnpDevice -Class Net | Where-Object { $_.InstanceId -like "PCI*" -and $_.Status -eq "OK" } | ForEach-Object {
    $base = "HKLM:\SYSTEM\CurrentControlSet\Enum\$($_.InstanceId)\Device Parameters\Interrupt Management"
    $msi = (Get-ItemProperty -Path "$base\MessageSignaledInterruptProperties" -ErrorAction SilentlyContinue).MSISupported
    Write-Output "Network;$($_.FriendlyName);$($_.InstanceId);$msi;"
}
Get-PnpDevice | Where-Object { $_.InstanceId -like "PCI*" -and $_.FriendlyName -like "*Host Controller*" -and $_.Status -eq "OK" } | ForEach-Object {
    $base = "HKLM:\SYSTEM\CurrentControlSet\Enum\$($_.InstanceId)\Device Parameters\Interrupt Management"
    $msi = (Get-ItemProperty -Path "$base\MessageSignaledInterruptProperties" -ErrorAction SilentlyContinue).MSISupported
    Write-Output "USB;$($_.FriendlyName);$($_.InstanceId);$msi;"
}
"#;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsiDeviceStatus {
    pub category: String,
    pub name: String,
    pub instance_id: String,
    pub msi_supported: Option<u32>,
    pub device_priority: Option<u32>,
}

impl MsiDeviceStatus {
    pub fn is_msi_enabled(&self) -> bool {
        self.msi_supported == Some(1)
    }

    pub fn is_gpu_high_priority(&self) -> bool {
        if self.category == "GPU" {
            self.device_priority == Some(3)
        } else {
            true
        }
    }
}

pub fn get_msi_status() -> Result<Vec<MsiDeviceStatus>, TweakError> {
    let output = run_powershell_script(PS_SCRIPT_QUERY_MSI_STATUS)?;
    let mut devices = Vec::new();

    for line in output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let parts: Vec<&str> = trimmed.split(';').collect();
        if parts.len() >= 4 {
            let category = parts[0].trim().to_string();
            let name = parts[1].trim().to_string();
            let instance_id = parts[2].trim().to_string();
            let msi_supported = parts[3].trim().parse::<u32>().ok();
            let device_priority = parts.get(4).and_then(|p| p.trim().parse::<u32>().ok());

            devices.push(MsiDeviceStatus {
                category,
                name,
                instance_id,
                msi_supported,
                device_priority,
            });
        }
    }

    Ok(devices)
}

pub fn is_all_msi_enabled() -> Result<bool, TweakError> {
    let devices = get_msi_status()?;
    if devices.is_empty() {
        return Ok(false);
    }

    let all_enabled = devices.iter().all(|d| {
        if d.category == "GPU" {
            d.is_msi_enabled() && d.is_gpu_high_priority()
        } else {
            d.is_msi_enabled()
        }
    });

    Ok(all_enabled)
}

pub fn enable_gpu_msi() -> Result<(), TweakError> {
    run_powershell_script(PS_SCRIPT_ENABLE_GPU_MSI)?;
    Ok(())
}

pub fn enable_network_msi() -> Result<(), TweakError> {
    run_powershell_script(PS_SCRIPT_ENABLE_NETWORK_MSI)?;
    Ok(())
}

pub fn enable_usb_msi() -> Result<(), TweakError> {
    run_powershell_script(PS_SCRIPT_ENABLE_USB_MSI)?;
    Ok(())
}

pub fn enable_all_msi() -> Result<(), TweakError> {
    enable_gpu_msi()?;
    enable_network_msi()?;
    enable_usb_msi()?;
    Ok(())
}

pub fn rollback_gpu_msi() -> Result<(), TweakError> {
    run_powershell_script(PS_SCRIPT_ROLLBACK_GPU_MSI)?;
    Ok(())
}

pub fn rollback_network_msi() -> Result<(), TweakError> {
    run_powershell_script(PS_SCRIPT_ROLLBACK_NETWORK_MSI)?;
    Ok(())
}

pub fn rollback_usb_msi() -> Result<(), TweakError> {
    run_powershell_script(PS_SCRIPT_ROLLBACK_USB_MSI)?;
    Ok(())
}

pub fn rollback_all_msi() -> Result<(), TweakError> {
    rollback_gpu_msi()?;
    rollback_network_msi()?;
    rollback_usb_msi()?;
    Ok(())
}

pub struct MsiModeTweak;

impl Tweak for MsiModeTweak {
    fn id(&self) -> &'static str {
        "msi_mode"
    }

    fn name(&self) -> &'static str {
        "Ativação do Modo MSI (Message Signaled-Based Interrupts)"
    }

    fn description(&self) -> &'static str {
        "Ativa o Modo MSI na GPU (Prioridade Alta), Placa de Rede e Controladoras USB para eliminar disputas de interrupção (IRQs) e reduzir latência no CS2."
    }

    fn requires_admin(&self) -> bool {
        true
    }

    fn status(&self) -> Result<TweakStatus, TweakError> {
        let is_applied = is_all_msi_enabled()?;
        if is_applied {
            Ok(TweakStatus::Applied)
        } else {
            Ok(TweakStatus::Default)
        }
    }

    fn apply(&self) -> Result<(), TweakError> {
        enable_all_msi()
    }

    fn rollback(&self) -> Result<(), TweakError> {
        rollback_all_msi()
    }
}
