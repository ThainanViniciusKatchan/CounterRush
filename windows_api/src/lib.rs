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
pub use launcher::launch_cs2;

pub mod powershell;
pub use powershell::run_powershell;

pub mod process;
pub use process::{is_process_running, wait_for_process};
