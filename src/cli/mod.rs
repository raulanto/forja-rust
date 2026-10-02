pub mod axum;
pub mod django;

use axum::AxumArgs;
use clap::{Parser, Subcommand};
use django::DjangoArgs;

#[derive(Parser, Debug)]
#[command(
    name = "forja",
    author,
    version,
    about = "Generador de proyectos CLI",
    long_about = None
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generar un proyecto Django con uv
    Django(DjangoArgs),

    /// Generar una API en Rust con Axum y arquitectura hexagonal
    Axum(AxumArgs),
}
