/** Formats an ISO-ish date string as e.g. "September 20, 2026"; returns the input unchanged if it can't be parsed. */
export function formatDate(iso: string): string {
  const date = new Date(iso);
  if (Number.isNaN(date.getTime())) return iso;
  return date.toLocaleDateString("en-US", { year: "numeric", month: "long", day: "numeric" });
}
