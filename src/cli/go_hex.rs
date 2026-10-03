use crate::domain::{DbOption, TransportOption};
use clap::Args;

#[derive(Args, Debug)]
pub struct GoHexArgs {
    /// Nombre del proyecto
    pub name: String,

    /// Nombre del módulo de Go (por defecto es igual al nombre del proyecto)
    #[arg(long)]
    pub module: Option<String>,

    /// Transporte habilitado (http, grpc, both)
    #[arg(long, default_value = "http")]
    pub transport: TransportOption,

    /// Base de datos para el driver de Go (postgres, sqlite)
    #[arg(long, default_value = "postgres")]
    pub db: DbOption,

    /// Genera Dockerfile y docker-compose.yml
    #[arg(long, default_value_t = false)]
    pub docker: bool,

    /// Añade el módulo de autenticación JWT (reservado para fase posterior)
    #[arg(long, default_value_t = false)]
    pub auth: bool,
}
