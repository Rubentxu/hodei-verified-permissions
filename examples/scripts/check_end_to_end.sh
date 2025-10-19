#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
EXAMPLES_DIR="$(dirname "$SCRIPT_DIR")"
ENV_FILE="${ENV_FILE:-$EXAMPLES_DIR/.env}"

if [ -f "$ENV_FILE" ]; then
  set -a
  # shellcheck disable=SC1090
  source "$ENV_FILE"
  set +a
fi

KEYCLOAK_HOST="${KEYCLOAK_HOST:-localhost}"
KEYCLOAK_PORT="${KEYCLOAK_PORT:-8080}"
KEYCLOAK_REALM="${KEYCLOAK_REALM:-demo}"
KEYCLOAK_CLIENT_ID="${KEYCLOAK_CLIENT_ID:-demo-app}"
KEYCLOAK_CLIENT_SECRET="${KEYCLOAK_CLIENT_SECRET:-demo-secret}"
KEYCLOAK_URL_BASE="http://$KEYCLOAK_HOST:$KEYCLOAK_PORT"
DEFAULT_PASSWORD="${KEYCLOAK_DEFAULT_PASSWORD:-Password123!}"

DEMO_ADDR="${DEMO_HTTP_ADDR:-0.0.0.0:3000}"
DEMO_HOST="${DEMO_HTTP_HOST:-localhost}"
DEMO_HTTP_PORT="${DEMO_HTTP_PORT:-3000}"

require() {
  if ! command -v "$1" >/dev/null; then
    echo "Se requiere '$1' en PATH" >&2
    exit 1
  fi
}

require curl
require jq

fetch_token() {
  local username=$1
  local password=$2
  curl -s -X POST "$KEYCLOAK_URL_BASE/realms/$KEYCLOAK_REALM/protocol/openid-connect/token" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "grant_type=password" \
    -d "client_id=$KEYCLOAK_CLIENT_ID" \
    -d "client_secret=$KEYCLOAK_CLIENT_SECRET" \
    -d "username=$username" \
    -d "password=$password" | jq -r '.access_token'
}

check_endpoint() {
  local token=$1
  local method=$2
  local url=$3
  local expected=$4
  echo "===> $method $url (esperado: $expected)"
  local status
  status=$(curl -s -o /dev/null -w "%{http_code}" \
    -H "Authorization: Bearer $token" \
    -X "$method" \
    "$url")
  echo "    status=$status"
  if [ "$status" != "$expected" ]; then
    echo "    ❌ Esperado $expected, obtenido $status" >&2
    return 1
  fi
  echo "    ✅"
}

pet_admin_token=$(fetch_token "pet_admin" "$DEFAULT_PASSWORD")
pet_vet_token=$(fetch_token "pet_vet" "$DEFAULT_PASSWORD")
pet_customer_token=$(fetch_token "pet_customer" "$DEFAULT_PASSWORD")

if [ -z "$pet_admin_token" ] || [ "$pet_admin_token" = "null" ]; then
  echo "No se pudo obtener token pet_admin" >&2
  exit 1
fi

BASE_URL="http://$DEMO_HOST:$DEMO_HTTP_PORT"

set -e
check_endpoint "$pet_admin_token" "GET" "$BASE_URL/pets" "200"
check_endpoint "$pet_vet_token" "GET" "$BASE_URL/pets" "200"
check_endpoint "$pet_customer_token" "GET" "$BASE_URL/pets" "200"

check_endpoint "$pet_admin_token" "POST" "$BASE_URL/pets" "200"
check_endpoint "$pet_vet_token" "POST" "$BASE_URL/pets" "403"
check_endpoint "$pet_customer_token" "POST" "$BASE_URL/pets" "403"

check_endpoint "$pet_vet_token" "POST" "$BASE_URL/pets/pet-1/appointments" "200"
check_endpoint "$pet_customer_token" "POST" "$BASE_URL/pets/pet-1/appointments" "403"

echo "===> Verificación end-to-end completada"
