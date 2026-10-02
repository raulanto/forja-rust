use crate::application::error::ApplicationError;
use crate::application::ports::{CommandRunner, FileSystem};
use crate::domain::DjangoSpec;
use crate::infrastructure::settings_patcher::{patch_apps_py, patch_settings_py};
use std::path::PathBuf;

pub fn generate_django(
    spec: &DjangoSpec,
    runner: &dyn CommandRunner,
    fs: &dyn FileSystem,
) -> Result<(), ApplicationError> {
    let project_dir = PathBuf::from(&spec.name);

    // 1. uv init <nombre>
    runner.run("uv", &["init", &spec.name], None)?;

    // 2. uv add "django==<versión>"
    let django_dep = format!("django=={}", spec.django_version);
    runner.run("uv", &["add", &django_dep], Some(&project_dir))?;

    // 3. uv run django-admin startproject config .
    runner.run(
        "uv",
        &["run", "django-admin", "startproject", "config", "."],
        Some(&project_dir),
    )?;

    // 4. mkdir apps y crear apps/__init__.py
    let apps_dir = project_dir.join("apps");
    fs.create_dir_all(&apps_dir)?;
    fs.write_file(&apps_dir.join("__init__.py"), "")?;

    // 5. uv run python manage.py startapp <app> apps/<app>
    let app_path_str = format!("apps/{}", spec.app_name);
    runner.run(
        "uv",
        &[
            "run",
            "python",
            "manage.py",
            "startapp",
            &spec.app_name,
            &app_path_str,
        ],
        Some(&project_dir),
    )?;

    // 6. Parchar apps/<app>/apps.py
    let apps_py_path = apps_dir.join(&spec.app_name).join("apps.py");
    let apps_py_content = fs.read_file(&apps_py_path)?;
    let patched_apps_py = patch_apps_py(&apps_py_content, &spec.app_name)?;
    fs.write_file(&apps_py_path, &patched_apps_py)?;

    // 7. Parchar config/settings.py
    let settings_py_path = project_dir.join("config").join("settings.py");
    let settings_content = fs.read_file(&settings_py_path)?;
    let patched_settings = patch_settings_py(&settings_content, &spec.app_name)?;
    fs.write_file(&settings_py_path, &patched_settings)?;

    // 8. uv run python manage.py migrate
    runner.run(
        "uv",
        &["run", "python", "manage.py", "migrate"],
        Some(&project_dir),
    )?;

    // 9. Si --run: uv run python manage.py runserver
    if spec.run_server {
        runner.run(
            "uv",
            &["run", "python", "manage.py", "runserver"],
            Some(&project_dir),
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::path::Path;

    struct FakeCommandRunner {
        calls: RefCell<Vec<(String, Vec<String>, Option<PathBuf>)>>,
    }

    impl FakeCommandRunner {
        fn new() -> Self {
            Self {
                calls: RefCell::new(Vec::new()),
            }
        }
    }

    impl CommandRunner for FakeCommandRunner {
        fn run(
            &self,
            program: &str,
            args: &[&str],
            cwd: Option<&Path>,
        ) -> Result<(), ApplicationError> {
            self.calls.borrow_mut().push((
                program.to_string(),
                args.iter().map(|s| s.to_string()).collect(),
                cwd.map(|p| p.to_path_buf()),
            ));
            Ok(())
        }
    }

    struct FakeFileSystem {
        files: RefCell<HashMap<PathBuf, String>>,
        dirs: RefCell<Vec<PathBuf>>,
    }

    impl FakeFileSystem {
        fn new() -> Self {
            Self {
                files: RefCell::new(HashMap::new()),
                dirs: RefCell::new(Vec::new()),
            }
        }
    }

    impl FileSystem for FakeFileSystem {
        fn create_dir_all(&self, path: &Path) -> Result<(), ApplicationError> {
            self.dirs.borrow_mut().push(path.to_path_buf());
            Ok(())
        }

        fn write_file(&self, path: &Path, content: &str) -> Result<(), ApplicationError> {
            self.files
                .borrow_mut()
                .insert(path.to_path_buf(), content.to_string());
            Ok(())
        }

        fn read_file(&self, path: &Path) -> Result<String, ApplicationError> {
            self.files.borrow().get(path).cloned().ok_found(path)
        }

        fn exists(&self, path: &Path) -> bool {
            self.files.borrow().contains_key(path)
                || self.dirs.borrow().contains(&path.to_path_buf())
        }
    }

    trait OptionExt<T> {
        fn ok_found(self, path: &Path) -> Result<T, ApplicationError>;
    }

    impl<T> OptionExt<T> for Option<T> {
        fn ok_found(self, path: &Path) -> Result<T, ApplicationError> {
            self.ok_or_else(|| ApplicationError::FileSystemFailed {
                path: path.to_path_buf(),
                reason: "Archivo no encontrado en fake fs".to_string(),
            })
        }
    }

    #[test]
    fn test_generate_django_flow() {
        let spec = DjangoSpec::new("demo", "6.1.1", "core", false).unwrap();
        let runner = FakeCommandRunner::new();
        let fs = FakeFileSystem::new();

        // Precargamos los archivos que startapp y startproject generarían
        let apps_py_path = PathBuf::from("demo/apps/core/apps.py");
        let settings_py_path = PathBuf::from("demo/config/settings.py");

        fs.write_file(
            &apps_py_path,
            "class CoreConfig(AppConfig):\n    name = 'core'\n",
        )
        .unwrap();

        fs.write_file(
            &settings_py_path,
            "INSTALLED_APPS = [\n    'django.contrib.staticfiles',\n]\n",
        )
        .unwrap();

        let result = generate_django(&spec, &runner, &fs);
        assert!(result.is_ok());

        // Verificar que los archivos se parcharon correctamente
        let patched_apps = fs.read_file(&apps_py_path).unwrap();
        assert!(patched_apps.contains("name = 'apps.core'"));

        let patched_settings = fs.read_file(&settings_py_path).unwrap();
        assert!(patched_settings.contains("'apps.core'"));

        // Verificar la secuencia de comandos ejecutados
        let calls = runner.calls.borrow();
        assert_eq!(calls.len(), 5); // init, add, startproject, startapp, migrate
        assert_eq!(calls[0].0, "uv");
        assert_eq!(calls[0].1, vec!["init", "demo"]);
    }
}
