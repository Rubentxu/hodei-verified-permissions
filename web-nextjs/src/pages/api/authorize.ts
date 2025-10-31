import type { NextApiRequest, NextApiResponse } from "next";

export default async function handler(
  req: NextApiRequest,
  res: NextApiResponse
) {
  if (req.method !== "POST") {
    return res.status(405).json({ error: "Method not allowed" });
  }

  try {
    const { grpcClients } = await import("../../lib/grpc/node-client");

    const {
      policy_store_id,
      principal,
      action,
      resource,
      context,
      entities,
    } = (req.body || {}) as {
      policy_store_id?: string;
      principal?: { entity_type?: string; entity_id?: string };
      action?: { entity_type?: string; entity_id?: string };
      resource?: { entity_type?: string; entity_id?: string };
      context?: string;
      entities?: any[];
    };

    // Validate payload
    const errors: string[] = [];
    if (!policy_store_id) errors.push("policy_store_id is required");
    if (!principal?.entity_type || !principal?.entity_id)
      errors.push("principal.entity_type and principal.entity_id are required");
    if (!action?.entity_type || !action?.entity_id)
      errors.push("action.entity_type and action.entity_id are required");
    if (!resource?.entity_type || !resource?.entity_id)
      errors.push("resource.entity_type and resource.entity_id are required");

    if (errors.length > 0) {
      return res.status(400).json({ error: "Invalid request", details: errors });
    }

    const request = {
      policy_store_id,
      principal: { entity_type: principal.entity_type!, entity_id: principal.entity_id! },
      action: { entity_type: action.entity_type!, entity_id: action.entity_id! },
      resource: { entity_type: resource.entity_type!, entity_id: resource.entity_id! },
      context: context || "{}",
      entities: entities || [],
    };

    const response = await grpcClients.isAuthorized(request);

    return res.status(200).json({
      decision: response.decision,
      determining_policies: response.determining_policies || [],
      errors: response.errors || [],
    });
  } catch (err: any) {
    const message = err?.message || "Unknown error";
    const code = err?.code || "UNKNOWN";
    console.error("Authorize API error:", code, message);
    return res.status(500).json({ error: "Internal server error", details: message, code });
  }
}
