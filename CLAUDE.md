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
- `--auth` añade el módulo de autenticación JWT (sección «Fase auth»)

Pasos, en orden:

1. `cargo new <nombre>`
2. `cargo add` de las dependencias: `axum`, `tokio` (full), `tower-http` (trace, cors), `serde` (derive), `sqlx` (según `--db`), `tracing`, `tracing-subscriber`, `thiserror`, `anyhow`, `dotenvy`, `uuid`
3. Renderizar plantillas (estructura hexagonal y módulo `user` de ejemplo)
4. Si `--auth`: `cargo add` de `jsonwebtoken`, `argon2`, `rand`, `sha2`, `time`; renderizar plantillas y migraciones de auth; generar `.env` con `JWT_SECRET` aleatorio
5. Si `--docker`: renderizar Dockerfile y compose
6. `cargo check` para validar que el proyecto generado compila

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

## Fase auth: `forja axum <nombre> --auth`

Agrega autenticación JWT con refresh tokens y roles al proyecto generado. Requiere el módulo `user` (siempre se genera).

Endpoints:

| Método | Ruta | Descripción |
|---|---|---|
| POST | `/auth/register` | Crea usuario (rol `user`) |
| POST | `/auth/login` | Devuelve access token y refresh token |
| POST | `/auth/refresh` | Rota el refresh token y emite un nuevo access token |
| POST | `/auth/logout` | Revoca el refresh token |
| GET | `/me` | Usuario autenticado (ruta protegida de ejemplo) |
| GET | `/admin/ping` | Ruta solo para rol `admin` (ejemplo de guard por rol) |

Capas añadidas al proyecto generado:

```
src/
├── domain/auth/
│   ├── role.rs              # enum Role { Admin, User }
│   ├── refresh_token.rs     # entidad RefreshToken (familia, expiración, revocado)
│   └── error.rs             # AuthError
├── application/auth/
│   ├── ports.rs             # PasswordHasher, TokenService, RefreshTokenRepository
│   ├── register.rs
│   ├── login.rs
│   ├── refresh.rs
│   └── logout.rs
├── infrastructure/
│   ├── security/
│   │   ├── argon2_hasher.rs # impl PasswordHasher (argon2id)
│   │   └── jwt_service.rs   # impl TokenService (jsonwebtoken, HS256)
│   └── persistence/
│       └── refresh_token_repo.rs
└── presentation/http/
    ├── extractors/
    │   ├── auth_user.rs     # AuthUser: FromRequestParts, valida el Bearer
    │   └── require_role.rs  # RequireRole<R>: guard por rol
    ├── dto/auth.rs
    └── handlers/auth.rs
migrations/                  # users += password_hash, role; tabla refresh_tokens
```

Decisiones de seguridad (no negociables):
- Contraseñas con **argon2id**; nunca se loguean ni se devuelven.
- Access token de vida corta (default 15 min); claims mínimos: `sub`, `role`, `iat`, `exp`, `jti`.
- Refresh token **opaco** (32 bytes aleatorios), guardado como hash SHA-256 en BD, nunca en claro.
- **Rotación** en cada uso. Si se reutiliza un refresh ya rotado, se revoca toda su familia.
- Login con mensaje de error genérico, y hash de relleno cuando el usuario no existe, para evitar enumeración por mensaje o por tiempo.
- `JWT_SECRET` se valida al arrancar (mínimo 32 bytes); si falta o es corto, la app no inicia.
- Config en `.env`: `JWT_SECRET`, `ACCESS_TOKEN_TTL_MINUTES`, `REFRESH_TOKEN_TTL_DAYS`. El `.env` generado queda en `.gitignore`; `.env.example` solo lleva placeholders.

Reglas de arquitectura:
- `application/auth` solo conoce los puertos; no importa `jsonwebtoken` ni `argon2`.
- Los extractores de `presentation` dependen del puerto `TokenService`, no de la implementación.
- Cambiar HS256 por RS256 debe implicar solo un adaptador nuevo en `infrastructure/security`.

Testing:
- Casos de uso con fakes de `PasswordHasher`, `TokenService` y `RefreshTokenRepository`.
- Integración con `tower::ServiceExt`: registro → login → `/me` → refresh → reutilizar el refresh viejo (debe fallar y revocar la familia) → `/admin/ping` con rol `user` (403).
- Cada test de plantilla verifica que, sin `--auth`, no se genere ningún archivo de auth.

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

- OAuth/OIDC, RS256, 2FA y recuperación de contraseña: fases posteriores
- Configuración por archivo (`forja.toml`)
- Nginx y CI (Docker solo existe como `forja axum --docker`)