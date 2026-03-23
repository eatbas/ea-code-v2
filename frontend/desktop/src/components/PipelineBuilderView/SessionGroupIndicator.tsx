import type { ReactNode } from "react";
import { sessionGroupClass } from "../../utils/sessionGroupClass";

interface SessionGroupIndicatorProps {
  group: string;
}

export function SessionGroupIndicator({
  group,
}: SessionGroupIndicatorProps): ReactNode {
  return (
    <span className={`rounded px-2 py-0.5 text-[11px] ${sessionGroupClass(group)}`}>
      Group {group}
    </span>
  );
}
