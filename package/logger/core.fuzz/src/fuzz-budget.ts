/**
 Shared run-plan for the property-based fuzz tests.

 The same `*.property.unit.test.ts` files serve two layers. Run by the
 normal unit suite (env unset), each property does a bounded number of
 runs so the suite stays fast and deterministic in wall-clock. Run as the
 `fuzz` campaign (env set to a per-property millisecond budget), each
 property runs effectively unbounded until its time budget elapses, so
 the campaign explores far deeper without a second copy of the
 arbitraries.

 The seed is left random in both layers (so coverage broadens across
 runs); a discovered counterexample is reproduced from the seed
 fast-check prints and then pinned as a fast-check `examples` entry on the
 offending property.

 The bounded run count is lower than toml-edit's because logger
 properties await real sink work and, for the deadline properties, real
 timers; each run costs milliseconds rather than microseconds.

 @module
 */

/**
 Environment variable naming the per-property campaign time budget in
 milliseconds. Unset (or non-positive) selects bounded mode; a positive
 integer selects campaign mode with that per-property budget.
 */
const FUZZ_BUDGET_ENV_NAME = 'LOGGER_FUZZ_BUDGET_MS';

/**
 Runs per property in bounded mode. High enough to explore each
 interleaving space, low enough that a file's properties stay well under
 the timeout even when every run awaits a few real milliseconds.
 */
const BOUNDED_NUM_RUNS = 60;

/**
 Per-property timeout in bounded mode. A property runs its body
 {@link BOUNDED_NUM_RUNS} times, so it needs a wider budget than a single
 assertion even though each run is a few milliseconds.
 */
const BOUNDED_TIMEOUT_MS = 60_000;

/**
 Extra time added to a campaign property's budget before the harness
 timeout fires, leaving room for fast-check to interrupt and shrink.
 */
const CAMPAIGN_TIMEOUT_MARGIN_MS = 30_000;

/**
 fast-check `Parameters` subset shared by both layers, structurally
 assignable to `assert`'s options argument.
 */
type FuzzRunParams = {
  readonly interruptAfterTimeLimit?: number;
  readonly markInterruptAsFailure?: boolean;
  readonly numRuns: number;
};

/**
 Resolved plan for one property: the options to hand `assert` and the
 harness timeout to set on the enclosing `it`.
 */
export type FuzzRunPlan = {
  readonly params: FuzzRunParams;
  readonly timeout: number;
};

/**
 Sentinel budget meaning bounded mode; a real campaign budget is always
 positive, so zero is an unambiguous non-nullish marker for "unset".
 */
const BOUNDED_BUDGET = 0;

/**
 Parse the campaign budget from the environment.

 @returns Positive millisecond budget, or {@link BOUNDED_BUDGET} for
   bounded mode.

 @example
 ```ts
 const budget = campaignBudgetMs();
 ```
 */
function campaignBudgetMs(): number {
  /**
   Raw budget string from the environment, if present.
   */
  const raw = process.env[FUZZ_BUDGET_ENV_NAME];
  if ((raw === undefined) || (raw === ''))
    return BOUNDED_BUDGET;
  /**
   Numeric budget parsed from the raw string.
   */
  const parsed = Number(raw,);
  if (!Number.isInteger(parsed,))
    return BOUNDED_BUDGET;
  if (parsed <= 0)
    return BOUNDED_BUDGET;
  return parsed;
}

/**
 Whether the campaign budget is active.

 @returns Whether a positive campaign budget is set.

 @example
 ```ts
 const deep = isCampaignMode();
 ```
 */
export function isCampaignMode(): boolean {
  return campaignBudgetMs() !== BOUNDED_BUDGET;
}

/**
 Resolve the run plan from the environment. Call once per file and reuse
 across that file's properties.

 @returns Bounded plan when the campaign budget is unset, otherwise a
   time-budgeted campaign plan.

 @example
 ```ts
 const run = fuzzRunPlan();
 await assert(asyncProperty(arb, predicate,), run.params,);
 ```
 */
export function fuzzRunPlan(): FuzzRunPlan {
  /**
   Campaign budget, or {@link BOUNDED_BUDGET} in bounded mode.
   */
  const budget = campaignBudgetMs();
  if (budget === BOUNDED_BUDGET) {
    return {
      params: { numRuns: BOUNDED_NUM_RUNS, },
      timeout: BOUNDED_TIMEOUT_MS,
    };
  }
  return {
    params: {
      numRuns: Number.MAX_SAFE_INTEGER,
      interruptAfterTimeLimit: budget,
      markInterruptAsFailure: false,
    },
    timeout: budget + CAMPAIGN_TIMEOUT_MARGIN_MS,
  };
}
