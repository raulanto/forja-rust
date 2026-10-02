use crate::application::error::ApplicationError;
use crate::application::ports::TemplateRenderer;
use tera::{Context, Tera};

pub struct TeraTemplateRenderer {
    tera: Tera,
}

impl TeraTemplateRenderer {
    pub fn new() -> Result<Self, ApplicationError> {
        let mut tera = Tera::default();

        tera.add_raw_templates(vec![
            (
                "main.rs.tera",
                r#"// {{ project_name }} - Axum API
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .with(tracing_subscriber::fmt::layer())
        .init();

    println!("Starting {{ project_name }}...");
    Ok(())
}
"#,
            ),
            (
                "Dockerfile.tera",
                r#"FROM rust:1.80 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
WORKDIR /app
COPY --from=builder /app/target/release/{{ project_name }} /app/server
CMD ["/app/server"]
"#,
            ),
            (
                "docker-compose.yml.tera",
                r#"version: '3.8'
services:
  app:
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL={{ db }}://user:pass@localhost/{{ project_name }}
"#,
            ),
        ])
        .map_err(|e| ApplicationError::TemplateRender(e.to_string()))?;

        Ok(Self { tera })
    }
}

impl TemplateRenderer for TeraTemplateRenderer {
    fn render(&self, template_name: &str, context: &Context) -> Result<String, ApplicationError> {
        self.tera
            .render(template_name, context)
            .map_err(|e| ApplicationError::TemplateRender(e.to_string()))
    }
}
