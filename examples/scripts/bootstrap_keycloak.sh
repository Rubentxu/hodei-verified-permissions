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
KEYCLOAK_URL_BASE="http://$KEYCLOAK_HOST:$KEYCLOAK_PORT"
REALM="${KEYCLOAK_REALM:-demo}"
CLIENT_ID="${KEYCLOAK_CLIENT_ID:-demo-app}"
CLIENT_SECRET="${KEYCLOAK_CLIENT_SECRET:-demo-secret}"
DEFAULT_PASSWORD="${KEYCLOAK_DEFAULT_PASSWORD:-Password123!}"

USERS_JSON=$(cat <<'JSON'
[
  {
    "username": "pet_admin",
    "roles": ["pet-admin"],
    "groups": ["administrators"]
  },
  {
    "username": "pet_vet",
    "roles": ["pet-vet"],
    "groups": ["veterinarians"]
  },
  {
    "username": "pet_customer",
    "roles": ["pet-customer"],
    "groups": ["customers"]
  }
]
JSON
)

wait_for_keycloak() {
  echo "===> Esperando a Keycloak en $KEYCLOAK_URL_BASE"
  until curl -sf "$KEYCLOAK_URL_BASE/realms/master" >/dev/null; do
    sleep 3
  done
}

admin_token() {
  curl -s -X POST "$KEYCLOAK_URL_BASE/realms/master/protocol/openid-connect/token" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "grant_type=password" \
    -d "client_id=admin-cli" \
    -d "username=${KEYCLOAK_ADMIN:-admin}" \
    -d "password=${KEYCLOAK_ADMIN_PASSWORD:-admin}" | jq -r '.access_token'
}

create_realm() {
  curl -sf -X POST "$KEYCLOAK_URL_BASE/admin/realms" \
    -H "Authorization: Bearer $ADMIN_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"realm\": \"$REALM\", \"enabled\": true}" || true
}

create_roles() {
  local roles=("pet-admin" "pet-vet" "pet-customer")
  for role in "${roles[@]}"; do
    curl -sf -X POST "$KEYCLOAK_URL_BASE/admin/realms/$REALM/roles" \
      -H "Authorization: Bearer $ADMIN_TOKEN" \
      -H "Content-Type: application/json" \
      -d "{\"name\": \"$role\"}" || true
  done
}

create_client() {
  cat <<JSON | curl -sf -X POST "$KEYCLOAK_URL_BASE/admin/realms/$REALM/clients" \
    -H "Authorization: Bearer $ADMIN_TOKEN" \
    -H "Content-Type: application/json" \
    -d @- || true
{
  "clientId": "$CLIENT_ID",
  "enabled": true,
  "publicClient": false,
  "secret": "$CLIENT_SECRET",
  "protocol": "openid-connect",
  "standardFlowEnabled": true,
  "directAccessGrantsEnabled": true,
  "redirectUris": ["http://localhost:3000/*"],
  "attributes": {
    "post.logout.redirect.uris": "http://localhost:3000/*"
  }
}
JSON
}

create_users() {
  echo "$USERS_JSON" | jq -c '.[]' | while read -r user; do
    local username=$(echo "$user" | jq -r '.username')
    local roles=$(echo "$user" | jq -r '.roles | join(",")')

    curl -sf -X POST "$KEYCLOAK_URL_BASE/admin/realms/$REALM/users" \
      -H "Authorization: Bearer $ADMIN_TOKEN" \
      -H "Content-Type: application/json" \
      -d "{\"username\": \"$username\", \"enabled\": true}" || true

    local user_id=$(curl -s "$KEYCLOAK_URL_BASE/admin/realms/$REALM/users" \
      -H "Authorization: Bearer $ADMIN_TOKEN" | jq -r ".[] | select(.username==\"$username\") | .id")

    if [ -n "$user_id" ]; then
      curl -sf -X PUT "$KEYCLOAK_URL_BASE/admin/realms/$REALM/users/$user_id/reset-password" \
        -H "Authorization: Bearer $ADMIN_TOKEN" \
        -H "Content-Type: application/json" \
        -d "{\"type\": \"password\", \"value\": \"$DEFAULT_PASSWORD\", \"temporary\": false}"

      IFS=',' read -ra role_array <<< "$roles"
      for role in "${role_array[@]}"; do
        local role_json=$(curl -s "$KEYCLOAK_URL_BASE/admin/realms/$REALM/roles/$role" \
          -H "Authorization: Bearer $ADMIN_TOKEN")
        curl -sf -X POST "$KEYCLOAK_URL_BASE/admin/realms/$REALM/users/$user_id/role-mappings/realm" \
          -H "Authorization: Bearer $ADMIN_TOKEN" \
          -H "Content-Type: application/json" \
          -d "[$role_json]" || true
      done
    fi
  done
}

wait_for_keycloak
ADMIN_TOKEN=$(admin_token)
if [ -z "$ADMIN_TOKEN" ] || [ "$ADMIN_TOKEN" = "null" ]; then
  echo "No se pudo obtener token admin" >&2
  exit 1
fi

create_realm
create_roles
create_client
create_users

echo "===> Keycloak inicializado (realm: $REALM, URL: $KEYCLOAK_URL_BASE)"
