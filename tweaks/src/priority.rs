use crate::{Tweak, TweakError, TweakStatus};
use windows_api::run_powershell;

pub struct Cs2PriorityTweak;

impl Tweak for Cs2PriorityTweak {
    fn id(&self) -> &'static str {
        "cs2_priority"
    }

    fn name(&self) -> &'static str {
        "Prioridade em Tempo Real do CS2"
    }

    fn description(&self) -> &'static str {
        "Define a classe de prioridade do processo cs2.exe para RealTime."
    }

    fn status(&self) -> Result<TweakStatus, TweakError> {
        let script = "(Get-Process -Name 'cs2').PriorityClass";
        match run_powershell(script) {
            Ok(output) if output.eq_ignore_ascii_case("RealTime") => Ok(TweakStatus::Applied),
            Ok(_) => Ok(TweakStatus::Default),
            Err(_) => Ok(TweakStatus::Unknown),
        }
    }

    fn apply(&self) -> Result<(), TweakError> {
        windows_api::wait_for_process("cs2", std::time::Duration::from_secs(60))?;
        let script = "(Get-Process -Name 'cs2').PriorityClass = 'RealTime'";
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
