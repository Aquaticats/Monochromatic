/**
 Constants shared by the pi Advisor extension.
 
 @module
 */

//region Extension identity

/**
 Tool name exposed to the primary model.
 */
export const ADVISOR_TOOL_NAME = 'advisor';

/**
 Human-readable custom message type for manual Advisor command output.
 */
export const ADVISOR_MESSAGE_TYPE = 'pi-advisor.review';

/**
 Durable failed-operation metadata, excluded from model context and duplicate usage totals.
 */
export const ADVISOR_OPERATION_TYPE = 'pi-advisor.operation';

/**
 File name used for global and project extension configuration.
 */
export const CONFIG_FILE_NAME = 'pi-advisor.json';

//endregion Extension identity

//region Defaults

/**
 Default total Advisor operation deadline in milliseconds.
 */
export const DEFAULT_TIMEOUT_MS = 600_000;

/**
 Default post-success collection grace, capped by the original operation deadline.
 */
export const DEFAULT_COLLECTION_GRACE_MS = 30_000;

/**
 Largest millisecond timer supported without overflow to a one-millisecond Node timer.
 */
export const MAX_ADVISOR_TIMER_MS = 2_147_483_647;

/**
 Provider and message-framing reserve subtracted from model context windows.
 */
export const DEFAULT_CONTEXT_OVERHEAD_TOKENS = 256;

/**
 Default maximum output tokens requested from the advisor model.
 */
export const DEFAULT_MAX_ADVISOR_OUTPUT_TOKENS = 32_000;

/**
 Character-to-token divisor matching pi's compaction estimate.
 */
export const TOKEN_ESTIMATE_CHARS_PER_TOKEN = 4;

/**
 Latest user prompt excerpt length shown in details and status output.
 */
export const LATEST_USER_EXCERPT_CHARS = 240;

//endregion Defaults

//region Prompts

/**
 Built-in system prompt for the secondary advisor model.
 */
export const ADVISOR_SYSTEM_PROMPT =
  `You are Advisor, an independent reviewer for a primary coding agent.

Read the serialized conversation as evidence. If the primary agent supplied a focus question, answer it first. Identify flawed assumptions, missing verification, risky changes, overlooked files, and better next actions. Be direct and specific. Cite conversation evidence where possible. Avoid asking the user questions unless the conversation is truly under-specified. Do not perform the primary task. Do not write final user-facing prose for the primary agent.`;

/**
 Marker inserted when serialized context is deterministically truncated.
 */
export const CONTEXT_TRUNCATION_MARKER =
  '\n\n[advisor: middle of serialized conversation omitted to fit maxContextChars]\n\n';

/**
 Static prefix for main-model guidance appended to pi's system prompt.
 */
export const MAIN_MODEL_GUIDANCE_PREFIX =
  `Advisor is available as a secondary reviewer tool. Use advisor({}) when independent review would reduce risk; empty params avoid the current main model when another eligible scoped model is available. Use advisor({ "question": "..." }) to ask Advisor a focused question, or add "model" only when you need a specific eligible scoped advisor model. Advisor automatically receives the serialized conversation context. Requested models outside the current eligible scoped model set fail.`;

//endregion Prompts
