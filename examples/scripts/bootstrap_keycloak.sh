#!/usr/bin/env bash
set -euo pipefail

KEYCLOAK_URL="${KEYCLOAK_URL:-http://localhost:8080}"
REALM="${KEYCLOAK_REALM:-demo}"
CLIENT_ID="${KEYCLOAK_CLIENT_ID:-demo-app}"
CLIENT_SECRET="${KEYCLOAK_CLIENT_SECRET:-demo-secret}"
USERS_JSON=$(cat <<'JSON'
[
  {
    "username": "admin_user",
    "password": "Password123!",
    "roles": ["admin"],
    "groups": ["administrators"]
  },
  {
    "username": "developer_user",
    "password": "Password123!",
    "roles": ["developer"],
    "groups": ["developers"]
  },
  {
    "username": "viewer_user",
    "password": "Password123!",
    "roles": ["viewer"],
    "groups": ["viewers"]
  }
]
JSON
)

echo "===> Esperando a Keycloak en $KEYCLOAK_URL"
until curl -sf "$KEYCLOAK_URL/realms/master" >/dev/null; do
  sleep 3
done

echo "===> Obteniendo token admin"
ADMIN_TOKEN=$(curl -s -X POST "$KEYCLOAK_URL/realms/master/protocol/openid-connect/token" \
  -H "Content-Type: application/x-www-form-urlencoded" \
  -d "grant_type=password" \
  -d "client_id=admin-cli" \
  -d "username=${KEYCLOAK_ADMIN:-admin}" \
  -d "password=${KEYCLOAK_ADMIN_PASSWORD:-admin}" | jq -r '.access_token')

if [ -z "$ADMIN_TOKEN" ] || [ "$ADMIN_TOKEN" = "null" ]; then
  echo "No se pudo obtener token admin" >&2
  exit 1
fi

create_realm() {
  echo "===> Creando realm $REALM"
  curl -sf -X POST "$KEYCLOAK_URL/admin/realms" \
    -H "Authorization: Bearer $ADMIN_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"realm\": \"$REALM\", \"enabled\": true}" || true
}

create_realm

ROLE_NAMES=(admin developer viewer)
echo "===> Creando roles"
for role in "${ROLE_NAMES[@]}"; do
  curl -sf -X POST "$KEYCLOAK_URL/admin/realms/$REALM/roles" \
    -H "Authorization: Bearer $ADMIN_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"name\": \"$role\"}" || true
done

echo "===> Creando cliente confidencial"
CLIENT_ID_PAYLOAD=$(cat <<JSON
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
)

curl -sf -X POST "$KEYCLOAK_URL/admin/realms/$REALM/clients" \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d "$CLIENT_ID_PAYLOAD" || true

echo "===> Creando usuarios"
echo "$USERS_JSON" | jq -c '.[]' | while read -r user; do
  username=$(echo "$user" | jq -r '.username')
  password=$(echo "$user" | jq -r '.password')
  roles=$(echo "$user" | jq -r '.roles | join(",")')

  curl -sf -X POST "$KEYCLOAK_URL/admin/realms/$REALM/users" \
    -H "Authorization: Bearer $ADMIN_TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"username\": \"$username\", \"enabled\": true}" || true

  USER_ID=$(curl -s "$KEYCLOAK_URL/admin/realms/$REALM/users" \
    -H "Authorization: Bearer $ADMIN_TOKEN" | jq -r ".[] | select(.username==\"$username\") | .id")

  if [ -n "$USER_ID" ]; then
    curl -sf -X PUT "$KEYCLOAK_URL/admin/realms/$REALM/users/$USER_ID/reset-password" \
      -H "Authorization: Bearer $ADMIN_TOKEN" \
      -H "Content-Type: application/json" \
      -d "{\"type\": \"password\", \"value\": \"$password\", \"temporary\": false}"

    IFS=',' read -ra ROLE_ARRAY <<< "$roles"
    for role in "${ROLE_ARRAY[@]}"; do
      ROLE_JSON=$(curl -s "$KEYCLOAK_URL/admin/realms/$REALM/roles/$role" \
        -H "Authorization: Bearer $ADMIN_TOKEN")
      curl -sf -X POST "$KEYCLOAK_URL/admin/realms/$REALM/users/$USER_ID/role-mappings/realm" \
        -H "Authorization: Bearer $ADMIN_TOKEN" \
        -H "Content-Type: application/json" \
        -d "[$ROLE_JSON]" || true
    done
  fi
done

echo "===> Keycloak inicializado"
