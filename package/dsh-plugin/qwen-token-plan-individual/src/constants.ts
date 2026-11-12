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

/**
 Attempts one mount-time route operation makes against a transient mount race
 before it gives up. Sized from measured contention: `watch-lock-hold.ts`
 observed `package.json.lock` held for 11151 ms at web-probe boot
 (2026-11-11), `@deepseek-ai/dsh-atomic-write` `withFileLock` abandons a
 contended lock after `DEFAULT_LOCK_WAIT_MS = 2e3`, and the plugin-manager
 disable/enable window measured at 2026-11-11T13:11:29-05:00 to
 2026-11-11T13:11:35-05:00 holds the same lock across a re-trigger. Ten
 attempts at `QWEN_TOKEN_PLAN_INDIVIDUAL_MOUNT_RACE_RETRY_DELAY_MS` span about
 40 s: past every measured hold. The same budget covers the credentials
 service, which the loader can register after this plugin's `apply`
 (web-probe boot 2026-11-11 logged the lookup failing at apply while
 `@deepseek-ai/dsh-llm-pi-ai` resolves it per request).
 */
export const QWEN_TOKEN_PLAN_INDIVIDUAL_MOUNT_RACE_RETRY_ATTEMPTS = 10;

/**
 Delay between the attempts of one mount-time route operation, matching the 2 s
 `DEFAULT_LOCK_WAIT_MS` wait `@deepseek-ai/dsh-atomic-write` `withFileLock`
 spends on a contended profile writer lock, so an attempt lands right behind
 the competing writer's release instead of idling through its whole hold.
 */
export const QWEN_TOKEN_PLAN_INDIVIDUAL_MOUNT_RACE_RETRY_DELAY_MS = 2_000;

//endregion Public API
