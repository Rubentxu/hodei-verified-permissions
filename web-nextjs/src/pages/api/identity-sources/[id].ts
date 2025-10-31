import type { NextApiRequest, NextApiResponse } from 'next';
import { z } from 'zod';
import { handleGRPCError } from '@/lib/grpc/handle-grpc-error';

// Validation schemas
const getIdentitySourceSchema = z.object({
  policy_store_id: z.string().min(1, 'Policy store ID is required'),
  identity_source_id: z.string().min(1, 'Identity source ID is required')
});

const updateIdentitySourceSchema = z.object({
  policy_store_id: z.string().min(1, 'Policy store ID is required'),
  identity_source_id: z.string().min(1, 'Identity source ID is required'),
  config: z.object({
    cognito: z.object({
      user_pool_id: z.string().min(1, 'User pool ID is required'),
      region: z.string().min(1, 'Region is required'),
      client_id: z.string().min(1, 'Client ID is required'),
      client_secret: z.string().min(1, 'Client secret is required'),
    }).optional(),
    oidc: z.object({
      issuer: z.string().url('Valid URL required'),
      client_id: z.string().min(1, 'Client ID is required'),
      client_secret: z.string().min(1, 'Client secret is required'),
      authorization_endpoint: z.string().url('Valid URL required'),
      token_endpoint: z.string().url('Valid URL required'),
      userinfo_endpoint: z.string().url('Valid URL required'),
      scopes: z.array(z.string()).default(['openid', 'profile', 'email']),
    }).optional(),
  })
});

const deleteIdentitySourceSchema = z.object({
  policy_store_id: z.string().min(1, 'Policy store ID is required'),
  identity_source_id: z.string().min(1, 'Identity source ID is required')
});

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  const { id: identitySourceId } = req.query;
  const { policy_store_id } = req.query;

  if (!identitySourceId || !policy_store_id) {
    return res.status(400).json({
      error: 'Missing required parameters',
      required: ['policy_store_id', 'identity_source_id']
    });
  }

  try {
    switch (req.method) {
      case 'GET': {
        const params = getIdentitySourceSchema.parse({
          policy_store_id,
          identity_source_id: identitySourceId
        });

        // TODO: Implement actual gRPC call when backend supports it
        // For now, return mock response
        const response = {
          identity_source_id: identitySourceId,
          policy_store_id: policy_store_id,
          config: {},
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
          status: 'active'
        };

        return res.status(200).json(response);
      }

      case 'PUT': {
        const body = updateIdentitySourceSchema.parse(req.body);
        const { policy_store_id, identity_source_id, config } = body;

        // TODO: Implement actual gRPC call when backend supports it
        // For now, return mock response
        const response = {
          identity_source_id,
          policy_store_id,
          config,
          updated_at: new Date().toISOString(),
          status: 'active'
        };

        return res.status(200).json(response);
      }

      case 'DELETE': {
        const params = deleteIdentitySourceSchema.parse({
          policy_store_id,
          identity_source_id: identitySourceId
        });

        // TODO: Implement actual gRPC call when backend supports it

        return res.status(204).send(null);
      }

      default:
        return res.status(405).json({
          error: 'Method not allowed',
          allowedMethods: ['GET', 'PUT', 'DELETE']
        });
    }
  } catch (error: any) {
    console.error('Identity Source API error:', error);

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
