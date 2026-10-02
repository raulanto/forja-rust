use crate::application::error::ApplicationError;
use crate::application::ports::FileSystem;
use std::fs;
use std::path::Path;

pub struct StdFileSystem;

impl FileSystem for StdFileSystem {
    fn create_dir_all(&self, path: &Path) -> Result<(), ApplicationError> {
        fs::create_dir_all(path).map_err(|e| ApplicationError::FileSystemFailed {
            path: path.to_path_buf(),
            reason: format!("No se pudo crear el directorio: {e}"),
        })
    }

    fn write_file(&self, path: &Path, content: &str) -> Result<(), ApplicationError> {
        fs::write(path, content).map_err(|e| ApplicationError::FileSystemFailed {
            path: path.to_path_buf(),
            reason: format!("No se pudo escribir en el archivo: {e}"),
        })
    }

    fn read_file(&self, path: &Path) -> Result<String, ApplicationError> {
        fs::read_to_string(path).map_err(|e| ApplicationError::FileSystemFailed {
            path: path.to_path_buf(),
            reason: format!("No se pudo leer el archivo: {e}"),
        })
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }
}
