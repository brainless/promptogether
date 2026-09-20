/** Extracts a YouTube video ID from a watch/share/embed/shorts URL, or null if `url` isn't a recognizable YouTube link. */
export function extractYouTubeId(url: string): string | null {
  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    return null;
  }

  if (parsed.hostname === "youtu.be") {
    return parsed.pathname.slice(1) || null;
  }

  if (parsed.hostname.endsWith("youtube.com")) {
    if (parsed.pathname === "/watch") return parsed.searchParams.get("v");
    const match = parsed.pathname.match(/^\/(?:embed|shorts)\/([^/]+)/);
    if (match) return match[1];
  }

  return null;
}
