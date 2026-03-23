import { StageEdgeCondition, type StageEdgeDefinition, type StageNodeDefinition } from "../../types";

const STAGE_ID_PREFIX = "stage";

function nextNumericId(ids: string[], prefix: string): string {
  const prefixWithDash = `${prefix}-`;
  const next = ids.reduce((currentMax, id) => {
    if (!id.startsWith(prefixWithDash)) {
      return currentMax;
    }
    const parsed = Number.parseInt(id.slice(prefixWithDash.length), 10);
    if (Number.isNaN(parsed)) {
      return currentMax;
    }
    return Math.max(currentMax, parsed);
  }, 0) + 1;
  return `${prefix}-${next}`;
}

export function nextStageId(nodes: StageNodeDefinition[]): string {
  return nextNumericId(nodes.map((node) => node.id), STAGE_ID_PREFIX);
}

export function makeLinearEdges(nodes: StageNodeDefinition[]): StageEdgeDefinition[] {
  const enabled = nodes.filter((node) => node.enabled);
  if (enabled.length < 2) return [];

  const edges: StageEdgeDefinition[] = [];
  for (let index = 0; index < enabled.length - 1; index += 1) {
    const sourceNode = enabled[index];
    const targetNode = enabled[index + 1];
    if (!sourceNode || !targetNode) continue;

    edges.push({
      id: `${sourceNode.id}-to-${targetNode.id}`,
      sourceNodeId: sourceNode.id,
      targetNodeId: targetNode.id,
      condition: StageEdgeCondition.Always,
      inputKey: null,
      loopControl: false,
    });
  }

  return edges;
}

export function reorderNodesById(
  nodes: StageNodeDefinition[],
  sourceNodeId: string,
  targetNodeId: string,
): StageNodeDefinition[] {
  const sourceIndex = nodes.findIndex((node) => node.id === sourceNodeId);
  const targetIndex = nodes.findIndex((node) => node.id === targetNodeId);
  if (sourceIndex < 0 || targetIndex < 0 || sourceIndex === targetIndex) {
    return nodes;
  }

  const copy = [...nodes];
  const [moved] = copy.splice(sourceIndex, 1);
  if (!moved) return nodes;
  copy.splice(targetIndex, 0, moved);
  return copy.map((node, index) => ({
    ...node,
    uiPosition: {
      x: index * 320,
      y: 0,
    },
  }));
}

function groupLetterFromIndex(index: number): string {
  const code = "A".charCodeAt(0) + Math.max(0, index);
  return String.fromCharCode(code);
}

export function computeResumeFlags(nodes: StageNodeDefinition[]): boolean[] {
  return nodes.map((node, index) => {
    if (index === 0) return false;
    const previous = nodes[index - 1];
    if (!previous) return false;
    return previous.sessionGroup === node.sessionGroup;
  });
}

export function applyResumeFlagsToNodes(
  nodes: StageNodeDefinition[],
  resumeFlags: boolean[],
): StageNodeDefinition[] {
  if (nodes.length === 0) return [];

  let currentGroupIndex = 0;
  return nodes.map((node, index) => {
    if (index === 0) {
      return {
        ...node,
        sessionGroup: groupLetterFromIndex(currentGroupIndex),
      };
    }

    const resumeFromPrevious = resumeFlags[index] ?? true;
    if (!resumeFromPrevious) {
      currentGroupIndex += 1;
    }
    return {
      ...node,
      sessionGroup: groupLetterFromIndex(currentGroupIndex),
    };
  });
}

export function normaliseNodePositions(nodes: StageNodeDefinition[]): StageNodeDefinition[] {
  return nodes.map((node, index) => ({
    ...node,
    uiPosition: {
      x: index * 320,
      y: 0,
    },
  }));
}
