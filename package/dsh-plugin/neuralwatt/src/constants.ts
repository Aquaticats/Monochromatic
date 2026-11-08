/**
 NeuralWatt route constants: settings namespace, re-exporting the shared
 route identity, endpoint, credential reference, and listing fetch bound. @module
 */

export {
  NEURALWATT_API,
  NEURALWATT_API_KEY_ENV,
  NEURALWATT_BASE_URL,
  NEURALWATT_LISTING_TIMEOUT_MS,
  NEURALWATT_LISTING_URL,
  NEURALWATT_ROUTE,
} from '@monochromatic-dev/agent-harness-shared-neuralwatt-listing/ts';

//region Public API

/**
 Settings namespace the route is written to: the `llm-pi-ai` profile entry
 owns the single pi-ai adapter instance, and its volatile `providers` field is
 the layer that merges this route in beside the hand-declared ones.
 */
export const NEURALWATT_SETTINGS_NAMESPACE = 'llm-pi-ai';

//endregion Public API
