import type { PoolPolicy, } from '../../dist/final/node/index.mjs';

//region Pool policy fixture
// THE POLICY OF AN INVOKER WHO SET NEITHER POOL VARIABLE, handed to every
// reader that pools settled artifacts in a case that is not about the policy.
// The readers take it as a value the entry files read from the environment,
// so a case says which policy it pools under rather than inheriting whatever
// the shell running the suite exported.
//
// AND THE REPOSITORY A POOL ASKED FOR NO COMMIT IS HANDED: a path that holds
// none, since such a pool never asks git anything, so a case that comes to need
// a repository fails loudly rather than reading the checkout it runs in.
//
// AND THE TWO REQUESTS THAT CONTRADICT EACH OTHER, as a built command's child
// is handed them and as the pool refuses them, for the as-built case of each
// command that pools.

/**
 No commit required and no mixed pool asked for.

 @example
 ```ts
 const pool = await resolvePool({ artifactsDir, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },);
 ```
 */
export const NO_POOL_POLICY: PoolPolicy = {
  requiredCommit: '',
  poolAll: false,
};

/**
 Repository handed to a pool that requires no commit, which asks it nothing: a
 path no directory stands at, so a git call made there refuses.

 @example
 ```ts
 const pool = await resolvePool({ artifactsDir, policy: NO_POOL_POLICY, repository: UNASKED_REPOSITORY, },);
 ```
 */
export const UNASKED_REPOSITORY: string = '/nonexistent/cattery/no-repository-here';

/**
 A required commit and a mixed pool asked for together, as variables a built
 command's child is handed.

 @example
 ```ts
 const run = await runBuiltCommand({
   command: 'score-probe',
   packageEntry: import.meta.resolve('@monochromatic-dev/translation-repair-probe'),
   env: { TRANSLATION_REPAIR_RUNS_DIR: runsDir, ...CONFLICTING_POOL_VARIABLES, },
 },);
 ```
 */
export const CONFLICTING_POOL_VARIABLES: Readonly<Record<string, string>> = {
  TRANSLATION_REPAIR_REQUIRED_COMMIT: 'tabby-tip',
  TRANSLATION_REPAIR_POOL_ALL: 'yes',
};

/**
 What the pool says when both are asked for, whole.

 @example
 ```ts
 expect(run.stderr,).toBe(`score-probe: ${CONFLICTING_POOL_SAYS}\n`,);
 ```
 */
export const CONFLICTING_POOL_SAYS: string = 'TRANSLATION_REPAIR_REQUIRED_COMMIT and TRANSLATION_REPAIR_POOL_ALL are both '
  + 'set, which asks for a filtered pool and an unfiltered one at the same time.\nUnset whichever was not '
  + 'meant. A required commit selects entries whose pipeline contains it; pooling all takes every generation '
  + 'and says so above the number.';

//endregion Pool policy fixture
