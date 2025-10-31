import type { NextApiRequest, NextApiResponse } from 'next';
import { z } from 'zod';
import { grpcClients } from '@/lib/grpc/node-client';
import { handleGRPCError } from '@/lib/grpc/handle-grpc-error';

// Validation schemas
const createPolicyStoreSchema = z.object({
  description: z.string().min(1, 'Description is required').max(500, 'Description too long')
});

const listPolicyStoresSchema = z.object({
  max_results: z.string().optional(),
  next_token: z.string().optional()
});

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  try {
    switch (req.method) {
      case 'GET': {
        // Parse query parameters
        const queryParams = listPolicyStoresSchema.parse(req.query);
        const { max_results, next_token } = queryParams;

        const response = await grpcClients.listPolicyStores({
          max_results: max_results ? parseInt(max_results) : undefined,
          next_token
        });

        return res.status(200).json(response);
      }

      case 'POST': {
        // Validate request body
        const body = createPolicyStoreSchema.parse(req.body);
        const { description } = body;

        // Call gRPC backend
        const response = await grpcClients.createPolicyStore({ description });

        return res.status(201).json(response);
      }

      default:
        return res.status(405).json({
          error: 'Method not allowed',
          allowedMethods: ['GET', 'POST']
        });
    }
  } catch (error: any) {
    console.error('Policy Stores API error:', error);

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
