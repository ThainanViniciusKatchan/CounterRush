use std::path::{Path, PathBuf};
use crate::CoreError;

pub trait StartupManager: Send + Sync {
    fn is_enabled(&self) -> Result<bool, CoreError>;
    fn enable(&self) -> Result<(), CoreError>;
    fn disable(&self) -> Result<(), CoreError>;
}

pub struct TaskSchedulerStartupManager {
    task_name: String,
    custom_exe_path: Option<PathBuf>,
}

impl Default for TaskSchedulerStartupManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskSchedulerStartupManager {
    pub fn new() -> Self {
        Self {
            task_name: windows_api::DEFAULT_STARTUP_TASK_NAME.to_string(),
            custom_exe_path: None,
        }
    }

    pub fn with_custom_target(task_name: &str, exe_path: PathBuf) -> Self {
        Self {
            task_name: task_name.to_string(),
            custom_exe_path: Some(exe_path),
        }
    }

    pub fn resolve_exe_path(&self) -> Result<PathBuf, CoreError> {
        if let Some(ref path) = self.custom_exe_path {
            return Ok(normalize_exe_path(path));
        }
        let raw_path = std::env::current_exe().map_err(CoreError::Io)?;
        Ok(normalize_exe_path(&raw_path))
    }
}

pub fn normalize_exe_path(path: &Path) -> PathBuf {
    let path_str = path.to_string_lossy();
    if let Some(stripped) = path_str.strip_prefix(r"\\?\") {
        PathBuf::from(stripped)
    } else {
        path.to_path_buf()
    }
}

impl StartupManager for TaskSchedulerStartupManager {
    fn is_enabled(&self) -> Result<bool, CoreError> {
        windows_api::is_task_registered(&self.task_name).map_err(CoreError::WindowsApi)
    }

    fn enable(&self) -> Result<(), CoreError> {
        let exe = self.resolve_exe_path()?;
        windows_api::create_logon_task(&self.task_name, &exe).map_err(CoreError::WindowsApi)
    }

    fn disable(&self) -> Result<(), CoreError> {
        windows_api::delete_task(&self.task_name).map_err(CoreError::WindowsApi)
    }
}

pub fn is_system_startup_enabled() -> Result<bool, CoreError> {
    TaskSchedulerStartupManager::new().is_enabled()
}

pub fn enable_system_startup() -> Result<(), CoreError> {
    TaskSchedulerStartupManager::new().enable()
}

pub fn disable_system_startup() -> Result<(), CoreError> {
    TaskSchedulerStartupManager::new().disable()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_exe_path_strips_unc_prefix() {
        let p = Path::new(r"\\?\C:\Program Files\CounterRush\counterrush.exe");
        let normalized = normalize_exe_path(p);
        assert_eq!(normalized, PathBuf::from(r"C:\Program Files\CounterRush\counterrush.exe"));
    }

    #[test]
    fn test_normalize_exe_path_leaves_standard_path_unchanged() {
        let p = Path::new(r"C:\CounterRush\counterrush.exe");
        let normalized = normalize_exe_path(p);
        assert_eq!(normalized, PathBuf::from(r"C:\CounterRush\counterrush.exe"));
    }

    #[test]
    fn test_custom_target_startup_manager_query() {
        let manager = TaskSchedulerStartupManager::with_custom_target(
            "CounterRush_NonExistent_Core_Test_12345",
            PathBuf::from(r"C:\CounterRush\app.exe"),
        );
        let status = manager.is_enabled().expect("is_enabled deve responder sem erro");
        assert!(!status);
    }

    #[test]
    fn test_custom_target_startup_manager_disable_idempotent() {
        let manager = TaskSchedulerStartupManager::with_custom_target(
            "CounterRush_NonExistent_Core_Test_98765",
            PathBuf::from(r"C:\CounterRush\app.exe"),
        );
        let res = manager.disable();
        assert!(res.is_ok(), "disable em tarefa inexistente deve ser Ok: {:?}", res.err());
    }

    #[test]
    fn test_resolve_exe_path_custom_and_current() {
        let custom_target = PathBuf::from(r"\\?\D:\Games\CounterRush\CounterRush.exe");
        let manager = TaskSchedulerStartupManager::with_custom_target("Test_Task", custom_target);
        let resolved = manager.resolve_exe_path().unwrap();
        assert_eq!(resolved, PathBuf::from(r"D:\Games\CounterRush\CounterRush.exe"));

        let default_manager = TaskSchedulerStartupManager::new();
        let current = default_manager.resolve_exe_path();
        assert!(current.is_ok());
        let cur_path = current.unwrap();
        assert!(!cur_path.to_string_lossy().starts_with(r"\\?\"));
    }
}
