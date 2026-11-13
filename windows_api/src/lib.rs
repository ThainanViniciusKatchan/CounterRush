use thiserror::Error;

#[derive(Error, Debug)]
pub enum WindowsApiError {
    #[error("Falha ao executar comando do sistema: {0}")]
    ExecutionFailed(String),
    #[error("Acesso negado. Privilégios de Administrador são necessários.")]
    AccessDenied,
    #[error("Chave ou valor do Registro não encontrado: {0}")]
    RegistryNotFound(String),
    #[error("Erro de I/O: {0}")]
    Io(#[from] std::io::Error),
}

pub mod launcher;
pub use launcher::{
    check_downloading_dir_has_730, ensure_steam_running, get_cs2_download_validation_status,
    get_cs2_download_validation_status_for_paths, get_steam_downloading_dirs,
    get_steam_downloading_dirs_from_base, inspect_downloading_dir_730,
    is_cs2_downloading_or_updating, launch_cs2, launch_cs2_economico, launch_cs2_intermediario,
    launch_cs2_potente, launch_cs2_with_args, launch_cs2_with_args_slice,
    launch_on_interactive_desktop, shell_execute, DownloadingDirCheckResult,
};

pub mod steamcmd;
pub use steamcmd::{
    check_bootstrap_log_file, check_content_log_file, check_steamcmd_logs_in_dir,
    get_running_steamcmd_processes, is_command_line_updating_app, monitor_child_stdout,
    parse_bootstrap_log_lines, parse_content_log_lines, parse_steamcmd_stdout_line,
    DownloadActivityState, SteamCmdClient, SteamCmdLogStatus, SteamCmdProcessInfo,
    SteamCmdStdoutEvent, SteamCmdUpdateSummary, CS2_APP_ID, DEFAULT_STEAMCMD_DIR,
};

pub mod powershell;
pub use powershell::{run_powershell, run_powershell_script};

pub mod process;
pub use process::{is_process_running, wait_for_process};

pub mod hardware;
pub use hardware::{detect_hardware, get_cpu_physical_cores, CpuInfo, GpuInfo, HardwareInfo, RamInfo};

pub mod memory;
pub use memory::empty_standby_list;

pub mod clipboard;
pub use clipboard::set_clipboard_text;

pub mod security;
pub use security::{is_elevated, relaunch_as_admin};

pub mod scheduler;
pub use scheduler::{create_logon_task, delete_task, is_task_registered, DEFAULT_STARTUP_TASK_NAME};
