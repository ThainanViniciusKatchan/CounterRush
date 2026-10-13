use std::process::Command;
use crate::WindowsApiError;

pub fn launch_cs2() -> Result<(), WindowsApiError> {
    let primary_status = Command::new("cmd")
        .args(["/c", "start steam://rungameid/730"])
        .status();

    match primary_status {
        Ok(status) if status.success() => Ok(()),
        _ => launch_cs2_with_args(&[]),
    }
}

pub fn launch_cs2_with_args(extra_args: &[&str]) -> Result<(), WindowsApiError> {
    let mut cmd = Command::new(r"C:\Program Files (x86)\Steam\steam.exe");
    cmd.arg("-applaunch").arg("730");
    for arg in extra_args {
        cmd.arg(arg);
    }

    match cmd.status() {
        Ok(status) if status.success() => Ok(()),
        Ok(status) => Err(WindowsApiError::ExecutionFailed(format!(
            "Steam retornou código de saída: {:?}",
            status.code()
        ))),
        Err(err) => {
            let mut full_cmd = format!(r#"start "" "C:\Program Files (x86)\Steam\steam.exe" -applaunch 730"#);
            for arg in extra_args {
                full_cmd.push(' ');
                full_cmd.push_str(arg);
            }
            let fallback_status = Command::new("cmd").args(["/c", &full_cmd]).status();
            match fallback_status {
                Ok(s) if s.success() => Ok(()),
                _ => Err(WindowsApiError::Io(err)),
            }
        }
    }
}

pub fn launch_cs2_economico() -> Result<(), WindowsApiError> {
    launch_cs2_with_args(&["-console", "-nojoy", "+fps_max", "120", "-vulkan"])
}

pub fn launch_cs2_intermediario() -> Result<(), WindowsApiError> {
    launch_cs2_with_args(&[
        "-fullscreen",
        "-console",
        "-nojoy",
        "+engine_low_latency_sleep_after_client_tick",
        "true",
        "+fps_max",
        "240",
        "+mat_queue_mode",
    ])
}

pub fn launch_cs2_potente() -> Result<(), WindowsApiError> {
    launch_cs2_with_args(&[
        "-console",
        "-nojoy",
        "-refresh",
        "240",
        "+fps_max",
        "400",
        "+engine_low_latency_sleep_after_client_tick",
        "true",
    ])
}
