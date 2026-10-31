use std::process::Command;
#[cfg(windows)]
use std::os::windows::process::CommandExt;
use crate::WindowsApiError;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub fn run_powershell(script: &str) -> Result<String, WindowsApiError> {
    let mut cmd = Command::new("powershell");
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd
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

fn base64_encode(bytes: &[u8]) -> String {
    const CHARSET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len() * 4 / 3 + 4);
    let mut chunks = bytes.chunks_exact(3);
    for chunk in chunks.by_ref() {
        let b = ((chunk[0] as u32) << 16) | ((chunk[1] as u32) << 8) | (chunk[2] as u32);
        out.push(CHARSET[((b >> 18) & 63) as usize] as char);
        out.push(CHARSET[((b >> 12) & 63) as usize] as char);
        out.push(CHARSET[((b >> 6) & 63) as usize] as char);
        out.push(CHARSET[(b & 63) as usize] as char);
    }
    let rem = chunks.remainder();
    if rem.len() == 1 {
        let b = (rem[0] as u32) << 16;
        out.push(CHARSET[((b >> 18) & 63) as usize] as char);
        out.push(CHARSET[((b >> 12) & 63) as usize] as char);
        out.push('=');
        out.push('=');
    } else if rem.len() == 2 {
        let b = ((rem[0] as u32) << 16) | ((rem[1] as u32) << 8);
        out.push(CHARSET[((b >> 18) & 63) as usize] as char);
        out.push(CHARSET[((b >> 12) & 63) as usize] as char);
        out.push(CHARSET[((b >> 6) & 63) as usize] as char);
        out.push('=');
    }
    out
}

pub fn run_powershell_script(script: &str) -> Result<String, WindowsApiError> {
    let utf16_bytes: Vec<u8> = script.encode_utf16().flat_map(|u| u.to_le_bytes()).collect();
    let encoded = base64_encode(&utf16_bytes);
    let mut cmd = Command::new("powershell");
    #[cfg(windows)]
    cmd.creation_flags(CREATE_NO_WINDOW);

    let output = cmd
        .args(["-NoProfile", "-EncodedCommand", &encoded])
        .output()
        .map_err(WindowsApiError::Io)?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        Err(WindowsApiError::ExecutionFailed(stderr))
    }
}

