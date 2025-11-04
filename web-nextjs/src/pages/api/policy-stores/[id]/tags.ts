import type { NextApiRequest, NextApiResponse } from "next";
// import { grpcClients } from '@/lib/grpc/node-client';

// Mock data store for tags (in production, this would be a database)
const mockTagsStore: Record<string, string[]> = {};

interface TagRequest {
  tags: string[];
}

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse,
) {
  // Extract policy store ID from URL
  const policyStoreId = req.query.id as string;

  if (!policyStoreId) {
    return res.status(400).json({ error: "Policy store ID is required" });
  }

  try {
    switch (req.method) {
      case "PUT": {
        // Update tags for policy store
        const { tags } = req.body as TagRequest;

        if (!Array.isArray(tags)) {
          return res.status(400).json({ error: "Tags must be an array" });
        }

        // Initialize mock store entry if doesn't exist
        if (!mockTagsStore[policyStoreId]) {
          mockTagsStore[policyStoreId] = [];
        }

        mockTagsStore[policyStoreId] = tags;

        return res.status(200).json({
          success: true,
          tags: tags,
          message: "Tags updated successfully",
        });
      }

      case "GET": {
        // Get current tags for policy store
        const tags = mockTagsStore[policyStoreId] || [];

        return res.status(200).json({
          tags: tags,
        });
      }

      case "POST": {
        // Add a single tag
        const { tag } = req.body as { tag: string };

        if (!tag || typeof tag !== "string") {
          return res
            .status(400)
            .json({ error: "Tag must be a non-empty string" });
        }

        // Initialize mock store entry if doesn't exist
        if (!mockTagsStore[policyStoreId]) {
          mockTagsStore[policyStoreId] = [];
        }

        // Add new tag if not exists
        if (!mockTagsStore[policyStoreId].includes(tag)) {
          mockTagsStore[policyStoreId].push(tag);
        }

        return res.status(200).json({
          success: true,
          tags: mockTagsStore[policyStoreId],
          message: "Tag added successfully",
        });
      }

      case "DELETE": {
        // Remove a single tag
        const { tag } = req.body as { tag: string };

        if (!tag || typeof tag !== "string") {
          return res
            .status(400)
            .json({ error: "Tag must be a non-empty string" });
        }

        // Initialize mock store entry if doesn't exist
        if (!mockTagsStore[policyStoreId]) {
          mockTagsStore[policyStoreId] = [];
        }

        // Remove tag if exists
        mockTagsStore[policyStoreId] = mockTagsStore[policyStoreId].filter(
          (t) => t !== tag,
        );

        return res.status(200).json({
          success: true,
          tags: mockTagsStore[policyStoreId],
          message: "Tag removed successfully",
        });
      }

      default:
        res.setHeader("Allow", ["GET", "PUT", "POST", "DELETE"]);
        return res.status(405).json({ error: "Method not allowed" });
    }
  } catch (error) {
    console.error("Failed to manage tags:", error);

    return res.status(500).json({
      error: "Failed to manage tags",
      message: (error as Error).message,
    });
  }
}
