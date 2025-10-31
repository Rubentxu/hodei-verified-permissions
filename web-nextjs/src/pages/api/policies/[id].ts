import type { NextApiRequest, NextApiResponse } from 'next';
import { z } from 'zod';
import { grpcClients } from '@/lib/grpc/node-client';
import { handleGRPCError } from '@/lib/grpc/handle-grpc-error';

// Validation schemas
const getPolicySchema = z.object({
  policy_store_id: z.string().min(1, 'Policy store ID is required'),
  policy_id: z.string().min(1, 'Policy ID is required')
});

const updatePolicySchema = z.object({
  policy_store_id: z.string().min(1, 'Policy store ID is required'),
  policy_id: z.string().min(1, 'Policy ID is required'),
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

const deletePolicySchema = z.object({
  policy_store_id: z.string().min(1, 'Policy store ID is required'),
  policy_id: z.string().min(1, 'Policy ID is required')
});

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  const { id: policyId } = req.query;
  const { policy_store_id } = req.query;

  if (!policyId || !policy_store_id) {
    return res.status(400).json({
      error: 'Missing required parameters',
      required: ['policy_store_id', 'policy_id']
    });
  }

  try {
    switch (req.method) {
      case 'GET': {
        const params = getPolicySchema.parse({
          policy_store_id,
          policy_id: policyId
        });

        const response = await grpcClients.getPolicy({
          policy_store_id: params.policy_store_id,
          policy_id: params.policy_id
        });

        return res.status(200).json(response);
      }

      case 'PUT': {
        const body = updatePolicySchema.parse(req.body);
        const { policy_store_id, policy_id, definition } = body;

        // Call gRPC backend - update policy
        const response = await grpcClients.updatePolicy({
          policy_store_id,
          policy_id,
          definition
        });

        return res.status(200).json(response);
      }

      case 'DELETE': {
        const params = deletePolicySchema.parse({
          policy_store_id,
          policy_id: policyId
        });

        // Call gRPC backend - delete policy
        await grpcClients.deletePolicy({
          policy_store_id: params.policy_store_id,
          policy_id: params.policy_id
        });

        return res.status(204).send(null);
      }

      default:
        return res.status(405).json({
          error: 'Method not allowed',
          allowedMethods: ['GET', 'PUT', 'DELETE']
        });
    }
  } catch (error: any) {
    console.error('Policy API error:', error);

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
