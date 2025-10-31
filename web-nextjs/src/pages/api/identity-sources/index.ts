import type { NextApiRequest, NextApiResponse } from 'next';
import { z } from 'zod';
import { handleGRPCError } from '@/lib/grpc/handle-grpc-error';

// Validation schemas
const createIdentitySourceSchema = z.object({
  policy_store_id: z.string().min(1, 'Policy store ID is required'),
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
  }).refine(
    (data) => data.config.cognito || data.config.oidc,
    {
      message: 'Must provide either cognito or oidc configuration'
    }
  )
});

const listIdentitySourcesSchema = z.object({
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
        const queryParams = listIdentitySourcesSchema.parse(req.query);
        const { policy_store_id, max_results, next_token } = queryParams;

        // TODO: Implement actual gRPC call when backend supports it
        // For now, return empty list as mock
        const response = {
          identity_sources: [],
          next_token: undefined
        };

        return res.status(200).json(response);
      }

      case 'POST': {
        // Validate request body
        const body = createIdentitySourceSchema.parse(req.body);
        const { policy_store_id, config } = body;

        // TODO: Implement actual gRPC call when backend supports it
        // For now, return mock response
        const identitySourceId = 'idn_' + Date.now();
        const response = {
          identity_source_id: identitySourceId,
          policy_store_id,
          config,
          created_at: new Date().toISOString(),
          updated_at: new Date().toISOString(),
          status: 'active'
        };

        return res.status(201).json(response);
      }

      default:
        return res.status(405).json({
          error: 'Method not allowed',
          allowedMethods: ['GET', 'POST']
        });
    }
  } catch (error: any) {
    console.error('Identity Sources API error:', error);

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
