use crate::application::error::ApplicationError;
use crate::application::ports::{CommandRunner, FileSystem, TemplateRenderer};
use crate::domain::FastApiSpec;
use std::path::Path;
use tera::Context;

pub fn generate_fastapi(
    spec: &FastApiSpec,
    runner: &dyn CommandRunner,
    fs: &dyn FileSystem,
    renderer: &dyn TemplateRenderer,
) -> Result<(), ApplicationError> {
    let project_dir = Path::new(&spec.name);

    // 1. Verificar disponibilidad de uv
    runner.run("uv", &["--version"], None)?;

    // 2. uv init --package <nombre>
    runner.run("uv", &["init", "--package", &spec.name], None)?;

    // 3. uv add de dependencias principales
    let driver = match spec.db {
        crate::domain::DbOption::Postgres => "asyncpg",
        crate::domain::DbOption::Sqlite => "aiosqlite",
    };

    runner.run(
        "uv",
        &[
            "add",
            "fastapi",
            "uvicorn[standard]",
            "pydantic-settings",
            "sqlalchemy[asyncio]",
            "alembic",
            driver,
        ],
        Some(project_dir),
    )?;

    // 4. uv add --dev de dependencias de desarrollo
    runner.run(
        "uv",
        &[
            "add",
            "--dev",
            "pytest",
            "pytest-asyncio",
            "httpx",
            "ruff",
            "mypy",
            "aiosqlite",
        ],
        Some(project_dir),
    )?;

    // 5. Renderizar plantillas
    let mut ctx = Context::new();
    ctx.insert("project_name", &spec.name);
    ctx.insert("package_name", &spec.package_name);
    ctx.insert("db", &spec.db.to_string());
    ctx.insert("docker", &spec.docker);

    let src_pkg_dir = project_dir.join("src").join(&spec.package_name);
    fs.create_dir_all(&src_pkg_dir)?;

    let main_py = renderer.render("fastapi_main.py.tera", &ctx)?;
    fs.write_file(&src_pkg_dir.join("main.py"), &main_py)?;

    // 6. Si --docker: renderizar Dockerfile y compose
    if spec.docker {
        let dockerfile = renderer.render("fastapi_Dockerfile.tera", &ctx)?;
        fs.write_file(&project_dir.join("Dockerfile"), &dockerfile)?;

        let compose = renderer.render("fastapi_docker_compose.yml.tera", &ctx)?;
        fs.write_file(&project_dir.join("docker-compose.yml"), &compose)?;
    }

    // 7. Validar el proyecto generado con ruff y pytest
    runner.run("uv", &["run", "ruff", "check"], Some(project_dir))?;
    runner.run("uv", &["run", "pytest", "-q"], Some(project_dir))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::DbOption;
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
            Ok(format!("# Rendered from {}", template_name))
        }
    }

    #[test]
    fn test_generate_fastapi_flow() {
        let spec = FastApiSpec::new("demo-api", DbOption::Postgres, true, false).unwrap();
        let runner = FakeRunner::new();
        let fs = FakeFs::new();
        let renderer = FakeRenderer;

        let result = generate_fastapi(&spec, &runner, &fs, &renderer);
        assert!(result.is_ok());

        let calls = runner.calls.borrow();
        assert_eq!(calls[0], "uv --version");
        assert_eq!(calls[1], "uv init --package demo-api");
        assert!(calls[2].starts_with("uv add fastapi"));
        assert!(calls[3].starts_with("uv add --dev pytest"));
        assert_eq!(calls[4], "uv run ruff check");
        assert_eq!(calls[5], "uv run pytest -q");

        let files = fs.files.borrow();
        assert!(files.iter().any(|(p, _)| p.ends_with("main.py")));
        assert!(files.iter().any(|(p, _)| p.ends_with("Dockerfile")));
        assert!(files.iter().any(|(p, _)| p.ends_with("docker-compose.yml")));
    }
}
