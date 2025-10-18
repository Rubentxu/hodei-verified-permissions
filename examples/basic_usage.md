# Basic Usage Example

This example demonstrates the complete workflow of using Hodei Verified Permissions.

## 1. Start the Server

```bash
cargo run --release
```

The server will start on `0.0.0.0:50051` by default.

## 2. Create a Policy Store

```bash
grpcurl -plaintext -d '{
  "description": "Document Management System"
}' localhost:50051 authorization.AuthorizationControl/CreatePolicyStore
```

Response:
```json
{
  "policyStoreId": "550e8400-e29b-41d4-a716-446655440000",
  "createdAt": "2024-01-15T10:30:00Z"
}
```

Save the `policyStoreId` for subsequent requests.

## 3. Define a Schema

```bash
grpcurl -plaintext -d '{
  "policy_store_id": "550e8400-e29b-41d4-a716-446655440000",
  "schema": "{\"DocApp\": {\"entityTypes\": {\"User\": {\"shape\": {\"type\": \"Record\", \"attributes\": {\"department\": {\"type\": \"String\"}}}}, \"Document\": {\"shape\": {\"type\": \"Record\", \"attributes\": {\"owner\": {\"type\": \"Entity\", \"name\": \"User\"}, \"confidential\": {\"type\": \"Boolean\"}}}}}, \"actions\": {\"view\": {\"appliesTo\": {\"principalTypes\": [\"User\"], \"resourceTypes\": [\"Document\"]}}, \"edit\": {\"appliesTo\": {\"principalTypes\": [\"User\"], \"resourceTypes\": [\"Document\"]}}}}}"
}' localhost:50051 authorization.AuthorizationControl/PutSchema
```

## 4. Create Policies

### Policy 1: Users can view their own documents

```bash
grpcurl -plaintext -d '{
  "policy_store_id": "550e8400-e29b-41d4-a716-446655440000",
  "policy_id": "view-own-docs",
  "definition": {
    "static": {
      "statement": "permit(principal, action == Action::\"view\", resource) when { resource.owner == principal };"
    }
  },
  "description": "Allow users to view their own documents"
}' localhost:50051 authorization.AuthorizationControl/CreatePolicy
```

### Policy 2: Managers can view all documents

```bash
grpcurl -plaintext -d '{
  "policy_store_id": "550e8400-e29b-41d4-a716-446655440000",
  "policy_id": "managers-view-all",
  "definition": {
    "static": {
      "statement": "permit(principal, action == Action::\"view\", resource) when { principal.department == \"management\" };"
    }
  },
  "description": "Allow managers to view all documents"
}' localhost:50051 authorization.AuthorizationControl/CreatePolicy
```

### Policy 3: Only owners can edit documents

```bash
grpcurl -plaintext -d '{
  "policy_store_id": "550e8400-e29b-41d4-a716-446655440000",
  "policy_id": "edit-own-only",
  "definition": {
    "static": {
      "statement": "permit(principal, action == Action::\"edit\", resource) when { resource.owner == principal };"
    }
  },
  "description": "Only document owners can edit"
}' localhost:50051 authorization.AuthorizationControl/CreatePolicy
```

## 5. Test Authorization Requests

### Scenario 1: Alice views her own document (ALLOW)

```bash
grpcurl -plaintext -d '{
  "policy_store_id": "550e8400-e29b-41d4-a716-446655440000",
  "principal": {
    "entity_type": "User",
    "entity_id": "alice"
  },
  "action": {
    "entity_type": "Action",
    "entity_id": "view"
  },
  "resource": {
    "entity_type": "Document",
    "entity_id": "doc123"
  },
  "entities": [
    {
      "identifier": {
        "entity_type": "User",
        "entity_id": "alice"
      },
      "attributes": {
        "department": "\"engineering\""
      }
    },
    {
      "identifier": {
        "entity_type": "Document",
        "entity_id": "doc123"
      },
      "attributes": {
        "owner": "{\"__entity\": {\"type\": \"User\", \"id\": \"alice\"}}",
        "confidential": "false"
      }
    }
  ]
}' localhost:50051 authorization.AuthorizationData/IsAuthorized
```

Expected Response:
```json
{
  "decision": "ALLOW",
  "determiningPolicies": ["view-own-docs"],
  "errors": []
}
```

### Scenario 2: Bob tries to view Alice's document (DENY)

```bash
grpcurl -plaintext -d '{
  "policy_store_id": "550e8400-e29b-41d4-a716-446655440000",
  "principal": {
    "entity_type": "User",
    "entity_id": "bob"
  },
  "action": {
    "entity_type": "Action",
    "entity_id": "view"
  },
  "resource": {
    "entity_type": "Document",
    "entity_id": "doc123"
  },
  "entities": [
    {
      "identifier": {
        "entity_type": "User",
        "entity_id": "bob"
      },
      "attributes": {
        "department": "\"engineering\""
      }
    },
    {
      "identifier": {
        "entity_type": "Document",
        "entity_id": "doc123"
      },
      "attributes": {
        "owner": "{\"__entity\": {\"type\": \"User\", \"id\": \"alice\"}}",
        "confidential": "false"
      }
    }
  ]
}' localhost:50051 authorization.AuthorizationData/IsAuthorized
```

Expected Response:
```json
{
  "decision": "DENY",
  "determiningPolicies": [],
  "errors": []
}
```

### Scenario 3: Manager views any document (ALLOW)

```bash
grpcurl -plaintext -d '{
  "policy_store_id": "550e8400-e29b-41d4-a716-446655440000",
  "principal": {
    "entity_type": "User",
    "entity_id": "charlie"
  },
  "action": {
    "entity_type": "Action",
    "entity_id": "view"
  },
  "resource": {
    "entity_type": "Document",
    "entity_id": "doc123"
  },
  "entities": [
    {
      "identifier": {
        "entity_type": "User",
        "entity_id": "charlie"
      },
      "attributes": {
        "department": "\"management\""
      }
    },
    {
      "identifier": {
        "entity_type": "Document",
        "entity_id": "doc123"
      },
      "attributes": {
        "owner": "{\"__entity\": {\"type\": \"User\", \"id\": \"alice\"}}",
        "confidential": "false"
      }
    }
  ]
}' localhost:50051 authorization.AuthorizationData/IsAuthorized
```

Expected Response:
```json
{
  "decision": "ALLOW",
  "determiningPolicies": ["managers-view-all"],
  "errors": []
}
```

## 6. List Policies

```bash
grpcurl -plaintext -d '{
  "policy_store_id": "550e8400-e29b-41d4-a716-446655440000"
}' localhost:50051 authorization.AuthorizationControl/ListPolicies
```

## 7. Batch Authorization (Multiple Requests)

```bash
grpcurl -plaintext -d '{
  "policy_store_id": "550e8400-e29b-41d4-a716-446655440000",
  "requests": [
    {
      "policy_store_id": "550e8400-e29b-41d4-a716-446655440000",
      "principal": {"entity_type": "User", "entity_id": "alice"},
      "action": {"entity_type": "Action", "entity_id": "view"},
      "resource": {"entity_type": "Document", "entity_id": "doc123"},
      "entities": []
    },
    {
      "policy_store_id": "550e8400-e29b-41d4-a716-446655440000",
      "principal": {"entity_type": "User", "entity_id": "alice"},
      "action": {"entity_type": "Action", "entity_id": "edit"},
      "resource": {"entity_type": "Document", "entity_id": "doc123"},
      "entities": []
    }
  ]
}' localhost:50051 authorization.AuthorizationData/BatchIsAuthorized
```

## Notes

- Replace `550e8400-e29b-41d4-a716-446655440000` with your actual policy store ID
- Entity attributes must be valid JSON strings
- The `__entity` format is used for entity references in attributes
- Context can be added for time-based or IP-based policies
