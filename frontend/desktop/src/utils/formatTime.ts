/**
 * Formats an ISO timestamp or date string for display.
 *
 * When `style` is `"time"`, returns HH:MM:SS only.
 * When `style` is `"full"` (the default), returns a full locale string.
 * Returns `"Never"` for null/undefined, or the raw string if unparseable.
 */
export function formatTime(
  value: string | null | undefined,
  style: "full" | "time" = "full",
): string {
  if (!value) return "Never";
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) return value;
  if (style === "time") {
    return parsed.toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  }
  return parsed.toLocaleString();
}
