import type { NextApiRequest, NextApiResponse } from 'next';
import { grpcClients } from '@/lib/grpc/node-client';

interface TagRequest {
  tags: string[];
}

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  // Extract policy store ID from URL
  const policyStoreId = req.query.id as string;

  if (!policyStoreId) {
    return res.status(400).json({ error: 'Policy store ID is required' });
  }

  try {
    switch (req.method) {
      case 'PUT': {
        // Update tags for policy store
        const { tags } = req.body as TagRequest;

        if (!Array.isArray(tags)) {
          return res.status(400).json({ error: 'Tags must be an array' });
        }

        // Call gRPC method to update tags
        const response = await grpcClients.updatePolicyStoreTags({
          policy_store_id: policyStoreId,
          tags: tags
        });

        return res.status(200).json({
          success: true,
          tags: response.tags || [],
          message: 'Tags updated successfully'
        });
      }

      case 'GET': {
        // Get current tags for policy store
        const response = await grpcClients.getPolicyStore({
          policy_store_id: policyStoreId
        });

        // Parse tags from JSON string
        const tags = response.tags ? JSON.parse(response.tags) : [];

        return res.status(200).json({
          tags: tags
        });
      }

      case 'POST': {
        // Add a single tag
        const { tag } = req.body as { tag: string };

        if (!tag || typeof tag !== 'string') {
          return res.status(400).json({ error: 'Tag must be a non-empty string' });
        }

        // First get current tags
        const currentStore = await grpcClients.getPolicyStore({
          policy_store_id: policyStoreId
        });

        const currentTags = currentStore.tags ? JSON.parse(currentStore.tags) : [];

        // Add new tag if not exists
        if (!currentTags.includes(tag)) {
          currentTags.push(tag);

          // Update with new tags
          await grpcClients.updatePolicyStoreTags({
            policy_store_id: policyStoreId,
            tags: currentTags
          });
        }

        return res.status(200).json({
          success: true,
          tags: currentTags,
          message: 'Tag added successfully'
        });
      }

      case 'DELETE': {
        // Remove a single tag
        const { tag } = req.body as { tag: string };

        if (!tag || typeof tag !== 'string') {
          return res.status(400).json({ error: 'Tag must be a non-empty string' });
        }

        // First get current tags
        const currentStore = await grpcClients.getPolicyStore({
          policy_store_id: policyStoreId
        });

        const currentTags = currentStore.tags ? JSON.parse(currentStore.tags) : [];

        // Remove tag if exists
        const updatedTags = currentTags.filter((t: string) => t !== tag);

        // Update with new tags
        await grpcClients.updatePolicyStoreTags({
          policy_store_id: policyStoreId,
          tags: updatedTags
        });

        return res.status(200).json({
          success: true,
          tags: updatedTags,
          message: 'Tag removed successfully'
        });
      }

      default:
        res.setHeader('Allow', ['GET', 'PUT', 'POST', 'DELETE']);
        return res.status(405).json({ error: 'Method not allowed' });
    }
  } catch (error) {
    console.error('Failed to manage tags:', error);

    return res.status(500).json({
      error: 'Failed to manage tags',
      message: (error as Error).message
    });
  }
}
