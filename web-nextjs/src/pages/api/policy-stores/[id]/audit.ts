import type { NextApiRequest, NextApiResponse } from 'next';
import { grpcClients } from '@/lib/grpc/node-client';

interface AuditLogEntry {
  id: number;
  policy_store_id: string;
  action: string;
  user_id: string;
  changes: string | null;
  ip_address: string | null;
  timestamp: string;
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

  // Only allow GET requests
  if (req.method !== 'GET') {
    res.setHeader('Allow', ['GET']);
    return res.status(405).json({ error: 'Method not allowed' });
  }

  try {
    // Call gRPC method to get audit log
    const response = await grpcClients.getPolicyStoreAuditLog({
      policy_store_id: policyStoreId
    });

    // Transform the response to include the metrics structure
    const auditLogs: AuditLogEntry[] = response.audit_logs?.map((log) => ({
      id: log.id || 0,
      policy_store_id: log.policy_store_id,
      action: log.action,
      user_id: log.user_id,
      changes: log.changes || null,
      ip_address: log.ip_address || null,
      timestamp: log.timestamp || new Date().toISOString()
    })) || [];

    return res.status(200).json({
      audit_logs: auditLogs,
      count: auditLogs.length
    });
  } catch (error) {
    console.error('Failed to fetch audit log:', error);

    // If gRPC call fails, return empty audit log
    return res.status(200).json({
      audit_logs: [],
      count: 0,
      error: 'Unable to fetch audit log'
    });
  }
}
