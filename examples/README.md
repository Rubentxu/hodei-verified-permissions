# Pet Store Demo with Hodei Permissions SDK

This example demonstrates a full integration of **authentication with Keycloak** and **authorization with Hodei Verified Permissions** using the Rust SDK. The demo includes:

- A Pet Store backend written with Axum and the `VerifiedPermissionsLayer` middleware
- Docker Compose environment (PostgreSQL, Keycloak, AVP server, demo app)
- Scripts to bootstrap Keycloak, seed the policy store, and run end-to-end checks

## Prerequisites

- Docker & Docker Compose
- Rust toolchain (for local builds)
- `jq` and `grpcurl` installed locally (used by helper scripts)

## Quick Start

```bash
cd examples
make env          # copies .env.example to .env
make up           # builds and starts postgres, keycloak, avp, demo-app
make keycloak     # (optional) run in a separate shell to bootstrap Keycloak
make seed         # seed policy store, identity source, schema, and policies
make demo-run     # run app locally instead of the container (optional)
```

### Services

- Keycloak: `http://localhost:8080`
- Hodei AVP gRPC: `localhost:50051`
- Demo app: `http://localhost:3000`
- PostgreSQL: `localhost:5432`

## Authentication Flow

1. Keycloak is configured with:
   - Realm `demo`
   - Client `demo-app`
   - Users:
     - `pet_admin` (role: `pet-admin`)
     - `pet_vet` (role: `pet-vet`)
     - `pet_customer` (role: `pet-customer`)
2. Obtain JWT tokens using the script:

```bash
./scripts/get_token.sh pet_admin Password123!
```

Or via raw curl:

```bash
curl -X POST "http://localhost:8080/realms/demo/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=password" \
  -d "client_id=demo-app" \
  -d "client_secret=demo-secret" \
  -d "username=pet_admin" \
  -d "password=Password123!"
```

## Authorization Setup

`make seed` performs the following:

- Creates policy store `demo-policy-store`
- Uploads Cedar schema (`schema/pet_store_schema.json`)
- Creates identity source pointing to Keycloak realm `demo`
- Loads policies from `policies/`:
  - `allow_admin_full_access.cedar`
  - `allow_vet_manage_appointments.cedar`
  - `allow_customer_view_pets.cedar`

## Demo App (Pet Store)

Routes exposed from `examples/app/src/main.rs`:

- `GET /health` – health check
- `GET /pets` – list pets
- `POST /pets` – create pet (admin only)
- `GET /pets/:id` – view pet details
- `GET /pets/:id/appointments` – list appointments for a pet
- `POST /pets/:id/appointments` – schedule appointment (vet only)

The app reads configuration from `examples/app/.env` and applies `VerifiedPermissionsLayer` to all routes.

## Testing Authorization

Run the end-to-end script after services are up:

```bash
./scripts/check_end_to_end.sh
```

Expected results:

- `pet_admin`: can list pets, create pets, and schedule appointments
- `pet_vet`: can list pets and schedule appointments, but cannot create pets
- `pet_customer`: can only list pets and view their own data

## Cleaning Up

```bash
make down        # stop services
make clean       # stop services and remove volumes
```

If you ran `make demo-run` manually, stop the process with `Ctrl+C`.

## Troubleshooting

- Ensure Keycloak is fully started before running `make keycloak` or `make seed`
- `grpcurl` required: install from https://github.com/fullstorydev/grpcurl
- `jq` required: install via package manager (e.g., `sudo apt install jq`)
- Ports in use: adjust `.env` to change exposed ports
- JWT verification issues: clear `examples/.env` and rerun setup scripts if tokens fail due to secret or realm mismatch
