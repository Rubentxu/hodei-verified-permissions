# Mapeo de Funcionalidades Backend → Frontend

## Endpoints gRPC Disponibles (21 total)

### 📦 POLICY STORES (4 endpoints)
1. ✅ CreatePolicyStore
    - Input: { description?: string }
    - Output: { policy_store_id: string, created_at: string }

2. ✅ GetPolicyStore
    - Input: { policy_store_id: string }
    - Output: { policy_store_id: string, description?: string, created_at: string, updated_at: string }

3. ✅ ListPolicyStores
    - Input: { max_results?: number, next_token?: string }
    - Output: { policy_stores: PolicyStoreItem[], next_token?: string }

4. ✅ DeletePolicyStore
    - Input: { policy_store_id: string }
    - Output: {} (empty)

   ---

### 📋 SCHEMAS (2 endpoints)
5. ✅ PutSchema
    - Input: { policy_store_id: string, schema: string (JSON) }
    - Output: { policy_store_id: string, namespaces: string[] }

6. ✅ GetSchema
    - Input: { policy_store_id: string }
    - Output: { policy_store_id: string, schema: string, created_at: string, updated_at: string }

   ---

### 📜 POLICIES (5 endpoints)
7. ✅ CreatePolicy
    - Input: { policy_store_id: string, policy_id: string, definition: PolicyDefinition, description?: string }
    - Output: { policy_store_id: string, policy_id: string, created_at: string }

8. ✅ GetPolicy
    - Input: { policy_store_id: string, policy_id: string }
    - Output: { policy_store_id: string, policy_id: string, definition: PolicyDefinition, description?: string,
      created_at: string, updated_at: string }

9. ✅ UpdatePolicy
    - Input: { policy_store_id: string, policy_id: string, definition: PolicyDefinition, description?: string }
    - Output: { policy_store_id: string, policy_id: string, updated_at: string }

10. ✅ DeletePolicy
    - Input: { policy_store_id: string, policy_id: string }
    - Output: {} (empty)

11. ✅ ListPolicies
    - Input: { policy_store_id: string, max_results?: number, next_token?: string }
    - Output: { policies: PolicyItem[], next_token?: string }

   ---

### 🔐 IDENTITY SOURCES (4 endpoints)
12. ✅ CreateIdentitySource
    - Input: { policy_store_id: string, configuration: IdentitySourceConfiguration, claims_mapping?:
      ClaimsMappingConfiguration, description?: string }
    - Output: { identity_source_id: string, created_at: string }

13. ✅ GetIdentitySource
    - Input: { policy_store_id: string, identity_source_id: string }
    - Output: { identity_source_id: string, policy_store_id: string, configuration: IdentitySourceConfiguration,
      claims_mapping?: ClaimsMappingConfiguration, description?: string, created_at: string, updated_at: string }

14. ✅ ListIdentitySources
    - Input: { policy_store_id: string, max_results?: number, next_token?: string }
    - Output: { identity_sources: IdentitySourceItem[], next_token?: string }

15. ✅ DeleteIdentitySource
    - Input: { policy_store_id: string, identity_source_id: string }
    - Output: { identity_source_id: string }

   ---

### 🧩 POLICY TEMPLATES (4 endpoints)
16. ✅ CreatePolicyTemplate
    - Input: { policy_store_id: string, template_id: string, statement: string, description?: string }
    - Output: { template_id: string, created_at: string }

17. ✅ GetPolicyTemplate
    - Input: { policy_store_id: string, template_id: string }
    - Output: { template_id: string, policy_store_id: string, statement: string, description?: string, created_at:
      string, updated_at: string }

18. ✅ ListPolicyTemplates
    - Input: { policy_store_id: string, max_results?: number, next_token?: string }
    - Output: { templates: PolicyTemplateItem[], next_token?: string }

19. ✅ DeletePolicyTemplate
    - Input: { policy_store_id: string, template_id: string }
    - Output: { template_id: string }

   ---

### 🧪 TESTING/PLAYGROUND (2 endpoints)
20. ✅ TestAuthorization
    - Input: { policy_store_id?: string, schema?: string, policies: string[], principal: EntityIdentifier, action:
      EntityIdentifier, resource: EntityIdentifier, context?: string, entities: Entity[] }
    - Output: { decision: Decision, determining_policies: string[], errors: string[], validation_warnings:
      ValidationIssue[], validation_errors: ValidationIssue[] }

21. ✅ ValidatePolicy
    - Input: { policy_store_id?: string, schema?: string, policy_statement: string }
    - Output: { is_valid: boolean, errors: ValidationIssue[], warnings: ValidationIssue[], policy_info?: PolicyInfo
      }

   ---

## EVALUACIÓN DE AUTORIZACIÓN (3 endpoints)

### ⚡ DATA PLANE - Evaluación de Alto Rendimiento

22. ✅ IsAuthorized
    - Input: { policy_store_id: string, principal: EntityIdentifier, action: EntityIdentifier, resource:
      EntityIdentifier, context?: string, entities: Entity[] }
    - Output: { decision: Decision, determining_policies: string[], errors: string[] }

23. ✅ BatchIsAuthorized
    - Input: { policy_store_id: string, requests: IsAuthorizedRequest[] }
    - Output: { responses: IsAuthorizedResponse[] }

24. ✅ IsAuthorizedWithToken
    - Input: { policy_store_id: string, identity_source_id: string, access_token: string, action: EntityIdentifier,
      resource: EntityIdentifier, context?: string, entities: Entity[] }
    - Output: { decision: Decision, determining_policies: string[], errors: string[] }

   ---

## TIPOS DE DATOS CLAVE PARA EL FRONTEND

### EntityIdentifier
   ```typescript
   {
     entity_type: string;  // "User", "Action", "Resource", "Role", etc.
     entity_id: string;    // "alice", "viewDocument", "doc123", etc.
   }
   ```

### Entity
   ```typescript
   {
     identifier: EntityIdentifier;
     attributes: Record<string, string>;  // JSON-encoded values
     parents: EntityIdentifier[];         // For hierarchies
   }
   ```

### PolicyDefinition (oneof)
   ```typescript
   // Static Policy
   {
     static: {
       statement: string;  // Cedar policy text
     }
   }

   // OR Template-Linked Policy
   {
     template_linked: {
       policy_template_id: string;
       principal: EntityIdentifier;
       resource: EntityIdentifier;
     }
   }
   ```

### IdentitySourceConfiguration (oneof)
   ```typescript
   // Cognito User Pool
   {
     cognito_user_pool: {
       user_pool_arn: string;
       client_ids: string;  // comma-separated
       group_configuration_group_claim: string;
     }
   }

   // OR OIDC
   {
     oidc: {
       issuer: string;
       client_ids: string[];
       jwks_uri: string;
       group_claim: string;
     }
   }
   ```

### ClaimsMappingConfiguration
   ```typescript
   {
     principal_id_claim: string;           // Default: "sub"
     group_claim: string;                   // e.g., "cognito:groups", "groups"
     attribute_mappings: Record<string, string>;  // claim_name -> cedar_attribute
   }
   ```

### ValidationIssue
   ```typescript
   {
     severity: 'ERROR' | 'WARNING' | 'INFO';
     message: string;
     location?: string;
     issue_type: string;
   }
   ```

### Decision
   ```typescript
   enum Decision {
     DECISION_UNSPECIFIED = 0,
     ALLOW = 1,
     DENY = 2
   }
   ```

   ---

## FLUJO DE TRABAJO RECOMENDADO PARA FRONTEND

### 1️⃣ Configuración Inicial
   ```
   CreatePolicyStore → PutSchema → CreateIdentitySource
   ```

### 2️⃣ Gestión de Políticas
   ```
   CreatePolicyTemplate (optional) → CreatePolicy (static or from template) → ListPolicies
   ```

### 3️⃣ Testing/Desarrollo
   ```
   TestAuthorization (playground) → ValidatePolicy (before saving)
   ```

### 4️⃣ Producción
   ```
   IsAuthorizedWithToken (with JWT) OR IsAuthorized (manual)
   ```

   ---

## CASOS DE USO COMPLEJOS

### Policy Templates (placeholder replacement)
   ```typescript
   // Template definition
   const template = {
     template_id: "user_doc_template",
     statement: `
       permit(
         principal == ?principal,
         action == Action::"view",
         resource == ?resource
       );
     `
   };

   // Create linked policy
   const policy = {
     policy_id: "alice_can_view_doc123",
     definition: {
       template_linked: {
         policy_template_id: "user_doc_template",
         principal: { entity_type: "User", entity_id: "alice" },
         resource: { entity_type: "Document", entity_id: "doc123" }
       }
     }
   };
   // Backend automatically replaces ?principal and ?resource
   ```

### JWT Integration with Claims Mapping
   ```typescript
   const identitySource = {
     configuration: {
       oidc: {
         issuer: "https://YOUR_DOMAIN.auth0.com/",
         client_ids: ["your-client-id"],
         jwks_uri: "https://YOUR_DOMAIN.auth0.com/.well-known/jwks.json",
         group_claim: "https://yourapp.com/groups"
       }
     },
     claims_mapping: {
       principal_id_claim: "sub",
       group_claim: "https://yourapp.com/groups",
       attribute_mappings: {
         "email": "email",
         "department": "department"
       }
     }
   };
   // JWT groups become parent Role entities
   // Claims become Cedar attributes
   ```

### Entity Hierarchies
   ```typescript
   const entities = [
     {
       identifier: { entity_type: "User", entity_id: "alice" },
       attributes: { "email": "alice@example.com" },
       parents: [
         { entity_type: "Role", entity_id: "Manager" },
         { entity_type: "Department", entity_id: "Engineering" }
       ]
     },
     {
       identifier: { entity_type: "Role", entity_id: "Manager" },
       attributes: {},
       parents: [
         { entity_type: "Role", entity_id: "Employee" }
       ]
     }
   ];
   // Alice inherits attributes from Manager and Employee
   ```

   ---

## IMPORTANTE PARA EL FRONTEND

### ⚠️ Validaciones del Backend
- IDs cannot be empty
- Cedar policy syntax is validated
- Schema must be valid JSON
- JWT tokens must have kid header
- Template placeholders must be replaced

### ⚠️ Manejo de Errores
- INVALID_ARGUMENT → Validation errors
- NOT_FOUND → Resource doesn't exist
- INTERNAL → Server error
- UNAUTHENTICATED → Invalid JWT token

### ⚠️ Performance Tips
- Use BatchIsAuthorized for multiple checks
- Cache policy stores and schemas
- JWT keys are cached automatically
- Prefer PostgreSQL for production

### ⚠️ Security
- Don't log full JWT tokens
- Validate tokens client-side before sending
- Use HTTPS in production
- Sanitize policy IDs and descriptions