# Makefile Guide - Hodei Verified Permissions

Este documento explica cómo usar el `Makefile` para ejecutar tests, compilar y gestionar la infraestructura del proyecto.

---

## 📋 Tabla de Contenidos

1. [Comandos Rápidos](#comandos-rápidos)
2. [Build](#build)
3. [Tests](#tests)
4. [Docker](#docker)
5. [Desarrollo](#desarrollo)
6. [Ejemplos](#ejemplos)

---

## 🚀 Comandos Rápidos

```bash
# Ver todos los comandos disponibles
make help

# Ver información del proyecto
make info

# Ejecutar tests unitarios (38 tests)
make test-unit

# Ejecutar todos los tests (unit + integration)
make test

# Ejecutar tests E2E (requiere Docker)
make test-e2e

# Compilar en release mode
make build

# Validación rápida (check + lint + unit tests)
make validate

# Pipeline CI completo
make ci
```

---

## 🏗️ Build

### `make build`
Compila todos los paquetes en **release mode** (optimizado para producción).

```bash
make build
```

**Salida esperada:**
```
Building all packages (release mode)...
✅ Build completed successfully
```

### `make build-debug`
Compila todos los paquetes en **debug mode** (más rápido, menos optimizado).

```bash
make build-debug
```

### `make clean`
Limpia todos los artefactos de build.

```bash
make clean
```

---

## 🧪 Tests

### `make test-unit`
Ejecuta **solo los tests unitarios** (38 tests).

```bash
make test-unit
```

**Características:**
- ✅ 38 tests pasando
- ⏱️ ~5 segundos
- 📍 No requiere infraestructura externa

**Salida esperada:**
```
Running Unit Tests (38 tests)
...
test result: ok. 38 passed; 0 failed
✅ Unit tests passed!
```

### `make test`
Ejecuta **tests unitarios + integration tests**.

```bash
make test
```

### `make test-integration`
Ejecuta **solo los tests de integración**.

```bash
make test-integration
```

### `make test-e2e`
Ejecuta **tests E2E con infraestructura SQLite**.

```bash
make test-e2e
```

**Requisitos:**
- Docker instalado
- Puertos 50051 (Hodei Server) y 3000 (TODO App) disponibles

**Proceso:**
1. Levanta contenedores Docker (SQLite profile)
2. Espera 15 segundos a que los servicios estén listos
3. Ejecuta tests E2E
4. Los contenedores permanecen corriendo (usar `make docker-down` para detenerlos)

### `make test-e2e-full`
Ejecuta **todos los tests E2E** con todos los perfiles (SQLite, PostgreSQL, SurrealDB).

```bash
make test-e2e-full
```

**Requisitos:**
- Docker instalado
- Puertos 50051-50053, 3000-3002, 5432, 8001 disponibles

**Duración:** ~5-10 minutos

### `make test-all`
Ejecuta **todos los tests** (unit + integration + E2E).

```bash
make test-all
```

---

## 🐳 Docker

### `make docker-up`
Levanta contenedores con el **perfil SQLite** (más rápido).

```bash
make docker-up
```

**Servicios levantados:**
- Hodei Server (gRPC): `http://localhost:50051`
- TODO App: `http://localhost:3000`

### `make docker-up-all`
Levanta **todos los contenedores** (todos los perfiles).

```bash
make docker-up-all
```

**Servicios levantados:**
- Hodei Server (SQLite): `http://localhost:50051`
- Hodei Server (PostgreSQL): `http://localhost:50052`
- Hodei Server (SurrealDB): `http://localhost:50053`
- TODO App (SQLite): `http://localhost:3000`
- TODO App (PostgreSQL): `http://localhost:3001`
- TODO App (SurrealDB): `http://localhost:3002`
- PostgreSQL: `localhost:5432`
- SurrealDB: `localhost:8001`

### `make docker-down`
Detiene los contenedores Docker.

```bash
make docker-down
```

### `make docker-logs`
Muestra los logs de los contenedores en tiempo real.

```bash
make docker-logs
```

### `make docker-clean`
Detiene y **elimina** todos los contenedores y volúmenes.

```bash
make docker-clean
```

### `make docker-status`
Muestra el estado actual de los contenedores.

```bash
make docker-status
```

---

## 👨‍💻 Desarrollo

### `make fmt`
Formatea el código usando `rustfmt`.

```bash
make fmt
```

### `make lint`
Ejecuta `clippy` para detectar problemas de código.

```bash
make lint
```

### `make check`
Verifica el código sin compilar (más rápido que `build`).

```bash
make check
```

### `make doc`
Genera documentación y la abre en el navegador.

```bash
make doc
```

### `make validate`
Ejecuta validación rápida: `check` + `lint` + `test-unit`.

```bash
make validate
```

### `make ci`
Ejecuta el pipeline CI completo: `build` + `fmt` + `lint` + `test-unit`.

```bash
make ci
```

### `make dev-setup`
Prepara el ambiente de desarrollo: `build` + `fmt` + `lint`.

```bash
make dev-setup
```

### `make watch`
Observa cambios en el código y ejecuta tests automáticamente.

```bash
make watch
```

---

## 📚 Ejemplos

### Ejemplo 1: Desarrollo Local Rápido

```bash
# 1. Preparar ambiente
make dev-setup

# 2. Hacer cambios en el código

# 3. Validar cambios
make validate

# 4. Si todo está bien, hacer commit
git add .
git commit -m "Mi cambio"
```

### Ejemplo 2: Ejecutar Tests Antes de Push

```bash
# Ejecutar pipeline CI completo
make ci

# Si todo pasa, hacer push
git push origin main
```

### Ejemplo 3: Debugging con Tests E2E

```bash
# 1. Levantar infraestructura
make docker-up

# 2. Ejecutar tests E2E con logs
make docker-logs &  # En otra terminal
make test-e2e

# 3. Cuando termines
make docker-down
```

### Ejemplo 4: Validación Completa Antes de Release

```bash
# 1. Ejecutar todos los tests
make test-all

# 2. Generar documentación
make doc

# 3. Compilar en release mode
make build

# 4. Si todo está bien, crear tag
git tag -a v1.0.0 -m "Release 1.0.0"
git push origin v1.0.0
```

### Ejemplo 5: Limpiar y Empezar de Cero

```bash
# Limpiar todo
make clean
make docker-clean

# Recompilar
make build

# Ejecutar tests
make test-unit
```

---

## 📊 Resumen de Comandos

| Comando | Descripción | Duración | Requisitos |
|---------|-------------|----------|-----------|
| `make test-unit` | Tests unitarios (38) | ~5s | Ninguno |
| `make test` | Unit + Integration | ~10s | Ninguno |
| `make test-e2e` | E2E con SQLite | ~2-3min | Docker |
| `make test-e2e-full` | E2E con todos | ~5-10min | Docker |
| `make test-all` | Todos los tests | ~10-15min | Docker |
| `make build` | Build release | ~1-2min | Ninguno |
| `make validate` | Validación rápida | ~30s | Ninguno |
| `make ci` | Pipeline CI | ~2-3min | Ninguno |

---

## 🎯 Flujo de Trabajo Recomendado

### Para Desarrollo Local

```bash
# 1. Inicio del día
make dev-setup

# 2. Durante desarrollo
make validate  # Después de cada cambio importante

# 3. Antes de commit
make ci

# 4. Antes de push
make test-all
```

### Para CI/CD

```bash
# En el servidor CI
make ci  # Rápido (unit tests)
make test-all  # Completo (con E2E)
```

---

## ⚠️ Troubleshooting

### Error: "docker: command not found"
**Solución:** Instalar Docker
```bash
sudo apt install docker.io
sudo usermod -aG docker $USER
```

### Error: "Port 50051 already in use"
**Solución:** Detener contenedores existentes
```bash
make docker-down
# o
docker ps -a
docker stop <container_id>
```

### Error: "Permission denied" en Docker
**Solución:** Agregar usuario al grupo docker
```bash
sudo usermod -aG docker $USER
newgrp docker
```

### Tests E2E lentos
**Solución:** Usar perfil SQLite en lugar de todos
```bash
make test-e2e  # Más rápido
# en lugar de
make test-e2e-full  # Más lento
```

---

## 📖 Más Información

- [TESTS_EXPLANATION.md](TESTS_EXPLANATION.md) - Explicación detallada de los tests
- [README.md](README.md) - Documentación general del proyecto
- [Cargo.toml](Cargo.toml) - Configuración de dependencias

