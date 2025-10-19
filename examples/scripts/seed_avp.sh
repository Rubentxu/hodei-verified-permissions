#!/usr/bin/env bash
set -euo pipefail

AVP_HOST="${AVP_HOST:-localhost}"
AVP_PORT="${AVP_PORT:-50051}"
GRPC_ENDPOINT="$AVP_HOST:$AVP_PORT"

STORE_ID="${POLICY_STORE_ID:-demo-policy-store}"
STORE_NAME="${POLICY_STORE_NAME:-Demo Policy Store}"
IDENTITY_ID="${IDENTITY_SOURCE_ID:-demo-identity-source}"
IDENTITY_DESCRIPTION="${IDENTITY_SOURCE_DESCRIPTION:-Keycloak Demo Identity}"
KEYCLOAK_ISSUER="${KEYCLOAK_ISSUER:-http://localhost:8080/realms/demo}"
KEYCLOAK_CLIENT_ID="${KEYCLOAK_CLIENT_ID:-demo-app}"
KEYCLOAK_GROUP_CLAIM="${KEYCLOAK_GROUP_CLAIM:-realm_access.roles}"

POLICIES_DIR="${POLICIES_DIR:-$(dirname "$0")/../policies}"

if ! command -v grpcurl >/dev/null; then
  echo "grpcurl no está instalado. Instala grpcurl para continuar." >&2
  exit 1
fi

json_escape() {
  echo "$1" | python3 -c 'import json,sys; print(json.dumps(sys.stdin.read().strip()))'
}

create_policy_store() {
  echo "===> Creando Policy Store $STORE_ID"
  cat <<JSON | grpcurl -plaintext -d @ "$GRPC_ENDPOINT" hodei.permissions.v1.Authorization/CreatePolicyStore >/dev/null
{
  "policy_store_id": "$STORE_ID",
  "name": "$(json_escape "$STORE_NAME")"
}
JSON
}

create_identity_source() {
  echo "===> Creando Identity Source $IDENTITY_ID"
  cat <<JSON | grpcurl -plaintext -d @ "$GRPC_ENDPOINT" hodei.permissions.v1.Authorization/CreateIdentitySource >/dev/null
{
  "policy_store_id": "$STORE_ID",
  "identity_source_id": "$IDENTITY_ID",
  "description": "$(json_escape "$IDENTITY_DESCRIPTION")",
  "oidc_configuration": {
    "issuer": "$(json_escape "$KEYCLOAK_ISSUER")",
    "client_ids": ["$(json_escape "$KEYCLOAK_CLIENT_ID")"],
    "jwks_uri": "$(json_escape "$KEYCLOAK_ISSUER")/.well-known/openid-configuration/jwks",
    "group_claim": "$(json_escape "$KEYCLOAK_GROUP_CLAIM")"
  },
  "claims_mapping": {
    "principal_id_claim": "sub",
    "group_claim": "$(json_escape "$KEYCLOAK_GROUP_CLAIM")"
  }
}
JSON
}

create_policy() {
  local policy_id=$1
  local file=$2
  echo "===> Creando política $policy_id desde $file"
  cat <<JSON | grpcurl -plaintext -d @ "$GRPC_ENDPOINT" hodei.permissions.v1.Authorization/CreatePolicy >/dev/null
{
  "policy_store_id": "$STORE_ID",
  "policy_id": "$policy_id",
  "policy": "$(json_escape "$(cat "$file")")"
}
JSON
}

echo "===> Verificando conexión con AVP ($GRPC_ENDPOINT)"
if ! grpcurl -plaintext "$GRPC_ENDPOINT" list >/dev/null; then
  echo "No se puede conectar con $GRPC_ENDPOINT" >&2
  exit 1
fi

create_policy_store || true
create_identity_source || true

echo "===> Cargando políticas desde $POLICIES_DIR"
for policy_file in "$POLICIES_DIR"/*.cedar; do
  [ -e "$policy_file" ] || continue
  policy_id=$(basename "$policy_file" .cedar)
  create_policy "$policy_id" "$policy_file" || true
done

echo "===> Seed AVP completado"
