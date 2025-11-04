import type { NextApiRequest, NextApiResponse } from "next";
import { NextResponse } from "next/server";

export async function GET(
  req: NextApiRequest,
  { params }: { params: { id: string } },
) {
  try {
    const { id } = params;
    const { searchParams } = new URL(req.url!);

    // Extract query parameters
    const eventTypes =
      searchParams.get("event_types")?.split(",").filter(Boolean) || [];
    const maxResults = parseInt(searchParams.get("max_results") || "100");
    const startTime = searchParams.get("start_time") || undefined;
    const endTime = searchParams.get("end_time") || undefined;

    // Build gRPC request
    const grpcRequest = {
      policy_store_id: id,
      event_types: eventTypes,
      max_results: maxResults,
      start_time: startTime,
      end_time: endTime,
    };

    // Call backend API
    const response = await fetch(
      `${process.env.API_URL || "http://localhost:50051"}/audit`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${process.env.API_KEY || ""}`,
        },
        body: JSON.stringify(grpcRequest),
      },
    );

    if (!response.ok) {
      throw new Error(`Backend API error: ${response.statusText}`);
    }

    const data = await response.json();
    return NextResponse.json(data);
  } catch (error) {
    console.error("Error fetching audit log:", error);
    return NextResponse.json(
      { error: "Failed to fetch audit log" },
      { status: 500 },
    );
  }
}

export async function POST(
  req: NextApiRequest,
  { params }: { params: { id: string } },
) {
  try {
    const { id } = params;

    // Export audit log
    const response = await fetch(
      `${process.env.API_URL || "http://localhost:50051"}/audit/export`,
      {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
          Authorization: `Bearer ${process.env.API_KEY || ""}`,
        },
        body: JSON.stringify({
          policy_store_id: id,
        }),
      },
    );

    if (!response.ok) {
      throw new Error(`Backend API error: ${response.statusText}`);
    }

    const data = await response.json();
    return NextResponse.json(data);
  } catch (error) {
    console.error("Error exporting audit log:", error);
    return NextResponse.json(
      { error: "Failed to export audit log" },
      { status: 500 },
    );
  }
}
