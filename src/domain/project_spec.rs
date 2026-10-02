use crate::domain::error::DomainError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DjangoSpec {
    pub name: String,
    pub django_version: String,
    pub app_name: String,
    pub run_server: bool,
}

impl DjangoSpec {
    pub fn new(
        name: impl Into<String>,
        django_version: impl Into<String>,
        app_name: impl Into<String>,
        run_server: bool,
    ) -> Result<Self, DomainError> {
        let name = name.into();
        let django_version = django_version.into();
        let app_name = app_name.into();

        if name.trim().is_empty() {
            return Err(DomainError::ProjectName(name));
        }

        if app_name.trim().is_empty() {
            return Err(DomainError::AppName(app_name));
        }

        if django_version.trim().is_empty() {
            return Err(DomainError::DjangoVersion(django_version));
        }

        Ok(Self {
            name,
            django_version,
            app_name,
            run_server,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_django_spec() {
        let spec = DjangoSpec::new("mi_proyecto", "6.1.1", "core", false);
        assert!(spec.is_ok());
        let spec = spec.unwrap();
        assert_eq!(spec.name, "mi_proyecto");
        assert_eq!(spec.django_version, "6.1.1");
        assert_eq!(spec.app_name, "core");
        assert!(!spec.run_server);
    }

    #[test]
    fn test_invalid_project_name() {
        let spec = DjangoSpec::new("", "6.1.1", "core", false);
        assert_eq!(spec, Err(DomainError::ProjectName("".into())));
    }

    #[test]
    fn test_invalid_app_name() {
        let spec = DjangoSpec::new("demo", "6.1.1", "   ", false);
        assert_eq!(spec, Err(DomainError::AppName("   ".into())));
    }
}
