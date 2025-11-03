import type { NextApiRequest, NextApiResponse } from 'next';
import { grpcClients } from '@/lib/grpc/node-client';

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  const { id: policyStoreId, snapshotId } = req.query;

  if (!policyStoreId || typeof policyStoreId !== 'string') {
    return res.status(400).json({ error: 'Policy store ID is required' });
  }

  if (!snapshotId || typeof snapshotId !== 'string') {
    return res.status(400).json({ error: 'Snapshot ID is required' });
  }

  try {
    if (req.method === 'GET') {
      // Get a specific snapshot
      const response = await grpcClients.getPolicyStoreSnapshot({
        policy_store_id: policyStoreId,
        snapshot_id: snapshotId,
      });

      return res.status(200).json(response);
    } else if (req.method === 'DELETE') {
      // Delete a snapshot
      await grpcClients.deleteSnapshot({
        policy_store_id: policyStoreId,
        snapshot_id: snapshotId,
      });

      return res.status(204).send(null);
    } else if (req.method === 'POST') {
      // Rollback to this snapshot
      const { description } = req.body;

      const response = await grpcClients.rollbackToSnapshot({
        policy_store_id: policyStoreId,
        snapshot_id: snapshotId,
        description: description || undefined,
      });

      return res.status(200).json(response);
    } else {
      res.setHeader('Allow', ['GET', 'POST', 'DELETE']);
      return res.status(405).json({ error: `Method ${req.method} Not Allowed` });
    }
  } catch (error) {
    console.error('Snapshot API error:', error);
    return res.status(500).json({
      error: error instanceof Error ? error.message : 'Internal server error',
    });
  }
}
