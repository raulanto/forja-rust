# CLAUDE.md

## Qué es este proyecto

`forja` es un CLI en Rust que genera proyectos desde cero ejecutando las herramientas oficiales de cada stack y aplicando la estructura que prefiero. Primer generador: **Django con uv**. Los siguientes (Angular, NestJS, Axum, Go hexagonal) se agregan como nuevos subcomandos.

## Stack

- Rust (edición 2021 o superior)
- `clap` (derive) para el CLI
- `anyhow` en el binario, `thiserror` en domain y application
- `tempfile` y `assert_cmd` para tests

## Arquitectura (hexagonal)

```
src/
├── main.rs                  # solo arranca el CLI
├── cli/                     # adaptador de entrada (clap)
│   ├── mod.rs
│   └── django.rs
├── domain/                  # sin dependencias externas
│   ├── project_spec.rs      # DjangoSpec: nombre, versión, app inicial
│   └── error.rs
├── application/
│   ├── ports.rs             # traits CommandRunner y FileSystem
│   └── generate_django.rs   # caso de uso
└── infrastructure/          # adaptadores de salida
    ├── process_runner.rs    # std::process::Command
    ├── fs.rs                # std::fs
    └── settings_patcher.rs  # edita settings.py y apps.py
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
```

## Cómo agregar un generador nuevo

1. Spec en `domain/` (por ejemplo `AxumSpec`).
2. Caso de uso en `application/generate_<stack>.rs` usando los puertos existentes.
3. Subcomando en `cli/<stack>.rs`.
4. Tests con fakes y un test de integración `#[ignore]`.

## Fuera de alcance por ahora

- Plantillas con `tera`
- Configuración por archivo (`forja.toml`)
- Generación de Docker, Nginx o CI (se añadirá después como opción)
