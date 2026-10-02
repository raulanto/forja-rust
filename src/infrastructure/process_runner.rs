use crate::application::error::ApplicationError;
use crate::application::ports::CommandRunner;
use std::path::Path;
use std::process::Command;

pub struct StdProcessRunner;

impl CommandRunner for StdProcessRunner {
    fn run(
        &self,
        program: &str,
        args: &[&str],
        cwd: Option<&Path>,
    ) -> Result<(), ApplicationError> {
        let mut cmd = Command::new(program);
        cmd.args(args);

        if let Some(dir) = cwd {
            cmd.current_dir(dir);
        }

        let status = cmd
            .status()
            .map_err(|e| ApplicationError::CommandExecutionFailed {
                command: format!("{program} {}", args.join(" ")),
                reason: format!("No se pudo ejecutar el proceso: {e}"),
            })?;

        if !status.success() {
            return Err(ApplicationError::CommandExecutionFailed {
                command: format!("{program} {}", args.join(" ")),
                reason: format!("El comando finalizó con código de salida: {status}"),
            });
        }

        Ok(())
    }
}
