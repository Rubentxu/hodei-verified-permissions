import type { NextApiRequest, NextApiResponse } from 'next';
import { z } from 'zod';
import { grpcClients } from '@/lib/grpc/node-client';
import { handleGRPCError } from '@/lib/grpc/handle-grpc-error';

// Validation schemas
const createPolicySchema = z.object({
  policy_store_id: z.string().min(1, 'Policy store ID is required'),
  definition: z.object({
    static_policy: z.object({
      description: z.string(),
      statement: z.string(),
      applies_to: z.array(z.object({
        resource_type: z.string(),
        resource_id: z.string().optional()
      }))
    }).optional(),
    template_linked_policy: z.object({
      policy_template_id: z.string(),
      principal_verification_context: z.object({
        token: z.string().optional(),
        entity: z.object({
          entity_type: z.string(),
          entity_id: z.string()
        }).optional()
      }).optional(),
      resource_verification_context: z.object({
        token: z.string().optional(),
        entity: z.object({
          entity_type: z.string(),
          entity_id: z.string()
        }).optional()
      }).optional()
    }).optional()
  }).refine(
    (data) => data.definition.static_policy || data.definition.template_linked_policy,
    {
      message: 'Policy must have either static_policy or template_linked_policy'
    }
  )
});

const listPoliciesSchema = z.object({
  policy_store_id: z.string().min(1, 'Policy store ID is required'),
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
        const queryParams = listPoliciesSchema.parse(req.query);
        const { policy_store_id, max_results, next_token } = queryParams;

        const response = await grpcClients.listPolicies({
          policy_store_id,
          max_results: max_results ? parseInt(max_results) : undefined,
          next_token
        });

        return res.status(200).json(response);
      }

      case 'POST': {
        // Validate request body
        const body = createPolicySchema.parse(req.body);

        // Call gRPC backend
        const response = await grpcClients.createPolicy(body);

        return res.status(201).json(response);
      }

      default:
        return res.status(405).json({
          error: 'Method not allowed',
          allowedMethods: ['GET', 'POST']
        });
    }
  } catch (error: any) {
    console.error('Policies API error:', error);

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
