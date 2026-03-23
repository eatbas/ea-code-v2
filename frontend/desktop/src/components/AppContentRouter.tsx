import type { ReactNode } from "react";
import { useAppContext } from "../contexts/AppContext";
import { IdleView } from "./IdleView";
import { ChatView } from "./ChatView";
import { PipelineBuilderView } from "./PipelineBuilderView";
import { PipelineGalleryView } from "./PipelineGalleryView";
import { HiveApiStatusView } from "./HiveApiStatusView";
import { SkillsView } from "./SkillsView";
import { McpView } from "./McpView";
import { SessionsView } from "./SessionsView";
import { AppSettingsView } from "./AppSettingsView";

export function AppContentRouter(): ReactNode {
  const { activeView } = useAppContext();

  switch (activeView) {
    case "home":
      return <IdleView />;
    case "chat":
      return <ChatView />;
    case "sessions":
      return <SessionsView />;
    case "pipeline-builder":
      return <PipelineBuilderView />;
    case "pipeline-gallery":
      return <PipelineGalleryView />;
    case "hive-api-status":
      return <HiveApiStatusView />;
    case "skills":
      return <SkillsView />;
    case "mcp":
      return <McpView />;
    case "settings":
      return <AppSettingsView />;
    case "agents":
      return <PipelineBuilderView />;
    default:
      return <IdleView />;
  }
}
