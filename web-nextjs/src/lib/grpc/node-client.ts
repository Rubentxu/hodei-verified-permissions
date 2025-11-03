// Real gRPC client for Next.js server-side usage
// Connects to Rust gRPC backend using @grpc/grpc-js

import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import path from 'path';

// TypeScript interfaces for our gRPC types
interface EntityIdentifier {
  entity_type: string;
  entity_id: string;
}

interface IsAuthorizedRequest {
  policy_store_id: string;
  principal: EntityIdentifier;
  action: EntityIdentifier;
  resource: EntityIdentifier;
  context?: string;
  entities?: Entity[];
}

interface Entity {
  identifier: EntityIdentifier;
  attributes: Record<string, string>;
  parents: EntityIdentifier[];
}

interface IsAuthorizedResponse {
  decision: 'DECISION_UNSPECIFIED' | 'ALLOW' | 'DENY';
  determining_policies: string[];
  errors: string[];
}

interface CreatePolicyStoreRequest {
  description?: string;
}

interface CreatePolicyStoreResponse {
  policy_store_id: string;
  created_at: string;
}

interface GetPolicyStoreRequest {
  policy_store_id: string;
}

interface GetPolicyStoreResponse {
  policy_store_id: string;
  name?: string;
  description?: string;
  status?: string;
  version?: string;
  author?: string;
  tags?: string; // JSON string of tags array
  created_at: string;
  updated_at: string;
}

interface ListPolicyStoresRequest {
  max_results?: number;
  next_token?: string;
}

interface PolicyStoreItem {
  policy_store_id: string;
  name?: string;
  description?: string;
  status?: string;
  version?: string;
  author?: string;
  tags?: string; // JSON string of tags array
  created_at: string;
  updated_at?: string;
}

interface ListPolicyStoresResponse {
  policy_stores: PolicyStoreItem[];
  next_token?: string;
}

interface UpdatePolicyStoreRequest {
  policy_store_id: string;
  description?: string;
}

interface UpdatePolicyStoreResponse {
  policy_store_id: string;
  description?: string;
  updated_at: string;
}

interface PutSchemaRequest {
  policy_store_id: string;
  schema: string;
}

interface PutSchemaResponse {
  policy_store_id: string;
  namespaces: string[];
}

interface GetSchemaRequest {
  policy_store_id: string;
}

interface GetSchemaResponse {
  policy_store_id: string;
  schema: string;
  created_at: string;
  updated_at: string;
}

interface TestAuthorizationRequest {
  policy_store_id?: string;
  schema?: string;
  policies: string[];
  principal: EntityIdentifier;
  action: EntityIdentifier;
  resource: EntityIdentifier;
  context?: string;
  entities?: Entity[];
}

interface TestAuthorizationResponse {
  decision: 'DECISION_UNSPECIFIED' | 'ALLOW' | 'DENY';
  determining_policies: string[];
  errors: string[];
  validation_warnings: ValidationIssue[];
  validation_errors: ValidationIssue[];
}

interface ValidationIssue {
  severity: 'SEVERITY_UNSPECIFIED' | 'ERROR' | 'WARNING' | 'INFO';
  message: string;
  location?: string;
  issue_type: string;
}

interface GetPolicyStoreAuditLogRequest {
  policy_store_id: string;
}

interface PolicyStoreAuditLogEntry {
  id?: number;
  policy_store_id: string;
  action: string;
  user_id: string;
  changes?: string;
  ip_address?: string;
  timestamp?: string;
}

interface GetPolicyStoreAuditLogResponse {
  audit_logs: PolicyStoreAuditLogEntry[];
}

interface UpdatePolicyStoreTagsRequest {
  policy_store_id: string;
  tags: string[];
}

interface UpdatePolicyStoreTagsResponse {
  success: boolean;
  tags: string[];
}

// Version Control / Snapshot Management interfaces
interface CreatePolicyStoreSnapshotRequest {
  policy_store_id: string;
  description?: string;
}

interface CreatePolicyStoreSnapshotResponse {
  snapshot_id: string;
  policy_store_id: string;
  created_at: string;
  description: string;
}

interface GetPolicyStoreSnapshotRequest {
  policy_store_id: string;
  snapshot_id: string;
}

interface PolicySummary {
  policy_id: string;
  description?: string;
  statement: string;
}

interface GetPolicyStoreSnapshotResponse {
  snapshot_id: string;
  policy_store_id: string;
  description: string;
  created_at: string;
  policy_count: number;
  has_schema: boolean;
  schema_json?: string;
  policies?: PolicySummary[];
}

interface ListPolicyStoreSnapshotsRequest {
  policy_store_id: string;
  max_results?: number;
  next_token?: string;
}

interface SnapshotItem {
  snapshot_id: string;
  policy_store_id: string;
  description?: string;
  created_at: string;
  policy_count: number;
  has_schema: boolean;
  size_bytes: number;
}

interface ListPolicyStoreSnapshotsResponse {
  snapshots: SnapshotItem[];
  next_token?: string;
}

interface RollbackToSnapshotRequest {
  policy_store_id: string;
  snapshot_id: string;
  description?: string;
}

interface RollbackToSnapshotResponse {
  policy_store_id: string;
  snapshot_id: string;
  rolled_back_at: string;
  policies_restored: number;
  schema_restored: boolean;
}

interface DeleteSnapshotRequest {
  policy_store_id: string;
  snapshot_id: string;
}

interface DeleteSnapshotResponse {
  snapshot_id: string;
}

// Batch Policy Management interfaces
interface BatchCreatePoliciesRequest {
  policy_store_id: string;
  policies: BatchPolicyItem[];
}

interface BatchPolicyItem {
  policy_id: string;
  definition: any;
  description?: string;
}

interface BatchCreatePoliciesResponse {
  results: BatchPolicyResult[];
  errors: string[];
}

interface BatchPolicyResult {
  policy_id: string;
  created_at: string;
  error?: string;
}

interface BatchUpdatePoliciesRequest {
  policy_store_id: string;
  policies: BatchPolicyItem[];
}

interface BatchUpdatePoliciesResponse {
  results: BatchUpdatePolicyResult[];
  errors: string[];
}

interface BatchUpdatePolicyResult {
  policy_id: string;
  updated_at: string;
  error?: string;
}

interface BatchDeletePoliciesRequest {
  policy_store_id: string;
  policy_ids: string[];
}

interface BatchDeletePoliciesResponse {
  results: BatchDeletePolicyResult[];
  errors: string[];
}

interface BatchDeletePolicyResult {
  policy_id: string;
  error?: string;
}

const VERIFIED_PERMISSIONS_ADDR = process.env.VERIFIED_PERMISSIONS_ADDR || 'localhost:50051';
const PROTO_PATH = path.join(process.cwd(), '../proto/authorization.proto');

// Load proto file
const packageDefinition = protoLoader.loadSync(PROTO_PATH, {
  keepCase: true,
  longs: String,
  enums: String,
  defaults: true,
  oneofs: true
});

const protoDescriptor = grpc.loadPackageDefinition(packageDefinition) as any;
const authorizationPackage = protoDescriptor.authorization;

// Create gRPC clients
const authorizationDataClient = new authorizationPackage.AuthorizationData(
  VERIFIED_PERMISSIONS_ADDR,
  grpc.credentials.createInsecure()
);

const authorizationControlClient = new authorizationPackage.AuthorizationControl(
  VERIFIED_PERMISSIONS_ADDR,
  grpc.credentials.createInsecure()
);

// Export client objects for API routes
export { authorizationDataClient, authorizationControlClient };

// Helper functions for common operations with proper typing
export const grpcClients = {
  // Data Plane operations
  isAuthorized: (request: IsAuthorizedRequest): Promise<IsAuthorizedResponse> => {
    return new Promise((resolve, reject) => {
      authorizationDataClient.isAuthorized(request, (err: any, response: IsAuthorizedResponse) => {
        if (err) {
          console.error('gRPC isAuthorized error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  batchIsAuthorized: (request: any): Promise<any> => {
    return new Promise((resolve, reject) => {
      authorizationDataClient.batchIsAuthorized(request, (err: any, response: any) => {
        if (err) {
          console.error('gRPC batchIsAuthorized error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  isAuthorizedWithToken: (request: any): Promise<any> => {
    return new Promise((resolve, reject) => {
      authorizationDataClient.isAuthorizedWithToken(request, (err: any, response: any) => {
        if (err) {
          console.error('gRPC isAuthorizedWithToken error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  // Control Plane operations
  createPolicyStore: (request: CreatePolicyStoreRequest): Promise<CreatePolicyStoreResponse> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.createPolicyStore(request, (err: any, response: CreatePolicyStoreResponse) => {
        if (err) {
          console.error('gRPC createPolicyStore error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  getPolicyStore: (request: GetPolicyStoreRequest): Promise<GetPolicyStoreResponse> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.getPolicyStore(request, (err: any, response: GetPolicyStoreResponse) => {
        if (err) {
          console.error('gRPC getPolicyStore error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  listPolicyStores: (request?: ListPolicyStoresRequest): Promise<ListPolicyStoresResponse> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.listPolicyStores(request || {}, (err: any, response: ListPolicyStoresResponse) => {
        if (err) {
          console.error('gRPC listPolicyStores error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  updatePolicyStore: (request: UpdatePolicyStoreRequest): Promise<UpdatePolicyStoreResponse> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.updatePolicyStore(request, (err: any, response: UpdatePolicyStoreResponse) => {
        if (err) {
          console.error('gRPC updatePolicyStore error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  deletePolicyStore: (request: { policy_store_id: string }): Promise<any> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.deletePolicyStore(request, (err: any, response: any) => {
        if (err) {
          console.error('gRPC deletePolicyStore error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  putSchema: (request: PutSchemaRequest): Promise<PutSchemaResponse> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.putSchema(request, (err: any, response: PutSchemaResponse) => {
        if (err) {
          console.error('gRPC putSchema error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  getSchema: (request: GetSchemaRequest): Promise<GetSchemaResponse> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.getSchema(request, (err: any, response: GetSchemaResponse) => {
        if (err) {
          console.error('gRPC getSchema error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  createPolicy: (request: any): Promise<any> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.createPolicy(request, (err: any, response: any) => {
        if (err) {
          console.error('gRPC createPolicy error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  getPolicy: (request: any): Promise<any> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.getPolicy(request, (err: any, response: any) => {
        if (err) {
          console.error('gRPC getPolicy error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  listPolicies: (request: any): Promise<any> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.listPolicies(request, (err: any, response: any) => {
        if (err) {
          console.error('gRPC listPolicies error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  updatePolicy: (request: any): Promise<any> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.updatePolicy(request, (err: any, response: any) => {
        if (err) {
          console.error('gRPC updatePolicy error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  deletePolicy: (request: any): Promise<any> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.deletePolicy(request, (err: any, response: any) => {
        if (err) {
          console.error('gRPC deletePolicy error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  testAuthorization: (request: TestAuthorizationRequest): Promise<TestAuthorizationResponse> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.testAuthorization(request, (err: any, response: TestAuthorizationResponse) => {
        if (err) {
          console.error('gRPC testAuthorization error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  getPolicyStoreAuditLog: (request: GetPolicyStoreAuditLogRequest): Promise<GetPolicyStoreAuditLogResponse> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.getPolicyStoreAuditLog(request, (err: any, response: GetPolicyStoreAuditLogResponse) => {
        if (err) {
          console.error('gRPC getPolicyStoreAuditLog error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  updatePolicyStoreTags: (request: UpdatePolicyStoreTagsRequest): Promise<UpdatePolicyStoreTagsResponse> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.updatePolicyStoreTags(request, (err: any, response: UpdatePolicyStoreTagsResponse) => {
        if (err) {
          console.error('gRPC updatePolicyStoreTags error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  // Version Control / Snapshot Management
  createPolicyStoreSnapshot: (request: CreatePolicyStoreSnapshotRequest): Promise<CreatePolicyStoreSnapshotResponse> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.createPolicyStoreSnapshot(request, (err: any, response: CreatePolicyStoreSnapshotResponse) => {
        if (err) {
          console.error('gRPC createPolicyStoreSnapshot error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  getPolicyStoreSnapshot: (request: GetPolicyStoreSnapshotRequest): Promise<GetPolicyStoreSnapshotResponse> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.getPolicyStoreSnapshot(request, (err: any, response: GetPolicyStoreSnapshotResponse) => {
        if (err) {
          console.error('gRPC getPolicyStoreSnapshot error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  listPolicyStoreSnapshots: (request: ListPolicyStoreSnapshotsRequest): Promise<ListPolicyStoreSnapshotsResponse> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.listPolicyStoreSnapshots(request, (err: any, response: ListPolicyStoreSnapshotsResponse) => {
        if (err) {
          console.error('gRPC listPolicyStoreSnapshots error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  rollbackToSnapshot: (request: RollbackToSnapshotRequest): Promise<RollbackToSnapshotResponse> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.rollbackToSnapshot(request, (err: any, response: RollbackToSnapshotResponse) => {
        if (err) {
          console.error('gRPC rollbackToSnapshot error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  deleteSnapshot: (request: DeleteSnapshotRequest): Promise<DeleteSnapshotResponse> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.deleteSnapshot(request, (err: any, response: DeleteSnapshotResponse) => {
        if (err) {
          console.error('gRPC deleteSnapshot error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  // Batch Policy Management
  batchCreatePolicies: (request: BatchCreatePoliciesRequest): Promise<BatchCreatePoliciesResponse> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.batchCreatePolicies(request, (err: any, response: BatchCreatePoliciesResponse) => {
        if (err) {
          console.error('gRPC batchCreatePolicies error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  batchUpdatePolicies: (request: BatchUpdatePoliciesRequest): Promise<BatchUpdatePoliciesResponse> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.batchUpdatePolicies(request, (err: any, response: BatchUpdatePoliciesResponse) => {
        if (err) {
          console.error('gRPC batchUpdatePolicies error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  },

  batchDeletePolicies: (request: BatchDeletePoliciesRequest): Promise<BatchDeletePoliciesResponse> => {
    return new Promise((resolve, reject) => {
      authorizationControlClient.batchDeletePolicies(request, (err: any, response: BatchDeletePoliciesResponse) => {
        if (err) {
          console.error('gRPC batchDeletePolicies error:', err);
          reject(new Error(`gRPC error: ${err.message || err.details || 'Unknown error'}`));
        } else {
          resolve(response);
        }
      });
    });
  }
};

// Export types for use in API routes
export type {
  EntityIdentifier,
  IsAuthorizedRequest,
  IsAuthorizedResponse,
  Entity,
  CreatePolicyStoreRequest,
  CreatePolicyStoreResponse,
  GetPolicyStoreRequest,
  GetPolicyStoreResponse,
  ListPolicyStoresRequest,
  ListPolicyStoresResponse,
  PolicyStoreItem,
  UpdatePolicyStoreRequest,
  UpdatePolicyStoreResponse,
  PutSchemaRequest,
  PutSchemaResponse,
  GetSchemaRequest,
  GetSchemaResponse,
  TestAuthorizationRequest,
  TestAuthorizationResponse,
  ValidationIssue,
  GetPolicyStoreAuditLogRequest,
  PolicyStoreAuditLogEntry,
  GetPolicyStoreAuditLogResponse,
  UpdatePolicyStoreTagsRequest,
  UpdatePolicyStoreTagsResponse,
  CreatePolicyStoreSnapshotRequest,
  CreatePolicyStoreSnapshotResponse,
  GetPolicyStoreSnapshotRequest,
  GetPolicyStoreSnapshotResponse,
  ListPolicyStoreSnapshotsRequest,
  ListPolicyStoreSnapshotsResponse,
  SnapshotItem,
  RollbackToSnapshotRequest,
  RollbackToSnapshotResponse,
  DeleteSnapshotRequest,
  DeleteSnapshotResponse,
  PolicySummary,
  BatchCreatePoliciesRequest,
  BatchCreatePoliciesResponse,
  BatchUpdatePoliciesRequest,
  BatchUpdatePoliciesResponse,
  BatchDeletePoliciesRequest,
  BatchDeletePoliciesResponse,
  BatchPolicyItem,
  BatchPolicyResult,
  BatchUpdatePolicyResult,
  BatchDeletePolicyResult
};
