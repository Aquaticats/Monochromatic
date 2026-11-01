//region NeuralWatt public index
// The names the NeuralWatt provider gives its siblings and its own tests, each
// from the module that declares it; split out of `index.ts` when the fifth
// provider put that file over the line budget.

export {
  NEURALWATT_AUTH_HEADER,
  NEURALWATT_BASE_URL,
  NEURALWATT_MODELS,
  neuralWattChatUrlFor,
  type NeuralWattModelInfo,
  neuralWattQuotaUrlFor,
  neuralWattModelsUrlFor,
  neuralWattUsageByModelUrlFor,
} from './neuralwatt-catalog.ts';
export { type NeuralWattReturn, } from './budget-routing.ts';
export {
  createNeuralWattClient,
  NEURALWATT_PER_MODEL_CONCURRENCY,
  type NeuralWattClient,
  NeuralWattModelNotServedError,
} from './neuralwatt-client.ts';
export {
  aliasNamesOf,
  type NeuralWattAliasMap,
} from './neuralwatt-aliases.ts';
export {
  type NeuralWattComments,
  readNeuralWattComments,
} from './neuralwatt-comments.ts';
export {
  NEURALWATT_DAY_START_VAR,
  type NeuralWattDayStart,
  NeuralWattDayStartOverrideError,
  neuralWattDayAfter,
  neuralWattDayStartAt,
  neuralWattDayStartFrom,
  neuralWattDayStartOfInstant,
} from './neuralwatt-day.ts';
export {
  dayIsClosed,
  energyOfDay,
  entryIsInDay,
} from './neuralwatt-day-energy.ts';
export {
  type NeuralWattDay,
  NeuralWattAllowanceError,
  type NeuralWattEnergy,
} from './neuralwatt-energy.ts';
export {
  blockedUntilOf,
  keyAllowanceOf,
  type NeuralWattKeyAllowance,
  type NeuralWattKeyAllowanceReading,
} from './neuralwatt-key-allowance.ts';
export {
  coveredWhOf,
  NeuralWattUsageShapeError,
} from './neuralwatt-usage.ts';
export {
  estimateNeuralWattWh,
  NEURALWATT_ENERGY_MARGIN,
  NEURALWATT_SMALLEST_ESTIMATE_WH,
  type NeuralWattEstimate,
  promptTokensOfBody,
} from './neuralwatt-estimate.ts';
export {
  type ImagePixelSize,
  imagePixelSizeOf,
} from './neuralwatt-image-size.ts';
export {
  NEURALWATT_UNSIZED_PICTURE_TOKENS,
  pictureTokensOfSize,
  pictureTokensOfUrl,
  promptTokensOfRequest,
} from './neuralwatt-picture-tokens.ts';
export {
  createNeuralWattLedger,
  type NeuralWattLedger,
  neuralWattLedgerFromEnv,
  type NeuralWattReservation,
  NeuralWattReservationSettledError,
  type NeuralWattReserveOutcome,
  type NeuralWattSettlement,
} from './neuralwatt-ledger.ts';
export {
  defaultNeuralWattLedgerPath,
  NEURALWATT_DAILY_WH,
  NEURALWATT_DAILY_WH_VAR,
  NEURALWATT_LEDGER_PATH_VAR,
  NeuralWattAllowanceOverrideError,
  neuralWattDailyWhFrom,
  NeuralWattLedgerShapeError,
  neuralWattLedgerPathFrom,
} from './neuralwatt-ledger-config.ts';
export {
  NEURALWATT_RECKONINGS,
  type NeuralWattLedgerEntry,
  type NeuralWattReckoning,
} from './neuralwatt-ledger-entry.ts';
export {
  neuralWattIsDry,
  neuralWattMeterLevel,
} from './neuralwatt-meter.ts';
export {
  describePlanSpend,
  type NeuralWattPlanSpend,
  type NeuralWattQuota,
  NeuralWattQuotaShapeError,
  type NeuralWattSubscription,
  parseNeuralWattQuota,
  planSpendBetween,
} from './neuralwatt-quota.ts';
export {
  createNeuralWattQuotaSession,
  type NeuralWattQuotaSession,
} from './neuralwatt-quota-session.ts';
export {
  neuralWattIdFor,
  type NeuralWattSpelling,
  neuralWattShowsPictures,
} from './neuralwatt-reach.ts';
export {
  NEURALWATT_RETURN_MARGIN_MS,
  NEURALWATT_RETURN_WAIT_BOUND_MS,
  type NeuralWattCause,
  neuralWattDrynessOf,
  type NeuralWattDryness,
} from './neuralwatt-return-hold.ts';
export { createNeuralWattReturnMark, } from './neuralwatt-return-mark.ts';

//endregion NeuralWatt public index
