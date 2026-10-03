use crate::application::error::ApplicationError;
use crate::application::ports::{CommandRunner, FileSystem, TemplateRenderer};
use crate::domain::GoHexSpec;
use std::path::Path;
use tera::Context;

pub fn generate_go_hex(
    spec: &GoHexSpec,
    runner: &dyn CommandRunner,
    fs: &dyn FileSystem,
    renderer: &dyn TemplateRenderer,
) -> Result<(), ApplicationError> {
    let project_dir = Path::new(&spec.name);

    // 1. Comprobar disponibilidad de go
    runner.run("go", &["version"], None)?;

    // Crear el directorio del proyecto
    fs.create_dir_all(project_dir)?;

    // 2. go mod init <módulo>
    runner.run("go", &["mod", "init", &spec.module], Some(project_dir))?;

    // 3. Renderizar plantillas
    let mut ctx = Context::new();
    ctx.insert("project_name", &spec.name);
    ctx.insert("module_name", &spec.module);
    ctx.insert("transport", &spec.transport.to_string());
    ctx.insert("db", &spec.db.to_string());
    ctx.insert("docker", &spec.docker);

    let cmd_dir = project_dir.join("cmd").join("api");
    fs.create_dir_all(&cmd_dir)?;

    let main_go = renderer.render("go_main.go.tera", &ctx)?;
    fs.write_file(&cmd_dir.join("main.go"), &main_go)?;

    // 4. Si --docker
    if spec.docker {
        let dockerfile = renderer.render("go_Dockerfile.tera", &ctx)?;
        fs.write_file(&project_dir.join("Dockerfile"), &dockerfile)?;
    }

    // 5. go mod tidy
    runner.run("go", &["mod", "tidy"], Some(project_dir))?;

    // 6. Validar compilación
    runner.run("go", &["build", "./..."], Some(project_dir))?;
    runner.run("go", &["vet", "./..."], Some(project_dir))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DbOption, TransportOption};
    use std::cell::RefCell;

    struct FakeRunner {
        calls: RefCell<Vec<String>>,
    }

    impl FakeRunner {
        fn new() -> Self {
            Self {
                calls: RefCell::new(Vec::new()),
            }
        }
    }

    impl CommandRunner for FakeRunner {
        fn run(
            &self,
            program: &str,
            args: &[&str],
            _cwd: Option<&Path>,
        ) -> Result<(), ApplicationError> {
            let cmd = format!("{} {}", program, args.join(" "));
            self.calls.borrow_mut().push(cmd);
            Ok(())
        }
    }

    struct FakeFs {
        files: RefCell<Vec<(String, String)>>,
    }

    impl FakeFs {
        fn new() -> Self {
            Self {
                files: RefCell::new(Vec::new()),
            }
        }
    }

    impl FileSystem for FakeFs {
        fn create_dir_all(&self, _path: &Path) -> Result<(), ApplicationError> {
            Ok(())
        }

        fn write_file(&self, path: &Path, content: &str) -> Result<(), ApplicationError> {
            self.files
                .borrow_mut()
                .push((path.to_string_lossy().to_string(), content.to_string()));
            Ok(())
        }

        fn read_file(&self, _path: &Path) -> Result<String, ApplicationError> {
            Ok(String::new())
        }

        fn exists(&self, _path: &Path) -> bool {
            false
        }
    }

    struct FakeRenderer;

    impl TemplateRenderer for FakeRenderer {
        fn render(
            &self,
            template_name: &str,
            _context: &Context,
        ) -> Result<String, ApplicationError> {
            Ok(format!("// Rendered from {}", template_name))
        }
    }

    #[test]
    fn test_generate_go_hex_flow() {
        let spec = GoHexSpec::new(
            "demo_go",
            Some("github.com/user/demo_go".into()),
            TransportOption::Http,
            DbOption::Postgres,
            true,
            false,
        )
        .unwrap();

        let runner = FakeRunner::new();
        let fs = FakeFs::new();
        let renderer = FakeRenderer;

        let result = generate_go_hex(&spec, &runner, &fs, &renderer);
        assert!(result.is_ok());

        let calls = runner.calls.borrow();
        assert_eq!(calls[0], "go version");
        assert_eq!(calls[1], "go mod init github.com/user/demo_go");
        assert_eq!(calls[2], "go mod tidy");
        assert_eq!(calls[3], "go build ./...");
        assert_eq!(calls[4], "go vet ./...");

        let files = fs.files.borrow();
        assert!(files.iter().any(|(p, _)| p.ends_with("main.go")));
        assert!(files.iter().any(|(p, _)| p.ends_with("Dockerfile")));
    }
}
