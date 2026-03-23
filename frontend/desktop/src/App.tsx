import type { ReactNode } from "react";
import { useState } from "react";
import { AppProvider } from "./contexts/AppContext";
import { PipelineProvider } from "./contexts/PipelineContext";
import { TemplateProvider } from "./contexts/TemplateContext";
import { AppContentRouter } from "./components/AppContentRouter";
import { Sidebar } from "./components/Sidebar";

function AppShell(): ReactNode {
  const [sidebarCollapsed, setSidebarCollapsed] = useState(false);

  return (
    <div className="flex h-full min-h-[720px] bg-slate-100">
      <Sidebar
        collapsed={sidebarCollapsed}
        onToggle={() => setSidebarCollapsed((previous) => !previous)}
      />
      <main className="min-h-0 flex-1 overflow-auto">
        <AppContentRouter />
      </main>
    </div>
  );
}

export default function App(): ReactNode {
  return (
    <AppProvider>
      <TemplateProvider>
        <PipelineProvider>
          <AppShell />
        </PipelineProvider>
      </TemplateProvider>
    </AppProvider>
  );
}
