use crate::domain::DbOption;
use clap::Args;

#[derive(Args, Debug)]
pub struct FastApiArgs {
    /// Nombre del proyecto FastAPI
    pub name: String,

    /// Motor de base de datos (postgres, sqlite)
    #[arg(long, default_value = "postgres")]
    pub db: DbOption,

    /// Genera Dockerfile y docker-compose.yml
    #[arg(long, default_value_t = false)]
    pub docker: bool,

    /// Añade el módulo de autenticación JWT (reservado para fase posterior)
    #[arg(long, default_value_t = false)]
    pub auth: bool,
}
