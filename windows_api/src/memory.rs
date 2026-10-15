use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::null_mut;
use crate::WindowsApiError;

type HANDLE = *mut c_void;
type BOOL = i32;

const FALSE: BOOL = 0;
const TOKEN_ADJUST_PRIVILEGES: u32 = 0x0020;
const TOKEN_QUERY: u32 = 0x0008;
const SE_PRIVILEGE_ENABLED: u32 = 0x0000_0002;
const ERROR_NOT_ALL_ASSIGNED: u32 = 1300;
const SYSTEM_MEMORY_LIST_INFORMATION: i32 = 80;
const MEMORY_PURGE_STANDBY_LIST: i32 = 4;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Luid {
    low_part: u32,
    high_part: i32,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct LuidAndAttributes {
    luid: Luid,
    attributes: u32,
}

#[repr(C)]
struct TokenPrivileges {
    privilege_count: u32,
    privileges: [LuidAndAttributes; 1],
}

#[link(name = "advapi32")]
extern "system" {
    fn OpenProcessToken(
        process_handle: HANDLE,
        desired_access: u32,
        token_handle: *mut HANDLE,
    ) -> BOOL;

    fn LookupPrivilegeValueW(
        system_name: *const u16,
        name: *const u16,
        luid: *mut Luid,
    ) -> BOOL;

    fn AdjustTokenPrivileges(
        token_handle: HANDLE,
        disable_all_privileges: BOOL,
        new_state: *const TokenPrivileges,
        buffer_length: u32,
        previous_state: *mut c_void,
        return_length: *mut u32,
    ) -> BOOL;
}

#[link(name = "kernel32")]
extern "system" {
    fn GetCurrentProcess() -> HANDLE;
    fn CloseHandle(handle: HANDLE) -> BOOL;
    fn GetLastError() -> u32;
    fn SetLastError(dw_err_code: u32);
}

#[link(name = "ntdll")]
extern "system" {
    fn NtSetSystemInformation(
        system_information_class: i32,
        system_information: *const c_void,
        system_information_length: u32,
    ) -> i32;
}

struct AutoCloseHandle(HANDLE);

impl Drop for AutoCloseHandle {
    fn drop(&mut self) {
        if !self.0.is_null() && self.0 != -1isize as HANDLE {
            unsafe {
                CloseHandle(self.0);
            }
        }
    }
}

pub fn enable_profile_privilege() -> Result<(), WindowsApiError> {
    unsafe {
        let mut token: HANDLE = null_mut();
        if OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token,
        ) == FALSE
        {
            return Err(WindowsApiError::AccessDenied);
        }

        let _guard = AutoCloseHandle(token);

        let priv_name: Vec<u16> = "SeProfileSingleProcessPrivilege\0"
            .encode_utf16()
            .collect();
        let mut luid = Luid::default();

        if LookupPrivilegeValueW(null_mut(), priv_name.as_ptr(), &mut luid) == FALSE {
            return Err(WindowsApiError::ExecutionFailed(format!(
                "Falha ao localizar privilégio SeProfileSingleProcessPrivilege: código {}",
                GetLastError()
            )));
        }

        let tp = TokenPrivileges {
            privilege_count: 1,
            privileges: [LuidAndAttributes {
                luid,
                attributes: SE_PRIVILEGE_ENABLED,
            }],
        };

        SetLastError(0);
        let adjusted = AdjustTokenPrivileges(
            token,
            FALSE,
            &tp,
            size_of::<TokenPrivileges>() as u32,
            null_mut(),
            null_mut(),
        );

        let last_error = GetLastError();
        if adjusted == FALSE || last_error == ERROR_NOT_ALL_ASSIGNED {
            return Err(WindowsApiError::AccessDenied);
        }

        Ok(())
    }
}

pub fn empty_standby_list() -> Result<(), WindowsApiError> {
    enable_profile_privilege()?;

    let command: i32 = MEMORY_PURGE_STANDBY_LIST;
    let status = unsafe {
        NtSetSystemInformation(
            SYSTEM_MEMORY_LIST_INFORMATION,
            &command as *const i32 as *const c_void,
            size_of::<i32>() as u32,
        )
    };

    if status != 0 {
        return Err(WindowsApiError::ExecutionFailed(format!(
            "NtSetSystemInformation falhou com NTSTATUS 0x{:08X}",
            status as u32
        )));
    }

    Ok(())
}
