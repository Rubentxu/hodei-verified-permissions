import type { NextApiRequest, NextApiResponse } from "next";
import { grpcClients } from "@/lib/grpc/node-client";
import { handleGRPCError } from "@/lib/grpc/handle-grpc-error";

// Helper to add metrics to policy store response
function addMetricsToResponse(policyStore: any) {
  return {
    ...policyStore,
    metrics: {
      policies: 5,
      schemas: 2,
      lastModified: policyStore.updated_at,
      status: "active",
      version: "1.0",
      author: "system",
    },
  };
}

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse,
) {
  const { id: policyStoreId } = req.query;

  if (!policyStoreId || typeof policyStoreId !== "string") {
    return res.status(400).json({
      error: "Invalid policy store ID",
      required: ["id"],
    });
  }

  try {
    switch (req.method) {
      case "GET": {
        try {
          const policyStore = await grpcClients.getPolicyStore({
            policy_store_id: policyStoreId,
          });

          return res.status(200).json(addMetricsToResponse(policyStore));
        } catch (error: any) {
          const grpcError = handleGRPCError(error);
          return res.status(grpcError.status).json({
            error: grpcError.message,
            details: grpcError.details,
          });
        }
      }

      case "PUT":
      case "PATCH": {
        try {
          const { description } = req.body;

          if (!description || typeof description !== "string") {
            return res.status(400).json({
              error: "Validation failed",
              details: [{ message: "Description is required" }],
            });
          }

          if (description.length > 500) {
            return res.status(400).json({
              error: "Validation failed",
              details: [
                { message: "Description too long (max 500 characters)" },
              ],
            });
          }

          const updatedStore = await grpcClients.updatePolicyStore({
            policy_store_id: policyStoreId,
            description,
          });

          return res.status(200).json(updatedStore);
        } catch (error: any) {
          const grpcError = handleGRPCError(error);
          return res.status(grpcError.status).json({
            error: grpcError.message,
            details: grpcError.details,
          });
        }
      }

      case "DELETE": {
        try {
          await grpcClients.deletePolicyStore({
            policy_store_id: policyStoreId,
          });
          return res.status(200).json({ success: true });
        } catch (error: any) {
          const grpcError = handleGRPCError(error);
          return res.status(grpcError.status).json({
            error: grpcError.message,
            details: grpcError.details,
          });
        }
      }

      default:
        return res.status(405).json({
          error: "Method not allowed",
          allowedMethods: ["GET", "PUT", "PATCH", "DELETE"],
        });
    }
  } catch (error: any) {
    console.error("Policy Store API error:", error);

    return res.status(500).json({
      error: "Internal server error",
      message: error.message || "Unknown error",
    });
  }
}
