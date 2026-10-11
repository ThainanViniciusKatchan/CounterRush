use std::thread::sleep;
use std::time::{Duration, Instant};
use crate::run_powershell;
use crate::WindowsApiError;

pub fn is_process_running(process_name: &str) -> Result<bool, WindowsApiError> {
    let script = format!("(Get-Process -Name '{}' -ErrorAction SilentlyContinue) -ne $null", process_name);
    let output = run_powershell(&script)?;
    Ok(output.eq_ignore_ascii_case("True"))
}

pub fn wait_for_process(process_name: &str, timeout: Duration) -> Result<(), WindowsApiError> {
    let start = Instant::now();
    let poll_interval = Duration::from_millis(1000);

    while start.elapsed() < timeout {
        if is_process_running(process_name)? {
            return Ok(());
        }
        sleep(poll_interval);
    }

    Err(WindowsApiError::ExecutionFailed(format!(
        "Tempo limite de {} segundos esgotado aguardando o processo '{}'.",
        timeout.as_secs(),
        process_name
    )))
}
