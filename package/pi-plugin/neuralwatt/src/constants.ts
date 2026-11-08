/**
 Package constants for the NeuralWatt pi extension:
 pi's documented model defaults,
 the identity pi displays,
 and the credential guidance text.

 @module
 */

import { NEURALWATT_API_KEY_ENV, } from '@monochromatic-dev/agent-harness-shared-neuralwatt-listing/ts';

//region Pi schema defaults

/**
 pi's `models-configuration` documentation states these defaults for
 `contextWindow` and `maxTokens`,
 so the projection fills them exactly where a listing discloses no limit
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
 Zero price rate one missing pricing field serves.
 `ModelCost` in `@earendil-works/pi-ai` 1.5.0 requires four finite numbers and
 carries no unknown marker,
 so the zero display is the recorded acceptance in
 `doc/planning/neuralwatt-pi-plugin.md`.
 */
export const NEURALWATT_COST_RATE_ZERO = 0;

//endregion Cost rates

//region Provider identity

/**
 Display name pi shows for the route,
 matching the DSH plugin's route presentation.
 */
export const NEURALWATT_PROVIDER_NAME = 'NeuralWatt';

//endregion Provider identity

//region Credential seam

/**
 Display name pi shows for the credential in status surfaces.
 */
export const NEURALWATT_CREDENTIAL_NAME = 'NeuralWatt API key';

/**
 Environment variable `@monochromatic-dev/env-local-json-core` consults for its
 store path override.
 The reader honors the name itself,
 so this package only inspects it to decide whether an explicit path is safe.
 */
export const ENV_LOCAL_JSON_PATH_VARIABLE = 'ENV_LOCAL_JSON_PATH';

/**
 Guidance logged when no credential resolves,
 naming every accepted home so the remediation is actionable without reading
 code.
 */
export const NEURALWATT_CREDENTIAL_GUIDANCE: string = `No NeuralWatt credential resolved, so requests will fail until one lands. Store ${NEURALWATT_API_KEY_ENV} in the sops/age-encrypted .env.local.json store, or hand it to pi (--api-key or pi's auth.json), or export ${NEURALWATT_API_KEY_ENV}. Models stay listed meanwhile.`;

//endregion Credential seam
