/**
 The provider pools the Pi runs of one machine share, what each of them holds, and which of them a run may take. Each limit
 sits beside the owner's words or the measured refusal it came from, because a number without its reason is a guess nobody can
 weigh.

 The pools are not the work a run does: they are how many runs a provider account answers at once. A run on another account, or
 on a provider nobody pools, is outside all of them.

 A pool whose limit is {@link CLOSED_POOL_LIMIT} takes no new run at all: the runs already on it are still counted where they
 are, and a run that names it is still read as a run naming it.

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
   How many runs of this pool may be going at once, or {@link CLOSED_POOL_LIMIT} while it takes no new run.
   */
  readonly limit: number;

  /**
   How much longer a run on this pool may last than the same name on a standard pool, because the pool answers more slowly.
   */
  readonly wallFactor: number;

  /**
   Whether a run that names no model of its own may take this pool, or whether only a run naming this model takes it.
   */
  readonly takesDefaultRuns: boolean;
};

/**
 The limit a pool carries while it takes no new run, which two of them do: one the owner replaced, and one they closed while its
 week is used up.

 A closed pool keeps its place in the table because its runs are still counted where they are, and because a run that names the
 launcher's default provider and model is still read as a default run, which the first pool with room then takes.
 */
export const CLOSED_POOL_LIMIT = 0;

/**
 The qwen plan's DeepSeek pool, which a run that names no model of its own takes first.

 The owner, 2026-11-11T17:5x-05:00: "I think there's no point to withholding all Qwen quota to Qwen itself. Please replace the
 neuralwatt standard deepseek pool with qwen-token-plan-individual  deepseek-v4.1-flash-1017". So default runs go here first,
 where they went to standard Neuralwatt, with the 18 slots that pool had. The plan's own concurrency limit is neither stated nor
 measured: a 429 that names one sets the real number, as it did for Neuralwatt.

 Those 18 are the size the owner had just given the pool this replaced, 2026-11-11T17:0x-05:00: "Change flex concurrency to
 22-2=20 per model and normal concurrency to 20-2=18 per model. Neuralwatt said to keep 1 to 2 below the limit because \"a slot
 frees up a moment after your client sees the response finish, so a request sent straight away can land as the 23rd\"."
 Neuralwatt's 429 of 2026-11-11T01:19Z, "Concurrent limit reached for deepseek-ai/DeepSeek-V4.1-Flash: 6/6 slots in use. ...
 Your subscription is in overage with auto-renew off, so pay-as-you-go limits apply until it renews.", ended 16 runs while the
 pool read 20, so the slot that frees a moment late is what the two spare slots cover.
 */
export const QWEN_PLAN_DEEPSEEK: ProviderPool = {
  name: 'qwen-plan-deepseek',
  provider: 'qwen-token-plan-individual',
  model: 'deepseek-v4.1-flash-1017',
  limit: 18,
  wallFactor: 1,
  takesDefaultRuns: true,
};

/**
 Neuralwatt's standard DeepSeek pool, which no new run takes since the qwen plan replaced it.

 The owner's words beside {@link QWEN_PLAN_DEEPSEEK} are the replacement, so the limit is {@link CLOSED_POOL_LIMIT} and it was
 18. The entry stays so that the runs already on standard Neuralwatt are counted where they are, and so a run that names the
 launcher's default provider and model is still read as a default run and placed on the first pool with room.

 The owner, 2026-11-09 kept the account's own limit at 20: "Try 20, and if it doesn't blow up (in terms of api rate limits),
 we're permanently raising the practical limit to 20." Historical 429s that lowered it for a few hours are void: the owner wrote
 on 2026-11-11T05:4x-05:00 "I've re-enabled auto-renew. Treat the prev Neuralwatt 429s void."
 */
export const NEURALWATT_STANDARD: ProviderPool = {
  name: 'neuralwatt',
  provider: 'neuralwatt',
  model: 'deepseek-v4.1-flash',
  limit: CLOSED_POOL_LIMIT,
  wallFactor: 1,
  takesDefaultRuns: true,
};

/**
 Neuralwatt's DeepSeek flex pool, which a run that names no model of its own takes once the qwen plan's pool is full.

 The owner, 2026-11-10: "You can use -flex postfix on neuralwatt models. They have their own separate pool of 22 slots. They're
 around 100% slower than standard in practice." A flex run therefore gets twice the wall clock, which is what `wallFactor` is
 for.

 The owner, 2026-11-11T17:0x-05:00 then set its limit: "Change flex concurrency to 22-2=20 per model and normal concurrency to
 20-2=18 per model. Neuralwatt said to keep 1 to 2 below the limit because \"a slot frees up a moment after your client sees the
 response finish, so a request sent straight away can land as the 23rd\"." The Kimi flex 429 at 2026-11-11T21:4xZ ("max 22 in
 flight at once") was that race, so the two slots left free are what covers it.
 */
export const NEURALWATT_DEEPSEEK_FLEX: ProviderPool = {
  name: 'neuralwatt-flex',
  provider: 'neuralwatt',
  model: 'deepseek-v4.1-flash-flex',
  limit: 20,
  wallFactor: 2,
  takesDefaultRuns: true,
};

/**
 The `opencode-go` pool, which no new run takes until the owner says its week has reset.

 The owner, 2026-11-11T17:1x-05:00: "Be careful with opencode go as it has a weekly limit and I'm 99% through the weekly limit."
 So the limit is {@link CLOSED_POOL_LIMIT}, and it was 20. The owner ranked it below Neuralwatt's flex: "In practice,
 opencode-go may have worse inference quality and worse speed than neuralwatt's flex, so prefer neuralwatt."
 */
export const OPENCODE_GO: ProviderPool = {
  name: 'opencode-go',
  provider: 'opencode-go',
  model: 'deepseek-v4.1-flash',
  limit: CLOSED_POOL_LIMIT,
  wallFactor: 1,
  takesDefaultRuns: true,
};

/**
 Neuralwatt's Kimi flex pool, which only a run naming its model takes.

 The owner, 2026-11-10: "Neuralwatt rate limits are per model." and "Use 22 against Kimi K3.1 Pro flex from neuralwatt for now,
 until upstream clarifies or we hit something." The 7 that a 2026-11-11 Kimi flex 429 named ("max 7 in flight at once") was an
 assumption never measured, and the owner restored 22 the same night with "Why did you downgrade everything to 6 and 7?". The
 owner then set every flex pool with "Change flex concurrency to 22-2=20 per model", which is where 20 comes from: 20 of 28
 reviews on it ended at turn 16 to 37 while the table read 22. Only a measured refusal seen since sets any lower number.
 */
export const NEURALWATT_KIMI_FLEX: ProviderPool = {
  name: 'neuralwatt-kimi-flex',
  provider: 'neuralwatt',
  model: 'kimi-k3.1-pro-flex',
  limit: 20,
  wallFactor: 2,
  takesDefaultRuns: false,
};

/**
 The qwen plan's `qwen4-max-preview` pool, which only a run naming that model takes.

 The owner, 2026-11-11T16:3x-05:00: "qwen-token-plan-individual  qwen4-max-preview is now available in Pi. Check if it can
 replace Kimi or Sonnet." The plan's own concurrency limit for it is not measured or stated; 4 is the size of the comparison
 runs, not a limit of the plan, and a 429 from it sets the real one.
 */
export const QWEN_PLAN_QWEN4_MAX_PREVIEW: ProviderPool = {
  name: 'qwen-token-plan',
  provider: 'qwen-token-plan-individual',
  model: 'qwen4-max-preview',
  limit: 4,
  wallFactor: 1,
  takesDefaultRuns: false,
};

/**
 The pools a run is counted in, in the owner's order: the qwen plan's DeepSeek pool, then Neuralwatt's standard pool, its
 DeepSeek flex pool and `opencode-go`, then the two pools only a run naming their own model takes.

 This is also the order a run that names no model of its own takes them in, skipping each closed pool and each pool that takes
 only its own model, and the order the counts line prints them in.
 */
export const PROVIDER_POOLS: readonly ProviderPool[] = [
  QWEN_PLAN_DEEPSEEK,
  NEURALWATT_STANDARD,
  NEURALWATT_DEEPSEEK_FLEX,
  OPENCODE_GO,
  NEURALWATT_KIMI_FLEX,
  QWEN_PLAN_QWEN4_MAX_PREVIEW,
];

/**
 The provider a run gets when its command line names none, which is what the launcher the coordinator used wrote into every run
 it started itself.
 */
export const PROVIDER_OF_A_RUN_NAMING_NONE = 'neuralwatt';

/**
 The model a run gets when its command line names none. Standard Neuralwatt's model still reads as a default run although that
 pool is closed, so a run naming nothing lands on the first pool with room.
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
 Reads whether a pool takes a new run at all.

 @param pool - the pool to ask about

 @returns whether the pool's limit is {@link CLOSED_POOL_LIMIT}

 @example
 ```ts
 const closed = poolTakesNoNewRun({ pool: NEURALWATT_STANDARD, },);
 ```
 */
export function poolTakesNoNewRun({ pool }: { readonly pool: ProviderPool },): boolean {
  return pool.limit === CLOSED_POOL_LIMIT;
}

/**
 Reads the pool a run's model names.

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
