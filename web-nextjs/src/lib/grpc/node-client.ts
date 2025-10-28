import * as grpc from '@grpc/grpc-js';
import * as protoLoader from '@grpc/proto-loader';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';
import { readFileSync } from 'fs';

// Compute absolute proto path (ESM in Next.js runtime)
let protoPath: string;
if (typeof window === 'undefined') {
  // Node runtime (API routes)
  const __filename = fileURLToPath(import.meta.url);
  const __dirname = dirname(__filename);
  protoPath = join(__dirname, '../../../../proto/authorization.proto');
} else {
  // Fallback: relative from web root
  protoPath = './proto/authorization.proto';
}

const packageDefinition = protoLoader.loadSync(protoPath, {
  keepCase: true,
  longs: String,
  enums: String,
  defaults: true,
  oneofs: true,
});
const protoDescriptor = grpc.loadPackageDefinition(packageDefinition) as any;
const { authorization } = protoDescriptor;

// Address from env or default
const GRPC_ADDR = process.env.VERIFIED_PERMISSIONS_ADDR || 'localhost:50051';

export function createAuthorizationControlClient() {
  return new authorization.AuthorizationControl(
    GRPC_ADDR,
    grpc.credentials.createInsecure(),
  );
}

export function createAuthorizationDataClient() {
  return new authorization.AuthorizationData(
    GRPC_ADDR,
    grpc.credentials.createInsecure(),
  );
}

// Helpers
export async function listPolicyStores(): Promise<any[]> {
  const client = createAuthorizationControlClient();
  return new Promise((resolve, reject) => {
    client.ListPolicyStores({}, (err: any, response: any) => {
      if (err) return reject(err);
      console.log('[node-client listPolicyStores] response:', response);
      resolve(response.policyStores || []);
    });
  });
}

export async function createPolicyStore(description?: string): Promise<any> {
  const client = createAuthorizationControlClient();
  const request = description ? { description } : {};
  return new Promise((resolve, reject) => {
    client.CreatePolicyStore(request, (err: any, response: any) => {
      if (err) return reject(err);
      console.log('[node-client createPolicyStore] response:', response);
      resolve(response);
    });
  });
}

export async function getPolicyStore(id: string): Promise<any> {
  const client = createAuthorizationControlClient();
  return new Promise((resolve, reject) => {
    client.GetPolicyStore({ policyStoreId: id }, (err: any, response: any) => {
      if (err) return reject(err);
      resolve(response);
    });
  });
}

export async function deletePolicyStore(id: string): Promise<void> {
  const client = createAuthorizationControlClient();
  return new Promise((resolve, reject) => {
    client.DeletePolicyStore({ policyStoreId: id }, (err: any) => {
      if (err) return reject(err);
      resolve();
    });
  });
}

export async function putSchema(policyStoreId: string, schema: string): Promise<any> {
  const client = createAuthorizationControlClient();
  return new Promise((resolve, reject) => {
    client.PutSchema({ policyStoreId, schema }, (err: any, response: any) => {
      if (err) return reject(err);
      resolve(response);
    });
  });
}

export async function getSchema(policyStoreId: string): Promise<string> {
  const client = createAuthorizationControlClient();
  return new Promise((resolve, reject) => {
    client.GetSchema({ policyStoreId }, (err: any, response: any) => {
      if (err) return reject(err);
      resolve(response.schema);
    });
  });
}

export async function isAuthorized(req: any): Promise<any> {
  const client = createAuthorizationDataClient();
  return new Promise((resolve, reject) => {
    client.IsAuthorized(req, (err: any, response: any) => {
      if (err) return reject(err);
      resolve(response);
    });
  });
}

// Simple health check: try to list policy stores (empty request is cheap)
export async function healthCheck(): Promise<boolean> {
  try {
    const result = await listPolicyStores();
    console.log('[node-client healthCheck] fetched stores count:', result.length);
    return true;
  } catch (e) {
    console.error('[node-client healthCheck] error', e);
    return false;
  }
}