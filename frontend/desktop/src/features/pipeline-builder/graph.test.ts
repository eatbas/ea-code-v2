import { describe, expect, it } from "vitest";
import { StageEdgeCondition, type StageEdgeDefinition, type StageNodeDefinition } from "../../types";
import {
  connectNodes,
  createNode,
  deleteNode,
  disconnectEdge,
  disconnectNodes,
  duplicateNode,
  moveNode,
  renameNode,
  validateGraph,
} from "./graph";
import { createPipelineBuilderGraphState } from "./model";

function makeNode(id: string, overrides: Partial<StageNodeDefinition> = {}): StageNodeDefinition {
  const { uiPosition, ...restOverrides } = overrides;

  return {
    id,
    label: `${id} label`,
    stageType: "analyse",
    handler: "chat",
    config: { purpose: "test" },
    provider: "claude",
    model: "sonnet",
    sessionGroup: "A",
    promptTemplate: "{{task}}",
    enabled: true,
    executionIntent: "text",
    ...restOverrides,
    uiPosition: uiPosition ? { x: uiPosition.x, y: uiPosition.y } : { x: 10, y: 20 },
  };
}

function makeEdge(id: string, overrides: Partial<StageEdgeDefinition> = {}): StageEdgeDefinition {
  return {
    id,
    sourceNodeId: "node-1",
    targetNodeId: "node-2",
    condition: StageEdgeCondition.Always,
    inputKey: null,
    loopControl: false,
    ...overrides,
  };
}

describe("pipeline builder graph operations", () => {
  it("creates nodes with explicit and generated ids", () => {
    const state = createPipelineBuilderGraphState([
      makeNode("node-2"),
      makeNode("node-9", { label: "Existing", uiPosition: { x: 40, y: 60 } }),
    ]);

    const next = createNode(state, {
      label: "Generated",
      stageType: "review",
      handler: "chat",
      provider: "anthropic",
      model: "opus",
      sessionGroup: "B",
      promptTemplate: "prompt",
      executionIntent: "code",
      uiPosition: { x: 120, y: 140 },
    });

    expect(next.nodes).toHaveLength(3);
    expect(next.nodes[2]).toEqual({
      id: "node-10",
      label: "Generated",
      stageType: "review",
      handler: "chat",
      config: null,
      provider: "anthropic",
      model: "opus",
      sessionGroup: "B",
      promptTemplate: "prompt",
      enabled: true,
      executionIntent: "code",
      uiPosition: { x: 120, y: 140 },
    });
    expect(state.nodes).toHaveLength(2);
    expect(() =>
      createNode(state, {
        id: "node-2",
        label: "Duplicate",
        stageType: "review",
        handler: "chat",
        provider: "anthropic",
        model: "opus",
        sessionGroup: "B",
        promptTemplate: "prompt",
        executionIntent: "text",
        uiPosition: { x: 0, y: 0 },
      }),
    ).toThrow("Node ID already exists: node-2");
  });

  it("deletes nodes and their incident edges", () => {
    const state = createPipelineBuilderGraphState(
      [makeNode("node-1"), makeNode("node-2"), makeNode("node-3")],
      [
        makeEdge("edge-1", { sourceNodeId: "node-1", targetNodeId: "node-2" }),
        makeEdge("edge-2", { sourceNodeId: "node-2", targetNodeId: "node-3" }),
      ],
    );

    const next = deleteNode(state, { nodeId: "node-2" });

    expect(next.nodes.map((node) => node.id)).toEqual(["node-1", "node-3"]);
    expect(next.edges).toEqual([]);
    expect(deleteNode(state, { nodeId: "missing" })).toBe(state);
  });

  it("moves a node without touching other graph data", () => {
    const state = createPipelineBuilderGraphState(
      [makeNode("node-1"), makeNode("node-2", { uiPosition: { x: 50, y: 70 } })],
      [makeEdge("edge-1", { sourceNodeId: "node-1", targetNodeId: "node-2" })],
    );

    const next = moveNode(state, { nodeId: "node-2", uiPosition: { x: 220, y: 180 } });

    expect(next.nodes).toEqual([
      makeNode("node-1"),
      makeNode("node-2", { uiPosition: { x: 220, y: 180 } }),
    ]);
    expect(next.edges).toEqual(state.edges);
    expect(() => moveNode(state, { nodeId: "missing", uiPosition: { x: 0, y: 0 } })).toThrow(
      "Node does not exist: missing",
    );
  });

  it("renames nodes with trimming and validation", () => {
    const state = createPipelineBuilderGraphState([makeNode("node-1", { label: "Old label" })]);

    const next = renameNode(state, { nodeId: "node-1", label: "  New label  " });

    expect(next.nodes[0]?.label).toBe("New label");
    expect(() => renameNode(state, { nodeId: "node-1", label: "   " })).toThrow(
      "Node label must not be empty",
    );
    expect(() => renameNode(state, { nodeId: "missing", label: "Next label" })).toThrow(
      "Node does not exist: missing",
    );
  });

  it("duplicates nodes with a stable offset and generated ids", () => {
    const state = createPipelineBuilderGraphState([
      makeNode("node-4", {
        label: "Analyse",
        config: { retryLimit: 3 },
        enabled: false,
        executionIntent: "code",
        uiPosition: { x: 90, y: 110 },
      }),
      makeNode("node-7", { label: "Peer", uiPosition: { x: 200, y: 240 } }),
    ]);

    const next = duplicateNode(state, { nodeId: "node-4" });
    const copy = next.nodes[2];

    expect(copy).toEqual({
      id: "node-8",
      label: "Analyse Copy",
      stageType: "analyse",
      handler: "chat",
      config: { retryLimit: 3 },
      provider: "claude",
      model: "sonnet",
      sessionGroup: "A",
      promptTemplate: "{{task}}",
      enabled: false,
      executionIntent: "code",
      uiPosition: { x: 120, y: 140 },
    });
    expect(() => duplicateNode(state, { nodeId: "missing" })).toThrow("Node does not exist: missing");
  });

  it("connects nodes with unique edge ids and rejects invalid requests", () => {
    const state = createPipelineBuilderGraphState(
      [makeNode("node-1"), makeNode("node-2", { uiPosition: { x: 70, y: 90 } })],
      [makeEdge("edge-7", { sourceNodeId: "node-1", targetNodeId: "node-2" })],
    );

    const generated = connectNodes(state, {
      sourceNodeId: "node-2",
      targetNodeId: "node-1",
      condition: StageEdgeCondition.OnSuccess,
      inputKey: "result",
    });

    expect(generated.edges[1]).toEqual({
      id: "edge-8",
      sourceNodeId: "node-2",
      targetNodeId: "node-1",
      condition: StageEdgeCondition.OnSuccess,
      inputKey: "result",
      loopControl: false,
    });
    expect(
      connectNodes(state, {
        sourceNodeId: "node-1",
        targetNodeId: "node-2",
      }),
    ).toBe(state);
    expect(() =>
      connectNodes(state, {
        id: "edge-7",
        sourceNodeId: "node-2",
        targetNodeId: "node-1",
      }),
    ).toThrow("Edge ID already exists: edge-7");
    expect(() =>
      connectNodes(state, {
        sourceNodeId: "node-1",
        targetNodeId: "node-1",
      }),
    ).toThrow("Self-connections are not allowed");
  });

  it("disconnects edges by id and by endpoints", () => {
    const state = createPipelineBuilderGraphState(
      [makeNode("node-1"), makeNode("node-2"), makeNode("node-3")],
      [
        makeEdge("edge-1", { sourceNodeId: "node-1", targetNodeId: "node-2" }),
        makeEdge("edge-2", { sourceNodeId: "node-2", targetNodeId: "node-1" }),
        makeEdge("edge-3", { sourceNodeId: "node-1", targetNodeId: "node-3" }),
      ],
    );

    const afterEdgeRemoval = disconnectEdge(state, { edgeId: "edge-2" });

    expect(afterEdgeRemoval.edges.map((edge) => edge.id)).toEqual(["edge-1", "edge-3"]);
    expect(disconnectNodes(state, { sourceNodeId: "node-1", targetNodeId: "node-2" }).edges).toEqual([
      makeEdge("edge-2", { sourceNodeId: "node-2", targetNodeId: "node-1" }),
      makeEdge("edge-3", { sourceNodeId: "node-1", targetNodeId: "node-3" }),
    ]);
  });

  it("validates blank labels and broken edge references", () => {
    const state = createPipelineBuilderGraphState(
      [makeNode("node-1", { label: "   " }), makeNode("node-2", { label: "Valid" })],
      [makeEdge("edge-1", { sourceNodeId: "ghost", targetNodeId: "ghost", condition: StageEdgeCondition.OnFailure })],
    );

    expect(validateGraph(state)).toEqual({
      nodeErrors: {
        "node-1": ["Label is required"],
      },
      edgeErrors: {
        "edge-1": [
          "Missing source node: ghost",
          "Missing target node: ghost",
          "Self-connections are not allowed",
        ],
      },
    });
  });
});
