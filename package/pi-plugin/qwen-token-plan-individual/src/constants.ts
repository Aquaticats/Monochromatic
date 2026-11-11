/**
 Package constants for the Qwen Token Plan Individual pi extension:
 pi's documented model defaults,
 the identity pi displays,
 the endpoint pi already streams over,
 and the credential homes the seam resolves.

 @module
 */

import {
  QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME,
} from '@monochromatic-dev/agent-harness-shared-qwen-token-plan-individual-listing/ts';

//region Pi schema defaults

/**
 pi's `models-configuration` documentation states these defaults for
 `contextWindow` and `maxTokens`,
 so the projection fills them exactly where the listing discloses no limit
 instead of inventing a ceiling the endpoint never stated.

 @see https://pi.dev/docs/latest/models-configuration
 */
export const PI_SCHEMA_DEFAULT_CONTEXT_WINDOW = 128_000;

/**
 pi's documented default output ceiling,
 cited beside {@link PI_SCHEMA_DEFAULT_CONTEXT_WINDOW}.
 */
export const PI_SCHEMA_DEFAULT_MAX_TOKENS = 16_384;

//endregion Pi schema defaults

//region Cost rates

/**
 Zero price rate the facts table serves.
 `ModelCost` in `@earendil-works/pi-ai` 1.5.2 requires four finite numbers and
 carries no unknown marker,
 and every row of the builtin catalog
 (`dist/providers/data/qwen-token-plan-individual.json`)
 prices all four at zero,
 so the projection copies that acceptance.
 */
export const QWEN_TOKEN_PLAN_COST_RATE_ZERO = 0;

//endregion Cost rates

//region Provider identity

/**
 Display name pi shows for the route,
 kept byte-identical to the builtin provider's own
 (`@earendil-works/pi-ai` 1.5.2 `dist/providers/qwen-token-plan-individual.js`),
 so the override changes the served catalog and nothing else on screen.
 */
export const QWEN_TOKEN_PLAN_INDIVIDUAL_PROVIDER_NAME = 'Qwen Token Plan Individual';

/**
 Chat endpoint base pi streams over,
 kept byte-identical to the builtin provider's own
 (`@earendil-works/pi-ai` 1.5.2 `dist/providers/qwen-token-plan-individual.js`)
 and to the `baseUrl` every builtin model row carries.
 */
export const QWEN_TOKEN_PLAN_INDIVIDUAL_BASE_URL =
  'https://token-plan.ap-southeast-1.maas.aliyuncs.com/compatible-mode/v1';

//endregion Provider identity

//region Credential seam

/**
 Display name pi shows for the credential in status surfaces,
 kept byte-identical to the builtin provider's own
 (`@earendil-works/pi-ai` 1.5.2 `dist/providers/qwen-token-plan-individual.js`).
 */
export const QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_LABEL =
  'Qwen Token Plan Individual API key';

/**
 The builtin provider's own environment name,
 kept as the last credential home so an export written for the builtin keeps
 working under the override
 (`@earendil-works/pi-ai` 1.5.2 `dist/env-api-keys.js` maps this route to it).
 */
export const QWEN_TOKEN_PLAN_INDIVIDUAL_BUILTIN_CREDENTIAL_ENV =
  'QWEN_TOKEN_PLAN_API_KEY';

/**
 Guidance logged when no credential resolves,
 naming every accepted home so the remediation is actionable without reading
 code.
 */
export const QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_GUIDANCE: string =
  `No Qwen Token Plan Individual credential resolved, so requests will fail until one lands. Hand the key to pi (--api-key, or the "qwen-token-plan-individual" entry in pi's auth.json), or export ${QWEN_TOKEN_PLAN_INDIVIDUAL_CREDENTIAL_NAME}, or export ${QWEN_TOKEN_PLAN_INDIVIDUAL_BUILTIN_CREDENTIAL_ENV}. Models stay listed meanwhile.`;

//endregion Credential seam
