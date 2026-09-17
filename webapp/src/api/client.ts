import type { GalleryProjectSummary, GalleryProjectDetail, ErrorResponse } from "./generated";

export class ApiError extends Error {
  constructor(
    public readonly status: number,
    public readonly code: string,
    message: string,
    public readonly details?: unknown,
  ) {
    super(message);
    this.name = "ApiError";
  }
}

export class NotFoundError extends ApiError {
  constructor(message = "Resource not found") {
    super(404, "not_found", message);
    this.name = "NotFoundError";
  }
}

const BASE = "/api";

async function request<T>(path: string): Promise<T> {
  let response: Response;

  try {
    response = await fetch(`${BASE}${path}`);
  } catch {
    throw new ApiError(0, "network_error", "Unable to reach the server. Please check your connection.");
  }

  if (!response.ok) {
    let code = "unknown";
    let message = `Request failed with status ${response.status}`;
    let details: unknown;

    try {
      const body: ErrorResponse = await response.json();
      code = body.code;
      message = body.message;
      details = body.details ?? undefined;
    } catch {
      // Response body is not valid JSON; keep defaults.
    }

    if (response.status === 404) {
      throw new NotFoundError(message);
    }

    throw new ApiError(response.status, code, message, details);
  }

  try {
    return (await response.json()) as T;
  } catch {
    throw new ApiError(0, "invalid_response", "Received an invalid response from the server.");
  }
}

export function fetchGalleryProjects(): Promise<GalleryProjectSummary[]> {
  return request<GalleryProjectSummary[]>("/gallery");
}

export function fetchGalleryProject(slug: string): Promise<GalleryProjectDetail> {
  return request<GalleryProjectDetail>(`/gallery/${encodeURIComponent(slug)}`);
}
