# Demo Pet Store con Hodei Permissions SDK

Esta demo muestra una integración completa de **autenticación con Keycloak** y **autorización con Hodei Verified Permissions** usando el SDK de Rust. Incluye:

- Backend Pet Store escrito con Axum y el middleware `VerifiedPermissionsLayer`
- Entorno Docker Compose (PostgreSQL, Keycloak, servidor AVP, demo app)
- Scripts para inicializar Keycloak, sembrar el policy store y ejecutar verificaciones end-to-end

## Prerrequisitos

- Docker & Docker Compose
- Toolchain de Rust
- `jq` y `grpcurl` instalados (utilizados por los scripts)

## Inicio Rápido

```bash
cd examples
make env          # copia .env.example a .env
make up           # levanta postgres, keycloak, avp, demo-app
make keycloak     # (opcional) bootstrap de Keycloak
make seed         # carga policy store, identity source, schema y políticas
make demo-run     # ejecutar app localmente (opcional)
```

### Servicios

- Keycloak: `http://localhost:8080`
- Hodei AVP gRPC: `localhost:50051`
- Demo app: `http://localhost:3000`
- PostgreSQL: `localhost:5432`

## Flujo de Autenticación

1. Keycloak se configura con realm `demo`, cliente `demo-app` y usuarios demo (`pet_admin`, `pet_vet`, `pet_customer`).
2. Obtener tokens JWT:

```bash
./scripts/get_token.sh pet_admin Password123!
```

O manualmente:

```bash
curl -X POST "http://localhost:8080/realms/demo/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=password" \
  -d "client_id=demo-app" \
  -d "client_secret=demo-secret" \
  -d "username=pet_admin" \
  -d "password=Password123!"
```

## Configuración de Autorización

`make seed` realiza:

- Creación del policy store `demo-policy-store`
- Carga del esquema Cedar (`schema/pet_store_schema.json`)
- Creación del identity source apuntando a Keycloak
- Carga de políticas Cedar (`policies/`):
  - `allow_admin_full_access.cedar`
  - `allow_vet_manage_appointments.cedar`
  - `allow_customer_view_pets.cedar`

## Aplicación Demo (Pet Store)

Rutas protegidas con `VerifiedPermissionsLayer`:

- `GET /health`
- `GET /pets`
- `POST /pets` (solo admin)
- `GET /pets/:id`
- `GET /pets/:id/appointments`
- `POST /pets/:id/appointments` (solo vet)

## Tests de Autorización

Ejecuta el script e2e:

```bash
./scripts/check_end_to_end.sh
```

Resultados esperados:

- `pet_admin`: acceso total
- `pet_vet`: puede listar mascotas y crear citas, pero no crear mascotas
- `pet_customer`: solo lectura

## Limpieza

```bash
make down        # detiene servicios
make clean       # detiene y elimina volúmenes
```

## Troubleshooting

- Asegúrate de que Keycloak esté listo antes de `make keycloak`/`make seed`
- Instala `grpcurl` y `jq`
- Ajusta puertos u otras variables en `.env`
- Si los tokens fallan, resetea `.env` y scripts
