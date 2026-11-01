//region Dryness readings
// READINGS OF THE PROVIDER METERS the tests of the re-ask, the seat derivation and the backfill share (ledger B1330):
// which providers have budget, as the router's own view of them is shaped. The type is written out here and not
// imported, since this package depends on no other package of the family; it is assignable to the provider package's
// `BudgetView`.

/**
 The providers a reading names.
 */
type DrynessProvider =
  | 'anthropic'
    | 'bedrock'
    | 'deepseek'
    | 'hyper'
    | 'neuralwatt'
    | 'openrouter'
    | 'synthetic'
    | 'xiaomi';

/**
 Whether each provider is out of budget, the shape of the provider package's `BudgetView`.
 */
export type DrynessReading = Readonly<Record<DrynessProvider, boolean>>;

/**
 A reading in which only the named providers have budget.

 @param wet - providers with budget

 @returns Reading with every other provider dry

 @example
 ```ts
 const dry = readingWith({ wet: ['hyper',], },);
 ```
 */
export function readingWith({ wet, }: { readonly wet: readonly (keyof DrynessReading)[]; },): DrynessReading {
  return {
    neuralwatt: !wet.includes('neuralwatt',),
    synthetic: !wet.includes('synthetic',),
    hyper: !wet.includes('hyper',),
    bedrock: !wet.includes('bedrock',),
    deepseek: !wet.includes('deepseek',),
    xiaomi: !wet.includes('xiaomi',),
    anthropic: !wet.includes('anthropic',),
    openrouter: !wet.includes('openrouter',),
  };
}

/**
 The reading in which Synthetic is the only provider with budget, so it would serve Qwen3.8-27B.
 */
export const SYNTHETIC_WET_ONLY: DrynessReading = readingWith({ wet: ['synthetic',], },);

/**
 The reading in which Hyper is the only provider with budget, so it would serve Qwen3.8-27B.
 */
export const HYPER_WET_ONLY: DrynessReading = readingWith({ wet: ['hyper',], },);

//endregion Dryness readings
