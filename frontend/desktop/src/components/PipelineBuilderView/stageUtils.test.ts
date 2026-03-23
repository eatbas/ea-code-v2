import { describe, expect, it } from "vitest";
import type { StageNodeDefinition } from "../../types";
import {
  applyResumeFlagsToNodes,
  computeResumeFlags,
  makeLinearEdges,
  nextStageId,
  normaliseNodePositions,
  reorderNodesById,
} from "./stageUtils";

function makeNode(
  id: string,
  overrides: Partial<StageNodeDefinition> = {},
): StageNodeDefinition {
  const { uiPosition, ...rest } = overrides;
  return {
    id,
    label: id,
    stageType: "analyse",
    handler: "analyse",
    config: null,
    provider: "claude",
    model: "sonnet",
    sessionGroup: "A",
    promptTemplate: "{{task}}",
    enabled: true,
    executionIntent: "text",
    uiPosition: uiPosition ?? { x: 0, y: 0 },
    ...rest,
  };
}

describe("stageUtils", () => {
  it("generates next stage id", () => {
    const nodes = [makeNode("stage-2"), makeNode("stage-10"), makeNode("custom-id")];
    expect(nextStageId(nodes)).toBe("stage-11");
  });

  it("creates linear edges only for enabled nodes", () => {
    const nodes = [
      makeNode("n1"),
      makeNode("n2", { enabled: false }),
      makeNode("n3"),
    ];
    expect(makeLinearEdges(nodes)).toEqual([
      {
        id: "n1-to-n3",
        sourceNodeId: "n1",
        targetNodeId: "n3",
        condition: "always",
        inputKey: null,
        loopControl: false,
      },
    ]);
  });

  it("reorders nodes and normalises positions", () => {
    const nodes = [makeNode("a"), makeNode("b"), makeNode("c")];
    const reordered = reorderNodesById(nodes, "c", "a");
    expect(reordered.map((node) => node.id)).toEqual(["c", "a", "b"]);
    expect(reordered.map((node) => node.uiPosition.x)).toEqual([0, 320, 640]);
  });

  it("derives and reapplies simple resume flags", () => {
    const nodes = [
      makeNode("a", { sessionGroup: "A" }),
      makeNode("b", { sessionGroup: "A" }),
      makeNode("c", { sessionGroup: "B" }),
      makeNode("d", { sessionGroup: "B" }),
    ];
    const flags = computeResumeFlags(nodes);
    expect(flags).toEqual([false, true, false, true]);

    const updated = applyResumeFlagsToNodes(nodes, flags);
    expect(updated.map((node) => node.sessionGroup)).toEqual(["A", "A", "B", "B"]);
  });

  it("normalises node positions to a horizontal line", () => {
    const nodes = [makeNode("a"), makeNode("b"), makeNode("c")];
    expect(normaliseNodePositions(nodes).map((node) => node.uiPosition.x)).toEqual([
      0, 320, 640,
    ]);
  });
});
