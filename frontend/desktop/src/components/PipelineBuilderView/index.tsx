import type { ReactNode } from "react";
import { useEffect, useMemo, useState } from "react";
import { useAppContext } from "../../contexts/AppContext";
import { useTemplateContext } from "../../contexts/TemplateContext";
import { useHiveApi } from "../../hooks/useHiveApi";
import type { StageExecutionIntent, StageNodeDefinition } from "../../types";
import { PromptEditorModal } from "./PromptEditorModal";
import { StageCard } from "./StageCard";
import { TemplateListPanel } from "./TemplateListPanel";
import { TemplateSettingsForm } from "./TemplateSettingsForm";
import {
  applyResumeFlagsToNodes,
  computeResumeFlags,
  makeLinearEdges,
  nextStageId,
  normaliseNodePositions,
  reorderNodesById,
} from "./stageUtils";

type SessionMode = "simple" | "advanced";

function defaultStageType(index: number): string {
  const rotation = ["analyse", "review", "implement", "test"];
  return rotation[index % rotation.length] ?? "analyse";
}

function inferIntent(stageType: string): StageExecutionIntent {
  if (stageType === "implement" || stageType === "test") {
    return "code";
  }
  return "text";
}

function buildDefaultNode(
  existingNodes: StageNodeDefinition[],
  provider: string,
  model: string,
): StageNodeDefinition {
  const stageType = defaultStageType(existingNodes.length);
  return {
    id: nextStageId(existingNodes),
    label: stageType.charAt(0).toUpperCase() + stageType.slice(1),
    stageType,
    handler: stageType,
    config: null,
    provider,
    model,
    sessionGroup: "A",
    promptTemplate: "Task: {{task}}\n\nContext:\n{{code_context}}",
    enabled: true,
    executionIntent: inferIntent(stageType),
    uiPosition: {
      x: existingNodes.length * 320,
      y: 0,
    },
  };
}

export function PipelineBuilderView(): ReactNode {
  const { dispatch: appDispatch } = useAppContext();
  const {
    templates,
    builtinTemplates,
    userTemplates,
    activeTemplate,
    dispatch: templateDispatch,
    createTemplate,
    updateTemplate,
    deleteTemplate,
    enhancePrompt,
    loading,
  } = useTemplateContext();
  const { providers, checkHealth, refreshProviders } = useHiveApi();

  const [name, setName] = useState("New Pipeline");
  const [description, setDescription] = useState("Custom pipeline");
  const [maxIterations, setMaxIterations] = useState(1);
  const [stopOnFirstPass, setStopOnFirstPass] = useState(true);
  const [sessionMode, setSessionMode] = useState<SessionMode>("simple");
  const [draftNodes, setDraftNodes] = useState<StageNodeDefinition[]>([]);
  const [resumeFromPrevious, setResumeFromPrevious] = useState<boolean[]>([]);
  const [draggedNodeId, setDraggedNodeId] = useState<string | null>(null);
  const [editingStageId, setEditingStageId] = useState<string | null>(null);
  const [isSaving, setIsSaving] = useState(false);

  useEffect(() => {
    void checkHealth();
    void refreshProviders();
  }, [checkHealth, refreshProviders]);

  useEffect(() => {
    if (!activeTemplate) return;
    const nextNodes = normaliseNodePositions(activeTemplate.nodes);
    setDraftNodes(nextNodes);
    setName(activeTemplate.name);
    setDescription(activeTemplate.description);
    setMaxIterations(activeTemplate.maxIterations);
    setStopOnFirstPass(activeTemplate.stopOnFirstPass);
    setSessionMode("simple");
    setResumeFromPrevious(computeResumeFlags(nextNodes));
  }, [activeTemplate]);

  const templateCountLabel = useMemo(() => {
    if (templates.length === 1) return "1 template";
    return `${templates.length} templates`;
  }, [templates.length]);

  const providerFallback = useMemo(() => {
    const firstProvider = providers[0];
    if (firstProvider) {
      return {
        provider: firstProvider.name,
        model: firstProvider.models[0] ?? "sonnet",
      };
    }
    return { provider: "claude", model: "sonnet" };
  }, [providers]);

  const stagesForRender = useMemo(() => {
    if (sessionMode === "advanced") return draftNodes;
    return applyResumeFlagsToNodes(draftNodes, resumeFromPrevious);
  }, [draftNodes, resumeFromPrevious, sessionMode]);

  const editingStage = useMemo(
    () => stagesForRender.find((stage) => stage.id === editingStageId) ?? null,
    [editingStageId, stagesForRender],
  );

  const setNodesAndSyncResume = (nextNodes: StageNodeDefinition[]) => {
    setDraftNodes(nextNodes);
    setResumeFromPrevious(computeResumeFlags(nextNodes));
  };

  const moveStage = (index: number, direction: "up" | "down") => {
    const targetIndex = direction === "up" ? index - 1 : index + 1;
    if (targetIndex < 0 || targetIndex >= stagesForRender.length) return;
    const sourceNode = stagesForRender[index];
    const targetNode = stagesForRender[targetIndex];
    if (!sourceNode || !targetNode) return;
    setNodesAndSyncResume(reorderNodesById(stagesForRender, sourceNode.id, targetNode.id));
  };

  const onDrop = (targetNodeId: string) => {
    if (!draggedNodeId || draggedNodeId === targetNodeId) return;
    setNodesAndSyncResume(reorderNodesById(stagesForRender, draggedNodeId, targetNodeId));
    setDraggedNodeId(null);
  };

  const addStage = () => {
    const base = stagesForRender;
    const next = [
      ...base,
      buildDefaultNode(base, providerFallback.provider, providerFallback.model),
    ];
    setDraftNodes(next);
    setResumeFromPrevious((previous) => [...previous, next.length > 1]);
  };

  const removeStage = (nodeId: string) => {
    setNodesAndSyncResume(stagesForRender.filter((node) => node.id !== nodeId));
    if (editingStageId === nodeId) setEditingStageId(null);
  };

  const toggleSessionMode = (nextMode: SessionMode) => {
    if (nextMode === sessionMode) return;
    if (nextMode === "advanced") {
      setDraftNodes(applyResumeFlagsToNodes(stagesForRender, resumeFromPrevious));
      setSessionMode(nextMode);
      return;
    }
    setSessionMode(nextMode);
    setResumeFromPrevious(computeResumeFlags(stagesForRender));
  };

  const save = async () => {
    const nodesForSave = normaliseNodePositions(stagesForRender);
    const payload = {
      name,
      description,
      maxIterations,
      stopOnFirstPass,
      nodes: nodesForSave,
      edges: makeLinearEdges(nodesForSave),
    };
    setIsSaving(true);
    if (activeTemplate && !activeTemplate.isBuiltin) {
      await updateTemplate(activeTemplate.id, payload);
      setIsSaving(false);
      return;
    }
    const created = await createTemplate(payload);
    if (created) {
      templateDispatch({ type: "SET_ACTIVE", templateId: created.id });
    }
    setIsSaving(false);
  };

  const useAsTemplate = async () => {
    if (!activeTemplate) return;
    const created = await createTemplate({
      name: `${activeTemplate.name} Copy`,
      description: activeTemplate.description,
      maxIterations: activeTemplate.maxIterations,
      stopOnFirstPass: activeTemplate.stopOnFirstPass,
      nodes: activeTemplate.nodes,
      edges: activeTemplate.edges,
    });
    if (created) {
      templateDispatch({ type: "SET_ACTIVE", templateId: created.id });
    }
  };

  const newTemplate = () => {
    templateDispatch({ type: "SET_ACTIVE", templateId: null });
    setName("New Pipeline");
    setDescription("Custom pipeline");
    setMaxIterations(1);
    setStopOnFirstPass(true);
    setSessionMode("simple");
    setDraftNodes([]);
    setResumeFromPrevious([]);
  };

  const deleteActiveTemplate = async () => {
    if (!activeTemplate || activeTemplate.isBuiltin) return;
    await deleteTemplate(activeTemplate.id);
    newTemplate();
  };

  return (
    <div className="flex h-full min-h-0 flex-col bg-white p-6">
      <div className="mb-4 flex items-center justify-between">
        <h1 className="text-xl font-semibold text-slate-900">Pipeline Builder</h1>
        <button
          className="rounded border border-slate-300 px-3 py-2 text-sm"
          onClick={() => appDispatch({ type: "SET_VIEW", view: "pipeline-gallery" })}
          type="button"
        >
          Open Gallery
        </button>
      </div>

      <div className="grid min-h-0 flex-1 gap-4 lg:grid-cols-[280px_1fr]">
        <TemplateListPanel
          activeTemplateId={activeTemplate?.id ?? null}
          builtinTemplates={builtinTemplates}
          onNewTemplate={newTemplate}
          onSelectTemplate={(id) =>
            templateDispatch({ type: "SET_ACTIVE", templateId: id })
          }
          onUseAsTemplate={() => {
            void useAsTemplate();
          }}
          templateCountLabel={templateCountLabel}
          userTemplates={userTemplates}
        />

        <section className="flex min-h-0 flex-col rounded border border-slate-200 p-3">
          <TemplateSettingsForm
            description={description}
            maxIterations={maxIterations}
            name={name}
            onDescriptionChange={setDescription}
            onMaxIterationsChange={setMaxIterations}
            onNameChange={setName}
            onSessionModeChange={toggleSessionMode}
            onStopOnFirstPassChange={setStopOnFirstPass}
            sessionMode={sessionMode}
            stopOnFirstPass={stopOnFirstPass}
          />

          <div className="mt-3 min-h-0 flex-1 space-y-2 overflow-auto">
            {stagesForRender.map((node, index) => {
              const previous = stagesForRender[index - 1];
              const sessionBreak =
                index > 0 && !!previous && previous.sessionGroup !== node.sessionGroup;
              return (
                <StageCard
                  advancedSessionMode={sessionMode === "advanced"}
                  index={index}
                  key={node.id}
                  node={node}
                  onDelete={removeStage}
                  onDragStart={setDraggedNodeId}
                  onDrop={onDrop}
                  onMove={moveStage}
                  onOpenPromptEditor={setEditingStageId}
                  onToggleResumeFromPrevious={(stageIndex, resume) => {
                    setResumeFromPrevious((previousFlags) => {
                      const nextFlags = [...previousFlags];
                      nextFlags[stageIndex] = resume;
                      setDraftNodes(applyResumeFlagsToNodes(stagesForRender, nextFlags));
                      return nextFlags;
                    });
                  }}
                  onUpdate={(updatedNode) => {
                    setDraftNodes((previousNodes) =>
                      previousNodes.map((nodeEntry) =>
                        nodeEntry.id === updatedNode.id ? updatedNode : nodeEntry,
                      ),
                    );
                  }}
                  providers={providers}
                  resumeFromPrevious={resumeFromPrevious[index] ?? false}
                  showSessionBreakBefore={sessionBreak}
                  total={stagesForRender.length}
                />
              );
            })}
          </div>

          <div className="mt-3 flex flex-wrap items-center gap-2">
            <button
              className="rounded border border-slate-300 px-3 py-2 text-sm"
              onClick={addStage}
              type="button"
            >
              + Add Stage
            </button>
            <button
              className="rounded bg-slate-900 px-3 py-2 text-sm text-white"
              disabled={isSaving || loading}
              onClick={() => {
                void save();
              }}
              type="button"
            >
              {isSaving ? "Saving..." : "Save Changes"}
            </button>
            <button
              className="rounded border border-red-300 px-3 py-2 text-sm text-red-700"
              disabled={!activeTemplate || activeTemplate.isBuiltin}
              onClick={() => {
                void deleteActiveTemplate();
              }}
              type="button"
            >
              Delete Pipeline
            </button>
          </div>
        </section>
      </div>

      {editingStage ? (
        <PromptEditorModal
          onCancel={() => setEditingStageId(null)}
          onEnhance={enhancePrompt}
          onSave={(updatedStage) => {
            setDraftNodes((previousNodes) =>
              previousNodes.map((node) =>
                node.id === updatedStage.id ? updatedStage : node,
              ),
            );
            setEditingStageId(null);
          }}
          providers={providers}
          stage={editingStage}
        />
      ) : null}
    </div>
  );
}
