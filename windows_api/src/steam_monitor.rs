use std::path::PathBuf;
use crate::WindowsApiError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SteamGameLaunchEvent {
    pub app_id: u32,
    pub pid: u32,
    pub exe_path: String,
    pub command_line: String,
}

#[cfg(windows)]
mod win32_impl {
    use super::*;
    use std::ffi::{c_void, OsStr};
    use std::fs::OpenOptions;
    use std::io::{Read, Seek, SeekFrom};
    use std::os::windows::ffi::OsStrExt;
    use std::os::windows::fs::OpenOptionsExt;

    type HANDLE = *mut c_void;
    type HKEY = isize;
    type BOOL = i32;
    type DWORD = u32;

    const HKEY_CURRENT_USER: HKEY = 0x80000001u32 as i32 as isize;
    const KEY_READ: DWORD = 0x20019;
    const KEY_NOTIFY: DWORD = 0x0010;
    const REG_NOTIFY_CHANGE_NAME: DWORD = 0x00000001;
    const REG_NOTIFY_CHANGE_LAST_SET: DWORD = 0x00000004;

    const TH32CS_SNAPPROCESS: DWORD = 0x00000002;
    const INVALID_HANDLE_VALUE: isize = -1;
    const PROCESS_QUERY_LIMITED_INFORMATION: DWORD = 0x1000;
    const SYNCHRONIZE: DWORD = 0x00100000;
    const STILL_ACTIVE: DWORD = 259;
    const WAIT_OBJECT_0: DWORD = 0;
    const WAIT_TIMEOUT: DWORD = 0x00000102;
    const ERROR_SUCCESS: i32 = 0;

    const FILE_SHARE_READ: u32 = 0x00000001;
    const FILE_SHARE_WRITE: u32 = 0x00000002;
    const FILE_SHARE_DELETE: u32 = 0x00000004;

    #[repr(C)]
    #[allow(non_snake_case)]
    struct PROCESSENTRY32W {
        dwSize: DWORD,
        cntUsage: DWORD,
        th32ProcessID: DWORD,
        th32DefaultHeapID: usize,
        th32ModuleID: DWORD,
        cntThreads: DWORD,
        th32ParentProcessID: DWORD,
        pcPriClassBase: i32,
        dwFlags: DWORD,
        szExeFile: [u16; 260],
    }

    #[link(name = "advapi32")]
    extern "system" {
        fn RegOpenKeyExW(
            hKey: HKEY,
            lpSubKey: *const u16,
            ulOptions: DWORD,
            samDesired: DWORD,
            phkResult: *mut HKEY,
        ) -> i32;

        fn RegQueryValueExW(
            hKey: HKEY,
            lpValueName: *const u16,
            lpReserved: *mut DWORD,
            lpType: *mut DWORD,
            lpData: *mut u8,
            lpcbData: *mut DWORD,
        ) -> i32;

        fn RegCloseKey(hKey: HKEY) -> i32;

        fn RegNotifyChangeKeyValue(
            hKey: HKEY,
            bWatchSubtree: BOOL,
            dwNotifyFilter: DWORD,
            hEvent: isize,
            fAsynchronous: BOOL,
        ) -> i32;
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateEventW(
            lpEventAttributes: *mut c_void,
            bManualReset: BOOL,
            bInitialState: BOOL,
            lpName: *const u16,
        ) -> isize;

        fn WaitForSingleObject(hHandle: isize, dwMilliseconds: DWORD) -> DWORD;

        fn CloseHandle(hObject: HANDLE) -> BOOL;

        fn CreateToolhelp32Snapshot(dwFlags: DWORD, th32ProcessID: DWORD) -> isize;

        fn Process32FirstW(hSnapshot: isize, lppe: *mut PROCESSENTRY32W) -> BOOL;

        fn Process32NextW(hSnapshot: isize, lppe: *mut PROCESSENTRY32W) -> BOOL;

        fn OpenProcess(dwDesiredAccess: DWORD, bInheritHandle: BOOL, dwProcessId: DWORD) -> HANDLE;

        fn GetExitCodeProcess(hProcess: HANDLE, lpExitCode: *mut DWORD) -> BOOL;
    }

    #[link(name = "user32")]
    extern "system" {
        fn GetForegroundWindow() -> isize;
        fn GetWindowThreadProcessId(hWnd: isize, lpdwProcessId: *mut DWORD) -> DWORD;
    }

    fn to_wide_chars(s: &str) -> Vec<u16> {
        OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
    }

    pub fn get_steam_running_app_id() -> Result<u32, WindowsApiError> {
        let subkey = to_wide_chars(r"Software\Valve\Steam");
        let val_name = to_wide_chars("RunningAppID");

        unsafe {
            let mut hkey: HKEY = 0;
            let status = RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_READ, &mut hkey);
            if status != ERROR_SUCCESS {
                return Err(WindowsApiError::RegistryNotFound(r"HKCU\Software\Valve\Steam".into()));
            }

            let mut data: u32 = 0;
            let mut data_len = std::mem::size_of::<u32>() as u32;
            let mut val_type: u32 = 0;

            let query_status = RegQueryValueExW(
                hkey,
                val_name.as_ptr(),
                std::ptr::null_mut(),
                &mut val_type,
                &mut data as *mut u32 as *mut u8,
                &mut data_len,
            );
            RegCloseKey(hkey);

            if query_status != ERROR_SUCCESS {
                return Err(WindowsApiError::RegistryNotFound("RunningAppID".into()));
            }

            Ok(data)
        }
    }

    pub fn is_steam_app_running_in_registry(app_id: u32) -> Result<bool, WindowsApiError> {
        if let Ok(running_id) = get_steam_running_app_id() {
            if running_id == app_id {
                return Ok(true);
            }
        }

        let subkey_str = format!(r"Software\Valve\Steam\Apps\{}", app_id);
        let subkey = to_wide_chars(&subkey_str);
        let val_name = to_wide_chars("Running");

        unsafe {
            let mut hkey: HKEY = 0;
            let status = RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_READ, &mut hkey);
            if status != ERROR_SUCCESS {
                return Ok(false);
            }

            let mut data: u32 = 0;
            let mut data_len = std::mem::size_of::<u32>() as u32;
            let mut val_type: u32 = 0;

            let query_status = RegQueryValueExW(
                hkey,
                val_name.as_ptr(),
                std::ptr::null_mut(),
                &mut val_type,
                &mut data as *mut u32 as *mut u8,
                &mut data_len,
            );
            RegCloseKey(hkey);

            if query_status == ERROR_SUCCESS && data == 1 {
                return Ok(true);
            }
        }

        Ok(false)
    }

    pub fn resolve_steam_install_path_from_registry() -> Result<PathBuf, WindowsApiError> {
        let subkey = to_wide_chars(r"Software\Valve\Steam");
        let val_name = to_wide_chars("SteamPath");

        unsafe {
            let mut hkey: HKEY = 0;
            let status = RegOpenKeyExW(HKEY_CURRENT_USER, subkey.as_ptr(), 0, KEY_READ, &mut hkey);
            if status == ERROR_SUCCESS {
                let mut buf = [0u16; 512];
                let mut data_len = (buf.len() * std::mem::size_of::<u16>()) as u32;
                let mut val_type: u32 = 0;

                let query_status = RegQueryValueExW(
                    hkey,
                    val_name.as_ptr(),
                    std::ptr::null_mut(),
                    &mut val_type,
                    buf.as_mut_ptr() as *mut u8,
                    &mut data_len,
                );
                RegCloseKey(hkey);

                if query_status == ERROR_SUCCESS {
                    let u16_len = (data_len as usize) / std::mem::size_of::<u16>();
                    let slice = &buf[..u16_len];
                    let end = slice.iter().position(|&c| c == 0).unwrap_or(slice.len());
                    let path_str = String::from_utf16_lossy(&slice[..end]);
                    let normalized = path_str.replace('/', "\\");
                    let p = PathBuf::from(normalized);
                    if p.exists() {
                        return Ok(p);
                    }
                }
            }
        }

        let default_path = PathBuf::from(r"C:\Program Files (x86)\Steam");
        if default_path.exists() {
            Ok(default_path)
        } else {
            Err(WindowsApiError::RegistryNotFound(r"HKCU\Software\Valve\Steam\SteamPath".into()))
        }
    }

    pub struct RegistryWatcher {
        h_key: HKEY,
        h_event: isize,
    }

    impl RegistryWatcher {
        pub fn new_steam_watcher() -> Result<Self, WindowsApiError> {
            let subkey = to_wide_chars(r"Software\Valve\Steam");
            unsafe {
                let mut h_key: HKEY = 0;
                let status = RegOpenKeyExW(
                    HKEY_CURRENT_USER,
                    subkey.as_ptr(),
                    0,
                    KEY_READ | KEY_NOTIFY,
                    &mut h_key,
                );
                if status != ERROR_SUCCESS {
                    return Err(WindowsApiError::RegistryNotFound(r"HKCU\Software\Valve\Steam".into()));
                }

                let h_event = CreateEventW(std::ptr::null_mut(), 0, 0, std::ptr::null());
                if h_event == 0 {
                    RegCloseKey(h_key);
                    return Err(WindowsApiError::ExecutionFailed("Falha ao criar Win32 Event".into()));
                }

                let filter = REG_NOTIFY_CHANGE_NAME | REG_NOTIFY_CHANGE_LAST_SET;
                let reg_res = RegNotifyChangeKeyValue(h_key, 1, filter, h_event, 1);
                if reg_res != ERROR_SUCCESS {
                    CloseHandle(h_event as HANDLE);
                    RegCloseKey(h_key);
                    return Err(WindowsApiError::ExecutionFailed(format!("RegNotifyChangeKeyValue falhou: {reg_res}")));
                }

                Ok(Self { h_key, h_event })
            }
        }

        pub fn rearm(&self) -> Result<(), WindowsApiError> {
            unsafe {
                let filter = REG_NOTIFY_CHANGE_NAME | REG_NOTIFY_CHANGE_LAST_SET;
                let reg_res = RegNotifyChangeKeyValue(self.h_key, 1, filter, self.h_event, 1);
                if reg_res != ERROR_SUCCESS {
                    return Err(WindowsApiError::ExecutionFailed(format!("RegNotifyChangeKeyValue rearm falhou: {reg_res}")));
                }
                Ok(())
            }
        }

        pub fn wait_for_change(&self, timeout_ms: u32) -> Result<bool, WindowsApiError> {
            unsafe {
                let wait_res = WaitForSingleObject(self.h_event, timeout_ms);
                if wait_res == WAIT_OBJECT_0 {
                    let _ = self.rearm();
                    Ok(true)
                } else if wait_res == WAIT_TIMEOUT {
                    Ok(false)
                } else {
                    Err(WindowsApiError::ExecutionFailed(format!("WaitForSingleObject falhou: {wait_res}")))
                }
            }
        }
    }

    impl Drop for RegistryWatcher {
        fn drop(&mut self) {
            unsafe {
                if self.h_event != 0 {
                    CloseHandle(self.h_event as HANDLE);
                    self.h_event = 0;
                }
                if self.h_key != 0 {
                    RegCloseKey(self.h_key);
                    self.h_key = 0;
                }
            }
        }
    }

    unsafe impl Send for RegistryWatcher {}
    unsafe impl Sync for RegistryWatcher {}

    pub struct SteamGameProcessLogWatcher {
        log_path: PathBuf,
        last_offset: u64,
    }

    impl SteamGameProcessLogWatcher {
        pub fn new(log_path: PathBuf) -> Self {
            let last_offset = if let Ok(meta) = std::fs::metadata(&log_path) {
                meta.len()
            } else {
                0
            };
            Self { log_path, last_offset }
        }

        pub fn with_initial_offset(log_path: PathBuf, last_offset: u64) -> Self {
            Self { log_path, last_offset }
        }

        pub fn read_new_lines(&mut self) -> Result<Vec<String>, WindowsApiError> {
            if !self.log_path.exists() {
                return Ok(Vec::new());
            }

            let mut file = OpenOptions::new()
                .read(true)
                .share_mode(FILE_SHARE_READ | FILE_SHARE_WRITE | FILE_SHARE_DELETE)
                .open(&self.log_path)?;

            let current_len = file.metadata()?.len();
            if current_len < self.last_offset {
                self.last_offset = 0;
            }

            if current_len == self.last_offset {
                return Ok(Vec::new());
            }

            file.seek(SeekFrom::Start(self.last_offset))?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;
            self.last_offset = current_len;

            let content = String::from_utf8_lossy(&buffer);
            let lines = content
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect();

            Ok(lines)
        }

        pub fn poll_new_launch_events(&mut self, target_app_id: u32) -> Result<Vec<SteamGameLaunchEvent>, WindowsApiError> {
            let lines = self.read_new_lines()?;
            let mut events = Vec::new();
            let prefix = format!("AppID {} adding PID ", target_app_id);

            for line in lines {
                if let Some(idx) = line.find(&prefix) {
                    let rest = &line[idx + prefix.len()..];
                    if let Some((pid_str, after_pid)) = rest.split_once(' ') {
                        if let Ok(pid) = pid_str.parse::<u32>() {
                            let mut exe_path = String::new();
                            let mut command_line = String::new();

                            if let Some(proc_idx) = after_pid.find("as a tracked process ") {
                                let cmd_part = after_pid[proc_idx + "as a tracked process ".len()..].trim();
                                command_line = cmd_part.to_string();
                                let trimmed_cmd = cmd_part.trim_matches('"');
                                if let Some((first, _)) = trimmed_cmd.split_once('"') {
                                    exe_path = first.to_string();
                                } else if let Some((first, _)) = trimmed_cmd.split_once(' ') {
                                    exe_path = first.to_string();
                                } else {
                                    exe_path = trimmed_cmd.to_string();
                                }
                            }

                            events.push(SteamGameLaunchEvent {
                                app_id: target_app_id,
                                pid,
                                exe_path,
                                command_line,
                            });
                        }
                    }
                }
            }

            Ok(events)
        }

        pub fn has_stop_event(&mut self, target_app_id: u32, target_pid: Option<u32>) -> Result<bool, WindowsApiError> {
            let lines = self.read_new_lines()?;
            let remove_pattern = format!("Remove {} from running list", target_app_id);
            let no_longer_pattern = format!("AppID {} no longer tracking PID", target_app_id);

            for line in lines {
                if line.contains(&remove_pattern) {
                    return Ok(true);
                }
                if line.contains(&no_longer_pattern) {
                    if let Some(pid) = target_pid {
                        let pid_pattern = format!("PID {}", pid);
                        if line.contains(&pid_pattern) {
                            return Ok(true);
                        }
                    } else {
                        return Ok(true);
                    }
                }
            }

            Ok(false)
        }
    }

    pub fn get_process_id_by_name(exe_name: &str) -> Result<Option<u32>, WindowsApiError> {
        let target = if exe_name.to_lowercase().ends_with(".exe") {
            exe_name.to_lowercase()
        } else {
            format!("{}.exe", exe_name).to_lowercase()
        };

        unsafe {
            let snapshot = CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0);
            if snapshot == INVALID_HANDLE_VALUE {
                return Err(WindowsApiError::ExecutionFailed("Falha ao criar Toolhelp32Snapshot".into()));
            }

            let mut entry: PROCESSENTRY32W = std::mem::zeroed();
            entry.dwSize = std::mem::size_of::<PROCESSENTRY32W>() as u32;

            if Process32FirstW(snapshot, &mut entry) == 0 {
                CloseHandle(snapshot as HANDLE);
                return Ok(None);
            }

            loop {
                let name_len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(entry.szExeFile.len());
                let current_name = String::from_utf16_lossy(&entry.szExeFile[..name_len]).to_lowercase();

                if current_name == target {
                    let pid = entry.th32ProcessID;
                    CloseHandle(snapshot as HANDLE);
                    return Ok(Some(pid));
                }

                if Process32NextW(snapshot, &mut entry) == 0 {
                    break;
                }
            }

            CloseHandle(snapshot as HANDLE);
            Ok(None)
        }
    }

    pub fn is_cs2_process_running_win32() -> Result<bool, WindowsApiError> {
        Ok(get_process_id_by_name("cs2.exe")?.is_some())
    }

    pub fn is_pid_alive(pid: u32) -> bool {
        if pid == 0 {
            return false;
        }
        unsafe {
            let handle = OpenProcess(SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if !handle.is_null() && handle as isize != INVALID_HANDLE_VALUE {
                let wait_res = WaitForSingleObject(handle as isize, 0);
                CloseHandle(handle);
                return wait_res == WAIT_TIMEOUT;
            }

            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if !handle.is_null() && handle as isize != INVALID_HANDLE_VALUE {
                let mut exit_code: DWORD = 0;
                let ok = GetExitCodeProcess(handle, &mut exit_code);
                CloseHandle(handle);
                return ok != 0 && exit_code == STILL_ACTIVE;
            }

            false
        }
    }

    pub fn get_foreground_process_id() -> Option<u32> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd == 0 {
                return None;
            }
            let mut pid: DWORD = 0;
            GetWindowThreadProcessId(hwnd, &mut pid);
            if pid == 0 {
                None
            } else {
                Some(pid)
            }
        }
    }
}

#[cfg(windows)]
pub use win32_impl::*;

#[cfg(not(windows))]
mod fallback_impl {
    use super::*;

    pub fn get_steam_running_app_id() -> Result<u32, WindowsApiError> {
        Ok(0)
    }

    pub fn is_steam_app_running_in_registry(_app_id: u32) -> Result<bool, WindowsApiError> {
        Ok(false)
    }

    pub fn resolve_steam_install_path_from_registry() -> Result<PathBuf, WindowsApiError> {
        Err(WindowsApiError::RegistryNotFound("Suporte apenas no Windows".into()))
    }

    pub struct RegistryWatcher;

    impl RegistryWatcher {
        pub fn new_steam_watcher() -> Result<Self, WindowsApiError> {
            Ok(Self)
        }
        pub fn wait_for_change(&self, _timeout_ms: u32) -> Result<bool, WindowsApiError> {
            std::thread::sleep(std::time::Duration::from_millis(_timeout_ms as u64));
            Ok(false)
        }
        pub fn rearm(&self) -> Result<(), WindowsApiError> {
            Ok(())
        }
    }

    pub struct SteamGameProcessLogWatcher;

    impl SteamGameProcessLogWatcher {
        pub fn new(_log_path: PathBuf) -> Self {
            Self
        }
        pub fn with_initial_offset(_log_path: PathBuf, _last_offset: u64) -> Self {
            Self
        }
        pub fn poll_new_launch_events(&mut self, _target_app_id: u32) -> Result<Vec<SteamGameLaunchEvent>, WindowsApiError> {
            Ok(Vec::new())
        }
        pub fn has_stop_event(&mut self, _target_app_id: u32, _target_pid: Option<u32>) -> Result<bool, WindowsApiError> {
            Ok(false)
        }
    }

    pub fn get_process_id_by_name(_exe_name: &str) -> Result<Option<u32>, WindowsApiError> {
        Ok(None)
    }

    pub fn is_cs2_process_running_win32() -> Result<bool, WindowsApiError> {
        Ok(false)
    }

    pub fn is_pid_alive(_pid: u32) -> bool {
        false
    }

    pub fn get_foreground_process_id() -> Option<u32> {
        None
    }
}

#[cfg(not(windows))]
pub use fallback_impl::*;
