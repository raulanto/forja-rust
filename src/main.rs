mod application;
mod cli;
mod domain;
mod infrastructure;

use anyhow::Context;
use clap::Parser;
use cli::{Cli, Commands};
use domain::DjangoSpec;
use infrastructure::{StdFileSystem, StdProcessRunner};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Django(args) => {
            let spec = DjangoSpec::new(args.name, args.django_version, args.app, args.run)
                .context("Parámetros del proyecto Django inválidos")?;

            let runner = StdProcessRunner;
            let fs = StdFileSystem;

            println!("🔨 Creando proyecto Django '{}' con uv...", spec.name);

            application::generate_django(&spec, &runner, &fs)
                .context("Error durante la generación del proyecto Django")?;

            println!("✨ ¡Proyecto Django '{}' creado exitosamente!", spec.name);
        }
        Commands::Axum(args) => {
            let spec = domain::AxumSpec::new(args.name, args.db, args.docker, args.auth)
                .context("Parámetros del proyecto Axum inválidos")?;

            let runner = StdProcessRunner;
            let fs = StdFileSystem;
            let renderer = infrastructure::TeraTemplateRenderer::new()
                .context("Error al inicializar motor de plantillas")?;

            println!("🔨 Creando proyecto Axum '{}'...", spec.name);

            application::generate_axum(&spec, &runner, &fs, &renderer)
                .context("Error durante la generación del proyecto Axum")?;

            println!("✨ ¡Proyecto Axum '{}' creado exitosamente!", spec.name);
        }
        Commands::GoHex(args) => {
            let spec = domain::GoHexSpec::new(
                args.name,
                args.module,
                args.transport,
                args.db,
                args.docker,
                args.auth,
            )
            .context("Parámetros del proyecto Go Hexagonal inválidos")?;

            let runner = StdProcessRunner;
            let fs = StdFileSystem;
            let renderer = infrastructure::TeraTemplateRenderer::new()
                .context("Error al inicializar motor de plantillas")?;

            println!("🔨 Creando servicio Go Hexagonal '{}'...", spec.name);

            application::generate_go_hex(&spec, &runner, &fs, &renderer)
                .context("Error durante la generación del proyecto Go Hexagonal")?;

            println!(
                "✨ ¡Servicio Go Hexagonal '{}' creado exitosamente!",
                spec.name
            );
        }
    }

    Ok(())
}
