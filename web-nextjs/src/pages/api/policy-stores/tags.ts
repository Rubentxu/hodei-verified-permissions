import type { NextApiRequest, NextApiResponse } from 'next';
import { grpcClients } from '@/lib/grpc/node-client';

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  // Only allow GET requests
  if (req.method !== 'GET') {
    res.setHeader('Allow', ['GET']);
    return res.status(405).json({ error: 'Method not allowed' });
  }

  try {
    // Call gRPC method to list all policy stores
    const response = await grpcClients.listPolicyStores({
      max_results: 1000
    });

    // Extract all unique tags from policy stores
    const allTags = new Set<string>();

    if (response.policy_stores) {
      response.policy_stores.forEach((store) => {
        if (store.tags) {
          try {
            // Parse tags JSON string
            const tags = JSON.parse(store.tags);
            if (Array.isArray(tags)) {
              tags.forEach((tag: string) => {
                if (tag && typeof tag === 'string' && tag.trim()) {
                  allTags.add(tag.trim());
                }
              });
            }
          } catch (e) {
            // Skip invalid JSON
            console.warn(`Failed to parse tags for store ${store.policy_store_id}:`, e);
          }
        }
      });
    }

    // Sort tags alphabetically
    const sortedTags = Array.from(allTags).sort();

    return res.status(200).json({
      tags: sortedTags,
      count: sortedTags.length
    });
  } catch (error) {
    console.error('Failed to fetch tags:', error);

    return res.status(500).json({
      error: 'Failed to fetch tags',
      message: (error as Error).message
    });
  }
}
