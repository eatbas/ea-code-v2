/** Returns a TailwindCSS badge class string for the given session group letter. */
export function sessionGroupClass(group: string | undefined): string {
  if (!group) return "bg-slate-100 text-slate-700";
  const map: Record<string, string> = {
    A: "bg-indigo-100 text-indigo-700",
    B: "bg-teal-100 text-teal-700",
    C: "bg-orange-100 text-orange-700",
    D: "bg-pink-100 text-pink-700",
  };
  return map[group] ?? "bg-violet-100 text-violet-700";
}
