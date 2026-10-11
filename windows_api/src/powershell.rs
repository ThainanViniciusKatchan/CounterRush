use std::process::Command;
use crate::WindowsApiError;

pub fn run_powershell(script: &str) -> Result<String, WindowsApiError> {
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", script])
        .output()
        .map_err(WindowsApiError::Io)?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(WindowsApiError::ExecutionFailed(stderr))
    }
}
