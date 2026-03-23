import type { ProviderInfo, StageNodeDefinition } from "../types";

/**
 * Returns the list of model options for a given stage node, driven by
 * the available provider data. Falls back to the node's current model
 * if no matching provider is found.
 */
export function inferModels(
  node: Pick<StageNodeDefinition, "provider" | "model">,
  providers: ProviderInfo[],
): string[] {
  const match = providers.find((provider) => provider.name === node.provider);
  if (!match || match.models.length === 0) {
    return [node.model];
  }
  if (match.models.includes(node.model)) {
    return match.models;
  }
  return [node.model, ...match.models];
}
