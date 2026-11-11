/**
 Qwen Token Plan Individual route constants: the DSH-side endpoint base,
 re-exporting the shared route identity, wire protocol, credential reference,
 and listing fetch bound. @module
 */

export {
  QWEN_TOKEN_PLAN_INDIVIDUAL_API,
  QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME,
  QWEN_TOKEN_PLAN_INDIVIDUAL_LISTING_TIMEOUT_MS,
  QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE,
} from '@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/ts';

//region Public API

/**
 Chat endpoint base this harness serves the route on: the host the hand
 configuration already named, kept per harness because pi pins a different host
 of the same endpoint and both resolve to the same upstream. The listing GET
 reads `{base}/models` from this base.
 */
export const QWEN_TOKEN_PLAN_INDIVIDUAL_BASE_URL = 'https://token-plan.maas.qwencloudapi.com/compatible-mode/v1';

/**
 Settings namespace the route is written to: the `llm-pi-ai` profile entry
 owns the single pi-ai adapter instance, and its volatile `providers` field is
 the layer that merges this route in beside the hand-declared ones.
 */
export const QWEN_TOKEN_PLAN_INDIVIDUAL_SETTINGS_NAMESPACE = 'llm-pi-ai';

//endregion Public API
