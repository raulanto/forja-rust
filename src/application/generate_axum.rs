use crate::application::error::ApplicationError;
use crate::application::ports::{CommandRunner, FileSystem, TemplateRenderer};
use crate::domain::AxumSpec;
use std::path::Path;
use tera::Context;

pub fn generate_axum(
    spec: &AxumSpec,
    runner: &dyn CommandRunner,
    fs: &dyn FileSystem,
    renderer: &dyn TemplateRenderer,
) -> Result<(), ApplicationError> {
    let project_dir = Path::new(&spec.name);

    // 1. cargo new <nombre>
    runner.run("cargo", &["new", &spec.name], None)?;

    // 2. cargo add de dependencias básicas
    let mut deps = vec![
        "axum",
        "tower-http",
        "serde",
        "tracing",
        "tracing-subscriber",
        "thiserror",
        "anyhow",
        "dotenvy",
        "uuid",
    ];

    match spec.db {
        crate::domain::DbOption::Postgres => deps.push("sqlx"),
        crate::domain::DbOption::Sqlite => deps.push("sqlx"),
    }

    let mut add_args = vec!["add"];
    add_args.extend(deps);

    runner.run("cargo", &add_args, Some(project_dir))?;

    // tokio requiere la característica 'full' o 'rt-multi-thread' y 'macros' para #[tokio::main]
    runner.run(
        "cargo",
        &["add", "tokio", "--features", "full"],
        Some(project_dir),
    )?;

    // tracing-subscriber requiere la característica 'env-filter' para EnvFilter
    runner.run(
        "cargo",
        &["add", "tracing-subscriber", "--features", "env-filter"],
        Some(project_dir),
    )?;

    // Si --auth, agregar dependencias adicionales
    if spec.auth {
        let auth_deps = vec!["jsonwebtoken", "argon2", "rand", "sha2", "time"];
        let mut auth_add_args = vec!["add"];
        auth_add_args.extend(auth_deps);
        runner.run("cargo", &auth_add_args, Some(project_dir))?;
    }

    // 3. Renderizar plantillas principales
    let mut ctx = Context::new();
    ctx.insert("project_name", &spec.name);
    ctx.insert("db", &spec.db.to_string());
    ctx.insert("docker", &spec.docker);
    ctx.insert("auth", &spec.auth);

    let src_dir = project_dir.join("src");
    fs.create_dir_all(&src_dir)?;

    let main_content = renderer.render("main.rs.tera", &ctx)?;
    fs.write_file(&src_dir.join("main.rs"), &main_content)?;

    // Renderizar archivos de auth si spec.auth está activado
    if spec.auth {
        let auth_domain_dir = src_dir.join("domain").join("auth");
        fs.create_dir_all(&auth_domain_dir)?;
        let auth_app_dir = src_dir.join("application").join("auth");
        fs.create_dir_all(&auth_app_dir)?;

        let role_content = renderer.render("role.rs.tera", &ctx)?;
        fs.write_file(&auth_domain_dir.join("role.rs"), &role_content)?;

        let auth_ports_content = renderer.render("auth_ports.rs.tera", &ctx)?;
        fs.write_file(&auth_app_dir.join("ports.rs"), &auth_ports_content)?;
    }

    // 4. Si --docker: renderizar Dockerfile y compose
    if spec.docker {
        let dockerfile = renderer.render("Dockerfile.tera", &ctx)?;
        fs.write_file(&project_dir.join("Dockerfile"), &dockerfile)?;

        let compose = renderer.render("docker-compose.yml.tera", &ctx)?;
        fs.write_file(&project_dir.join("docker-compose.yml"), &compose)?;
    }

    // 5. cargo check para validar compilación
    runner.run("cargo", &["check"], Some(project_dir))?;

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
            Ok(format!("// Rendered from {}", template_name))
        }
    }

    #[test]
    fn test_generate_axum_flow() {
        let spec = AxumSpec::new("demo_api", DbOption::Postgres, true, false).unwrap();
        let runner = FakeRunner::new();
        let fs = FakeFs::new();
        let renderer = FakeRenderer;

        let result = generate_axum(&spec, &runner, &fs, &renderer);
        assert!(result.is_ok());

        let calls = runner.calls.borrow();
        assert_eq!(calls[0], "cargo new demo_api");
        assert!(calls[1].starts_with("cargo add axum tower-http"));
        assert_eq!(calls[2], "cargo add tokio --features full");
        assert_eq!(
            calls[3],
            "cargo add tracing-subscriber --features env-filter"
        );
        assert_eq!(calls[4], "cargo check");

        let files = fs.files.borrow();
        assert!(files.iter().any(|(p, _)| p.ends_with("main.rs")));
        assert!(files.iter().any(|(p, _)| p.ends_with("Dockerfile")));
        assert!(files.iter().any(|(p, _)| p.ends_with("docker-compose.yml")));
    }

    #[test]
    fn test_generate_axum_with_auth_flow() {
        let spec = AxumSpec::new("demo_auth_api", DbOption::Postgres, false, true).unwrap();
        let runner = FakeRunner::new();
        let fs = FakeFs::new();
        let renderer = FakeRenderer;

        let result = generate_axum(&spec, &runner, &fs, &renderer);
        assert!(result.is_ok());

        let calls = runner.calls.borrow();
        assert_eq!(calls[0], "cargo new demo_auth_api");
        assert!(calls[1].starts_with("cargo add axum tower-http"));
        assert_eq!(calls[2], "cargo add tokio --features full");
        assert_eq!(
            calls[3],
            "cargo add tracing-subscriber --features env-filter"
        );
        assert!(calls[4].starts_with("cargo add jsonwebtoken argon2"));
        assert_eq!(calls[5], "cargo check");

        let files = fs.files.borrow();
        assert!(files.iter().any(|(p, _)| p.ends_with("role.rs")));
        assert!(files.iter().any(|(p, _)| p.ends_with("ports.rs")));
    }
}
