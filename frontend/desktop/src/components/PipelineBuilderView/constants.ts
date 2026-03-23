import type { StageExecutionIntent } from "../../types";

export const EXECUTION_INTENTS: StageExecutionIntent[] = ["text", "code"];
export const SESSION_GROUP_OPTIONS = ["A", "B", "C", "D", "E", "F"];
export const VARIABLE_CHIPS = [
  "{{task}}",
  "{{code_context}}",
  "{{previous_output}}",
  "{{file_list}}",
  "{{iteration_number}}",
  "{{test_results}}",
];
