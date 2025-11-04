# Hodei CLI

Command-line tools for Hodei Verified Permissions.

## Installation

```bash
cargo install --path .
```

Or build from source:

```bash
cargo build --release
```

## Commands

### Generate Schema

Generate a Cedar schema from an OpenAPI specification following the SimpleRest mapping pattern.

```bash
hodei-cli generate-schema \
  --api-spec path/to/openapi.json \
  --namespace MyApp \
  --base-path /v1 \
  --output ./schemas
```

**Arguments:**
- `--api-spec`: Path to OpenAPI v3 specification file (JSON format)
- `--namespace`: Cedar namespace for your application (e.g., `MyApp`)
- `--base-path`: (Optional) Base path of your API. Required if your OpenAPI spec has multiple servers
- `--output` / `-o`: Output directory for generated schema files (default: current directory)

**Output:**
- `v4.cedarschema.json`: Cedar schema in v4 format (compatible with Cedar 4.x)
- `v2.cedarschema.json`: Cedar schema in v2 format (if available, for backwards compatibility)

### Generate Policies

Generate sample Cedar policies from a Cedar schema.

```bash
hodei-cli generate-policies \
  --schema path/to/v4.cedarschema.json \
  --output ./policies
```

**Arguments:**
- `--schema`: Path to Cedar schema file (v4 JSON format)
- `--output` / `-o`: Output directory for generated policy files (default: `./policies`)

**Output:**
- `policy_1.cedar`: Admin policy allowing all actions
- `policy_2.cedar`: Role-based policy template

### Generate Least Privilege Policies

Generate complete authorization bundle (schema + policies) from OpenAPI specification with least privilege defaults.

```bash
hvp generate-least-privilege \
  --spec path/to/openapi.json \
  --namespace MyApp \
  --mode strict \
  --roles admin,developer,viewer \
  --output ./authorization
```

**Arguments:**
- `--spec`: Path to OpenAPI v3 specification file (JSON format)
- `--namespace`: Cedar namespace for your application
- `--base-path`: (Optional) Base path of your API
- `--output` / `-o`: Output directory for generated files (default: `./authorization`)
- `--roles`: Comma-separated list of roles (default: `admin,developer,viewer`)
- `--mode`: Analysis mode: `strict`, `moderate`, or `permissive` (default: `strict`)

**Privilege Modes:**
- **strict** (recommended): Default deny, explicit allow only
- **moderate**: Read access for all authenticated users, write for privileged
- **permissive**: Common CRUD patterns allowed

**Output:**
- `v4.cedarschema.json`: Generated Cedar schema
- `policies/`: Directory with generated Cedar policies
  - Policies for each role and endpoint
  - Default deny policy
  - CORS OPTIONS policy
- `security_report.md`: Security analysis report with warnings and recommendations

**Example:**
```bash
# Generate with strict least privilege
hvp generate-least-privilege \
  --spec api_spec.json \
  --namespace MyApp \
  --mode strict \
  --roles admin,developer,viewer,auditor \
  --output ./auth_config

# This generates:
# ./auth_config/v4.cedarschema.json
# ./auth_config/policies/policy_1.cedar (admin GET users)
# ./auth_config/policies/policy_2.cedar (admin POST users)
# ./auth_config/policies/policy_3.cedar (developer GET users)
# ... (one policy per role-action combination)
# ./auth_config/security_report.md
```

**Security Report:**
The security report includes:
- Security score (0-100)
- Risk level (Low, Medium, High, Critical)
- Warnings for wildcard actions, overly permissive principals
- Missing default deny policy detection
- CRUD symmetry analysis
- Recommendations for improvement

## Example Workflow

### Option 1: Step-by-step

1. **Generate schema from OpenAPI:**

```bash
hodei-cli generate-schema \
  --api-spec ../examples/openapi-sample.json \
  --namespace DocumentApp \
  --base-path /v1
```

2. **Generate sample policies:**

```bash
hodei-cli generate-policies \
  --schema v4.cedarschema.json \
  --output ./policies
```

3. **Review and customize the generated policies** in `./policies/`

4. **Use the schema** with your Hodei authorization service

### Option 2: All-in-one (Recommended)

1. **Generate complete authorization bundle with least privilege:**

```bash
hvp generate-least-privilege \
  --spec ../examples/openapi-sample.json \
  --namespace DocumentApp \
  --mode strict \
  --roles admin,developer,viewer \
  --output ./authorization
```

2. **Review the security report** in `./authorization/security_report.md`

3. **Customize policies** in `./authorization/policies/` as needed

4. **Use the schema** with your Hodei authorization service

## SimpleRest Mapping Pattern

The `generate-schema` command follows the **SimpleRest** mapping pattern:

- **HTTP methods** → Cedar **actions** (e.g., `GET /documents/{id}` → `getDocument` action)
- **OpenAPI paths** → Cedar **resource types** (e.g., `/documents/{id}` → `Document` resource)
- **Path/query parameters** → Cedar **context** attributes

### Example

Given this OpenAPI operation:

```json
{
  "paths": {
    "/documents/{id}": {
      "get": {
        "operationId": "getDocument",
        "x-cedar": {
          "appliesToResourceTypes": ["Document"]
        },
        "parameters": [
          {
            "name": "id",
            "in": "path",
            "required": true,
            "schema": { "type": "string" }
          }
        ]
      }
    }
  }
}
```

The generated Cedar schema will include:

```json
{
  "DocumentApp": {
    "actions": {
      "getDocument": {
        "appliesTo": {
          "principalTypes": ["User"],
          "resourceTypes": ["Document"],
          "context": {
            "type": "Record",
            "attributes": {
              "pathParameters": {
                "type": "Record",
                "attributes": {
                  "id": { "type": "String", "required": true }
                }
              }
            }
          }
        },
        "annotations": {
          "httpVerb": "get",
          "httpPathTemplate": "/v1/documents/{id}"
        }
      }
    }
  }
}
```

## Logging

Set the `RUST_LOG` environment variable to control logging verbosity:

```bash
RUST_LOG=debug hodei-cli generate-schema --api-spec openapi.json --namespace MyApp
```

## See Also

- [Hodei Verified Permissions Documentation](../docs/)
- [Cedar Policy Language](https://www.cedarpolicy.com/)
- [OpenAPI Specification](https://swagger.io/specification/)
