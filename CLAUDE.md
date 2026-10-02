# CLAUDE.md

## Qué es este proyecto

`forja` es un CLI en Rust que genera proyectos desde cero ejecutando las herramientas oficiales de cada stack y aplicando la estructura que prefiero. Generadores actuales: **Django con uv** y **Axum (hexagonal)**. Los siguientes (Angular, NestJS, Go hexagonal) se agregan como nuevos subcomandos.

## Stack

- Rust (edición 2021 o superior)
- `clap` (derive) para el CLI
- `tera` para renderizar plantillas (embebidas con `include_str!`)
- `anyhow` en el binario, `thiserror` en domain y application
- `tempfile` y `assert_cmd` para tests

## Arquitectura (hexagonal)

```
src/
├── main.rs                  # solo arranca el CLI
├── cli/                     # adaptador de entrada (clap)
│   ├── mod.rs
│   ├── django.rs
│   └── axum.rs
├── domain/                  # sin dependencias externas
│   ├── project_spec.rs      # DjangoSpec y AxumSpec
│   └── error.rs
├── application/
│   ├── ports.rs             # traits CommandRunner, FileSystem y TemplateRenderer
│   ├── generate_django.rs   # caso de uso
│   └── generate_axum.rs     # caso de uso
└── infrastructure/          # adaptadores de salida
    ├── process_runner.rs    # std::process::Command
    ├── fs.rs                # std::fs
    ├── template_renderer.rs # tera con plantillas embebidas
    └── settings_patcher.rs  # edita settings.py y apps.py

templates/
└── axum/                    # plantillas (.tera) del proyecto generado
```

Reglas:
- `domain` no importa nada de `application` ni `infrastructure`.
- Los casos de uso reciben los puertos por inyección (`&dyn CommandRunner`, `&dyn FileSystem`).
- Los tests de casos de uso usan fakes de los puertos; no ejecutan `uv` real.
- Un test de integración (marcado `#[ignore]`) ejecuta el flujo real con `uv` instalado.

## Comando actual: `forja django <nombre>`

Flags:
- `--django-version <v>` (default `6.1.1`)
- `--app <nombre>` (default `core`)
- `--run` ejecuta `runserver` al final (por defecto no)

Pasos, en orden (cada uno falla rápido y reporta qué comando falló):

1. `uv init <nombre>` y entrar al directorio
2. `uv add "django==<versión>"`
3. `uv run django-admin startproject config .`
4. `mkdir apps` y crear `apps/__init__.py`
5. `uv run python manage.py startapp <app> apps/<app>`
6. Parchar `apps/<app>/apps.py`: `name = "apps.<app>"`
7. Parchar `config/settings.py`: agregar `"apps.<app>"` a `INSTALLED_APPS`
8. `uv run python manage.py migrate`
9. Si `--run`: `uv run python manage.py runserver`

Parches de texto:
- Deben ser idempotentes (no duplicar si ya existe).
- Si no encuentran el punto de inserción, devuelven error claro; nunca modifican a ciegas.
- `settings.py`: insertar tras la línea de `django.contrib.staticfiles`.

## Comando: `forja axum <nombre>`

Flags:
- `--db <postgres|sqlite>` (default `postgres`)
- `--docker` genera `Dockerfile` y `docker-compose.yml` (por defecto no)
- `--auth` reservado para la siguiente fase (JWT); por ahora no hace nada

Pasos, en orden:

1. `cargo new <nombre>`
2. `cargo add` de las dependencias: `axum`, `tokio` (full), `tower-http` (trace, cors), `serde` (derive), `sqlx` (según `--db`), `tracing`, `tracing-subscriber`, `thiserror`, `anyhow`, `dotenvy`, `uuid`
3. Renderizar plantillas (estructura hexagonal y módulo `user` de ejemplo)
4. Si `--docker`: renderizar Dockerfile y compose
5. `cargo check` para validar que el proyecto generado compila

Estructura del proyecto generado:

```
src/
├── main.rs, lib.rs, config.rs
├── domain/user/            # entity.rs, error.rs
├── application/user/       # ports.rs (UserRepository), create_user.rs
├── infrastructure/persistence/  # pool.rs, user_repo.rs (sqlx)
└── presentation/http/      # router, state, error, dto/, handlers/
tests/api_health.rs
```

Reglas del proyecto generado:
- Dependencias: `presentation → application → domain`; `infrastructure` implementa los puertos de `application`.
- Los handlers solo convierten DTO ↔ caso de uso; sin lógica de negocio.
- Un único `AppError` traduce errores de dominio a respuestas HTTP.
- `AppState` guarda los casos de uso (`Arc`), no el pool.
- Incluye `/health`, tracing y graceful shutdown desde el inicio.
- SQL Server no está soportado por `sqlx`; si se necesita, iría como otro adaptador (`tiberius`) sin tocar domain ni application.

Testing:
- Los tests de `generate_axum` usan fakes de `CommandRunner`, `FileSystem` y `TemplateRenderer`, y verifican qué archivos y comandos se piden.
- Las plantillas se validan con tests de snapshot del renderizado.
- Un test `#[ignore]` genera un proyecto real y ejecuta `cargo check` sobre él.

## Convenciones

- Código e identificadores en inglés; mensajes al usuario y docs en español.
- Errores con contexto (`.context("ejecutando uv add")`).
- Sin `unwrap()` ni `expect()` fuera de tests.
- `cargo fmt` y `cargo clippy -- -D warnings` antes de cada commit.
- Commits pequeños, un paso del flujo por commit.

## Comandos de desarrollo

```bash
cargo build
cargo test
cargo test -- --ignored   # integración real (requiere uv)
cargo run -- django demo --app core
cargo run -- axum demo_api --db postgres
```

## Cómo agregar un generador nuevo

1. Spec en `domain/` (por ejemplo `AxumSpec`).
2. Caso de uso en `application/generate_<stack>.rs` usando los puertos existentes.
3. Subcomando en `cli/<stack>.rs`.
4. Tests con fakes y un test de integración `#[ignore]`.

## Fuera de alcance por ahora

- `--auth` (JWT) para Axum: siguiente fase
- Configuración por archivo (`forja.toml`)
- Nginx y CI (Docker solo existe como `forja axum --docker`)