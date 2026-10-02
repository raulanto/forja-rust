# forja

Generador de proyectos en Rust. Ejecuta las herramientas oficiales de cada stack y deja la estructura lista para trabajar.

Generadores disponibles:

| Comando | Descripción |
|---|---|
| `forja django <nombre>` | Proyecto Django con uv y estructura `apps/` |
| `forja axum <nombre>` | API en Rust (Axum) con arquitectura hexagonal |
| `forja go-hex <nombre>` | Servicio en Go con arquitectura hexagonal (HTTP / gRPC) |

## Requisitos

- [Rust](https://rustup.rs) (stable)
- [uv](https://docs.astral.sh/uv/) instalado y en el `PATH` (para `forja django`)
- [Go](https://go.dev/) (1.22+) instalado y en el `PATH` (para `forja go-hex`)


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
forja go-hex mi_servicio
```

Opciones:

| Flag | Default | Descripción |
|---|---|---|
| `--module <nombre>` | igual al nombre | Nombre del módulo para `go mod init` |
| `--transport <http\|grpc\|both>` | `http` | Transportes habilitados |
| `--db <postgres\|sqlite>` | `postgres` | Driver de base de datos |
| `--docker` | desactivado | Genera `Dockerfile` y `docker-compose.yml` |

Pasos: comprobación de `go` en `PATH`, `go mod init`, renderizado de plantillas, `go get`, `go mod tidy`, `go build ./...` y `go vet ./...`.

Resultado:

```
mi_servicio/
├── cmd/api/main.go              # único lugar de wiring
├── internal/
│   ├── domain/user/
│   ├── application/user/        # ports.go + service.go
│   ├── adapters/
│   │   ├── inbound/httpapi/     # y grpcapi/ si aplica
│   │   └── outbound/postgres/
│   └── platform/{config,logger}/
├── api/proto/                   # solo con gRPC
├── migrations/
└── Makefile
```

Usa la biblioteca estándar Go siempre que es posible (`net/http` con `ServeMux`, `log/slog` y `signal.NotifyContext`).

## Arquitectura

Hexagonal: `domain` → `application` (casos de uso y puertos) → `infrastructure` (procesos, filesystem, parches de texto) → `cli` (clap). Detalles en [CLAUDE.md](CLAUDE.md).

## Desarrollo

```bash
cargo test
cargo test -- --ignored   # integración real, requiere uv/go
cargo fmt && cargo clippy -- -D warnings
```

## Roadmap

- [x] `forja django`
- [x] `forja axum`
- [x] `forja axum --auth`
- [x] `forja go-hex`
- [ ] `forja nest`
- [ ] Nginx y GitHub Actions como opciones

## Licencia

MIT