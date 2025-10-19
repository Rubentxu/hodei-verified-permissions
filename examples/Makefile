DOCKER_COMPOSE := docker compose -f docker-compose.yml --env-file .env

.PHONY: help env up down logs keycloak seed demo-build demo-run clean

help:
	@echo "Objetivos disponibles:"
	@echo "  make env           -> Copia .env.example a .env"
	@echo "  make up            -> Levanta la infraestructura demo"
	@echo "  make down          -> Detiene y limpia servicios"
	@echo "  make logs          -> Muestra logs de todos los servicios"
	@echo "  make keycloak      -> Ejecuta bootstrap de Keycloak"
	@echo "  make seed          -> Carga policy store, identity source y políticas"
	@echo "  make demo-build    -> Compila la app Axum de ejemplo"
	@echo "  make demo-run      -> Ejecuta la app Axum localmente"
	@echo "  make clean         -> Limpia volúmenes y binarios"

env:
	@test -f .env || cp .env.example .env
	@echo ".env preparado"

up: env
	$(DOCKER_COMPOSE) up -d --build

keycloak:
	@$(DOCKER_COMPOSE) run --rm keycloak ./scripts/bootstrap_keycloak.sh

seed:
	@AVP_HOST=$$(grep '^AVP_HOST' .env | cut -d'=' -f2 || echo localhost) \
	AVP_PORT=$$(grep '^AVP_PORT' .env | cut -d'=' -f2 || echo 50051) \
	POLICY_STORE_ID=$$(grep '^POLICY_STORE_ID' .env | cut -d'=' -f2 || echo demo-policy-store) \
	POLICY_STORE_NAME=$$(grep '^POLICY_STORE_NAME' .env | cut -d'=' -f2 || echo "Demo Policy Store") \
	IDENTITY_SOURCE_ID=$$(grep '^IDENTITY_SOURCE_ID' .env | cut -d'=' -f2 || echo demo-identity-source) \
	KEYCLOAK_ISSUER=$$(grep '^KEYCLOAK_ISSUER' .env | cut -d'=' -f2 || echo http://localhost:8080/realms/demo) \
	./scripts/seed_avp.sh

logs:
	$(DOCKER_COMPOSE) logs -f

down:
	$(DOCKER_COMPOSE) down

clean:
	$(DOCKER_COMPOSE) down -v

demo-build:
	cd app && cargo build

demo-run:
	cd app && cargo run
