/**
 The one fast-check run budget a fuzz sidecar reads.

 A fuzz sidecar runs its `*.property.unit.test.ts` files in two layers: the normal unit suite, which needs a cheap
 bounded count, and its `fuzz` task, which sets an environment variable for a longer campaign. Issue 1419 found that
 count, its environment read and its fallback copied character-identically into ten sidecars, so the default and the
 policy for a value that does not parse were decided in ten places with no owner. Both are decided here once, and a
 sidecar passes only the name of its own variable.

 @module
 */

//region Run budget

/**
 Count a property runs when no campaign variable supplies one.

 Low enough that the property files stay cheap inside the normal unit suite, high enough to explore each one's input
 space. A sidecar whose properties cost milliseconds per run passes its own count to
 {@link fuzzRunsFromEnvironment}.
 */
export const DEFAULT_FUZZ_RUNS = 200;

/**
 What one sidecar supplies: the name of its own campaign variable, and the fallback and environment it needs.
 */
export type FuzzRunBudget = {
  /**
   Environment variable the sidecar's `fuzz` task sets, such as `P_MAP_FORK_FUZZ_RUNS`.
   */
  readonly envName: string;
  /**
   Count used when the variable asks for nothing usable; {@link DEFAULT_FUZZ_RUNS} unless a sidecar's properties need
   their own.
   */
  readonly defaultRuns?: number;
  /**
   Where the variable is read; the process environment unless a test gives its own.
   */
  readonly environment?: Readonly<NodeJS.ProcessEnv>;
};

/**
 Number of fast-check runs one sidecar's properties take.

 A variable that is unset, blank, not a number, negative, zero or fractional falls back to the count, because
 fast-check takes `numRuns` verbatim and `0` would run every property through zero iterations: an empty variable would
 silently turn the gate green instead of failing loudly. Only a positive integer raises the count, which is what a
 campaign passes.

 The read happens per call, and a sidecar calls this once at module load, so the campaign set before node starts is the
 campaign its properties run.

 @param envName - variable a campaign sets; anything but a positive integer leaves the count at the fallback

 @param defaultRuns - count a sidecar's properties need when no campaign runs, since a fallback too low explores little
   and too high slows the normal unit suite

 @param environment - process environment to read, so a test can state one without changing the process's own

 @returns runs per property: the campaign's positive integer, or the fallback

 @example
 ```ts
 export const fuzzRuns: number = fuzzRunsFromEnvironment({ envName: 'P_MAP_FORK_FUZZ_RUNS' });
 ```
 */
export function fuzzRunsFromEnvironment({
  envName,
  defaultRuns = DEFAULT_FUZZ_RUNS,
  environment = process.env,
}: FuzzRunBudget,): number {
  /**
   The variable read as a number, which is `NaN` when the variable is unset or holds text.
   */
  const override = Number(environment[envName],);
  if (Number.isInteger(override,) && (override > 0))
    return override;
  return defaultRuns;
}

//endregion Run budget
