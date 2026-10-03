# forja

Generador de proyectos en Rust. Ejecuta las herramientas oficiales de cada stack y deja la estructura lista para trabajar.

Generadores disponibles:

| Comando | Descripción |
|---|---|
| `forja django <nombre>` | Proyecto Django con uv y estructura `apps/` |
| `forja axum <nombre>` | API en Rust (Axum) con arquitectura hexagonal |
| `forja go-hex <nombre>` | Servicio en Go con arquitectura hexagonal (HTTP y/o gRPC) |
| `forja fastapi <nombre>` | API en Python (FastAPI) con arquitectura hexagonal y uv |

## Requisitos

- [Rust](https://rustup.rs) (stable)
- [uv](https://docs.astral.sh/uv/) instalado y en el `PATH`

## Instalación

```bash
git clone <repo> forja && cd forja
cargo install --path .
```

## Uso

```bash
forja django mi_proyecto
```

Opciones:

| Flag | Default | Descripción |
|---|---|---|
| `--django-version <v>` | `6.1.1` | Versión de Django a fijar |
| `--app <nombre>` | `core` | Nombre de la app inicial |
| `--run` | desactivado | Levanta `runserver` al terminar |

Ejemplo:

```bash
forja django tienda --app catalogo --django-version 6.0.1 --run
```

## Qué hace `forja django`

Equivale a ejecutar:

```bash
uv init mi_proyecto && cd mi_proyecto
uv add "django==6.1.1"
uv run django-admin startproject config .

mkdir apps && touch apps/__init__.py
uv run python manage.py startapp core apps/core

# ajustes automáticos
#   apps/core/apps.py     -> name = "apps.core"
#   config/settings.py    -> INSTALLED_APPS += ["apps.core"]

uv run python manage.py migrate
```

Resultado:

```
mi_proyecto/
├── pyproject.toml
├── manage.py
├── config/
│   ├── settings.py
│   └── urls.py
└── apps/
    ├── __init__.py
    └── core/
```

## Qué hace `forja axum`

```bash
forja axum mi_api
```

Opciones:

| Flag | Default | Descripción |
|---|---|---|
| `--db <postgres\|sqlite>` | `postgres` | Base de datos para `sqlx` |
| `--docker` | desactivado | Genera `Dockerfile` y `docker-compose.yml` |
| `--auth` | desactivado | Autenticación JWT con refresh tokens y roles |

Pasos: `cargo new`, `cargo add` de dependencias, renderizado de plantillas y `cargo check` final para confirmar que compila.

Resultado:

```
mi_api/
├── Cargo.toml
├── .env.example
├── migrations/
├── src/
│   ├── main.rs, lib.rs, config.rs
│   ├── domain/user/
│   ├── application/user/
│   ├── infrastructure/persistence/
│   └── presentation/http/
│       ├── router.rs, state.rs, error.rs
│       ├── dto/
│       └── handlers/
└── tests/api_health.rs
```

Incluye un módulo `user` de ejemplo que recorre todas las capas, un endpoint `/health`, tracing y graceful shutdown.

### Con `--auth`

```bash
forja axum mi_api --auth
```

Agrega registro, login, refresh con rotación, logout, ruta protegida (`/me`) y guard por rol (`/admin/ping`).

| Método | Ruta | Descripción |
|---|---|---|
| POST | `/auth/register` | Crea usuario |
| POST | `/auth/login` | Access token + refresh token |
| POST | `/auth/refresh` | Rota el refresh token |
| POST | `/auth/logout` | Revoca el refresh token |
| GET | `/me` | Usuario autenticado |
| GET | `/admin/ping` | Solo rol `admin` |

Seguridad por defecto: contraseñas con argon2id, access token de 15 min, refresh token opaco guardado como hash y rotado en cada uso (con detección de reutilización), y `JWT_SECRET` aleatorio generado en `.env`.

## Qué hace `forja go-hex`

```bash
forja go-hex mi_servicio --module github.com/usuario/mi_servicio
```

Requiere [Go](https://go.dev/dl/) en el `PATH`.

Opciones:

| Flag | Default | Descripción |
|---|---|---|
| `--module <ruta>` | `<nombre>` | Ruta del módulo de Go |
| `--transport <http\|grpc\|both>` | `http` | Adaptador de entrada |
| `--db <postgres\|sqlite>` | `postgres` | Adaptador de persistencia |
| `--docker` | desactivado | Genera `Dockerfile` y `docker-compose.yml` |

Pasos: `go mod init`, renderizado de plantillas, `go get`, `go mod tidy` y `go build ./... && go vet ./...` final para confirmar que compila.

Resultado:

```
mi_servicio/
├── cmd/api/main.go
├── internal/
│   ├── domain/user/
│   ├── application/user/
│   ├── adapters/
│   │   ├── inbound/httpapi/   (y grpcapi/ si aplica)
│   │   └── outbound/postgres/
│   └── platform/{config,logger}/
├── api/proto/                 (solo con gRPC)
├── migrations/
└── Makefile
```

Usa solo biblioteca estándar donde se puede (`net/http`, `log/slog`), incluye `/health`, graceful shutdown y un módulo `user` de ejemplo que recorre todas las capas.

## Qué hace `forja fastapi`

```bash
forja fastapi mi_api
```

Opciones:

| Flag | Default | Descripción |
|---|---|---|
| `--db <postgres\|sqlite>` | `postgres` | Base de datos (SQLAlchemy async + Alembic) |
| `--docker` | desactivado | Genera `Dockerfile` y `docker-compose.yml` |
| `--auth` | desactivado | Reservado para JWT (fase posterior) |

Pasos: `uv init --package`, `uv add` de dependencias, renderizado de plantillas y validación final con `ruff` y `pytest`.

Resultado:

```
mi_api/
├── pyproject.toml
├── alembic.ini
├── migrations/
├── src/mi_api/
│   ├── main.py, config.py
│   ├── domain/user/
│   ├── application/user/
│   ├── infrastructure/persistence/
│   └── presentation/http/
│       ├── router.py, dependencies.py, errors.py
│       ├── schemas/
│       └── routes/
└── tests/{unit,integration}/
```

Dominio en Python puro, puertos como `Protocol`, Pydantic solo en la capa HTTP, SQLAlchemy 2.0 async, `/health` y un módulo `user` de ejemplo que recorre todas las capas. Configura `ruff` y `mypy --strict` desde el inicio.

## Arquitectura

Hexagonal: `domain` → `application` (casos de uso y puertos) → `infrastructure` (procesos, filesystem, parches de texto) → `cli` (clap). Detalles en [CLAUDE.md](CLAUDE.md).

## Desarrollo

```bash
cargo test
cargo test -- --ignored   # integración real, requiere uv
cargo fmt && cargo clippy -- -D warnings
```

## Roadmap

- [x] `forja django`
- [x] `forja axum` 
- [x] `forja axum --auth`
- [x] `forja go-hex`
- [x] `forja fastapi`
- [ ] `forja nest`
- [ ] Nginx y GitHub Actions como opciones


## Licencia

MIT