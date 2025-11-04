import type { NextApiRequest, NextApiResponse } from "next";
// import { grpcClients } from '@/lib/grpc/node-client';

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse,
) {
  // Only allow GET requests
  if (req.method !== "GET") {
    res.setHeader("Allow", ["GET"]);
    return res.status(405).json({ error: "Method not allowed" });
  }

  try {
    // Return mock tags for autocomplete
    const mockTags = [
      "production",
      "staging",
      "development",
      "testing",
      "critical",
      "internal",
      "external",
      "deprecated",
      "beta",
      "stable",
    ];

    return res.status(200).json({
      tags: mockTags,
      count: mockTags.length,
    });
  } catch (error) {
    console.error("Failed to fetch tags:", error);

    return res.status(500).json({
      error: "Failed to fetch tags",
      message: (error as Error).message,
    });
  }
}
