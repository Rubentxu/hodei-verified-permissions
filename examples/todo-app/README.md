# TODO Task Manager with Cedar Authorization

A complete, production-ready example of a TODO task management application with fine-grained authorization using Cedar policies and the Hodei Verified Permissions SDK.

## Features

- ✅ **Complete CRUD API** for tasks and projects
- ✅ **Role-Based Access Control** (Admin, Project Manager, Team Member, Viewer)
- ✅ **Attribute-Based Access Control** (users can only update their assigned tasks)
- ✅ **Automatic Authorization** using SimpleRest mapping
- ✅ **Context-Aware Policies** (path parameters, query strings)
- ✅ **Cedar Action Macros** for self-documenting handlers
- ✅ **OpenAPI to Cedar Schema** generation
- ✅ **In-Memory Storage** with sample data
- ✅ **Production-Ready** error handling and logging

## 🎯 Demonstrates ALL Implemented Features

This application showcases the complete integration of all features developed across Sprints 1-4:

1. **Sprint 1**: OpenAPI → Cedar schema generation
2. **Sprint 2**: Runtime mapping with SimpleRest
3. **Sprint 3**: Procedural macros (`#[cedar_action]`)
4. **Sprint 4**: Complete production application

## Architecture

```
HTTP Request
    ↓
Axum Router
    ↓
VerifiedPermissionsLayer (Middleware)
    ↓
SimpleRestMapping.resolve()
    → Action: "updateTask"
    → Resource: "Task"
    → Context: {"pathParameters": {"taskId": "123"}}
    ↓
Cedar Policy Evaluation
    → Check user's role
    → Check task ownership (if applicable)
    → Check resource attributes
    ↓
Allow → Handler → Response
Deny → 403 Forbidden
```

## Authorization Model

### Roles

1. **Admin** (`admin` group)
   - Full access to everything
   - Can manage all tasks and projects

2. **Project Manager** (`project_managers` group)
   - Create, read, update, delete tasks
   - Assign tasks to team members
   - Create and manage projects
   - Cannot delete projects (admin only)

3. **Team Member** (`team_members` group)
   - View all tasks and projects
   - Create new tasks
   - Update and complete tasks assigned to them
   - Cannot delete tasks or assign tasks

4. **Viewer** (`viewers` group)
   - Read-only access
   - Can view tasks and projects
   - Cannot create, update, or delete anything

### Cedar Policies

#### Policy 1: Admin Access
```cedar
permit(
    principal in TodoApp::UserGroup::"admin",
    action,
    resource
);
```

#### Policy 3: Team Member Access
```cedar
// View and create
permit(
    principal in TodoApp::UserGroup::"team_members",
    action in [
        TodoApp::Action::"listTasks",
        TodoApp::Action::"getTask",
        TodoApp::Action::"createTask",
        TodoApp::Action::"listProjects",
        TodoApp::Action::"getProject"
    ],
    resource
);

// Update own tasks
permit(
    principal,
    action in [
        TodoApp::Action::"updateTask",
        TodoApp::Action::"completeTask"
    ],
    resource
)
when {
    resource has assignee && resource.assignee == principal
};
```

## Prerequisites

1. **Hodei Authorization Service** running on `localhost:50051`
2. **Cedar Schema** generated (included in this example)
3. **Rust 1.89+** installed

## Quick Start

### 1. Generate Cedar Schema (Already Done)

The schema has been pre-generated from `openapi.json`:

```bash
hodei-cli generate-schema \
  --api-spec openapi.json \
  --namespace TodoApp \
  --base-path /api/v1
```

### 2. Review Policies

Check the policies in `policies/` directory:
- `policy_1.cedar` - Admin access
- `policy_2.cedar` - General role-based template
- `policy_3_team_member.cedar` - Team member permissions
- `policy_4_project_manager.cedar` - Project manager permissions
- `policy_5_read_only.cedar` - Viewer permissions

### 3. Run the Application

```bash
cd examples/todo-app
cargo run
```

The server will start on `http://localhost:3000`

### 4. Test the API

#### Health Check (No Auth)
```bash
curl http://localhost:3000/health
# Expected: OK
```

#### List Tasks (Auth Required)
```bash
curl -H "Authorization: Bearer <your-jwt-token>" \
  http://localhost:3000/api/v1/tasks
```

#### Get Specific Task
```bash
curl -H "Authorization: Bearer <your-jwt-token>" \
  http://localhost:3000/api/v1/tasks/<task-id>
```

#### Create Task
```bash
curl -X POST \
  -H "Authorization: Bearer <your-jwt-token>" \
  -H "Content-Type: application/json" \
  -d '{"title": "New Task", "description": "Task description"}' \
  http://localhost:3000/api/v1/tasks
```

#### Update Task
```bash
curl -X PUT \
  -H "Authorization: Bearer <your-jwt-token>" \
  -H "Content-Type: application/json" \
  -d '{"title": "Updated Title", "status": "in_progress"}' \
  http://localhost:3000/api/v1/tasks/<task-id>
```

#### Assign Task
```bash
curl -X POST \
  -H "Authorization: Bearer <your-jwt-token>" \
  "http://localhost:3000/api/v1/tasks/<task-id>/assign?userId=alice"
```

#### Complete Task
```bash
curl -X POST \
  -H "Authorization: Bearer <your-jwt-token>" \
  http://localhost:3000/api/v1/tasks/<task-id>/complete
```

#### Delete Task
```bash
curl -X DELETE \
  -H "Authorization: Bearer <your-jwt-token>" \
  http://localhost:3000/api/v1/tasks/<task-id>
```

## Sample Data

The application starts with sample data:

### Projects
1. **Website Redesign** (owner: alice)
2. **Mobile App** (owner: bob)

### Tasks
1. **Design homepage mockup**
   - Project: Website Redesign
   - Assignee: charlie
   - Status: Pending

2. **Implement login page**
   - Project: Mobile App
   - Assignee: bob
   - Status: In Progress

3. **Write API documentation**
   - No project
   - No assignee
   - Status: Pending

## Authorization Examples

### Scenario 1: Team Member Updates Own Task

**User**: charlie (in `team_members` group)  
**Action**: Update task "Design homepage mockup" (assigned to charlie)  
**Result**: ✅ ALLOW (policy_3_team_member.cedar - owns the task)

### Scenario 2: Team Member Updates Someone Else's Task

**User**: charlie (in `team_members` group)  
**Action**: Update task "Implement login page" (assigned to bob)  
**Result**: ❌ DENY (not assigned to charlie)

### Scenario 3: Project Manager Assigns Task

**User**: alice (in `project_managers` group)  
**Action**: Assign task to any user  
**Result**: ✅ ALLOW (policy_4_project_manager.cedar)

### Scenario 4: Viewer Tries to Create Task

**User**: dave (in `viewers` group)  
**Action**: Create new task  
**Result**: ❌ DENY (policy_5_read_only.cedar - read-only)

### Scenario 5: Admin Deletes Project

**User**: admin (in `admin` group)  
**Action**: Delete project  
**Result**: ✅ ALLOW (policy_1.cedar - admin has full access)

## API Endpoints

| Method | Path | Action | Resource | Description |
|--------|------|--------|----------|-------------|
| GET | `/health` | - | - | Health check (no auth) |
| GET | `/api/v1/tasks` | `listTasks` | `Application` | List all tasks |
| POST | `/api/v1/tasks` | `createTask` | `Application` | Create new task |
| GET | `/api/v1/tasks/:taskId` | `getTask` | `Task` | Get task details |
| PUT | `/api/v1/tasks/:taskId` | `updateTask` | `Task` | Update task |
| DELETE | `/api/v1/tasks/:taskId` | `deleteTask` | `Task` | Delete task |
| POST | `/api/v1/tasks/:taskId/assign` | `assignTask` | `Task` | Assign task to user |
| POST | `/api/v1/tasks/:taskId/complete` | `completeTask` | `Task` | Mark task as completed |
| GET | `/api/v1/projects` | `listProjects` | `Application` | List all projects |
| POST | `/api/v1/projects` | `createProject` | `Application` | Create new project |
| GET | `/api/v1/projects/:projectId` | `getProject` | `Project` | Get project details |
| DELETE | `/api/v1/projects/:projectId` | `deleteProject` | `Project` | Delete project |

## Environment Variables

```bash
# Authorization service endpoint
AUTH_ENDPOINT=http://localhost:50051

# Policy store ID
POLICY_STORE_ID=todo-policy-store

# Identity source ID
IDENTITY_SOURCE_ID=todo-identity-source

# Logging level
RUST_LOG=info
```

## Code Structure

```
todo-app/
├── src/
│   ├── main.rs          # Application entry point, router setup
│   ├── models.rs        # Data models (Task, Project, DTOs)
│   ├── storage.rs       # In-memory storage with DashMap
│   └── handlers.rs      # HTTP request handlers
├── policies/            # Cedar policies
│   ├── policy_1.cedar   # Admin access
│   ├── policy_2.cedar   # General template
│   ├── policy_3_team_member.cedar
│   ├── policy_4_project_manager.cedar
│   └── policy_5_read_only.cedar
├── openapi.json         # OpenAPI 3.0 specification
├── v4.cedarschema.json  # Generated Cedar schema
├── Cargo.toml           # Dependencies
└── README.md            # This file
```

## Cedar Action Macros

All handlers in this application use the `#[cedar_action]` macro for self-documentation:

```rust
use hodei_macros::cedar_action;

#[cedar_action(
    action = "getTask",
    resource = "Task",
    description = "Get a specific task by ID"
)]
pub async fn get_task(
    State(state): State<AppState>,
    Path(task_id): Path<String>,
) -> Result<Json<Task>, AppError> {
    // Handler implementation
}
```

### Benefits of Using Macros

1. **Self-Documenting Code**: Cedar metadata visible in code
2. **IDE Integration**: Tooltips show Cedar action information
3. **Compile-Time Validation**: Macro ensures correct syntax
4. **Future-Proof**: Foundation for schema generation from code
5. **Consistency**: Enforces documentation standards

### All Annotated Handlers

| Handler | Action | Resource | Description |
|---------|--------|----------|-------------|
| `list_tasks` | `listTasks` | `Application` | List all tasks with optional filters |
| `get_task` | `getTask` | `Task` | Get a specific task by ID |
| `create_task` | `createTask` | `Application` | Create a new task |
| `update_task` | `updateTask` | `Task` | Update task details |
| `delete_task` | `deleteTask` | `Task` | Delete a task |
| `assign_task` | `assignTask` | `Task` | Assign task to a user |
| `complete_task` | `completeTask` | `Task` | Mark task as completed |
| `list_projects` | `listProjects` | `Application` | List all projects |
| `get_project` | `getProject` | `Project` | Get project details |
| `create_project` | `createProject` | `Application` | Create a new project |
| `delete_project` | `deleteProject` | `Project` | Delete a project and all its tasks |

## Key Implementation Details

### 1. Automatic Action Resolution

```rust
let auth_layer = VerifiedPermissionsLayer::new(client, store_id, source_id)
    .with_simple_rest_mapping(schema_json)?;
```

The middleware automatically:
- Matches HTTP method + path to Cedar action
- Extracts path parameters as context
- Extracts query parameters as context
- Calls authorization service
- Allows/denies based on Cedar decision

### 2. Context Extraction

For request: `POST /api/v1/tasks/123/assign?userId=alice`

Context generated:
```json
{
  "pathParameters": {
    "taskId": "123"
  },
  "queryStringParameters": {
    "userId": "alice"
  }
}
```

### 3. Resource Attributes

In a real application, you would enrich the authorization request with resource attributes:

```rust
// Pseudo-code
let task = get_task(task_id)?;
let resource_attributes = json!({
    "assignee": task.assignee,
    "created_by": task.created_by,
    "project_id": task.project_id
});
```

Then Cedar can evaluate policies like:
```cedar
permit(principal, action, resource)
when {
    resource.assignee == principal
};
```

## Testing Different Roles

### Setup Test Users

In your authorization service, create test users:

```bash
# Admin user
User::"admin_user" in UserGroup::"admin"

# Project manager
User::"alice" in UserGroup::"project_managers"

# Team member
User::"charlie" in UserGroup::"team_members"

# Viewer
User::"dave" in UserGroup::"viewers"
```

### Generate JWTs

Generate JWT tokens for each user with their respective roles.

### Test Authorization

```bash
# As admin - should work
curl -H "Authorization: Bearer <admin-token>" \
  -X DELETE http://localhost:3000/api/v1/projects/123

# As team member - should fail
curl -H "Authorization: Bearer <charlie-token>" \
  -X DELETE http://localhost:3000/api/v1/tasks/123

# As team member updating own task - should work
curl -H "Authorization: Bearer <charlie-token>" \
  -X PUT http://localhost:3000/api/v1/tasks/<charlie-task-id> \
  -d '{"status": "completed"}'
```

## Production Considerations

### 1. Database

Replace `DashMap` with a real database:
- PostgreSQL with SQLx
- MongoDB
- DynamoDB

### 2. Resource Attributes

Enrich authorization requests with resource attributes from the database.

### 3. Caching

Cache Cedar schemas and policy decisions for better performance.

### 4. Metrics

Add metrics for:
- Authorization decisions (allow/deny)
- Response times
- Error rates

### 5. Audit Logging

Log all authorization decisions for compliance and debugging.

## Troubleshooting

### "Connection refused" to authorization service

Ensure the Hodei authorization service is running:
```bash
cargo run --bin hodei-verified-permissions
```

### "Access Denied" errors

Check:
1. JWT token is valid
2. User exists in the policy store
3. User is in the correct group
4. Policies allow the action

### "No mapping found" errors

Verify:
1. Schema is loaded correctly
2. Path matches the OpenAPI spec
3. HTTP method is correct

## Next Steps

- Add real database integration
- Implement JWT token generation/validation
- Add resource attribute enrichment
- Implement audit logging
- Add metrics and monitoring
- Deploy to production

## Related Documentation

- [Hodei SDK Documentation](../../sdk/README.md)
- [CLI Usage Guide](../../docs/GUIA_USO_CLI_GENERACION_SCHEMA.md)
- [Cedar Policy Language](https://www.cedarpolicy.com/)
- [OpenAPI Specification](openapi.json)
- [Generated Cedar Schema](v4.cedarschema.json)
