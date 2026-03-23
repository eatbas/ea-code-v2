import type { ReactNode } from "react";
import type { PipelineTemplate } from "../../types";

interface TemplateListPanelProps {
  builtinTemplates: PipelineTemplate[];
  userTemplates: PipelineTemplate[];
  activeTemplateId: string | null;
  templateCountLabel: string;
  onSelectTemplate: (templateId: string) => void;
  onNewTemplate: () => void;
  onUseAsTemplate: () => void;
}

export function TemplateListPanel({
  builtinTemplates,
  userTemplates,
  activeTemplateId,
  templateCountLabel,
  onSelectTemplate,
  onNewTemplate,
  onUseAsTemplate,
}: TemplateListPanelProps): ReactNode {
  return (
    <aside className="flex min-h-0 flex-col rounded border border-slate-200 bg-slate-50 p-3">
      <div className="mb-2 flex items-center justify-between">
        <strong className="text-sm text-slate-900">
          Templates ({templateCountLabel})
        </strong>
        <button
          className="rounded border border-slate-300 px-2 py-1 text-xs"
          onClick={onNewTemplate}
          type="button"
        >
          + New
        </button>
      </div>

      <div className="min-h-0 flex-1 space-y-3 overflow-auto">
        <div>
          <p className="mb-1 text-xs font-semibold uppercase text-slate-500">Built-in</p>
          <div className="space-y-1">
            {builtinTemplates.map((template) => (
              <button
                className={`w-full rounded px-2 py-1 text-left text-sm ${
                  activeTemplateId === template.id
                    ? "bg-slate-900 text-white"
                    : "bg-white text-slate-800"
                }`}
                key={template.id}
                onClick={() => onSelectTemplate(template.id)}
                type="button"
              >
                {template.name}
              </button>
            ))}
          </div>
        </div>

        <div>
          <p className="mb-1 text-xs font-semibold uppercase text-slate-500">My Pipelines</p>
          <div className="space-y-1">
            {userTemplates.map((template) => (
              <button
                className={`w-full rounded px-2 py-1 text-left text-sm ${
                  activeTemplateId === template.id
                    ? "bg-slate-900 text-white"
                    : "bg-white text-slate-800"
                }`}
                key={template.id}
                onClick={() => onSelectTemplate(template.id)}
                type="button"
              >
                {template.name}
              </button>
            ))}
          </div>
        </div>
      </div>

      <div className="mt-3 flex gap-2">
        <button
          className="rounded border border-slate-300 px-2 py-1 text-xs"
          onClick={onUseAsTemplate}
          type="button"
        >
          Use as Template
        </button>
      </div>
    </aside>
  );
}
