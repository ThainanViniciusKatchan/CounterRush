use crate::{Tweak, TweakError, TweakStatus};
use windows_api::{run_powershell, run_powershell_script, WindowsApiError};

pub fn get_memory_compression_status() -> Result<bool, TweakError> {
    let script = r#"
try {
    (Get-MMAgent -ErrorAction Stop).MemoryCompression
} catch {
    $msg = $_.Exception.Message
    $fqid = $_.FullyQualifiedErrorId
    if ($msg -match 'Acesso negado|Access is denied|PermissionDenied|System Error 5' -or $fqid -match 'PermissionDenied|Windows System Error 5') {
        if ($null -ne (Get-Process -Name 'Memory Compression' -ErrorAction SilentlyContinue)) {
            'True'
        } else {
            'False'
        }
    } else {
        Write-Error $msg
        exit 1
    }
}
"#;
    let output = match run_powershell_script(script) {
        Ok(out) => out,
        Err(WindowsApiError::ExecutionFailed(err)) => {
            let lower = err.to_lowercase();
            if lower.contains("acesso negado")
                || lower.contains("access is denied")
                || lower.contains("permissiondenied")
                || lower.contains("system error 5")
            {
                return Err(TweakError::WindowsApi(WindowsApiError::AccessDenied));
            }
            return Err(TweakError::WindowsApi(WindowsApiError::ExecutionFailed(err)));
        }
        Err(other) => return Err(TweakError::WindowsApi(other)),
    };
    let last_line = output
        .lines()
        .map(str::trim)
        .rfind(|line| !line.is_empty())
        .unwrap_or("");

    if last_line.eq_ignore_ascii_case("True") {
        Ok(true)
    } else if last_line.eq_ignore_ascii_case("False") {
        Ok(false)
    } else {
        Err(TweakError::ValidationFailed(format!(
            "Saída inesperada ao consultar Get-MMAgent: '{}'",
            output.trim()
        )))
    }
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
