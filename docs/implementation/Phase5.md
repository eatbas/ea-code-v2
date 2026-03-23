# Phase 5 - Frontend Builder and Execution UX

## Objective

Deliver the v2 user experience: pipeline template builder, dynamic run visualisation, and state management overhaul, while retaining the familiar app shell.

## Scope

- Replace 34-prop drilling with React Context + useReducer.
- Add pipeline template gallery and editor views.
- Add stage-level configuration UI with drag-and-drop.
- Update execution views to render dynamic stages and session groups.
- Add hive-api status view.
- Add pipeline selector to home view.

## Implementation status (updated 23 March 2026, pass 3)

### Completed

- **Item 1 (State management architecture):** `AppContext`, `TemplateContext`, and `PipelineContext` are implemented and provider-wired in `App.tsx`.
- **Item 2 (ActiveView + router):** router uses context and all Phase 5 views are routed without prop drilling.
- **Item 3 (New hooks):** `usePipelineExecution` and upgraded `useHiveApi` are implemented and integrated into runtime views.
- **Item 4 (Pipeline selector on IdleView):** selector, template description, quick-access template shortcuts, edit/browse actions, and default selection via `settings.defaultPipelineId` are implemented.
- **Item 5 (Pipeline Gallery):** built-in and user template sections with use/clone/edit/delete/duplicate actions are implemented.
- **Item 6 (Pipeline Builder view):** two-panel template + stage editor implemented, including:
  - stage reorder via drag-and-drop and move controls,
  - session group indicators and visual session breaks,
  - simple/advanced session-group modes,
  - provider/model selectors driven by `useHiveApi().providers`,
  - enabled toggle, delete action, add-stage action,
  - save and delete pipeline actions.
- **Item 7 (Prompt editor modal):** `PromptEditorModal.tsx` implemented with variable chips, enhance action, before/after diff, and accept/reject flow.
- **Item 8 (Dynamic ChatView):** dynamic stage cards from template, session-group tags, streamed logs, code diff viewer, and iteration/progress indicators implemented.
- **Item 9 (Dynamic RunTimeline):** dynamic stage names, session-group colouring, adaptive compact/expanded layouts, and iteration-aware rendering implemented.
- **Item 10 (hive-api Status view):** status, providers, drone inventory, start/stop/restart controls, CLI versions, and error log implemented.
- **Workflow alignment update:** provider/model selection is now driven by hive-api surfaces (`HiveApiStatusView` + builder provider/model selectors). Legacy "pick agents in Settings/CLI Setup" workflow is not part of v2 Phase 5.
- **Item 11 (Sidebar updates):** pipelines and hive-api entries added; sessions/skills/mcp/settings navigation entries present.

### Deferred / N/A in this repo snapshot

- **Item 12 (remove deprecated components/hooks):**
  - `AgentsView/`, `CliSetupView/`, `useCliHealth`, and `useCliVersions` were already absent in this trimmed v2 frontend snapshot.
  - Equivalent migration is complete by routing to `PipelineBuilderView` and `HiveApiStatusView`.
- **Out of Phase 5 scope for this pass:** full "feature parity" rewiring of legacy Sessions/Settings screens beyond routed presence remains deferred.

### Validation status

- Completed:
  - `npm run typecheck` (`tsc --noEmit`) passes.
  - `npm test` (`vitest`) passes: `3` files, `19` tests.
  - `cargo check --all-targets --all-features` passes.
  - `cargo test --all-targets --all-features` passes: `107` tests.
- Pending before formal close-out:
  - Execute full manual Phase 5 test matrix below (builder interaction walkthroughs, multi-template execution checks, session-group behaviour verification, and desktop layout regression pass).

### Exit criteria status

- **Mostly complete in code:** architecture migration and Phase 5 surfaces are implemented.
- **Remaining for final sign-off:** run and document manual usability/regression verification for all scenarios in the Testing section.

## Work items

### 1. State management architecture

Replace the 34-prop drilling through `AppContentRouter` with 3 React Contexts:

**File: `frontend/desktop/src/contexts/AppContext.tsx` (NEW)**

```typescript
interface AppContextType {
  activeView: ActiveView;
  workspace: WorkspaceInfo | null;
  settings: AppSettings;
  dispatch: React.Dispatch<AppAction>;
}

// Actions: SET_VIEW, SET_WORKSPACE, SET_SETTINGS
```

**File: `frontend/desktop/src/contexts/PipelineContext.tsx` (NEW)**

```typescript
interface PipelineContextType {
  run: PipelineRun | null;        // active run state
  stages: StageStatus[];          // dynamic stage list from template
  logs: LogEntry[];               // streaming log output
  sessionGroups: Record<string, string>; // group â†’ session ref
  dispatch: React.Dispatch<PipelineAction>;
}

// Actions: START_RUN, UPDATE_STAGE, APPEND_LOG, SET_SESSION_REF, COMPLETE_RUN, etc.
```

**File: `frontend/desktop/src/contexts/TemplateContext.tsx` (NEW)**

```typescript
interface TemplateContextType {
  templates: PipelineTemplate[];
  activeTemplate: PipelineTemplate | null;
  dispatch: React.Dispatch<TemplateAction>;
}

// Actions: SET_TEMPLATES, SET_ACTIVE, UPDATE_TEMPLATE, etc.
```

**Integration:** Wrap the app in providers in `App.tsx`:

```typescript
<AppContext.Provider>
  <TemplateContext.Provider>
    <PipelineContext.Provider>
      <AppContentRouter />
    </PipelineContext.Provider>
  </TemplateContext.Provider>
</AppContext.Provider>
```

Child components consume context directly via `useContext()` â€” no prop drilling.

### 2. ActiveView enum updates

**File: `frontend/desktop/src/types/navigation.ts` (MODIFY)**

```typescript
export type ActiveView =
  | "home"
  | "chat"                // live execution (was part of home in v1)
  | "pipeline-builder"    // NEW: drag-and-drop pipeline editor
  | "pipeline-gallery"    // NEW: browse/clone templates
  | "hive-api-status"     // NEW: provider health dashboard (replaces "cli-setup")
  | "skills"
  | "mcp"
  | "agents";             // DEPRECATED: kept for transition, eventually removed
```

**File: `frontend/desktop/src/components/AppContentRouter.tsx` (REWRITE)**

Simplified â€” no more prop drilling. Each view reads from context:

```typescript
function AppContentRouter() {
  const { activeView } = useContext(AppContext);

  switch (activeView) {
    case "home": return <IdleView />;
    case "chat": return <ChatView />;
    case "pipeline-builder": return <PipelineBuilderView />;
    case "pipeline-gallery": return <PipelineGalleryView />;
    case "hive-api-status": return <HiveApiStatusView />;
    case "skills": return <SkillsView />;
    case "mcp": return <McpView />;
  }
}
```

### 3. New hooks

**File: `frontend/desktop/src/hooks/usePipelineTemplates.ts`** â€” Already defined in Phase 3. Consumed by gallery and builder views.

**File: `frontend/desktop/src/hooks/useHiveApi.ts` (NEW)**

```typescript
useHiveApi()
  â†’ {
    status: "starting" | "ready" | "disconnected" | "error",
    providers: ProviderInfo[],      // available providers with models
    drones: DroneInfo[],            // active drone inventory
    startApi: () => Promise<void>,
    stopApi: () => Promise<void>,
    checkHealth: () => Promise<HealthStatus>,
  }
```

**File: `frontend/desktop/src/hooks/usePipelineExecution.ts` (NEW)**

Replaces `usePipeline` + `usePipelineEvents` with a single hook for dynamic pipeline execution:

```typescript
usePipelineExecution()
  â†’ {
    run: PipelineRun | null,
    isRunning: boolean,
    currentStage: StageStatus | null,
    stages: StageStatus[],           // dynamic from template
    logs: LogEntry[],
    startPipeline: (templateId: string, prompt: string) => Promise<void>,
    pausePipeline: () => Promise<void>,
    resumePipeline: () => Promise<void>,
    cancelPipeline: () => Promise<void>,
    answerQuestion: (answer: string) => Promise<void>,
  }
```

### 4. Pipeline selector on IdleView

**File: `frontend/desktop/src/components/IdleView.tsx` (MODIFY)**

Add a pipeline template selector dropdown above or beside the prompt input bar:

- Shows template name and description.
- Default: the template set in `settings.default_pipeline_id`.
- Quick access to "Full Review Loop", "Quick Fix", "Research Only".
- "Edit Pipeline" link opens the pipeline builder for the selected template.
- "Browse All" link navigates to pipeline gallery.

The existing "Direct Task" and "No Plan" checkboxes remain.

### 5. Pipeline Gallery view

**File: `frontend/desktop/src/components/PipelineGalleryView/index.tsx` (NEW)**

Two-section layout:
- **Built-in Templates:** Card grid showing name, description, stage count, iteration limit. "Use" button selects it. "Clone" button creates a user copy.
- **My Pipelines:** User templates with "Edit", "Delete", "Duplicate" actions.
- **"+ New Pipeline"** button at the bottom.

### 6. Pipeline Builder view

**File: `frontend/desktop/src/components/PipelineBuilderView/index.tsx` (NEW)**

Two-column layout (see architecture doc section 10.2 for ASCII mockup):

**Left panel:** Template list (built-in + user). "Use as Template" and "+ New Pipeline" buttons.

**Right panel:** Active template editor:
- Name and description fields.
- Max iterations and "Stop on first pass" toggle.
- Drag-and-drop stage list with:
  - Drag handle (`::`) for reordering.
  - Session group indicator (colour-coded letter: A, B, C...).
  - Provider/model selectors (populated from `useHiveApi().providers`).
  - Enabled toggle.
  - Delete button.
  - "Edit Prompt" button â†’ opens prompt editor modal.
- Visual session break line between different session groups.
- "+ Add Stage" button.
- "Delete Pipeline" and "Save Changes" action buttons.

**Session group UX:**
- **Simple mode (default):** Each stage has a "Resume from previous?" toggle. If yes, joins the previous stage's group. Groups auto-calculated.
- **Advanced mode:** User picks explicit group labels (A, B, C...) for full control.

### 7. Prompt editor modal

**File: `frontend/desktop/src/components/PipelineBuilderView/PromptEditorModal.tsx` (NEW)**

- Stage label, type, provider/model, session group fields (read from stage, editable).
- Prompt template text area (full-height, monospace).
- Variable insertion bar: clickable chips for `{{task}}`, `{{code_context}}`, `{{previous_output}}`, `{{file_list}}`, etc. Clicking inserts at cursor.
- "Enhance" button: calls `enhance_prompt` command, shows before/after diff, accept/reject.
- "Cancel" and "Save Stage" buttons.

### 8. Dynamic ChatView

**File: `frontend/desktop/src/components/ChatView.tsx` (REWRITE)**

Replace hardcoded 13-stage rendering with dynamic stage list from template:

- Stage cards rendered from `usePipelineExecution().stages` (dynamic labels and count).
- Session group colour tags on each stage card.
- Log output streamed per-stage.
- Diff viewer shown after stages with `execution_intent: "code"`.
- Iteration counter and progress indicator.

### 9. Dynamic RunTimeline

**File: `frontend/desktop/src/components/RunTimeline.tsx` (REWRITE)**

Replace fixed iteration/stage timeline with dynamic renderer:

- Stage names from template (not hardcoded enum).
- Session group colour coding.
- Iteration loop visualisation adapts to template's `max_iterations`.
- Compact view for 2-stage templates, expanded for 5+ stage templates.

### 10. hive-api Status view

**File: `frontend/desktop/src/components/HiveApiStatusView/index.tsx` (NEW)** â€” Replaces `CliSetupView`.

- Connection status (starting / ready / disconnected / error).
- Provider list with availability indicators (from `useHiveApi().providers`).
- Drone inventory (active drones per provider/model).
- "Start" / "Stop" / "Restart" buttons.
- CLI version info per provider (from hive-api version endpoints).
- Error log display if hive-api fails to start.

### 11. Sidebar updates

**File: `frontend/desktop/src/components/Sidebar.tsx` (MODIFY)**

- Add "Pipelines" navigation item (links to gallery).
- Replace "CLI Setup" with "hive-api" status indicator.
- Keep existing: Sessions, Skills, MCP, Settings.

### 12. Remove deprecated components

Mark for removal (actual deletion in Phase 6):
- `AgentsView/` â€” Replaced by Pipeline Builder.
- `CliSetupView/` â€” Replaced by HiveApiStatusView.
- `useCliHealth` hook.
- `useCliVersions` hook.

## File paths summary

| Action | Path |
|--------|------|
| NEW | `frontend/desktop/src/contexts/AppContext.tsx` |
| NEW | `frontend/desktop/src/contexts/PipelineContext.tsx` |
| NEW | `frontend/desktop/src/contexts/TemplateContext.tsx` |
| NEW | `frontend/desktop/src/hooks/useHiveApi.ts` |
| NEW | `frontend/desktop/src/hooks/usePipelineExecution.ts` |
| NEW | `frontend/desktop/src/components/PipelineGalleryView/index.tsx` |
| NEW | `frontend/desktop/src/components/PipelineBuilderView/index.tsx` |
| NEW | `frontend/desktop/src/components/PipelineBuilderView/PromptEditorModal.tsx` |
| NEW | `frontend/desktop/src/components/PipelineBuilderView/StageCard.tsx` |
| NEW | `frontend/desktop/src/components/PipelineBuilderView/SessionGroupIndicator.tsx` |
| NEW | `frontend/desktop/src/components/HiveApiStatusView/index.tsx` |
| REWRITE | `frontend/desktop/src/components/AppContentRouter.tsx` |
| REWRITE | `frontend/desktop/src/components/ChatView.tsx` |
| REWRITE | `frontend/desktop/src/components/RunTimeline.tsx` |
| MODIFY | `frontend/desktop/src/components/IdleView.tsx` (add pipeline selector) |
| MODIFY | `frontend/desktop/src/components/Sidebar.tsx` (add pipelines nav, replace CLI) |
| MODIFY | `frontend/desktop/src/App.tsx` (wrap in context providers) |
| MODIFY | `frontend/desktop/src/types/navigation.ts` (new ActiveView variants) |
| DEPRECATE | `frontend/desktop/src/components/AgentsView/` |
| DEPRECATE | `frontend/desktop/src/components/CliSetupView/` |
| DEPRECATE | `frontend/desktop/src/hooks/useCliHealth.ts` |
| DEPRECATE | `frontend/desktop/src/hooks/useCliVersions.ts` |

## Testing

- **Pipeline builder interaction:** Create a template with 4 stages. Drag-reorder stages. Toggle enable/disable. Verify positions update correctly.
- **Template selection:** Select different templates on IdleView. Start a run. Verify correct template is used.
- **Dynamic stage rendering:** Run templates with 2, 4, and 5 stages. Verify ChatView renders the correct number of stage cards with correct labels.
- **Session group visualisation:** Run Full Review Loop. Verify group A (Analyse + Review) and group B (Implement + Test) are colour-coded differently.
- **Prompt editor:** Open prompt editor, insert variables via chips, click Enhance, verify diff display.
- **hive-api status:** Disconnect hive-api. Verify status view shows "disconnected". Reconnect. Verify "ready".
- **State management:** Navigate between views (home â†’ builder â†’ chat â†’ history). Verify no stale state or prop drilling errors.
- **Layout regression:** Test on 1280Ã—800 (minimum desktop) and 1920Ã—1080. Verify no overflow or broken layouts.

## Deliverables

- End users can configure and run custom pipelines from UI only.
- Run view clearly shows current stage, iteration, and group continuity.
- No dependency on legacy fixed agent assignment grid.
- Prop drilling eliminated via React Context.
- hive-api status view functional.

## Dependencies

- Phase 4 dynamic orchestrator and stage events.
- Phase 3 template CRUD hooks.
- Phase 2 hive-api health/status data.

## Risks and mitigations

- Risk: Builder complexity hurts usability.
  Mitigation: Provide built-in starter templates, in-form validation, and minimal required fields. Simple mode for session groups.
- Risk: Drag-and-drop library adds significant bundle size.
  Mitigation: Evaluate lightweight options (dnd-kit, native HTML5 drag). If too heavy, use simple up/down arrow buttons instead.
- Risk: State management migration breaks existing flows.
  Mitigation: Migrate incrementally â€” start with AppContext, verify, then add PipelineContext and TemplateContext.

## Exit criteria

- Usability pass completed on template create, edit, and run flows.
- Dynamic execution view handles both short (2-stage) and long (5+ stage, multi-iteration) runs.
- No major layout regressions on desktop standard resolutions.
- All existing features (workspace selection, session history, skills, MCP) still work.
- `npx tsc --noEmit` passes with zero errors.

## Estimated duration

1.5 to 2 weeks
