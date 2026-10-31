use crate::silent_command;
use crate::{Tweak, TweakError, TweakStatus};
use windows_api::WindowsApiError;

pub const SUB_PCIEXPRESS: &str = "SUB_PCIEXPRESS";
pub const PCI_ASPM_GUID: &str = "ee12f906-d277-404b-b6da-e5fa1a576df5";
pub const PCI_ASPM_ALIAS: &str = "ASPM";
pub const SUB_USB_GUID: &str = "2a737441-1930-4402-8d77-b2bebba308a3";
pub const USB_SELECTIVE_SUSPEND_GUID: &str = "48e6b7a6-50f5-4782-a5d4-53bb8f07e226";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsbDevicePowerStatus {
    pub instance_name: String,
    pub power_saving_enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerSavingStatus {
    pub pcie_ac_setting: Option<u32>,
    pub pcie_dc_setting: Option<u32>,
    pub usb_suspend_ac_setting: Option<u32>,
    pub usb_suspend_dc_setting: Option<u32>,
    pub usb_devices: Vec<UsbDevicePowerStatus>,
}

impl PowerSavingStatus {
    pub fn is_pcie_optimized(&self) -> bool {
        self.pcie_ac_setting == Some(0) && (self.pcie_dc_setting == Some(0) || self.pcie_dc_setting.is_none())
    }

    pub fn is_usb_suspend_optimized(&self) -> bool {
        self.usb_suspend_ac_setting == Some(0) && (self.usb_suspend_dc_setting == Some(0) || self.usb_suspend_dc_setting.is_none())
    }

    pub fn is_usb_hubs_optimized(&self) -> bool {
        if self.usb_devices.is_empty() {
            true
        } else {
            self.usb_devices.iter().all(|dev| !dev.power_saving_enabled)
        }
    }

    pub fn is_all_optimized(&self) -> bool {
        self.is_pcie_optimized() && self.is_usb_suspend_optimized() && self.is_usb_hubs_optimized()
    }
}

fn set_powercfg_ac_dc(subgroup: &str, setting: &str, value: u32) -> Result<(), TweakError> {
    let val_str = value.to_string();

    let output_ac = silent_command("powercfg")
        .args(["/setacvalueindex", "SCHEME_CURRENT", subgroup, setting, &val_str])
        .output()
        .map_err(|e| TweakError::WindowsApi(WindowsApiError::Io(e)))?;

    let stdout_ac = String::from_utf8_lossy(&output_ac.stdout);
    let stderr_ac = String::from_utf8_lossy(&output_ac.stderr);
    let combined_ac = format!("{} {}", stdout_ac, stderr_ac).to_lowercase();

    if combined_ac.contains("acesso negado") || combined_ac.contains("access is denied") {
        return Err(TweakError::WindowsApi(WindowsApiError::AccessDenied));
    }

    if !output_ac.status.success() {
        return Err(TweakError::WindowsApi(WindowsApiError::ExecutionFailed(format!(
            "Falha ao definir valor AC em powercfg para subgrupo {} e config {}: {}",
            subgroup, setting, combined_ac.trim()
        ))));
    }

    let output_dc = silent_command("powercfg")
        .args(["/setdcvalueindex", "SCHEME_CURRENT", subgroup, setting, &val_str])
        .output()
        .map_err(|e| TweakError::WindowsApi(WindowsApiError::Io(e)))?;

    let stdout_dc = String::from_utf8_lossy(&output_dc.stdout);
    let stderr_dc = String::from_utf8_lossy(&output_dc.stderr);
    let combined_dc = format!("{} {}", stdout_dc, stderr_dc).to_lowercase();

    if combined_dc.contains("acesso negado") || combined_dc.contains("access is denied") {
        return Err(TweakError::WindowsApi(WindowsApiError::AccessDenied));
    }

    if !output_dc.status.success() {
        return Err(TweakError::WindowsApi(WindowsApiError::ExecutionFailed(format!(
            "Falha ao definir valor DC em powercfg para subgrupo {} e config {}: {}",
            subgroup, setting, combined_dc.trim()
        ))));
    }

    Ok(())
}

fn apply_powercfg_active_scheme() -> Result<(), TweakError> {
    let output = silent_command("powercfg")
        .args(["/setactive", "SCHEME_CURRENT"])
        .output()
        .map_err(|e| TweakError::WindowsApi(WindowsApiError::Io(e)))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(TweakError::WindowsApi(WindowsApiError::ExecutionFailed(format!(
            "Falha ao recarregar esquema ativo com powercfg /setactive SCHEME_CURRENT: {}",
            stderr.trim()
        ))));
    }

    Ok(())
}

fn parse_powercfg_ac_dc(stdout: &str) -> (Option<u32>, Option<u32>) {
    let mut ac = None;
    let mut dc = None;

    for line in stdout.lines() {
        let lower = line.to_lowercase();
        if (lower.contains("alternadas") || lower.contains("ac power setting")) && lower.contains("0x") {
            if let Some(pos) = lower.find("0x") {
                let hex_part = &lower[pos + 2..];
                let hex_digits: String = hex_part.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
                if let Ok(val) = u32::from_str_radix(&hex_digits, 16) {
                    ac = Some(val);
                }
            }
        } else if (lower.contains("cont") || lower.contains("dc power setting")) && lower.contains("0x") {
            if let Some(pos) = lower.find("0x") {
                let hex_part = &lower[pos + 2..];
                let hex_digits: String = hex_part.chars().take_while(|c| c.is_ascii_hexdigit()).collect();
                if let Ok(val) = u32::from_str_radix(&hex_digits, 16) {
                    dc = Some(val);
                }
            }
        }
    }

    (ac, dc)
}

pub fn disable_pcie_power_saving() -> Result<(), TweakError> {
    let result = set_powercfg_ac_dc(SUB_PCIEXPRESS, PCI_ASPM_GUID, 0);
    if result.is_err() {
        set_powercfg_ac_dc(SUB_PCIEXPRESS, PCI_ASPM_ALIAS, 0)?;
    }
    apply_powercfg_active_scheme()?;
    Ok(())
}

pub fn rollback_pcie_power_saving() -> Result<(), TweakError> {
    let result = set_powercfg_ac_dc(SUB_PCIEXPRESS, PCI_ASPM_GUID, 1);
    if result.is_err() {
        set_powercfg_ac_dc(SUB_PCIEXPRESS, PCI_ASPM_ALIAS, 1)?;
    }
    apply_powercfg_active_scheme()?;
    Ok(())
}

pub fn disable_usb_selective_suspend() -> Result<(), TweakError> {
    set_powercfg_ac_dc(SUB_USB_GUID, USB_SELECTIVE_SUSPEND_GUID, 0)?;
    apply_powercfg_active_scheme()?;
    Ok(())
}

pub fn rollback_usb_selective_suspend() -> Result<(), TweakError> {
    set_powercfg_ac_dc(SUB_USB_GUID, USB_SELECTIVE_SUSPEND_GUID, 1)?;
    apply_powercfg_active_scheme()?;
    Ok(())
}

pub fn disable_usb_hubs_power_saving() -> Result<(), TweakError> {
    let script = r#"
$ProgressPreference = 'SilentlyContinue'
Get-CimInstance -ClassName MSPower_DeviceEnable -Namespace root\wmi | Where-Object { $_.InstanceName -match 'USB' } | Set-CimInstance -Property @{ Enable = $false }
"#;

    windows_api::run_powershell_script(script).map_err(|e| {
        match e {
            WindowsApiError::ExecutionFailed(msg) if msg.to_lowercase().contains("unauthorized") || msg.to_lowercase().contains("access") => {
                TweakError::WindowsApi(WindowsApiError::AccessDenied)
            }
            other => TweakError::WindowsApi(other),
        }
    })?;

    Ok(())
}

pub fn rollback_usb_hubs_power_saving() -> Result<(), TweakError> {
    let script = r#"
$ProgressPreference = 'SilentlyContinue'
Get-CimInstance -ClassName MSPower_DeviceEnable -Namespace root\wmi | Where-Object { $_.InstanceName -match 'USB' } | Set-CimInstance -Property @{ Enable = $true }
"#;

    windows_api::run_powershell_script(script).map_err(|e| {
        match e {
            WindowsApiError::ExecutionFailed(msg) if msg.to_lowercase().contains("unauthorized") || msg.to_lowercase().contains("access") => {
                TweakError::WindowsApi(WindowsApiError::AccessDenied)
            }
            other => TweakError::WindowsApi(other),
        }
    })?;

    Ok(())
}

pub fn disable_all_power_saving() -> Result<(), TweakError> {
    disable_pcie_power_saving()?;
    disable_usb_selective_suspend()?;
    disable_usb_hubs_power_saving()?;
    apply_powercfg_active_scheme()?;
    Ok(())
}

pub fn rollback_all_power_saving() -> Result<(), TweakError> {
    rollback_pcie_power_saving()?;
    rollback_usb_selective_suspend()?;
    rollback_usb_hubs_power_saving()?;
    apply_powercfg_active_scheme()?;
    Ok(())
}

pub fn get_power_saving_status() -> Result<PowerSavingStatus, TweakError> {
    let output_pcie = silent_command("powercfg")
        .args(["/q", "SCHEME_CURRENT", SUB_PCIEXPRESS, PCI_ASPM_GUID])
        .output();

    let (pcie_ac, pcie_dc) = match output_pcie {
        Ok(out) if out.status.success() => {
            parse_powercfg_ac_dc(&String::from_utf8_lossy(&out.stdout))
        }
        _ => {
            if let Ok(out_alias) = silent_command("powercfg")
                .args(["/q", "SCHEME_CURRENT", SUB_PCIEXPRESS, PCI_ASPM_ALIAS])
                .output()
            {
                if out_alias.status.success() {
                    parse_powercfg_ac_dc(&String::from_utf8_lossy(&out_alias.stdout))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            }
        }
    };

    let output_usb = silent_command("powercfg")
        .args(["/q", "SCHEME_CURRENT", SUB_USB_GUID, USB_SELECTIVE_SUSPEND_GUID])
        .output();

    let (usb_ac, usb_dc) = match output_usb {
        Ok(out) if out.status.success() => {
            parse_powercfg_ac_dc(&String::from_utf8_lossy(&out.stdout))
        }
        _ => (None, None),
    };

    let script = r#"
$ProgressPreference = 'SilentlyContinue'
Get-CimInstance -ClassName MSPower_DeviceEnable -Namespace root\wmi | Where-Object { $_.InstanceName -match 'USB' } | ForEach-Object {
    "$($_.InstanceName);$($_.Enable)"
}
"#;

    let ps_output = windows_api::run_powershell_script(script).unwrap_or_default();
    let mut usb_devices = Vec::new();

    for line in ps_output.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parts: Vec<&str> = trimmed.split(';').collect();
        if parts.len() >= 2 {
            let instance_name = parts[0].trim().to_string();
            let is_enabled = parts[1].trim().eq_ignore_ascii_case("true");
            usb_devices.push(UsbDevicePowerStatus {
                instance_name,
                power_saving_enabled: is_enabled,
            });
        }
    }

    Ok(PowerSavingStatus {
        pcie_ac_setting: pcie_ac,
        pcie_dc_setting: pcie_dc,
        usb_suspend_ac_setting: usb_ac,
        usb_suspend_dc_setting: usb_dc,
        usb_devices,
    })
}

pub struct PcieUsbPowerSavingTweak;

impl Tweak for PcieUsbPowerSavingTweak {
    fn id(&self) -> &'static str {
        "pcie_usb_power_saving"
    }

    fn name(&self) -> &'static str {
        "Desativação de Economia de Energia PCIe e USB"
    }

    fn description(&self) -> &'static str {
        "Desativa o Link State Power Management (ASPM) no PCIe, a Suspensão Seletiva USB e desmarca a economia de energia em Hubs USB para eliminar stutters e latência no CS2."
    }

    fn requires_admin(&self) -> bool {
        true
    }

    fn status(&self) -> Result<TweakStatus, TweakError> {
        let status = get_power_saving_status()?;
        if status.is_all_optimized() {
            Ok(TweakStatus::Applied)
        } else {
            Ok(TweakStatus::Default)
        }
    }

    fn apply(&self) -> Result<(), TweakError> {
        disable_all_power_saving()
    }

    fn rollback(&self) -> Result<(), TweakError> {
        rollback_all_power_saving()
    }
}
