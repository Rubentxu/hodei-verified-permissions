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
  description?: string;
  created_at: string;
  updated_at: string;
}

interface ListPolicyStoresRequest {
  max_results?: number;
  next_token?: string;
}

interface PolicyStoreItem {
  policy_store_id: string;
  description?: string;
  created_at: string;
}

interface ListPolicyStoresResponse {
  policy_stores: PolicyStoreItem[];
  next_token?: string;
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
  }
};

// Health check client
const healthClient = {
  check: async (request: any) => {
    return { status: 'SERVING' };
  },
};

export { healthClient };

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
  PutSchemaRequest,
  PutSchemaResponse,
  GetSchemaRequest,
  GetSchemaResponse,
  TestAuthorizationRequest,
  TestAuthorizationResponse,
  ValidationIssue
};
