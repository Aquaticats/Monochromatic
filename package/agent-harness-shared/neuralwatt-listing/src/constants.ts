/**
 NeuralWatt provider constants shared by every harness: route identity,
 endpoints, credential reference, and the listing fetch bound. @module
 */

//region Public API

/**
 Provider route key the agent settings name (`neuralwatt/mimo-v2.6-pro`) and
 the key both harnesses register the provider under, so a model slug works
 identically in DSH and pi.
 */
export const NEURALWATT_ROUTE = 'neuralwatt';

/**
 Wire protocol the route speaks: the OpenAI chat-completions surface both
 harnesses already stream over.
 */
export const NEURALWATT_API = 'openai-completions';

/**
 Chat endpoint base; the harness adapters append their completions path.
 */
export const NEURALWATT_BASE_URL = 'https://api.neuralwatt.com/v1/';

/**
 Public model-listing endpoint the mount fetch reads; the listing carries no
 credential requirement.
 */
export const NEURALWATT_LISTING_URL = 'https://api.neuralwatt.com/v1/models';

/**
 Credential reference name each harness resolves in its own store: DSH
 addresses a record in its credentials service (`$DSH_HOME/.credentials.yaml`,
 written from the web Models page) with the launching environment as the
 fallback source, and pi walks its own credential precedence first. The name
 is a reference, not an environment requirement.
 */
export const NEURALWATT_API_KEY_ENV = 'NEURALWATT_API_KEY';

/**
 Bound on the mount listing fetch: the checked-in snapshot serves until the
 fetch lands, so a slow endpoint delays fresh metadata instead of delaying
 route registration.
 */
export const NEURALWATT_LISTING_TIMEOUT_MS = 10_000;

//endregion Public API
