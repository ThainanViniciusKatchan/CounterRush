use crate::{Tweak, TweakError, TweakStatus};
use windows_api::run_powershell;

pub struct Cs2AffinityTweak;

impl Tweak for Cs2AffinityTweak {
    fn id(&self) -> &'static str {
        "cs2_affinity"
    }

    fn name(&self) -> &'static str {
        "Afinidade de Processador do CS2"
    }

    fn description(&self) -> &'static str {
        "Remove o Core 0 da afinidade do processo cs2.exe (-band -2) para evitar contenção de tarefas do sistema operacional."
    }

    fn status(&self) -> Result<TweakStatus, TweakError> {
        let script = "(Get-Process -Name 'cs2' -ErrorAction SilentlyContinue | Select-Object -First 1 | ForEach-Object { ($_.ProcessorAffinity.ToInt64() -band 1) }) -eq 0";
        match run_powershell(script) {
            Ok(output) if output.eq_ignore_ascii_case("True") => Ok(TweakStatus::Applied),
            Ok(output) if output.eq_ignore_ascii_case("False") => Ok(TweakStatus::Default),
            _ => Ok(TweakStatus::Unknown),
        }
    }

    fn apply(&self) -> Result<(), TweakError> {
        let physical_cores = windows_api::get_cpu_physical_cores();
        if physical_cores <= 4 {
            return Ok(());
        }

        windows_api::wait_for_process("cs2", std::time::Duration::from_secs(60))?;
        let script = "Get-Process -Name 'cs2' | ForEach-Object { $_.ProcessorAffinity = [IntPtr]($_.ProcessorAffinity.ToInt64() -band -2) }";
        run_powershell(script).map_err(|err| match err {
            windows_api::WindowsApiError::ExecutionFailed(msg)
                if msg.contains("Cannot find a process with the name 'cs2'")
                    || msg.contains("não é possível localizar um processo") =>
            {
                TweakError::ValidationFailed("O processo 'cs2' não está em execução.".to_string())
            }
            other => TweakError::WindowsApi(other),
        })?;
        Ok(())
    }

    fn rollback(&self) -> Result<(), TweakError> {
        Ok(())
    }
}
