import type { NextApiRequest, NextApiResponse } from "next";
// import { grpcClients } from '@/lib/grpc/node-client';

// Mock snapshots data
const mockSnapshots: Record<string, any[]> = {};

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse,
) {
  const { id: policyStoreId } = req.query;

  if (!policyStoreId || typeof policyStoreId !== "string") {
    return res.status(400).json({ error: "Policy store ID is required" });
  }

  try {
    if (req.method === "GET") {
      // List all snapshots for the policy store
      // Initialize mock snapshots if doesn't exist
      if (!mockSnapshots[policyStoreId]) {
        mockSnapshots[policyStoreId] = [
          {
            snapshot_id: "snap-001",
            policy_store_id: policyStoreId,
            version: 1,
            created_at: "2024-01-15T10:30:00Z",
            created_by: "admin",
            description: "Initial snapshot",
            size: 1024,
          },
          {
            snapshot_id: "snap-002",
            policy_store_id: policyStoreId,
            version: 2,
            created_at: "2024-01-16T14:20:00Z",
            created_by: "admin",
            description: "After policy updates",
            size: 1152,
          },
        ];
      }

      return res.status(200).json({
        snapshots: mockSnapshots[policyStoreId],
      });
    } else if (req.method === "POST") {
      // Create a new snapshot
      const { description } = req.body;

      // Initialize mock snapshots if doesn't exist
      if (!mockSnapshots[policyStoreId]) {
        mockSnapshots[policyStoreId] = [];
      }

      const newSnapshot = {
        snapshot_id: `snap-${Date.now()}`,
        policy_store_id: policyStoreId,
        version: mockSnapshots[policyStoreId].length + 1,
        created_at: new Date().toISOString(),
        created_by: "admin",
        description: description || "Manual snapshot",
        size: 1024,
      };

      mockSnapshots[policyStoreId].unshift(newSnapshot);

      return res.status(201).json(newSnapshot);
    } else {
      res.setHeader("Allow", ["GET", "POST"]);
      return res
        .status(405)
        .json({ error: `Method ${req.method} Not Allowed` });
    }
  } catch (error) {
    console.error("Snapshots API error:", error);
    return res.status(500).json({
      error: error instanceof Error ? error.message : "Internal server error",
    });
  }
}
