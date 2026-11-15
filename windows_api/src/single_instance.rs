use std::sync::Arc;
use std::thread::JoinHandle;

use crate::WindowsApiError;

pub const DEFAULT_SINGLE_INSTANCE_ID: &str = "CounterRush_SingleInstance";

pub enum SingleInstanceOutcome {
    Primary(SingleInstanceGuard),
    SecondaryNotified,
}

#[cfg(windows)]
mod win32_impl {
    use super::*;
    use std::ffi::c_void;
    use std::mem::size_of;
    use std::ptr::null_mut;
    use std::thread;
    use std::time::Duration;

    use crate::security::{current_user_sid_string, to_wide, AutoCloseHandle, LocalFree, BOOL, HANDLE};

    const FALSE: BOOL = 0;
    const ERROR_ACCESS_DENIED: u32 = 5;
    const ERROR_FILE_NOT_FOUND: u32 = 2;
    const ERROR_ALREADY_EXISTS: u32 = 183;
    const SYNCHRONIZE: u32 = 0x0010_0000;
    const EVENT_MODIFY_STATE: u32 = 0x0002;
    const SDDL_REVISION_1: u32 = 1;
    const WAIT_OBJECT_0: u32 = 0;
    const INFINITE: u32 = 0xFFFF_FFFF;
    const ASFW_ANY: u32 = u32::MAX;

    #[repr(C)]
    struct SecurityAttributes {
        n_length: u32,
        lp_security_descriptor: *mut c_void,
        b_inherit_handle: BOOL,
    }

    #[link(name = "kernel32")]
    extern "system" {
        fn CreateMutexW(
            lp_mutex_attributes: *mut c_void,
            b_initial_owner: BOOL,
            lp_name: *const u16,
        ) -> HANDLE;
        fn OpenMutexW(dw_desired_access: u32, b_inherit_handle: BOOL, lp_name: *const u16) -> HANDLE;
        fn CreateEventW(
            lp_event_attributes: *mut c_void,
            b_manual_reset: BOOL,
            b_initial_state: BOOL,
            lp_name: *const u16,
        ) -> isize;
        fn OpenEventW(dw_desired_access: u32, b_inherit_handle: BOOL, lp_name: *const u16) -> HANDLE;
        fn SetEvent(h_event: HANDLE) -> BOOL;
        fn WaitForMultipleObjects(
            n_count: u32,
            lp_handles: *const HANDLE,
            b_wait_all: BOOL,
            dw_milliseconds: u32,
        ) -> u32;
        fn GetLastError() -> u32;
    }

    #[link(name = "advapi32")]
    extern "system" {
        fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
            string_security_descriptor: *const u16,
            string_sd_revision: u32,
            security_descriptor: *mut *mut c_void,
            security_descriptor_size: *mut u32,
        ) -> BOOL;
    }

    #[link(name = "user32")]
    extern "system" {
        fn AllowSetForegroundWindow(dw_process_id: u32) -> BOOL;
    }

    struct SendHandle(AutoCloseHandle);

    unsafe impl Send for SendHandle {}
    unsafe impl Sync for SendHandle {}

    impl SendHandle {
        fn new(handle: HANDLE) -> Self {
            Self(AutoCloseHandle(handle))
        }

        fn raw(&self) -> HANDLE {
            self.0 .0
        }
    }

    struct SecurityDescriptorGuard(*mut c_void);

    impl Drop for SecurityDescriptorGuard {
        fn drop(&mut self) {
            if !self.0.is_null() {
                unsafe {
                    LocalFree(self.0);
                }
            }
        }
    }

    fn build_user_security_descriptor() -> Option<SecurityDescriptorGuard> {
        let user_sid = current_user_sid_string()?;
        // Inclui o SID do usuário atual (compartilhado entre token Médio e Elevado)
        // com Low Mandatory Label para sinalização entre níveis sem erro de acesso.
        let sddl = format!("D:(A;;GA;;;{user_sid})(A;;GA;;;BA)(A;;GA;;;SY)S:(ML;;NW;;;LW)");
        let wide = to_wide(&sddl);
        let mut sd_ptr: *mut c_void = null_mut();
        let ok = unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                wide.as_ptr(),
                SDDL_REVISION_1,
                &mut sd_ptr,
                null_mut(),
            )
        };
        if ok != FALSE && !sd_ptr.is_null() {
            Some(SecurityDescriptorGuard(sd_ptr))
        } else {
            None
        }
    }

    fn sanitize_object_id(instance_id: &str) -> String {
        let cleaned: String = instance_id
            .chars()
            .map(|c| match c {
                '\\' | '\0' => '_',
                _ => c,
            })
            .collect();
        if cleaned.is_empty() {
            DEFAULT_SINGLE_INSTANCE_ID.to_string()
        } else {
            cleaned
        }
    }

    fn kernel_object_names(instance_id: &str) -> (String, String) {
        let safe_id = sanitize_object_id(instance_id);
        (format!("Local\\{safe_id}_Mutex"), format!("Local\\{safe_id}_Activate"))
    }

    fn signal_activation_event_with_retry(event_name: &str) {
        unsafe {
            AllowSetForegroundWindow(ASFW_ANY);
        }

        let wide_event = to_wide(event_name);
        for attempt in 0..20 {
            let h_event = unsafe { OpenEventW(EVENT_MODIFY_STATE, FALSE, wide_event.as_ptr()) };
            if !h_event.is_null() {
                let _guard = AutoCloseHandle(h_event);
                unsafe {
                    SetEvent(h_event);
                }
                return;
            }
            let err = unsafe { GetLastError() };
            if err != ERROR_FILE_NOT_FOUND || attempt == 19 {
                return;
            }
            thread::sleep(Duration::from_millis(25));
        }
    }

    pub struct SingleInstanceGuard {
        _mutex_handle: SendHandle,
        activate_event: Arc<SendHandle>,
        stop_event: Arc<SendHandle>,
        listener_handle: Option<JoinHandle<()>>,
    }

    impl SingleInstanceGuard {
        pub fn start_activation_listener<F>(&mut self, mut on_activate: F) -> Result<(), WindowsApiError>
        where
            F: FnMut() + Send + 'static,
        {
            self.stop_listener();

            let activate_event = Arc::clone(&self.activate_event);
            let stop_event = Arc::clone(&self.stop_event);

            let handle = thread::Builder::new()
                .name("counterrush-single-instance".to_string())
                .spawn(move || {
                    let handles = [activate_event.raw(), stop_event.raw()];
                    loop {
                        let wait_res = unsafe { WaitForMultipleObjects(2, handles.as_ptr(), FALSE, INFINITE) };
                        if wait_res == WAIT_OBJECT_0 {
                            on_activate();
                        } else {
                            break;
                        }
                    }
                })
                .map_err(WindowsApiError::Io)?;

            self.listener_handle = Some(handle);
            Ok(())
        }

        fn stop_listener(&mut self) {
            if let Some(handle) = self.listener_handle.take() {
                unsafe {
                    SetEvent(self.stop_event.raw());
                }
                let _ = handle.join();
            }
        }
    }

    impl Drop for SingleInstanceGuard {
        fn drop(&mut self) {
            self.stop_listener();
        }
    }

    pub fn notify_existing_instance(instance_id: &str) -> Result<bool, WindowsApiError> {
        let (mutex_name, event_name) = kernel_object_names(instance_id);
        let wide_mutex = to_wide(&mutex_name);

        let h_mutex = unsafe { OpenMutexW(SYNCHRONIZE, FALSE, wide_mutex.as_ptr()) };
        if !h_mutex.is_null() {
            let _guard = AutoCloseHandle(h_mutex);
            signal_activation_event_with_retry(&event_name);
            return Ok(true);
        }

        let err = unsafe { GetLastError() };
        if err == ERROR_ACCESS_DENIED {
            signal_activation_event_with_retry(&event_name);
            return Ok(true);
        }

        Ok(false)
    }

    pub fn acquire_single_instance(instance_id: &str) -> Result<SingleInstanceOutcome, WindowsApiError> {
        let (mutex_name, event_name) = kernel_object_names(instance_id);
        let wide_mutex = to_wide(&mutex_name);

        let sd_guard = build_user_security_descriptor();
        let mut sec_attrs = sd_guard.as_ref().map(|sd| SecurityAttributes {
            n_length: size_of::<SecurityAttributes>() as u32,
            lp_security_descriptor: sd.0,
            b_inherit_handle: FALSE,
        });
        let sa_ptr = sec_attrs
            .as_mut()
            .map(|sa| sa as *mut SecurityAttributes as *mut c_void)
            .unwrap_or_else(null_mut);

        let h_mutex = unsafe { CreateMutexW(sa_ptr, FALSE, wide_mutex.as_ptr()) };
        let last_err = unsafe { GetLastError() };

        if h_mutex.is_null() {
            if last_err == ERROR_ACCESS_DENIED {
                signal_activation_event_with_retry(&event_name);
                return Ok(SingleInstanceOutcome::SecondaryNotified);
            }
            return Err(WindowsApiError::ExecutionFailed(format!(
                "CreateMutexW falhou com erro Win32 {last_err}"
            )));
        }

        let mutex_handle = SendHandle::new(h_mutex);
        if last_err == ERROR_ALREADY_EXISTS {
            signal_activation_event_with_retry(&event_name);
            return Ok(SingleInstanceOutcome::SecondaryNotified);
        }

        let wide_event = to_wide(&event_name);
        let h_activate = unsafe { CreateEventW(sa_ptr, FALSE, FALSE, wide_event.as_ptr()) } as HANDLE;
        if h_activate.is_null() {
            let ev_err = unsafe { GetLastError() };
            return Err(WindowsApiError::ExecutionFailed(format!(
                "CreateEventW (ativação) falhou com erro Win32 {ev_err}"
            )));
        }
        let activate_event = Arc::new(SendHandle::new(h_activate));

        let h_stop = unsafe { CreateEventW(null_mut(), 1, FALSE, std::ptr::null()) } as HANDLE;
        if h_stop.is_null() {
            let stop_err = unsafe { GetLastError() };
            return Err(WindowsApiError::ExecutionFailed(format!(
                "CreateEventW (parada) falhou com erro Win32 {stop_err}"
            )));
        }
        let stop_event = Arc::new(SendHandle::new(h_stop));

        Ok(SingleInstanceOutcome::Primary(SingleInstanceGuard {
            _mutex_handle: mutex_handle,
            activate_event,
            stop_event,
            listener_handle: None,
        }))
    }
}

#[cfg(not(windows))]
mod win32_impl {
    use super::*;

    pub struct SingleInstanceGuard;

    impl SingleInstanceGuard {
        pub fn start_activation_listener<F>(&mut self, _on_activate: F) -> Result<(), WindowsApiError>
        where
            F: FnMut() + Send + 'static,
        {
            Ok(())
        }
    }

    pub fn notify_existing_instance(_instance_id: &str) -> Result<bool, WindowsApiError> {
        Ok(false)
    }

    pub fn acquire_single_instance(_instance_id: &str) -> Result<SingleInstanceOutcome, WindowsApiError> {
        Ok(SingleInstanceOutcome::Primary(SingleInstanceGuard))
    }
}

pub use win32_impl::{acquire_single_instance, notify_existing_instance, SingleInstanceGuard};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    fn unique_test_id(suffix: &str) -> String {
        format!("CounterRush_Test_{}_{suffix}", std::process::id())
    }

    #[test]
    fn test_notify_existing_without_instance_returns_false() {
        let id = unique_test_id("none");
        let notified = notify_existing_instance(&id).expect("Não deve falhar sem instância ativa");
        assert!(!notified);
    }

    #[test]
    fn test_primary_then_secondary_notifies_listener_and_drop_releases() {
        let id = unique_test_id("lifecycle");

        let mut guard = match acquire_single_instance(&id).expect("Primeira aquisição deve funcionar") {
            SingleInstanceOutcome::Primary(g) => g,
            SingleInstanceOutcome::SecondaryNotified => panic!("Não deveria existir instância prévia"),
        };

        let activations = Arc::new(AtomicUsize::new(0));
        let activations_cb = Arc::clone(&activations);
        guard
            .start_activation_listener(move || {
                activations_cb.fetch_add(1, Ordering::SeqCst);
            })
            .expect("Listener deve iniciar");

        let notified = notify_existing_instance(&id).expect("Notificação explícita deve suceder");
        assert!(notified, "Instância ativa deve ser detectada por notify_existing_instance");

        let second = acquire_single_instance(&id).expect("Segunda tentativa deve notificar e retornar Ok");
        assert!(
            matches!(second, SingleInstanceOutcome::SecondaryNotified),
            "Segunda aquisição deve retornar SecondaryNotified"
        );

        let deadline = Instant::now() + Duration::from_secs(2);
        while activations.load(Ordering::SeqCst) < 2 && Instant::now() < deadline {
            std::thread::sleep(Duration::from_millis(20));
        }
        assert_eq!(
            activations.load(Ordering::SeqCst),
            2,
            "Listener deve receber os dois sinais de ativação"
        );

        drop(guard);

        let reacquired = acquire_single_instance(&id).expect("Reaquisição após Drop deve funcionar");
        assert!(
            matches!(reacquired, SingleInstanceOutcome::Primary(_)),
            "Após Drop do guard anterior, nova execução deve assumir como Primary"
        );
    }
}
