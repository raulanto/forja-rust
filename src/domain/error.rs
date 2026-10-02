use thiserror::Error;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum DomainError {
    #[error("El nombre del proyecto '{0}' es inválido")]
    ProjectName(String),

    #[error("El nombre de la app '{0}' es inválido")]
    AppName(String),

    #[error("La versión de Django '{0}' es inválida")]
    DjangoVersion(String),
}
