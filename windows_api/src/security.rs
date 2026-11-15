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
pub(crate) type HANDLE = *mut c_void;
#[cfg(windows)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) type BOOL = i32;

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

    fn ConvertSidToStringSidW(sid: *const c_void, string_sid: *mut *mut u16) -> BOOL;
}

#[cfg(windows)]
#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcess() -> HANDLE;
    pub(crate) fn CloseHandle(handle: HANDLE) -> BOOL;
    pub(crate) fn LocalFree(hMem: *mut c_void) -> *mut c_void;
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
pub(crate) struct AutoCloseHandle(pub(crate) HANDLE);

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
pub(crate) fn to_wide(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

/// Retorna o SID do usuário dono do token do processo atual (ex.: `"S-1-5-21-..."`).
#[cfg(windows)]
pub(crate) fn current_user_sid_string() -> Option<String> {
    const TOKEN_USER_CLASS: i32 = 1;

    unsafe {
        let mut token: HANDLE = null_mut();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) == FALSE {
            return None;
        }
        let _guard = AutoCloseHandle(token);

        let mut needed: u32 = 0;
        GetTokenInformation(token, TOKEN_USER_CLASS, null_mut(), 0, &mut needed);
        if needed == 0 {
            return None;
        }

        let mut buffer = vec![0u8; needed as usize];
        if GetTokenInformation(
            token,
            TOKEN_USER_CLASS,
            buffer.as_mut_ptr() as *mut c_void,
            needed,
            &mut needed,
        ) == FALSE
        {
            return None;
        }

        let sid_ptr = *(buffer.as_ptr() as *const *const c_void);
        if sid_ptr.is_null() {
            return None;
        }

        let mut wide_sid: *mut u16 = null_mut();
        if ConvertSidToStringSidW(sid_ptr, &mut wide_sid) == FALSE || wide_sid.is_null() {
            return None;
        }

        let mut len = 0usize;
        while *wide_sid.add(len) != 0 {
            len += 1;
        }
        let slice = std::slice::from_raw_parts(wide_sid, len);
        let sid_string = String::from_utf16_lossy(slice);
        LocalFree(wide_sid as *mut c_void);
        Some(sid_string)
    }
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

    #[cfg(windows)]
    #[test]
    fn test_current_user_sid_string() {
        let sid = current_user_sid_string().expect("Deve resolver o SID do usuário atual");
        assert!(sid.starts_with("S-1-"), "SID inesperado: {sid}");
    }
}
