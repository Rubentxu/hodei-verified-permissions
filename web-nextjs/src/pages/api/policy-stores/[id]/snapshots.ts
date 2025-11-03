import type { NextApiRequest, NextApiResponse } from 'next';
import { grpcClients } from '@/lib/grpc/node-client';

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  const { id: policyStoreId } = req.query;

  if (!policyStoreId || typeof policyStoreId !== 'string') {
    return res.status(400).json({ error: 'Policy store ID is required' });
  }

  try {
    if (req.method === 'GET') {
      // List all snapshots for the policy store
      const response = await grpcClients.listPolicyStoreSnapshots({
        policy_store_id: policyStoreId,
      });

      return res.status(200).json(response);
    } else if (req.method === 'POST') {
      // Create a new snapshot
      const { description } = req.body;

      const response = await grpcClients.createPolicyStoreSnapshot({
        policy_store_id: policyStoreId,
        description: description || undefined,
      });

      return res.status(201).json(response);
    } else {
      res.setHeader('Allow', ['GET', 'POST']);
      return res.status(405).json({ error: `Method ${req.method} Not Allowed` });
    }
  } catch (error) {
    console.error('Snapshots API error:', error);
    return res.status(500).json({
      error: error instanceof Error ? error.message : 'Internal server error',
    });
  }
}
