use crate::application::error::ApplicationError;
use std::path::Path;

pub trait CommandRunner {
    fn run(&self, program: &str, args: &[&str], cwd: Option<&Path>)
        -> Result<(), ApplicationError>;
}

pub trait FileSystem {
    fn create_dir_all(&self, path: &Path) -> Result<(), ApplicationError>;
    fn write_file(&self, path: &Path, content: &str) -> Result<(), ApplicationError>;
    fn read_file(&self, path: &Path) -> Result<String, ApplicationError>;
    #[allow(dead_code)]
    fn exists(&self, path: &Path) -> bool;
}

pub trait TemplateRenderer {
    fn render(
        &self,
        template_name: &str,
        context: &tera::Context,
    ) -> Result<String, ApplicationError>;
}
