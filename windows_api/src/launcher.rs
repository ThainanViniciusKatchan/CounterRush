use std::process::Command;
use crate::WindowsApiError;

pub fn launch_cs2() -> Result<(), WindowsApiError> {
    let primary_status = Command::new("cmd")
        .args(["/c", "start steam://rungameid/730"])
        .status();

    match primary_status {
        Ok(status) if status.success() => Ok(()),
        _ => {
            let fallback_status = Command::new(r"C:\Program Files (x86)\Steam\steam.exe")
                .arg("-applaunch")
                .arg("730")
                .status();

            match fallback_status {
                Ok(status) if status.success() => Ok(()),
                Ok(status) => Err(WindowsApiError::ExecutionFailed(format!(
                    "Fallback retornou código de saída: {:?}",
                    status.code()
                ))),
                Err(err) => Err(WindowsApiError::Io(err)),
            }
        }
    }
}
