/**
 Shared run plan for the property files.

 The same `*.property.unit.test.ts` files serve two layers. Run by the
 normal unit suite (environment unset), each property does a bounded number
 of runs from one fixed seed, so `test:unit` and the `fuzz:coverage` gate are
 reproducible. Run by `src/campaign.ts` (environment set), each property does
 one campaign round: the round's run count from a fresh seed the campaign
 chooses and records, so a counterexample replays from the recorded seed.

 The campaign is unbounded across rounds, not within one: an in-process
 unbounded property would starve every later property of the same file.

 @module
 */

/**
 Environment variable carrying the campaign round's seed.
 */
const SEED_ENV_NAME = 'DEEPMERGE_FUZZ_SEED';

/**
 Environment variable carrying the campaign round's run count per property.
 */
const NUM_RUNS_ENV_NAME = 'DEEPMERGE_FUZZ_NUM_RUNS';

/**
 Fixed seed of the bounded layer. Any constant works; changing it changes
 which inputs the coverage gate reaches, so refreeze the baseline with it.
 */
const BOUNDED_SEED = 20_260_923;

/**
 Runs per property in the bounded layer. Each run merges a few small trees
 in microseconds, so this stays well inside the timeout.
 */
const BOUNDED_NUM_RUNS = 400;

/**
 Harness timeout per property in the bounded layer.
 */
const BOUNDED_TIMEOUT_MS = 60_000;

/**
 Harness timeout per property in a campaign round; rounds are sized by run
 count, so this is a stall guard rather than a budget.
 */
const CAMPAIGN_TIMEOUT_MS = 3_600_000;

/**
 fast-check `Parameters` subset shared by both layers, structurally
 assignable to `assert`'s options argument.
 */
type FuzzRunParams = {
  readonly numRuns: number;
  readonly seed: number;
};

/**
 Resolved plan for one property file: options for `assert` and the harness
 timeout for the enclosing `it`.
 */
export type FuzzRunPlan = {
  readonly params: FuzzRunParams;
  readonly timeout: number;
};

/**
 Error thrown when a campaign variable holds something other than an integer,
 so a typo in the campaign driver fails loudly instead of silently running
 the bounded layer.
 */
export class FuzzPlanError extends Error {
  /**
   @param name - Offending environment variable, named so the fix is obvious.
   
   @param raw - Offending value, echoed for the same reason.
   */
  constructor({
    name,
    raw,
  }: {
    readonly name: string;
    readonly raw: string
  },) {
    super(`${name} must be an integer, got ${JSON.stringify(raw,)}`,);
    this.name = 'FuzzPlanError';
  }
}

/**
 One campaign variable's reading: unset selects the bounded layer.
 */
type EnvInteger =
  | {
    readonly set: false;
  }
    | {
      readonly set: true;
      readonly value: number;
    };

/**
 Reading of a variable that is unset or empty.
 */
const UNSET: EnvInteger = { set: false, };

/**
 Read one integer campaign variable.

 @param name - Variable to read; absence selects the bounded layer.

 @returns Parsed integer, or {@link UNSET} when the variable is unset or empty.

 @throws {@link FuzzPlanError} When set to a non-integer.

 @example
 ```ts
 const seed = readIntegerEnv('DEEPMERGE_FUZZ_SEED');
 ```
 */
function readIntegerEnv(name: string,): EnvInteger {
  /**
   Raw value from the environment.
   */
  const raw = process.env[name];
  if ((raw === undefined) || (raw === ''))
    return UNSET;
  /**
   Numeric reading of the raw value.
   */
  const parsed = Number(raw,);
  if (!Number.isInteger(parsed,)) {
    throw new FuzzPlanError({
      name,
      raw,
    },);
  }
  return {
    set: true,
    value: parsed,
  };
}

/**
 Resolve the run plan from the environment. Call once per file and reuse it
 for every property in that file.

 @returns Bounded fixed-seed plan when no campaign variables are set,
   otherwise the campaign round's plan.

 @throws {@link FuzzPlanError} When a campaign variable is malformed.

 @example
 ```ts
 const RUN = fuzzRunPlan();
 assert(property(arb, predicate,), RUN.params,);
 ```
 */
export function fuzzRunPlan(): FuzzRunPlan {
  /**
   Campaign seed, when a campaign round is running.
   */
  const seed = readIntegerEnv(SEED_ENV_NAME,);
  /**
   Campaign run count, when a campaign round is running.
   */
  const numRuns = readIntegerEnv(NUM_RUNS_ENV_NAME,);
  if ((!seed.set) || (!numRuns.set)) {
    return {
      params: {
        numRuns: BOUNDED_NUM_RUNS,
        seed: BOUNDED_SEED,
      },
      timeout: BOUNDED_TIMEOUT_MS,
    };
  }
  return {
    params: {
      numRuns: numRuns.value,
      seed: seed.value,
    },
    timeout: CAMPAIGN_TIMEOUT_MS,
  };
}
