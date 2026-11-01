/**
 DSH-adapter constants shared by catalog construction, request mapping,
 ranking, and status assembly. @module
 */

/**
 Reasoning-effort label treated as the provider's maximum effort level.

 Advisor parity requests the highest level strictly below the provider
 maximum, so an effort carrying this label on its id or name is never
 selected.
 */
export const DSH_MAX_REASONING_EFFORT_LABEL = 'max';

/**
 Context-window stand-in for routes whose `LlmResolvedModelInfo.context` is
 absent.

 The shared budget math needs a concrete token count; `128_000` is the usual
 floor of modern chat routes. This is a DSH behavior pick recorded in the
 plugin README: unknown windows budget conservatively against real routes and
 rank below any route advertising a larger window. Veto open.
 */
export const DSH_UNKNOWN_CONTEXT_WINDOW_TOKENS = 128_000;

/**
 API protocol label stamped on catalog models built from `ctx.llm` routes.

 The Advisor core reads `Model.api` only as an opaque protocol identifier; the
 DSH runtime owns actual dispatch, so this label marks adapter-built records.
 */
export const DSH_MODEL_API = 'dsh';

/**
 Base-url stand-in for catalog models built from `ctx.llm` routes.

 Routes have no pi-ai base URL; the LLM runtime resolves routing from its own
 provider registration.
 */
export const DSH_MODEL_BASE_URL = '';

/**
 Tool-name stand-in for tool-result messages whose originating tool call is
 missing from the derived history.

 Kept distinct from `ADVISOR_TOOL_NAME` so prior-result filtering never
 mistakes it for an Advisor result.
 */
export const DSH_UNKNOWN_TOOL_NAME = 'unknown-tool';

/**
 Provider-neutral failure codes that mean exhausted quota or balance.

 Terminal `finish` failures carrying these codes are rewritten to include the
 `insufficient_quota` diagnostic marker the shared credit classifier matches.
 */
export const DSH_CREDIT_EXHAUSTION_CODES = [
  'QUOTA',
  'ACCOUNT_QUOTA',
] as const;

/**
 Diagnostic marker appended to quota-code failures for shared credit
 classification.
 */
export const DSH_CREDIT_EXHAUSTION_MARKER = 'insufficient_quota';
