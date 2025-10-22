/**
 * API Types - gRPC response types and interfaces
 */

export interface PolicyStore {
  policyStoreId: string;
  description?: string;
  createdAt: string;
  updatedAt: string;
}

export interface Policy {
  policyId: string;
  policyStoreId: string;
  statement: string;
  description?: string;
  createdAt: string;
  updatedAt: string;
}

export interface Schema {
  policyStoreId: string;
  schema: string;
  createdAt: string;
  updatedAt: string;
}

export interface PolicyTemplate {
  templateId: string;
  policyStoreId: string;
  statement: string;
  description?: string;
  createdAt: string;
  updatedAt: string;
}

export interface EntityIdentifier {
  entityType: string;
  entityId: string;
}

export interface Entity {
  identifier?: EntityIdentifier;
  attributes?: Record<string, string>;
  parents?: EntityIdentifier[];
}

export interface AuthorizationRequest {
  principal?: EntityIdentifier;
  action?: EntityIdentifier;
  resource?: EntityIdentifier;
  context?: Record<string, unknown>;
  policies: string[];
  entities: Entity[];
  policyStoreId?: string;
  schema?: string;
}

export interface AuthorizationResponse {
  decision: 'ALLOW' | 'DENY';
  determiningPolicies: string[];
  errors: string[];
}

export interface ValidationResponse {
  isValid: boolean;
  errors: string[];
  warnings: string[];
}

export interface ListPolicyStoresResponse {
  policyStores: PolicyStore[];
  nextToken?: string;
}

export interface ListPoliciesResponse {
  policies: Policy[];
  nextToken?: string;
}

export interface ListPolicyTemplatesResponse {
  templates: PolicyTemplate[];
  nextToken?: string;
}
