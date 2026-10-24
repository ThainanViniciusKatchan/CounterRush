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
    launch_cs2, launch_cs2_economico, launch_cs2_intermediario, launch_cs2_potente,
    launch_cs2_with_args, launch_cs2_with_args_slice,
};

pub mod powershell;
pub use powershell::{run_powershell, run_powershell_script};

pub mod process;
pub use process::{is_process_running, wait_for_process};

pub mod hardware;
pub use hardware::{detect_hardware, get_cpu_physical_cores, CpuInfo, GpuInfo, HardwareInfo, RamInfo};

pub mod memory;
pub use memory::empty_standby_list;
