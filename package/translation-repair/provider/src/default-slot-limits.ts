import { DEEPSEEK_PER_MODEL_CONCURRENCY, } from './deepseek-client.ts';
import { NEURALWATT_PER_MODEL_CONCURRENCY, } from './neuralwatt-client.ts';
import { SYNTHETIC_PER_MODEL_CONCURRENCY, } from './synthetic-client.ts';
import type { SlotLimits, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Default slot limits
// SPLIT OUT OF `provider-router.ts` at its line budget.

/**
 Per-model slots the providers grant by default: no ceiling on any of them.
 Synthetic had a measured five until 2026-10-19, when its rate-limit page stopped
 naming a concurrency limit; the entry still reads the client's own default, so
 the router's count and the client's limiter cannot drift apart.
 */
export const DEFAULT_SLOT_LIMITS: SlotLimits = {
  // NeuralWatt publishes no per-model ceiling that was found (2026-10-20); the
  // energy ledger's in-flight hold is what bounds what is risked at once.
  neuralwatt: NEURALWATT_PER_MODEL_CONCURRENCY,
  synthetic: SYNTHETIC_PER_MODEL_CONCURRENCY,
  hyper: Number.POSITIVE_INFINITY,
  bedrock: Number.POSITIVE_INFINITY,
  // DeepSeek states 2,500 concurrent requests for its one model (2026-10-22,
  // `deepseek-client.ts`), the only real ceiling in this table; Xiaomi states
  // a request rate instead, which its pacer meets (`paces`).
  deepseek: DEEPSEEK_PER_MODEL_CONCURRENCY,
  xiaomi: Number.POSITIVE_INFINITY,
  openrouter: Number.POSITIVE_INFINITY,
};

//endregion Default slot limits
