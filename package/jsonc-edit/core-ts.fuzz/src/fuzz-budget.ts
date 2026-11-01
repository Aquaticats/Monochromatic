/**
 Per-property fast-check run count.
 
 Default keeps the property files cheap when they run as part of the normal
 unit suite; the `fuzz` task raises `JSONC_EDIT_FUZZ_RUNS` for a longer
 campaign.
 
 @module
 */

//region Run budget

/**
 Run count used when no campaign budget is supplied.
 */
const DEFAULT_RUNS = 200;

/**
 Parsed override from the environment, or `NaN` when unset or unparseable.
 */
const override = Number(process.env
  .JSONC_EDIT_FUZZ_RUNS,);

/**
 Number of fast-check runs per property. Set `JSONC_EDIT_FUZZ_RUNS` to override.
 
 @example
 ```ts
 assert(property(arb, predicate), { numRuns: fuzzRuns });
 ```
 */
export const fuzzRuns: number = Number.isFinite(override,)
  ? override
  : DEFAULT_RUNS;

//endregion Run budget
