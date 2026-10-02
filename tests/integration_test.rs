use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::tempdir;

#[test]
#[ignore]
fn test_real_django_generation_with_uv() {
    let temp_dir = tempdir().expect("falló al crear directorio temporal");
    let mut cmd = Command::cargo_bin("forja").expect("falló al encontrar el binario forja");

    cmd.current_dir(temp_dir.path())
        .arg("django")
        .arg("demo_test")
        .arg("--app")
        .arg("core")
        .arg("--django-version")
        .arg("6.1.1");

    cmd.assert().success().stdout(predicate::str::contains(
        "¡Proyecto Django 'demo_test' creado exitosamente!",
    ));

    let project_path = temp_dir.path().join("demo_test");
    assert!(project_path.join("pyproject.toml").exists());
    assert!(project_path.join("manage.py").exists());
    assert!(project_path.join("config").join("settings.py").exists());
    assert!(project_path
        .join("apps")
        .join("core")
        .join("apps.py")
        .exists());

    let apps_py =
        std::fs::read_to_string(project_path.join("apps").join("core").join("apps.py")).unwrap();
    assert!(apps_py.contains("name = 'apps.core'"));

    let settings_py =
        std::fs::read_to_string(project_path.join("config").join("settings.py")).unwrap();
    assert!(settings_py.contains("'apps.core'"));
}
