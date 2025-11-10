import { NextRequest, NextResponse } from "next/server";

const API_BASE_URL = process.env.API_BASE_URL || "http://localhost:3001";

// GET /api/time-entries/current - Get currently running timer
export async function GET(request: NextRequest) {
  try {
    const authHeader = request.headers.get("authorization");
    if (!authHeader) {
      return NextResponse.json({ error: "Unauthorized" }, { status: 401 });
    }

    const response = await fetch(`${API_BASE_URL}/api/time-entries/current`, {
      method: "GET",
      headers: {
        Authorization: authHeader,
        "Content-Type": "application/json",
      },
    });

    if (response.status === 404) {
      // No running timer is not an error - return null
      return NextResponse.json({ data: null });
    }

    if (!response.ok) {
      const errorData = await response
        .json()
        .catch(() => ({ error: "Request failed" }));
      return NextResponse.json(
        { error: errorData.error || "Failed to fetch current timer" },
        { status: response.status },
      );
    }

    const data = await response.json();
    // Wrap the response in a data object to match client expectations
    return NextResponse.json({ data });
  } catch (error) {
    console.error("Current timer API error:", error);
    return NextResponse.json(
      { error: "Internal server error" },
      { status: 500 },
    );
  }
}
