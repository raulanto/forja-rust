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
    }

    Ok(())
}
