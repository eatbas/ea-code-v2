import type { ReactNode } from "react";
import type { ActiveView } from "../types";
import { useAppContext } from "../contexts/AppContext";

interface NavItem {
  view: ActiveView;
  label: string;
}

const NAV_ITEMS: NavItem[] = [
  { view: "home", label: "Home" },
  { view: "chat", label: "Chat" },
  { view: "sessions", label: "Sessions" },
  { view: "pipeline-gallery", label: "Pipelines" },
  { view: "pipeline-builder", label: "Builder" },
  { view: "hive-api-status", label: "hive-api" },
  { view: "skills", label: "Skills" },
  { view: "mcp", label: "MCP" },
  { view: "settings", label: "Settings" },
];

interface SidebarProps {
  collapsed: boolean;
  onToggle: () => void;
}

export function Sidebar({ collapsed, onToggle }: SidebarProps): ReactNode {
  const { activeView, dispatch } = useAppContext();

  return (
    <aside
      className={`border-r border-slate-200 bg-slate-50 transition-all ${
        collapsed ? "w-16" : "w-64"
      }`}
    >
      <div className="flex items-center justify-between border-b border-slate-200 px-3 py-3">
        {!collapsed ? <strong className="text-sm">ea-code v2</strong> : null}
        <button
          className="rounded border border-slate-300 px-2 py-1 text-xs"
          onClick={onToggle}
          type="button"
        >
          {collapsed ? ">" : "<"}
        </button>
      </div>

      <nav className="flex flex-col gap-1 p-2">
        {NAV_ITEMS.map((item) => {
          const active = item.view === activeView;
          return (
            <button
              className={`rounded px-3 py-2 text-left text-sm ${
                active ? "bg-slate-800 text-white" : "bg-white text-slate-800"
              }`}
              key={item.view}
              onClick={() => dispatch({ type: "SET_VIEW", view: item.view })}
              type="button"
            >
              {collapsed ? item.label.charAt(0) : item.label}
            </button>
          );
        })}
      </nav>
    </aside>
  );
}
