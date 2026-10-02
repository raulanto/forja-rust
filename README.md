# forja

Generador de proyectos en Rust. Ejecuta las herramientas oficiales de cada stack y deja la estructura lista para trabajar.

Generadores disponibles:

| Comando | Descripción |
|---|---|
| `forja django <nombre>` | Proyecto Django con uv y estructura `apps/` |

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

## Arquitectura

Hexagonal: `domain` → `application` (casos de uso y puertos) → `infrastructure` (procesos, filesystem, parches de texto) → `cli` (clap). Detalles en [CLAUDE.md](CLAUDE.md).

## Desarrollo

```bash
cargo test
cargo test -- --ignored   # integración real, requiere uv
cargo fmt && cargo clippy -- -D warnings
```

## Roadmap

- [ ] `forja django` (en progreso)
- [ ] `forja axum`
- [ ] `forja nest`
- [ ] `forja go-hex`
- [ ] Opciones de Docker, Nginx y GitHub Actions

## Licencia

MIT
