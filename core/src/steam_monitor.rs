use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};
use crate::CoreError;

pub const CS2_STEAM_APP_ID: u32 = 730;

pub trait SteamRegistryDetector: Send + Sync {
    fn get_running_app_id(&self) -> Result<u32, CoreError>;
    fn is_app_running(&self, app_id: u32) -> Result<bool, CoreError>;
    fn wait_for_registry_change(&self, timeout_ms: u32) -> Result<bool, CoreError>;
}

pub trait SteamLogDetector: Send + Sync {
    fn poll_launch_events(&mut self, app_id: u32) -> Result<Vec<windows_api::SteamGameLaunchEvent>, CoreError>;
    fn has_stop_event(&mut self, app_id: u32, pid: Option<u32>) -> Result<bool, CoreError>;
}

pub trait ProcessQuery: Send + Sync {
    fn is_process_running(&self, exe_name: &str) -> Result<bool, CoreError>;
    fn get_pid_by_name(&self, exe_name: &str) -> Result<Option<u32>, CoreError>;
    fn is_pid_alive(&self, pid: u32) -> bool;
    fn get_foreground_pid(&self) -> Option<u32>;
}

pub struct Win32SteamRegistryDetector {
    watcher: Option<windows_api::RegistryWatcher>,
}

impl Win32SteamRegistryDetector {
    pub fn new() -> Self {
        let watcher = windows_api::RegistryWatcher::new_steam_watcher().ok();
        Self { watcher }
    }
}

impl Default for Win32SteamRegistryDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl SteamRegistryDetector for Win32SteamRegistryDetector {
    fn get_running_app_id(&self) -> Result<u32, CoreError> {
        windows_api::get_steam_running_app_id().map_err(CoreError::WindowsApi)
    }

    fn is_app_running(&self, app_id: u32) -> Result<bool, CoreError> {
        windows_api::is_steam_app_running_in_registry(app_id).map_err(CoreError::WindowsApi)
    }

    fn wait_for_registry_change(&self, timeout_ms: u32) -> Result<bool, CoreError> {
        if let Some(watcher) = &self.watcher {
            watcher.wait_for_change(timeout_ms).map_err(CoreError::WindowsApi)
        } else {
            std::thread::sleep(Duration::from_millis(timeout_ms as u64));
            Ok(false)
        }
    }
}

pub struct Win32SteamLogDetector {
    watcher: windows_api::SteamGameProcessLogWatcher,
}

impl Win32SteamLogDetector {
    pub fn new() -> Self {
        let log_path = match windows_api::resolve_steam_install_path_from_registry() {
            Ok(steam_path) => steam_path.join("logs").join("gameprocess_log.txt"),
            Err(_) => std::path::PathBuf::from(r"C:\Program Files (x86)\Steam\logs\gameprocess_log.txt"),
        };
        let watcher = windows_api::SteamGameProcessLogWatcher::new(log_path);
        Self { watcher }
    }
}

impl Default for Win32SteamLogDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl SteamLogDetector for Win32SteamLogDetector {
    fn poll_launch_events(&mut self, app_id: u32) -> Result<Vec<windows_api::SteamGameLaunchEvent>, CoreError> {
        self.watcher.poll_new_launch_events(app_id).map_err(CoreError::WindowsApi)
    }

    fn has_stop_event(&mut self, app_id: u32, pid: Option<u32>) -> Result<bool, CoreError> {
        self.watcher.has_stop_event(app_id, pid).map_err(CoreError::WindowsApi)
    }
}

pub struct Win32ProcessQuery;

impl Win32ProcessQuery {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Win32ProcessQuery {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessQuery for Win32ProcessQuery {
    fn is_process_running(&self, exe_name: &str) -> Result<bool, CoreError> {
        windows_api::get_process_id_by_name(exe_name)
            .map(|opt| opt.is_some())
            .map_err(CoreError::WindowsApi)
    }

    fn get_pid_by_name(&self, exe_name: &str) -> Result<Option<u32>, CoreError> {
        windows_api::get_process_id_by_name(exe_name).map_err(CoreError::WindowsApi)
    }

    fn is_pid_alive(&self, pid: u32) -> bool {
        windows_api::is_pid_alive(pid)
    }

    fn get_foreground_pid(&self) -> Option<u32> {
        windows_api::get_foreground_process_id()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MonitorState {
    Stopped,
    IdleMonitoring,
    GameDetected { pid: u32 },
    Optimizing { pid: u32 },
    WaitingForGameForeground { pid: u32 },
    GameRunningInBackground { pid: u32 },
    GameExited { pid: u32 },
}

pub type GameEventCallback = Arc<dyn Fn(u32) + Send + Sync + 'static>;

pub struct GameMonitorService {
    state: Arc<RwLock<MonitorState>>,
    running: Arc<AtomicBool>,
    worker_handle: Arc<Mutex<Option<std::thread::JoinHandle<()>>>>,
    on_launch: Arc<RwLock<Option<GameEventCallback>>>,
    on_exit: Arc<RwLock<Option<GameEventCallback>>>,
}

impl GameMonitorService {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(MonitorState::Stopped)),
            running: Arc::new(AtomicBool::new(false)),
            worker_handle: Arc::new(Mutex::new(None)),
            on_launch: Arc::new(RwLock::new(None)),
            on_exit: Arc::new(RwLock::new(None)),
        }
    }

    pub fn set_on_launch<F>(&self, cb: F)
    where
        F: Fn(u32) + Send + Sync + 'static,
    {
        *self.on_launch.write().unwrap() = Some(Arc::new(cb));
    }

    pub fn set_on_exit<F>(&self, cb: F)
    where
        F: Fn(u32) + Send + Sync + 'static,
    {
        *self.on_exit.write().unwrap() = Some(Arc::new(cb));
    }

    pub fn current_state(&self) -> MonitorState {
        *self.state.read().unwrap()
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn start(&self) {
        if self.running.swap(true, Ordering::SeqCst) {
            return;
        }

        let state_arc = Arc::clone(&self.state);
        let running_arc = Arc::clone(&self.running);
        let on_launch_arc = Arc::clone(&self.on_launch);
        let on_exit_arc = Arc::clone(&self.on_exit);

        let handle = std::thread::Builder::new()
            .name("counterrush-steam-monitor".into())
            .spawn(move || {
                let registry_detector = Win32SteamRegistryDetector::new();
                let mut log_detector = Win32SteamLogDetector::new();
                let process_query = Win32ProcessQuery::new();

                *state_arc.write().unwrap() = MonitorState::IdleMonitoring;

                while running_arc.load(Ordering::SeqCst) {
                    let current = *state_arc.read().unwrap();
                    match current {
                        MonitorState::Stopped => break,
                        MonitorState::IdleMonitoring => {
                            let _ = registry_detector.wait_for_registry_change(1500);
                            if !running_arc.load(Ordering::SeqCst) {
                                break;
                            }

                            let is_cs2_in_reg = registry_detector
                                .is_app_running(CS2_STEAM_APP_ID)
                                .unwrap_or(false);

                            let mut detected_pid = None;

                            if is_cs2_in_reg {
                                if let Ok(events) = log_detector.poll_launch_events(CS2_STEAM_APP_ID) {
                                    if let Some(last_evt) = events.last() {
                                        if process_query.is_pid_alive(last_evt.pid) {
                                            detected_pid = Some(last_evt.pid);
                                        }
                                    }
                                }

                                if detected_pid.is_none() {
                                    if let Ok(Some(pid)) = process_query.get_pid_by_name("cs2.exe") {
                                        detected_pid = Some(pid);
                                    }
                                }
                            }

                            if let Some(pid) = detected_pid {
                                *state_arc.write().unwrap() = MonitorState::GameDetected { pid };
                                *state_arc.write().unwrap() = MonitorState::Optimizing { pid };

                                if let Some(cb) = on_launch_arc.read().unwrap().as_ref() {
                                    cb(pid);
                                }

                                *state_arc.write().unwrap() = MonitorState::WaitingForGameForeground { pid };

                                let foreground_start = Instant::now();
                                let max_foreground_wait = Duration::from_secs(45);

                                while running_arc.load(Ordering::SeqCst) && foreground_start.elapsed() < max_foreground_wait {
                                    if !process_query.is_pid_alive(pid) {
                                        break;
                                    }
                                    if process_query.get_foreground_pid() == Some(pid) {
                                        break;
                                    }
                                    std::thread::sleep(Duration::from_millis(250));
                                }

                                *state_arc.write().unwrap() = MonitorState::GameRunningInBackground { pid };
                            }
                        }
                        MonitorState::GameRunningInBackground { pid } => {
                            std::thread::sleep(Duration::from_millis(1500));
                            if !running_arc.load(Ordering::SeqCst) {
                                break;
                            }

                            let still_alive = process_query.is_pid_alive(pid);
                            let steam_reg_stopped = !registry_detector
                                .is_app_running(CS2_STEAM_APP_ID)
                                .unwrap_or(true);
                            let has_stop_evt = log_detector
                                .has_stop_event(CS2_STEAM_APP_ID, Some(pid))
                                .unwrap_or(false);

                            if !still_alive || (steam_reg_stopped && has_stop_evt) {
                                *state_arc.write().unwrap() = MonitorState::GameExited { pid };

                                if let Some(cb) = on_exit_arc.read().unwrap().as_ref() {
                                    cb(pid);
                                }

                                *state_arc.write().unwrap() = MonitorState::IdleMonitoring;
                            }
                        }
                        _ => {
                            std::thread::sleep(Duration::from_millis(500));
                        }
                    }
                }

                *state_arc.write().unwrap() = MonitorState::Stopped;
            });

        if let Ok(handle) = handle {
            *self.worker_handle.lock().unwrap() = Some(handle);
        }
    }

    pub fn stop(&self) {
        if !self.running.swap(false, Ordering::SeqCst) {
            return;
        }

        *self.state.write().unwrap() = MonitorState::Stopped;

        if let Some(handle) = self.worker_handle.lock().unwrap().take() {
            let _ = handle.join();
        }
    }
}

impl Default for GameMonitorService {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for GameMonitorService {
    fn drop(&mut self) {
        self.stop();
    }
}
