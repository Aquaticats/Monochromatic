/**
 Per-property fast-check run count.
 
 Default keeps the property files cheap when they run as part of the normal
 unit suite; the `fuzz` task raises `MAKE_ASYNCHRONOUS_FORK_FUZZ_RUNS` for a
 longer campaign.
 
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
  .MAKE_ASYNCHRONOUS_FORK_FUZZ_RUNS,);

/**
 Number of fast-check runs per property. Set
 `MAKE_ASYNCHRONOUS_FORK_FUZZ_RUNS` to override.
 
 @example
 ```ts
 assert(property(arb, predicate), { numRuns: fuzzRuns, });
 ```
 */
export const fuzzRuns: number = Number.isFinite(override,)
  ? override
  : DEFAULT_RUNS;

/**
 Wall clock one fast-check run costs on a four-CPU host, measured on this
 package's differential property file: 200 runs took 13.2s, 1000 runs 58.2s
 and 5000 runs 292.3s, so one run costs about 56ms and the cost is linear in
 the run count.
 */
const MEASURED_RUN_WALL_MS = 56;

/**
 Multiplier over {@link MEASURED_RUN_WALL_MS}: four times the measured cost
 covers this campaign's own two-CPU container, which is half the host the 56ms
 was measured on, and leaves a slow runner unable to report a working property
 as a hang.
 */
const SLOW_RUNNER_FACTOR = 4;

/**
 Millisecond floor of one property's timeout, which keeps a short unit-suite
 run far inside the guard while still bounding a property that never settles.
 */
const PROPERTY_TIMEOUT_FLOOR_MS = 60_000;

/**
 Wall clock one run is allowed once {@link SLOW_RUNNER_FACTOR} is applied.
 */
const ALLOWED_RUN_WALL_MS = MEASURED_RUN_WALL_MS * SLOW_RUNNER_FACTOR;

/**
 Wall clock the whole campaign budget is allowed before the floor is added.
 */
const CAMPAIGN_WALL_MS = ALLOWED_RUN_WALL_MS * fuzzRuns;

/**
 Harness timeout for one property, scaled by {@link fuzzRuns} so a raised
 campaign budget cannot exceed a fixed authored limit.
 
 At the default 200 runs this is 104.8s and at the `fuzz` task's 5000 runs
 1180s, against 13.2s and 292.3s measured here.
 
 @example
 ```ts
 it({ name, fn, timeout: fuzzPropertyTimeoutMs, });
 ```
 */
export const fuzzPropertyTimeoutMs: number = PROPERTY_TIMEOUT_FLOOR_MS + CAMPAIGN_WALL_MS;

//endregion Run budget
