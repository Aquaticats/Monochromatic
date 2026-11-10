/**
 Per-property fast-check run count.

 The default and the policy for an override that does not parse live once, in the shared fuzz tooling; this package
 names the environment variable its `fuzz` task raises for a longer campaign.

 @module
 */

import { fuzzRunsFromEnvironment, } from '@monochromatic-dev/fuzz-core/ts';

//region Run budget

/**
 Number of fast-check runs per property. Set `PIFY_FORK_FUZZ_RUNS` to override.

 @example
 ```ts
 assert(property(arb, predicate), { numRuns: fuzzRuns, });
 ```
 */
export const fuzzRuns: number = fuzzRunsFromEnvironment({
  envName: 'PIFY_FORK_FUZZ_RUNS',
},);

//endregion Run budget
