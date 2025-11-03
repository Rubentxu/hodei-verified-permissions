import type { NextApiRequest, NextApiResponse } from 'next';
import { z } from 'zod';
import { grpcClients } from '@/lib/grpc/node-client';
import { handleGRPCError } from '@/lib/grpc/handle-grpc-error';

// Validation schemas
const updatePolicyStoreSchema = z.object({
  description: z.string().min(1, 'Description is required').max(500, 'Description too long')
});

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  const { id: policyStoreId } = req.query;

  if (!policyStoreId || typeof policyStoreId !== 'string') {
    return res.status(400).json({
      error: 'Invalid policy store ID',
      required: ['id']
    });
  }

  try {
    switch (req.method) {
      case 'GET': {
        // Get a specific policy store with detailed information including metrics
        const [policyStoreResponse, schemaResponse] = await Promise.all([
          grpcClients.getPolicyStore({ policy_store_id: policyStoreId }),
          grpcClients.getSchema({ policy_store_id: policyStoreId }).catch(() => null)
        ]);

        // Get real policy count - fetch ALL policies to count them
        const allPolicies = await grpcClients.listPolicies({
          policy_store_id: policyStoreId,
          max_results: 1000 // Fetch up to 1000 policies to get real count
        }).catch(() => ({ policies: [] }));

        // Calculate real metrics from available data
        const metrics = {
          policies: allPolicies.policies?.length || 0,
          schemas: schemaResponse ? 1 : 0,
          lastModified: policyStoreResponse.updated_at,
          status: 'active', // Available from policy store data
          version: '1.0', // Default version (can be enhanced later)
          author: policyStoreResponse.description?.includes('Created by') ?
            policyStoreResponse.description.split('Created by')[1]?.trim() || 'system' : 'system',
          tags: [] // Can be implemented later with separate field
        };

        return res.status(200).json({
          ...policyStoreResponse,
          metrics
        });
      }

      case 'PUT':
      case 'PATCH': {
        // Validate request body
        const body = updatePolicyStoreSchema.parse(req.body);
        const { description } = body;

        // Call gRPC backend to update policy store
        const response = await grpcClients.updatePolicyStore({
          policy_store_id: policyStoreId,
          description
        });

        return res.status(200).json(response);
      }

      case 'DELETE': {
        // Call gRPC backend to delete policy store
        await grpcClients.deletePolicyStore({ policy_store_id: policyStoreId });
        return res.status(200).json({ success: true });
      }

      default:
        return res.status(405).json({
          error: 'Method not allowed',
          allowedMethods: ['GET', 'PUT', 'PATCH', 'DELETE']
        });
    }
  } catch (error: any) {
    console.error('Policy Store API error:', error);

    // Handle Zod validation errors
    if (error.name === 'ZodError') {
      return res.status(400).json({
        error: 'Validation failed',
        details: error.errors
      });
    }

    // Handle gRPC errors
    const grpcError = handleGRPCError(error);
    return res.status(grpcError.status).json({
      error: grpcError.message,
      details: grpcError.details
    });
  }
}
