import { useMemo, type ReactNode } from "react";
import type { DroneInfo } from "../../types";

interface DroneInventoryProps {
  drones: DroneInfo[];
}

export function DroneInventory({ drones }: DroneInventoryProps): ReactNode {
  const groupedDrones = useMemo(() => {
    const grouped = new Map<string, number>();
    for (const drone of drones) {
      const key = `${drone.provider}/${drone.model}`;
      grouped.set(key, (grouped.get(key) ?? 0) + 1);
    }
    return Array.from(grouped.entries()).sort(([left], [right]) =>
      left.localeCompare(right),
    );
  }, [drones]);

  return (
    <section className="rounded border border-slate-200 p-4">
      <h2 className="mb-2 text-sm font-semibold text-slate-800">
        Drone Inventory ({drones.length})
      </h2>
      {groupedDrones.length === 0 ? (
        <p className="text-xs text-slate-600">No active drones reported.</p>
      ) : (
        <ul className="space-y-1 text-sm text-slate-700">
          {groupedDrones.map(([key, count]) => (
            <li key={key}>
              {key} · {count} active
            </li>
          ))}
        </ul>
      )}
    </section>
  );
}
