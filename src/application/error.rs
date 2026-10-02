use std::path::PathBuf;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Error de dominio: {0}")]
    Domain(#[from] crate::domain::DomainError),

    #[error("Error al ejecutar el comando '{command}': {reason}")]
    CommandExecutionFailed { command: String, reason: String },

    #[error("Error de sistema de archivos en '{path}': {reason}")]
    FileSystemFailed { path: PathBuf, reason: String },

    #[error("Error al aplicar parche en '{file}': {reason}")]
    PatchFailed { file: PathBuf, reason: String },
}
