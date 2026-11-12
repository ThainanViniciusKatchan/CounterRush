use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::null_mut;
use crate::WindowsApiError;

#[cfg(windows)]
use std::ffi::OsStr;
#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

#[cfg(windows)]
#[allow(clippy::upper_case_acronyms)]
type HANDLE = *mut c_void;
#[cfg(windows)]
#[allow(clippy::upper_case_acronyms)]
type BOOL = i32;

#[cfg(windows)]
const FALSE: BOOL = 0;
#[cfg(windows)]
const TOKEN_QUERY: u32 = 0x0008;
#[cfg(windows)]
const SW_SHOWNORMAL: i32 = 1;

#[cfg(windows)]
#[repr(C)]
struct TokenElevation {
    token_is_elevated: u32,
}

#[cfg(windows)]
#[link(name = "advapi32")]
extern "system" {
    fn OpenProcessToken(
        process_handle: HANDLE,
        desired_access: u32,
        token_handle: *mut HANDLE,
    ) -> BOOL;

    fn GetTokenInformation(
        token_handle: HANDLE,
        token_information_class: i32,
        token_information: *mut c_void,
        token_information_length: u32,
        return_length: *mut u32,
    ) -> BOOL;
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcess() -> HANDLE;
    fn CloseHandle(handle: HANDLE) -> BOOL;
}

#[cfg(windows)]
#[link(name = "shell32")]
extern "system" {
    fn ShellExecuteW(
        hwnd: *mut c_void,
        lpOperation: *const u16,
        lpFile: *const u16,
        lpParameters: *const u16,
        lpDirectory: *const u16,
        nShowCmd: i32,
    ) -> isize;
}

#[cfg(windows)]
struct AutoCloseHandle(HANDLE);

#[cfg(windows)]
impl Drop for AutoCloseHandle {
    fn drop(&mut self) {
        if !self.0.is_null() && self.0 != -1isize as HANDLE {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}

#[cfg(windows)]
fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

/// Verifica se o processo atual está executando com privilégios de Administrador (Token elevado).
#[cfg(windows)]
pub fn is_elevated() -> bool {
    unsafe {
        let mut token: HANDLE = null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == FALSE {
            return false;
        }
        let _guard = AutoCloseHandle(token);

        let mut elevation = TokenElevation { token_is_elevated: 0 };
        let mut return_length: u32 = 0;
        let success = GetTokenInformation(
            token,
            20, // TokenElevation (TOKEN_INFORMATION_CLASS)
            &mut elevation as *mut _ as *mut c_void,
            size_of::<TokenElevation>() as u32,
            &mut return_length,
        );

        if success != FALSE {
            elevation.token_is_elevated != 0
        } else {
            false
        }
    }
}

#[cfg(not(windows))]
pub fn is_elevated() -> bool {
    true
}

/// Reinicia o executável atual solicitando elevação UAC ("runas") e repassando argumentos.
#[cfg(windows)]
pub fn relaunch_as_admin() -> Result<(), WindowsApiError> {
    let current_exe = std::env::current_exe().map_err(WindowsApiError::Io)?;
    let exe_str = current_exe.to_string_lossy();
    let clean_exe = exe_str.strip_prefix(r"\\?\").unwrap_or(&exe_str);

    let args: Vec<String> = std::env::args().skip(1).collect();
    let params_str = if args.is_empty() {
        None
    } else {
        Some(args.join(" "))
    };

    let operation = to_wide("runas");
    let file_wide = to_wide(clean_exe);
    let params_wide = params_str.as_deref().map(to_wide);

    let params_ptr = match &params_wide {
        Some(w) => w.as_ptr(),
        None => std::ptr::null(),
    };

    let res = unsafe {
        ShellExecuteW(
            null_mut(),
            operation.as_ptr(),
            file_wide.as_ptr(),
            params_ptr,
            null_mut(),
            SW_SHOWNORMAL,
        )
    };

    if res > 32 {
        Ok(())
    } else {
        Err(WindowsApiError::ExecutionFailed(format!(
            "ShellExecuteW 'runas' falhou com código de retorno: {res}"
        )))
    }
}

#[cfg(not(windows))]
pub fn relaunch_as_admin() -> Result<(), WindowsApiError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_elevated_callable() {
        // Confirma que a chamada ao Win32 TokenElevation retorna sem erro/pânico
        let _ = is_elevated();
    }
}
