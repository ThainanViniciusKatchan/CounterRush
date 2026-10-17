use crate::{Tweak, TweakError, TweakStatus};
use windows_api::run_powershell;

pub fn get_memory_compression_status() -> Result<bool, TweakError> {
    let output = run_powershell("(Get-MMAgent).MemoryCompression")?;
    Ok(output.trim().eq_ignore_ascii_case("True"))
}

pub fn disable_memory_compression() -> Result<(), TweakError> {
    let hardware = windows_api::detect_hardware()?;
    if hardware.ram.total_gb <= 16.0 {
        return Err(TweakError::ValidationFailed(format!(
            "A desativação da compactação de memória requer mais de 16 GB de RAM. Detectado: {:.2} GB.",
            hardware.ram.total_gb
        )));
    }

    let is_enabled = get_memory_compression_status()?;
    if !is_enabled {
        return Ok(());
    }

    run_powershell("Disable-MMAgent -mc")?;
    Ok(())
}

pub fn enable_memory_compression() -> Result<(), TweakError> {
    run_powershell("Enable-MMAgent -mc")?;
    Ok(())
}

pub struct MemoryCompressionTweak;

impl Tweak for MemoryCompressionTweak {
    fn id(&self) -> &'static str {
        "memory_compression"
    }

    fn name(&self) -> &'static str {
        "Desativação de Compactação de Memória (MMAgent)"
    }

    fn description(&self) -> &'static str {
        "Desativa a compactação de memória do Windows (Disable-MMAgent -mc) em PCs com mais de 16GB de RAM para reduzir latência de CPU e micro-stutters."
    }

    fn requires_admin(&self) -> bool {
        true
    }

    fn status(&self) -> Result<TweakStatus, TweakError> {
        let is_enabled = get_memory_compression_status()?;
        if is_enabled {
            Ok(TweakStatus::Default)
        } else {
            Ok(TweakStatus::Applied)
        }
    }

    fn apply(&self) -> Result<(), TweakError> {
        disable_memory_compression()
    }

    fn rollback(&self) -> Result<(), TweakError> {
        enable_memory_compression()
    }
}
