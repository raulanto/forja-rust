use crate::domain::DbOption;
use clap::Args;

#[derive(Args, Debug)]
pub struct AxumArgs {
    /// Nombre del proyecto Axum
    pub name: String,

    /// Motor de base de datos para sqlx (postgres, sqlite)
    #[arg(long, default_value = "postgres")]
    pub db: DbOption,

    /// Genera Dockerfile y docker-compose.yml
    #[arg(long, default_value_t = false)]
    pub docker: bool,

    /// Añade autenticación JWT con refresh tokens y roles
    #[arg(long, default_value_t = false)]
    pub auth: bool,
}
