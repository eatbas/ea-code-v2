import type { ReactNode } from "react";
import { useState } from "react";
import { useAppContext } from "../../contexts/AppContext";
import { useTemplateContext } from "../../contexts/TemplateContext";

export function PipelineGalleryView(): ReactNode {
  const { dispatch: appDispatch } = useAppContext();
  const {
    builtinTemplates,
    userTemplates,
    activeTemplate,
    dispatch,
    cloneTemplate,
    deleteTemplate,
  } = useTemplateContext();
  const [cloningTemplateId, setCloningTemplateId] = useState<string | null>(null);

  const clone = async (templateId: string, sourceName: string) => {
    setCloningTemplateId(templateId);
    await cloneTemplate(templateId, { newName: `${sourceName} Copy` });
    setCloningTemplateId(null);
  };

  return (
    <div className="flex h-full flex-col gap-6 bg-white p-6">
      <div className="flex items-center justify-between">
        <h1 className="text-xl font-semibold text-slate-900">Pipeline Gallery</h1>
        <button
          className="rounded border border-slate-300 px-3 py-2 text-sm"
          onClick={() => appDispatch({ type: "SET_VIEW", view: "home" })}
          type="button"
        >
          Back to Home
        </button>
      </div>

      <section>
        <h2 className="mb-2 text-sm font-semibold text-slate-800">Built-in Templates</h2>
        <div className="grid gap-3 md:grid-cols-2">
          {builtinTemplates.map((template) => {
            const selected = activeTemplate?.id === template.id;
            return (
              <article className="rounded border border-slate-200 p-4" key={template.id}>
                <h3 className="text-sm font-semibold text-slate-900">{template.name}</h3>
                <p className="mt-1 text-xs text-slate-600">{template.description}</p>
                <p className="mt-2 text-xs text-slate-500">
                  {template.nodes.length} stages · {template.maxIterations} max iterations
                </p>
                <div className="mt-3 flex gap-2">
                  <button
                    className="rounded border border-slate-300 px-2 py-1 text-xs"
                    onClick={() => {
                      dispatch({ type: "SET_ACTIVE", templateId: template.id });
                      appDispatch({ type: "SET_VIEW", view: "home" });
                    }}
                    type="button"
                  >
                    {selected ? "Selected" : "Use"}
                  </button>
                  <button
                    className="rounded border border-slate-300 px-2 py-1 text-xs"
                    disabled={cloningTemplateId === template.id}
                    onClick={() => {
                      void clone(template.id, template.name);
                    }}
                    type="button"
                  >
                    Clone
                  </button>
                </div>
              </article>
            );
          })}
        </div>
      </section>

      <section>
        <h2 className="mb-2 text-sm font-semibold text-slate-800">My Pipelines</h2>
        <div className="grid gap-3 md:grid-cols-2">
          {userTemplates.map((template) => (
            <article className="rounded border border-slate-200 p-4" key={template.id}>
              <h3 className="text-sm font-semibold text-slate-900">{template.name}</h3>
              <p className="mt-1 text-xs text-slate-600">{template.description}</p>
              <div className="mt-3 flex gap-2">
                <button
                  className="rounded border border-slate-300 px-2 py-1 text-xs"
                  onClick={() => {
                    dispatch({ type: "SET_ACTIVE", templateId: template.id });
                    appDispatch({ type: "SET_VIEW", view: "pipeline-builder" });
                  }}
                  type="button"
                >
                  Edit
                </button>
                <button
                  className="rounded border border-slate-300 px-2 py-1 text-xs"
                  onClick={() => {
                    void clone(template.id, template.name);
                  }}
                  type="button"
                >
                  Duplicate
                </button>
                <button
                  className="rounded border border-red-300 px-2 py-1 text-xs text-red-700"
                  onClick={() => {
                    void deleteTemplate(template.id);
                  }}
                  type="button"
                >
                  Delete
                </button>
              </div>
            </article>
          ))}
        </div>
      </section>

      <button
        className="w-fit rounded bg-slate-900 px-4 py-2 text-sm text-white"
        onClick={() => appDispatch({ type: "SET_VIEW", view: "pipeline-builder" })}
        type="button"
      >
        + New Pipeline
      </button>
    </div>
  );
}
