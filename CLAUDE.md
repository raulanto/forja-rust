# CLAUDE.md

## Qué es este proyecto

`forja` es un CLI en Rust que genera proyectos desde cero ejecutando las herramientas oficiales de cada stack y aplicando la estructura que prefiero. Generadores actuales: **Django con uv**, **Axum (hexagonal)**, **Go hexagonal** y **FastAPI (hexagonal)**. Los siguientes (Angular, NestJS) se agregan como nuevos subcomandos.

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
│   ├── axum.rs
│   ├── go_hex.rs
│   └── fastapi.rs
├── domain/                  # sin dependencias externas
│   ├── project_spec.rs      # DjangoSpec, AxumSpec, GoHexSpec y FastApiSpec
│   └── error.rs
├── application/
│   ├── ports.rs             # traits CommandRunner, FileSystem y TemplateRenderer
│   ├── generate_django.rs   # caso de uso
│   ├── generate_axum.rs     # caso de uso
│   ├── generate_go_hex.rs   # caso de uso
│   └── generate_fastapi.rs  # caso de uso
└── infrastructure/          # adaptadores de salida
    ├── process_runner.rs    # std::process::Command
    ├── fs.rs                # std::fs
    ├── template_renderer.rs # tera con plantillas embebidas
    └── settings_patcher.rs  # edita settings.py y apps.py

templates/
├── axum/                    # plantillas (.tera) del proyecto generado
├── go-hex/
└── fastapi/
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

## Comando: `forja go-hex <nombre>`

Flags:
- `--module <ruta>` módulo de Go (default: `<nombre>`; ej. `github.com/rau/mi_servicio`)
- `--transport <http|grpc|both>` (default `http`)
- `--db <postgres|sqlite>` (default `postgres`)
- `--docker` genera `Dockerfile` y `docker-compose.yml` (por defecto no)

Pasos, en orden:

1. Verificar que `go` está en el `PATH`
2. Crear el directorio y ejecutar `go mod init <módulo>`
3. Renderizar plantillas (estructura hexagonal y módulo `user` de ejemplo)
4. `go get` de dependencias según flags: `pgx/v5` (postgres) o `modernc.org/sqlite`; `google.golang.org/grpc` y `google.golang.org/protobuf` si hay gRPC
5. Si `--docker`: renderizar Dockerfile y compose
6. `go mod tidy`
7. `go build ./...` y `go vet ./...` para validar que el proyecto generado compila

Estructura del proyecto generado:

```
mi_servicio/
├── go.mod
├── Makefile                     # run, test, lint, build
├── .env.example
├── cmd/
│   └── api/main.go              # composition root: config, wiring, serve, shutdown
├── internal/
│   ├── domain/user/             # user.go (entidad, value objects), errors.go
│   ├── application/user/        # ports.go, service.go, service_test.go
│   ├── adapters/
│   │   ├── inbound/
│   │   │   ├── httpapi/         # router, handlers, dto, middleware, errors
│   │   │   └── grpcapi/         # solo con --transport grpc|both
│   │   └── outbound/
│   │       └── postgres/        # user_repository.go (o sqlite/)
│   └── platform/
│       ├── config/              # variables de entorno tipadas
│       └── logger/              # log/slog
├── api/proto/                   # .proto, solo con gRPC
└── migrations/
```

Reglas del proyecto generado:
- Dependencias: `adapters/inbound → application → domain`; `adapters/outbound` implementa los puertos de `application`. `domain` no importa nada del proyecto.
- Las interfaces (puertos) viven en `application`, del lado del consumidor, y son pequeñas.
- Todo bajo `internal/` para que nada se importe desde fuera del módulo.
- Único lugar de wiring: `cmd/api/main.go`. Sin globals ni `init()` con efectos.
- `context.Context` como primer parámetro en casos de uso y repositorios.
- Errores de dominio como sentinelas (`ErrUserNotFound`); los adaptadores los traducen con `errors.Is` (HTTP → status, gRPC → `codes`).
- Solo biblioteca estándar donde sea posible: `net/http` con `ServeMux` (patrones de Go 1.22+), `log/slog`, `signal.NotifyContext` para graceful shutdown.
- Incluye `/health` y logging estructurado desde el inicio.

Testing:
- Los tests de `generate_go_hex` usan fakes de `CommandRunner`, `FileSystem` y `TemplateRenderer`; verifican archivos y comandos pedidos, y que `grpcapi/` y `api/proto/` solo existan con gRPC.
- Plantillas con tests de snapshot.
- Un test `#[ignore]` genera un proyecto real por cada combinación de `--transport` y ejecuta `go build ./...`.
- El proyecto generado trae tests de tabla para el servicio y un test de handlers con `httptest`.

## Comando: `forja fastapi <nombre>`

Flags:
- `--db <postgres|sqlite>` (default `postgres`)
- `--docker` genera `Dockerfile` y `docker-compose.yml` (por defecto no)
- `--auth` reservado para la fase de auth (mismo diseño que Axum); por ahora no hace nada

Pasos, en orden:

1. Verificar que `uv` está en el `PATH`
2. `uv init --package <nombre>` (layout `src/`)
3. `uv add fastapi "uvicorn[standard]" pydantic-settings "sqlalchemy[asyncio]" alembic` más el driver (`asyncpg` o `aiosqlite`)
4. `uv add --dev pytest pytest-asyncio httpx ruff mypy aiosqlite`
5. Renderizar plantillas (estructura hexagonal, módulo `user` de ejemplo, `alembic.ini` y config de `ruff`/`mypy`/`pytest` en `pyproject.toml`)
6. Si `--docker`: renderizar Dockerfile y compose
7. Validar el proyecto generado: `uv run ruff check` y `uv run pytest -q`

Estructura del proyecto generado (el paquete toma el nombre normalizado, con `_` en vez de `-`):

```
mi_api/
├── pyproject.toml
├── .env.example
├── alembic.ini
├── migrations/
├── src/mi_api/
│   ├── main.py                    # create_app() y lifespan
│   ├── config.py                  # pydantic-settings
│   ├── domain/user/               # entity.py (dataclass frozen, Email VO), errors.py
│   ├── application/user/          # ports.py (Protocol), use_cases.py
│   ├── infrastructure/persistence/
│   │   ├── database.py            # engine y async_sessionmaker
│   │   ├── models.py              # modelos ORM, separados del dominio
│   │   └── user_repository.py     # implementa el puerto y mapea ORM ↔ dominio
│   └── presentation/http/
│       ├── router.py
│       ├── dependencies.py        # wiring: sesión → repositorio → caso de uso
│       ├── errors.py              # exception handlers
│       ├── schemas/               # Pydantic request/response
│       └── routes/                # health.py, users.py
└── tests/
    ├── unit/                      # casos de uso con repositorio en memoria
    └── integration/               # httpx.AsyncClient + SQLite en memoria
```

Reglas del proyecto generado:
- Dependencias: `presentation → application → domain`; `infrastructure` implementa los puertos de `application`.
- `domain` es Python puro: `dataclass(frozen=True)` y excepciones propias; sin Pydantic ni SQLAlchemy.
- Los puertos son `typing.Protocol`; los casos de uso son clases con un método `execute`.
- Pydantic solo en `presentation` (schemas); los modelos ORM solo en `infrastructure`. Los repositorios mapean entre ORM y dominio.
- Async de punta a punta (SQLAlchemy 2.0 async).
- Transacción por request: la dependencia `get_session` hace commit si todo va bien y rollback si hay excepción.
- Un único lugar de wiring (`dependencies.py`), con `Depends`; los casos de uso nunca importan `fastapi`.
- Las excepciones de dominio se traducen a HTTP en `errors.py`; los handlers no tienen lógica de negocio.
- Sin repositorios base genéricos ni capas extra «por si acaso»: hexagonal pragmático.
- Config validada al arrancar; `/health` y logging desde el inicio.
- `ruff` y `mypy --strict` configurados desde el primer commit.

Testing:
- Los tests de `generate_fastapi` usan fakes de `CommandRunner`, `FileSystem` y `TemplateRenderer`; verifican comandos, archivos y el nombre de paquete normalizado.
- Plantillas con tests de snapshot.
- Un test `#[ignore]` genera un proyecto real y ejecuta `uv run pytest` sobre él.
- El proyecto generado trae tests unitarios (con repositorio en memoria) y de integración (`httpx.AsyncClient` con `ASGITransport`).

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
cargo run -- go-hex demo_svc --transport http
cargo run -- fastapi demo_api --db postgres
```

## Cómo agregar un generador nuevo

1. Spec en `domain/` (por ejemplo `AxumSpec`).
2. Caso de uso en `application/generate_<stack>.rs` usando los puertos existentes.
3. Subcomando en `cli/<stack>.rs`.
4. Tests con fakes y un test de integración `#[ignore]`.

## Fuera de alcance por ahora

- OAuth/OIDC, RS256, 2FA y recuperación de contraseña: fases posteriores
- `--auth` para go-hex y fastapi: fase posterior (mismo diseño que Axum)
- Configuración por archivo (`forja.toml`)
- Nginx y CI (Docker solo existe como `forja axum --docker`)