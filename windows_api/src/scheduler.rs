use std::path::Path;
use std::process::Command;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use crate::WindowsApiError;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub const DEFAULT_STARTUP_TASK_NAME: &str = "CounterRush_Startup";

#[cfg(windows)]
fn run_schtasks(args: &[&str]) -> Result<String, WindowsApiError> {
    let mut cmd = Command::new("schtasks");
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd.args(args).output().map_err(WindowsApiError::Io)?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        Ok(stdout)
    } else {
        let combined = format!("{} {}", stdout, stderr).to_lowercase();
        if combined.contains("acesso negado") || combined.contains("access is denied") {
            return Err(WindowsApiError::AccessDenied);
        }
        Err(WindowsApiError::ExecutionFailed(if !stderr.is_empty() { stderr } else { stdout }))
    }
}

#[cfg(not(windows))]
fn run_schtasks(_args: &[&str]) -> Result<String, WindowsApiError> {
    Ok(String::new())
}

pub fn create_logon_task(task_name: &str, exe_path: &Path) -> Result<(), WindowsApiError> {
    #[cfg(windows)]
    {
        let quoted_target = format!("\"{}\"", exe_path.display());
        run_schtasks(&[
            "/create",
            "/tn",
            task_name,
            "/tr",
            &quoted_target,
            "/sc",
            "ONLOGON",
            "/rl",
            "HIGHEST",
            "/f",
        ])?;
        Ok(())
    }
    #[cfg(not(windows))]
    {
        let _ = (task_name, exe_path);
        Ok(())
    }
}

fn is_not_found_message(msg: &str) -> bool {
    let lower = msg.to_lowercase();
    lower.contains("pode encontrar")
        || lower.contains("arquivo especificado")
        || lower.contains("cannot find")
        || lower.contains("could not be found")
        || lower.contains("não encontrada")
        || lower.contains("nao encontrada")
        || lower.contains("not found")
}

pub fn is_task_registered(task_name: &str) -> Result<bool, WindowsApiError> {
    #[cfg(windows)]
    {
        match run_schtasks(&["/query", "/tn", task_name]) {
            Ok(_) => Ok(true),
            Err(WindowsApiError::ExecutionFailed(msg)) => {
                if is_not_found_message(&msg) {
                    Ok(false)
                } else {
                    Err(WindowsApiError::ExecutionFailed(msg))
                }
            }
            Err(err) => Err(err),
        }
    }
    #[cfg(not(windows))]
    {
        let _ = task_name;
        Ok(false)
    }
}

pub fn delete_task(task_name: &str) -> Result<(), WindowsApiError> {
    #[cfg(windows)]
    {
        match run_schtasks(&["/delete", "/tn", task_name, "/f"]) {
            Ok(_) => Ok(()),
            Err(WindowsApiError::ExecutionFailed(msg)) => {
                if is_not_found_message(&msg) {
                    Ok(())
                } else {
                    Err(WindowsApiError::ExecutionFailed(msg))
                }
            }
            Err(err) => Err(err),
        }
    }
    #[cfg(not(windows))]
    {
        let _ = task_name;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_task_registered_non_existent() {
        let task_name = "CounterRush_NonExistent_Test_987654";
        let registered = is_task_registered(task_name).expect("Consulta de tarefa inexistente deve retornar Ok(false)");
        assert!(!registered);
    }

    #[test]
    fn test_delete_task_idempotent() {
        let task_name = "CounterRush_NonExistent_Delete_Test_123456";
        let res = delete_task(task_name);
        assert!(res.is_ok(), "Exclusão de tarefa inexistente deve ser tratada como idempotente: {:?}", res.err());
    }

    #[test]
    fn test_is_not_found_message_variations() {
        assert!(is_not_found_message("ERRO: O sistema não pode encontrar o arquivo especificado."));
        assert!(is_not_found_message("ERRO: O sistema no pode encontrar o arquivo especificado."));
        assert!(is_not_found_message("ERROR: The system cannot find the file specified."));
        assert!(is_not_found_message("ERROR: The specified task name could not be found."));
        assert!(is_not_found_message("Tarefa não encontrada no agendador."));
        assert!(is_not_found_message("Tarefa nao encontrada."));
        assert!(!is_not_found_message("ERRO: Acesso negado."));
        assert!(!is_not_found_message("ERROR: Access is denied."));
    }
}
