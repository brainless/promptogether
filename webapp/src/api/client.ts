import type {
  ErrorResponse,
  GalleryProjectDetail,
  GalleryProjectFile,
  GalleryProjectSummary,
} from "./generated";

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

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function isJsonValue(value: unknown): boolean {
  if (value === null || typeof value === "string" || typeof value === "boolean") {
    return true;
  }

  if (typeof value === "number") {
    return Number.isFinite(value);
  }

  if (Array.isArray(value)) {
    return value.every(isJsonValue);
  }

  return isRecord(value) && Object.values(value).every(isJsonValue);
}

function isErrorResponse(value: unknown): value is ErrorResponse {
  return (
    isRecord(value) &&
    typeof value.code === "string" &&
    typeof value.message === "string" &&
    (!("details" in value) || value.details === null || isJsonValue(value.details))
  );
}

function isGalleryProjectSummary(value: unknown): value is GalleryProjectSummary {
  return (
    isRecord(value) &&
    typeof value.slug === "string" &&
    typeof value.title === "string" &&
    typeof value.summary === "string"
  );
}

function isGalleryProjectFile(value: unknown): value is GalleryProjectFile {
  return (
    isRecord(value) &&
    (value.phase === "initial" || value.phase === "result") &&
    typeof value.path === "string" &&
    (!("language" in value) || value.language === null || typeof value.language === "string") &&
    typeof value.content === "string"
  );
}

function isGalleryProjectDetail(value: unknown): value is GalleryProjectDetail {
  return (
    isGalleryProjectSummary(value) &&
    typeof value.prompt === "string" &&
    typeof value.createdAt === "string" &&
    typeof value.updatedAt === "string" &&
    Array.isArray(value.initialFiles) &&
    value.initialFiles.every(isGalleryProjectFile) &&
    Array.isArray(value.resultFiles) &&
    value.resultFiles.every(isGalleryProjectFile)
  );
}

async function request<T>(path: string, validate: (value: unknown) => value is T): Promise<T> {
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
      const body: unknown = await response.json();
      if (isErrorResponse(body)) {
        code = body.code;
        message = body.message;
        details = body.details ?? undefined;
      }
    } catch {
      // Response body is not valid JSON; keep defaults.
    }

    if (response.status === 404) {
      throw new NotFoundError(message);
    }

    throw new ApiError(response.status, code, message, details);
  }

  try {
    const body: unknown = await response.json();
    if (validate(body)) {
      return body;
    }
  } catch {
    // Fall through to the visitor-safe invalid response error.
  }

  throw new ApiError(0, "invalid_response", "Received an invalid response from the server.");
}

export function fetchGalleryProjects(): Promise<GalleryProjectSummary[]> {
  return request(
    "/gallery",
    (value): value is GalleryProjectSummary[] =>
      Array.isArray(value) && value.every(isGalleryProjectSummary),
  );
}

export function fetchGalleryProject(slug: string): Promise<GalleryProjectDetail> {
  return request(`/gallery/${encodeURIComponent(slug)}`, isGalleryProjectDetail);
}
