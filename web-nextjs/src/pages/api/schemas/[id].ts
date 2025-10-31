import type { NextApiRequest, NextApiResponse } from 'next';
import { z } from 'zod';
import { grpcClients } from '@/lib/grpc/node-client';
import { handleGRPCError } from '@/lib/grpc/handle-grpc-error';

// Validation schemas
const getSchemaSchema = z.object({
  policy_store_id: z.string().min(1, 'Policy store ID is required')
});

const putSchemaSchema = z.object({
  policy_store_id: z.string().min(1, 'Policy store ID is required'),
  schema: z.string().min(1, 'Schema is required')
});

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  const { id: policyStoreId } = req.query;

  if (!policyStoreId) {
    return res.status(400).json({
      error: 'Missing required parameter: policy_store_id'
    });
  }

  try {
    switch (req.method) {
      case 'GET': {
        const params = getSchemaSchema.parse({ policy_store_id: policyStoreId });

        const response = await grpcClients.getSchema({
          policy_store_id: params.policy_store_id
        });

        return res.status(200).json(response);
      }

      case 'PUT': {
        const body = putSchemaSchema.parse(req.body);
        const { policy_store_id, schema } = body;

        // Validate JSON schema format
        try {
          JSON.parse(schema);
        } catch (e) {
          return res.status(400).json({
            error: 'Invalid JSON schema',
            details: e.message
          });
        }

        // Call gRPC backend
        const response = await grpcClients.putSchema({
          policy_store_id,
          schema
        });

        return res.status(200).json(response);
      }

      default:
        return res.status(405).json({
          error: 'Method not allowed',
          allowedMethods: ['GET', 'PUT']
        });
    }
  } catch (error: any) {
    console.error('Schema API error:', error);

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
