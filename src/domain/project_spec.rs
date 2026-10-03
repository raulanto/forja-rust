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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DbOption {
    Postgres,
    Sqlite,
}

impl std::str::FromStr for DbOption {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "postgres" => Ok(DbOption::Postgres),
            "sqlite" => Ok(DbOption::Sqlite),
            _ => Err(format!("Base de datos no soportada: {}", s)),
        }
    }
}

impl std::fmt::Display for DbOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DbOption::Postgres => write!(f, "postgres"),
            DbOption::Sqlite => write!(f, "sqlite"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AxumSpec {
    pub name: String,
    pub db: DbOption,
    pub docker: bool,
    pub auth: bool,
}

impl AxumSpec {
    pub fn new(
        name: impl Into<String>,
        db: DbOption,
        docker: bool,
        auth: bool,
    ) -> Result<Self, DomainError> {
        let name = name.into();

        if name.trim().is_empty() {
            return Err(DomainError::ProjectName(name));
        }

        Ok(Self {
            name,
            db,
            docker,
            auth,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportOption {
    Http,
    Grpc,
    Both,
}

impl std::str::FromStr for TransportOption {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "http" => Ok(TransportOption::Http),
            "grpc" => Ok(TransportOption::Grpc),
            "both" => Ok(TransportOption::Both),
            _ => Err(format!("Transporte no soportado: {}", s)),
        }
    }
}

impl std::fmt::Display for TransportOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransportOption::Http => write!(f, "http"),
            TransportOption::Grpc => write!(f, "grpc"),
            TransportOption::Both => write!(f, "both"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GoHexSpec {
    pub name: String,
    pub module: String,
    pub transport: TransportOption,
    pub db: DbOption,
    pub docker: bool,
    pub auth: bool,
}

impl GoHexSpec {
    pub fn new(
        name: impl Into<String>,
        module: Option<String>,
        transport: TransportOption,
        db: DbOption,
        docker: bool,
        auth: bool,
    ) -> Result<Self, DomainError> {
        let name = name.into();

        if name.trim().is_empty() {
            return Err(DomainError::ProjectName(name.clone()));
        }

        let module = match module {
            Some(m) if !m.trim().is_empty() => m,
            _ => name.clone(),
        };

        Ok(Self {
            name,
            module,
            transport,
            db,
            docker,
            auth,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FastApiSpec {
    pub name: String,
    pub package_name: String,
    pub db: DbOption,
    pub docker: bool,
    pub auth: bool,
}

impl FastApiSpec {
    pub fn new(
        name: impl Into<String>,
        db: DbOption,
        docker: bool,
        auth: bool,
    ) -> Result<Self, DomainError> {
        let name = name.into();

        if name.trim().is_empty() {
            return Err(DomainError::ProjectName(name.clone()));
        }

        let package_name = name.trim().replace('-', "_").to_lowercase();

        Ok(Self {
            name,
            package_name,
            db,
            docker,
            auth,
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

    #[test]
    fn test_valid_axum_spec() {
        let spec = AxumSpec::new("mi_api", DbOption::Postgres, true, false);
        assert!(spec.is_ok());
        let spec = spec.unwrap();
        assert_eq!(spec.name, "mi_api");
        assert_eq!(spec.db, DbOption::Postgres);
        assert!(spec.docker);
        assert!(!spec.auth);
    }

    #[test]
    fn test_invalid_axum_project_name() {
        let spec = AxumSpec::new("   ", DbOption::Sqlite, false, false);
        assert_eq!(spec, Err(DomainError::ProjectName("   ".into())));
    }

    #[test]
    fn test_valid_go_hex_spec() {
        let spec = GoHexSpec::new(
            "mi_servicio",
            Some("github.com/user/mi_servicio".into()),
            TransportOption::Both,
            DbOption::Postgres,
            true,
            false,
        );
        assert!(spec.is_ok());
        let spec = spec.unwrap();
        assert_eq!(spec.name, "mi_servicio");
        assert_eq!(spec.module, "github.com/user/mi_servicio");
        assert_eq!(spec.transport, TransportOption::Both);
    }

    #[test]
    fn test_go_hex_spec_default_module() {
        let spec = GoHexSpec::new(
            "mi_servicio",
            None,
            TransportOption::Http,
            DbOption::Sqlite,
            false,
            false,
        );
        assert!(spec.is_ok());
        let spec = spec.unwrap();
        assert_eq!(spec.module, "mi_servicio");
    }

    #[test]
    fn test_valid_fastapi_spec() {
        let spec = FastApiSpec::new("mi-api", DbOption::Postgres, true, false);
        assert!(spec.is_ok());
        let spec = spec.unwrap();
        assert_eq!(spec.name, "mi-api");
        assert_eq!(spec.package_name, "mi_api");
        assert_eq!(spec.db, DbOption::Postgres);
        assert!(spec.docker);
        assert!(!spec.auth);
    }

    #[test]
    fn test_invalid_fastapi_project_name() {
        let spec = FastApiSpec::new("   ", DbOption::Sqlite, false, false);
        assert_eq!(spec, Err(DomainError::ProjectName("   ".into())));
    }
}
