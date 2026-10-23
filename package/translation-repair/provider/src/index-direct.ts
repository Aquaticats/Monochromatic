//region Direct public index
// The names DeepSeek's and Xiaomi's own APIs give their siblings and their own
// tests, each from the module that declares it; split out of `index.ts` for the
// reason `index-neuralwatt.ts` gives.

export {
  DEEPSEEK_BASE_URL,
  DEEPSEEK_MODELS,
  deepSeekBalanceUrlFor,
  deepSeekChatUrlFor,
  DIRECT_AUTH_HEADER,
  type DirectModelInfo,
  XIAOMI_BASE_URL,
  XIAOMI_MODELS,
  xiaomiChatUrlFor,
} from './direct-catalog.ts';
export {
  createDirectClient,
  type DirectClient,
  type DirectRoute,
} from './direct-client.ts';
export { withoutRefusedKeyBody, } from './refused-key-body.ts';
export {
  DIRECT_COST_UNREPORTED,
  deepSeekCostOf,
  xiaomiCostOf,
} from './direct-cost.ts';
export {
  deepSeekIdFor,
  deepSeekShowsPictures,
  type DeepSeekSpelling,
  xiaomiIdFor,
  xiaomiShowsPictures,
  type XiaomiSpelling,
} from './direct-reach.ts';
export {
  type DeepSeekCacheSplit,
  deepSeekCacheSplitOf,
  NOT_REPORTED,
  systemFingerprintOf,
} from './direct-usage.ts';
export * from './index-claude.ts';
export { DEFAULT_SLOT_LIMITS, } from './provider-slot-limits.ts';
export {
  RETRY_AFTER_HEADER,
  retryAfterHeaderMsOf,
} from './retry-after-header.ts';
export { requireWholeStream, } from './require-whole-stream.ts';
export {
  DEEPSEEK_DRY_FLOOR_USD,
  deepSeekIsDry,
  deepSeekMeterLevel,
  claudeIsDry,
  claudeMeterLevel,
  xiaomiIsDry,
  xiaomiMeterLevel,
} from './budget-routing.ts';
export {
  type DeepSeekBalance,
  DeepSeekBalanceShapeError,
  parseDeepSeekBalance,
} from './deepseek-balance.ts';
export {
  createDeepSeekClient,
  DEEPSEEK_PER_MODEL_CONCURRENCY,
  type DeepSeekClient,
  DeepSeekModelNotServedError,
} from './deepseek-client.ts';
export {
  createXiaomiClient,
  XIAOMI_PACE_WINDOW_MS,
  XIAOMI_REQUESTS_PER_MINUTE,
  type XiaomiClient,
  XiaomiModelNotServedError,
} from './xiaomi-client.ts';
export {
  xiaomiSpendNow,
  type XiaomiSpend,
} from './xiaomi-spend.ts';

//endregion Direct public index
