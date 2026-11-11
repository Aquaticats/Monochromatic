/**
 Qwen Token Plan individual provider constants shared by every harness: route
 identity, wire protocol, credential reference, and the listing fetch bound.
 @module
 */

//region Public API

/**
 Provider route key the agent settings name
 (`qwen-token-plan-individual/qwen3.8-max`) and the key both harnesses register
 the provider under, so a model slug works identically in DSH and pi
 (`doc/planning/qwen-token-plan-individual-provider.md` section "Identity and
 behavior invariants").
 */
export const QWEN_TOKEN_PLAN_INDIVIDUAL_ROUTE = 'qwen-token-plan-individual';

/**
 Wire protocol the route speaks: the OpenAI chat-completions surface both
 harnesses already stream over, matching the pi-ai builtin catalog's `api`
 field for every row it carries.
 */
export const QWEN_TOKEN_PLAN_INDIVIDUAL_API = 'openai-completions';

/**
 Credential record name each harness resolves in its own store: pi walks its
 stored credential and environment fallbacks first, DSH addresses this record
 in its credentials service (`$DSH_HOME/.credentials.yaml`, already present,
 measured in the planning doc). The name is a reference, not an environment
 requirement; this package holds no credential itself.
 */
export const QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME =
  'QWEN_TOKEN_PLAN_INDIVIDUAL_API_KEY';

/**
 Bound on the mount listing fetch. Owner-pinned beside the value in
 `doc/planning/qwen-token-plan-individual-provider.md` section "Shared core
 export surface"; workload basis: the fetch runs at mount while the checked-in
 snapshot already serves, so the bound delays fresh membership, never route
 registration or chat dispatch.
 */
export const QWEN_TOKEN_PLAN_INDIVIDUAL_LISTING_TIMEOUT_MS = 10_000;

//endregion Public API
