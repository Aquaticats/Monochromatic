/**
 The provider pools the Pi runs of one machine share, what each of them holds, and which of them a run may take. Each limit
 sits beside the owner's words or the measured refusal it came from, because a number without its reason is a guess nobody can
 weigh.

 The pools are not the work a run does: they are how many runs a provider account answers at once. A run on another account, or
 on a provider nobody pools, is outside all of them.

 @module
 */

/**
 One provider pool: the provider and model that share an account's room, how many runs it takes at once, and how long a run on
 it may last.
 */
export type ProviderPool = {
  /**
   The name the counts are printed under.
   */
  readonly name: string;

  /**
   The provider a run on this pool names on its command line.
   */
  readonly provider: string;

  /**
   The model a run on this pool names on its command line.
   */
  readonly model: string;

  /**
   How many runs of this pool may be going at once.
   */
  readonly limit: number;

  /**
   How much longer a run on this pool may last than the same name on a standard pool, because the pool answers more slowly.
   */
  readonly wallFactor: number;

  /**
   Whether a run that names no provider or model of its own may take this pool, or whether only a run naming this model takes
   it.
   */
  readonly takesDefaultRuns: boolean;
};

/**
 Neuralwatt's standard DeepSeek pool.

 The owner, 2026-11-09: "Try 20, and if it doesn't blow up (in terms of api rate limits), we're permanently raising the practical
 limit to 20." Historical 429s that lowered it for a few hours are void: the owner wrote on 2026-11-11T05:4x-05:00 "I've
 re-enabled auto-renew. Treat the prev Neuralwatt 429s void."
 */
export const NEURALWATT_STANDARD: ProviderPool = {
  name: 'neuralwatt',
  provider: 'neuralwatt',
  model: 'deepseek-v4.1-flash',
  limit: 20,
  wallFactor: 1,
  takesDefaultRuns: true,
};

/**
 Neuralwatt's DeepSeek flex pool.

 The owner, 2026-11-10: "You can use -flex postfix on neuralwatt models. They have their own separate pool of 22 slots.
 They're around 100% slower than standard in practice." A flex run therefore gets twice the wall clock, which is what
 `wallFactor` is for.
 */
export const NEURALWATT_DEEPSEEK_FLEX: ProviderPool = {
  name: 'neuralwatt-flex',
  provider: 'neuralwatt',
  model: 'deepseek-v4.1-flash-flex',
  limit: 22,
  wallFactor: 2,
  takesDefaultRuns: true,
};

/**
 The `opencode-go` pool, which the owner ranked below Neuralwatt's flex: "In practice, opencode-go may have worse inference
 quality and worse speed than neuralwatt's flex, so prefer neuralwatt." So it is taken only once both Neuralwatt pools above
 it are full.
 */
export const OPENCODE_GO: ProviderPool = {
  name: 'opencode-go',
  provider: 'opencode-go',
  model: 'deepseek-v4.1-flash',
  limit: 20,
  wallFactor: 1,
  takesDefaultRuns: true,
};

/**
 Neuralwatt's Kimi flex pool, which only a run naming its model takes.

 The owner, 2026-11-10: "Neuralwatt rate limits are per model." and "Use 22 against Kimi K3.1 Pro flex from neuralwatt for now,
 until upstream clarifies or we hit something." The 7 that a 2026-11-11 Kimi flex 429 named ("max 7 in flight at once") was an
 assumption never measured, and the owner restored 22 the same night with "Why did you downgrade everything to 6 and 7?", so
 only a measured refusal seen since sets any lower number.
 */
export const NEURALWATT_KIMI_FLEX: ProviderPool = {
  name: 'neuralwatt-kimi-flex',
  provider: 'neuralwatt',
  model: 'kimi-k3.1-pro-flex',
  limit: 22,
  wallFactor: 2,
  takesDefaultRuns: false,
};

/**
 The pools a run may be counted in, in the owner's order of preference: standard Neuralwatt, then flex, then `opencode-go`.
 This is the order a run that names no model of its own takes them in.
 */
export const PROVIDER_POOLS: readonly ProviderPool[] = [
  NEURALWATT_STANDARD,
  NEURALWATT_DEEPSEEK_FLEX,
  OPENCODE_GO,
  NEURALWATT_KIMI_FLEX,
];

/**
 The provider a run gets when its command line names none, which is what the launcher the coordinator used wrote into every run
 it started itself.
 */
export const PROVIDER_OF_A_RUN_NAMING_NONE = 'neuralwatt';

/**
 The model a run gets when its command line names none.
 */
export const MODEL_OF_A_RUN_NAMING_NONE = 'deepseek-v4.1-flash';

/**
 The pool a run is counted in when neither a launcher process nor its unit's own command line names one. The scratch launcher
 kept this answer for every unit whose launcher had gone, which is how runs sat on standard Neuralwatt while it read empty until
 2026-11-11; reading the unit's `--provider` and `--model` replaced that, and this is only what is left when that read fails.
 */
export const POOL_OF_A_RUN_NAMING_NO_POOL: ProviderPool = OPENCODE_GO;

/**
 The start of the run name of an audit. Audits run on other accounts and are in no pool, so a counted pool never holds one.
 */
export const AUDIT_RUN_PREFIX = 'sol-';

/**
 The start of the run name of a Kimi run. The owner: "Kimi trials don't count within 20 slots because 20 is neuralwatt's
 effective concurrency and kimi runs on Synthetic." A Kimi run that does name a pool, such as Neuralwatt's Kimi flex, is
 counted in it like any other run.
 */
export const KIMI_RUN_PREFIX = 'kimi-';

/**
 What a run's pool membership is: the pool it holds a slot in, or the fact that no shared pool holds it.
 */
export type PoolMembership =
  | {
    readonly inAPool: true;
    readonly pool: ProviderPool;
  }
    | { readonly inAPool: false };

/**
 Reads the pool a run's command line names.

 @param provider - the provider the run names

 @param model - the model the run names

 @returns the pool that provider and model belong to, or nothing when they belong to none

 @example
 ```ts
 const pool = poolNamed({ provider: 'neuralwatt', model: 'deepseek-v4.1-flash', },);
 ```
 */
export function poolNamed({
  provider,
  model,
}: {
  readonly provider: string;
  readonly model: string;
}): PoolMembership {
  /**
   The pool that this provider and model share.
   */
  const found = PROVIDER_POOLS.find(function isNamed(pool,) {
    return (pool.provider === provider) && (pool.model === model);
  },);
  return (found === undefined) ? { inAPool: false, } : {
    inAPool: true,
    pool: found,
  };
}

/**
 Reads which pool a run counts against.

 @param runName - the run's name, which says whether it is an audit or a Kimi run

 @param pool - the pool its command line names, when one was read

 @returns the pool it holds a slot in, or that no shared pool holds it

 @example
 ```ts
 const membership = membershipOf({ runName: 'kimi-review-1', },);
 ```
 */
export function membershipOf({
  runName,
  pool,
}: {
  readonly runName: string;
  readonly pool?: PoolMembership;
}): PoolMembership {
  if (runName.startsWith(AUDIT_RUN_PREFIX,)) {
    return { inAPool: false, };
  }
  if (pool !== undefined) {
    return pool;
  }
  if (runName.startsWith(KIMI_RUN_PREFIX,)) {
    return { inAPool: false, };
  }
  return {
    inAPool: true,
    pool: POOL_OF_A_RUN_NAMING_NO_POOL,
  };
}
