use crate::application::error::ApplicationError;
use std::path::Path;

pub fn patch_apps_py(content: &str, app_name: &str) -> Result<String, ApplicationError> {
    let target_line = format!("name = '{app_name}'");
    let replacement = format!("name = 'apps.{app_name}'");

    if content.contains(&replacement) {
        return Ok(content.to_string());
    }

    if !content.contains(&target_line) {
        // También probamos comillas dobles
        let target_line_double = format!("name = \"{app_name}\"");
        let replacement_double = format!("name = \"apps.{app_name}\"");

        if content.contains(&replacement_double) {
            return Ok(content.to_string());
        }

        if content.contains(&target_line_double) {
            return Ok(content.replace(&target_line_double, &replacement_double));
        }

        return Err(ApplicationError::PatchFailed {
            file: Path::new("apps").join(app_name).join("apps.py"),
            reason: format!(
                "No se encontró la declaración 'name = \"{app_name}\"' o 'name = '{app_name}'' en apps.py"
            ),
        });
    }

    Ok(content.replace(&target_line, &replacement))
}

pub fn patch_settings_py(content: &str, app_name: &str) -> Result<String, ApplicationError> {
    let app_entry = format!("'apps.{app_name}'");
    let app_entry_double = format!("\"apps.{app_name}\"");

    if content.contains(&app_entry) || content.contains(&app_entry_double) {
        return Ok(content.to_string());
    }

    let staticfiles_entry = "'django.contrib.staticfiles',";
    let staticfiles_entry_double = "\"django.contrib.staticfiles\",";

    if let Some(pos) = content.find(staticfiles_entry) {
        let insert_idx = pos + staticfiles_entry.len();
        let mut new_content = String::with_capacity(content.len() + app_entry.len() + 10);
        new_content.push_str(&content[..insert_idx]);
        new_content.push_str(&format!("\n    'apps.{app_name}',"));
        new_content.push_str(&content[insert_idx..]);
        return Ok(new_content);
    }

    if let Some(pos) = content.find(staticfiles_entry_double) {
        let insert_idx = pos + staticfiles_entry_double.len();
        let mut new_content = String::with_capacity(content.len() + app_entry_double.len() + 10);
        new_content.push_str(&content[..insert_idx]);
        new_content.push_str(&format!("\n    \"apps.{app_name}\","));
        new_content.push_str(&content[insert_idx..]);
        return Ok(new_content);
    }

    Err(ApplicationError::PatchFailed {
        file: Path::new("config/settings.py").to_path_buf(),
        reason: "No se encontró 'django.contrib.staticfiles' en INSTALLED_APPS".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_patch_apps_py_single_quotes() {
        let input = "from django.apps import AppConfig\n\nclass CoreConfig(AppConfig):\n    default_auto_field = 'django.db.models.BigAutoField'\n    name = 'core'\n";
        let patched = patch_apps_py(input, "core").unwrap();
        assert!(patched.contains("name = 'apps.core'"));
    }

    #[test]
    fn test_patch_apps_py_idempotent() {
        let input = "from django.apps import AppConfig\n\nclass CoreConfig(AppConfig):\n    name = 'apps.core'\n";
        let patched = patch_apps_py(input, "core").unwrap();
        assert_eq!(patched, input);
    }

    #[test]
    fn test_patch_settings_py() {
        let input = "INSTALLED_APPS = [\n    'django.contrib.admin',\n    'django.contrib.staticfiles',\n]\n";
        let patched = patch_settings_py(input, "core").unwrap();
        assert!(patched.contains("'django.contrib.staticfiles',\n    'apps.core',"));
    }

    #[test]
    fn test_patch_settings_py_idempotent() {
        let input = "INSTALLED_APPS = [\n    'django.contrib.staticfiles',\n    'apps.core',\n]\n";
        let patched = patch_settings_py(input, "core").unwrap();
        assert_eq!(patched, input);
    }
}
