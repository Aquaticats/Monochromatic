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

  /**
   Why this pool takes no run now, when the owner has set it aside: a weekly or monthly quota nearly spent, a provider
   refusing. A closed pool keeps the room it has, so the reason and the number it returns to are both still readable, and a
   caller reopens it by naming its limit again (`PI_RUNS_POOL_LIMITS`).
   */
  readonly closed?: {
    readonly reason: string;
  };
};

/**
 Neuralwatt's standard DeepSeek pool.

 The owner, 2026-11-11: Change flex concurrency to 22-2=20 per model and normal concurrency to 20-2=18 per model. Neuralwatt
 said to keep 1 to 2 below the limit because "a slot frees up a moment after your client sees the response finish, so a request
 sent straight away can land as the 23rd". The 20 this held until then was the owner's earlier reading: "Try 20, and if it
 doesn't blow up (in terms of api rate limits), we're permanently raising the practical limit to 20." Historical 429s that
 lowered it for a few hours are void: the owner wrote on 2026-11-11T05:4x-05:00 "I've re-enabled auto-renew. Treat the prev
 Neuralwatt 429s void."
 */
export const NEURALWATT_STANDARD: ProviderPool = {
  name: 'neuralwatt',
  provider: 'neuralwatt',
  model: 'deepseek-v4.1-flash',
  limit: 18,
  wallFactor: 1,
  takesDefaultRuns: true,
};

/**
 Neuralwatt's DeepSeek flex pool.

 The owner, 2026-11-11: Change flex concurrency to 22-2=20 per model and normal concurrency to 20-2=18 per model. Neuralwatt
 said to keep 1 to 2 below the limit because "a slot frees up a moment after your client sees the response finish, so a request
 sent straight away can land as the 23rd". The 22 this held until then was the pool's own size: "You can use -flex postfix on
 neuralwatt models. They have their own separate pool of 22 slots. They're around 100% slower than standard in practice." A
 flex run therefore gets twice the wall clock, which is what `wallFactor` is for.
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
 The `opencode-go` pool, which the owner ranked below Neuralwatt's flex: "In practice, opencode-go may have worse inference
 quality and worse speed than neuralwatt's flex, so prefer neuralwatt." So it is taken only once both Neuralwatt pools above
 it are full.

 It is closed now, and its 20 is the room it returns to. The owner, 2026-11-11: "Be careful with opencode go as it has a weekly
 limit and I'm 99% through the weekly limit." A run a caller must not risk there is started by reopening it, which the README
 says how to do, rather than by changing this file.
 */
export const OPENCODE_GO: ProviderPool = {
  name: 'opencode-go',
  provider: 'opencode-go',
  model: 'deepseek-v4.1-flash',
  limit: 20,
  wallFactor: 1,
  takesDefaultRuns: true,
  closed: {
    reason: "the owner's weekly limit was 99% spent on 2026-11-11",
  },
};

/**
 Neuralwatt's Kimi flex pool, which only a run naming its model takes.

 The owner, 2026-11-11: Change flex concurrency to 22-2=20 per model and normal concurrency to 20-2=18 per model. Neuralwatt
 said to keep 1 to 2 below the limit because "a slot frees up a moment after your client sees the response finish, so a request
 sent straight away can land as the 23rd". The 22 this held until then was the pool's own size, as the owner read it on
 2026-11-10: "Neuralwatt rate limits are per model." and "Use 22 against Kimi K3.1 Pro flex from neuralwatt for now, until
 upstream clarifies or we hit something." The 7 that a 2026-11-11 Kimi flex 429 named ("max 7 in flight at once") was an
 assumption never measured, and the owner restored 22 the same night with "Why did you downgrade everything to 6 and 7?", so
 only a measured refusal or the owner's words set any other number.
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
 The variable that sets one or more pools' limits for one command, as `name=<n>` pairs separated by commas, so a pool whose
 provider has a weekly or monthly quota is closed and reopened without a code change.

 A limit of zero closes the pool, and any number above zero is the room it takes again, so `PI_RUNS_POOL_LIMITS=opencode-go=20`
 reopens the pool the owner set aside, and `PI_RUNS_POOL_LIMITS=neuralwatt-kimi-flex=0` closes another for the day.
 */
export const POOL_LIMITS_VARIABLE = 'PI_RUNS_POOL_LIMITS';

/**
 A limits setting this command cannot use, such as a pool it does not have or a limit that is not a whole number.
 */
export class PoolLimitsError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'PoolLimitsError';
}

/**
 Reads the limits a caller set, from the value of {@link POOL_LIMITS_VARIABLE}.

 Every pair is checked before anything is counted: a name no pool has, or a limit that is not a whole number, is refused rather
 than silently counting a pool the caller meant to close.

 @param listed - the variable's value, empty when the caller set nothing

 @returns the limit each named pool was given

 @throws PoolLimitsError when a pair is not `name=<whole number>` or names no pool

 @example
 ```ts
 const limits = limitsIn({ listed: 'opencode-go=20,neuralwatt-kimi-flex=0', },);
 ```
 */
export function limitsIn({ listed, }: { readonly listed: string },): ReadonlyMap<string, number> {
  /**
   The limits read so far, by pool name.
   */
  const limits = new Map<string, number>();
  for (const pair of listed.split(',',)) {
    if (pair.trim() === '') {
      continue;
    }
    /**
     Where the pool's name ends.
     */
    const split = pair.indexOf('=',);
    /**
     The pool's name, as the caller wrote it.
     */
    const name = (split === (-1)) ? '' : pair.slice(
      0,
      split,
    )
      .trim();
    /**
     The limit, as the caller wrote it.
     */
    const written = (split === (-1)) ? '' : pair.slice(split + 1,)
      .trim();
    if ((!PROVIDER_POOLS.some(function isNamed(pool,) {
      return pool.name === name;
    },)) || (!isWholeNumber(written,))) {
      throw new PoolLimitsError(`${POOL_LIMITS_VARIABLE} holds "${pair}", which is no <pool name>=<whole number> of this table: ${PROVIDER_POOLS.map(function nameOf(pool,) {
        return pool.name;
      },)
        .join(', ',)}`);
    }
    limits.set(
      name,
      Number(written,),
    );
  }
  return limits;
}

/**
 Reads whether text is a whole number.

 @param text - the text to read

 @returns whether it is one or more digits and nothing else

 @example
 ```ts
 const whole = isWholeNumber('20',);
 ```
 */
function isWholeNumber(text: string,): boolean {
  if (text === '') {
    return false;
  }
  for (const character of text) {
    if ((character < '0') || (character > '9')) {
      return false;
    }
  }
  return true;
}

/**
 Reads the pools with the limits a caller set: a pool named with a limit above zero is open at that limit, and one named with
 zero is closed, saying that a caller closed it.

 @param limits - the limit each named pool was given

 @param pools - the table to apply them to, which is the shipped one unless a test passes another

 @returns the pools, in the order given, each with its own limit

 @example
 ```ts
 const pools = poolsWithTheseLimits({ limits: new Map([['opencode-go', 0],]), },);
 ```
 */
export function poolsWithTheseLimits({
  limits,
  pools = PROVIDER_POOLS,
}: {
  readonly limits: ReadonlyMap<string, number>;
  readonly pools?: readonly ProviderPool[];
},): readonly ProviderPool[] {
  return pools.map(function atItsLimit(pool,) {
    /**
     The limit this caller gave this pool, when it gave one.
     */
    const limit = limits.get(pool.name,);
    if (limit === undefined) {
      return pool;
    }
    return {
      name: pool.name,
      provider: pool.provider,
      model: pool.model,
      limit,
      wallFactor: pool.wallFactor,
      takesDefaultRuns: pool.takesDefaultRuns,
      ...((limit > 0)
        ? {}
        : { closed: pool.closed ?? { reason: `${POOL_LIMITS_VARIABLE} set its limit to zero`, }, }),
    };
  },);
}

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
 What a run's pool membership is: the pool it holds a slot in, every pool it could be running on when nothing could be read
 about it, or the fact that no shared pool holds it.

 A run whose pool is unknown holds a slot in each pool it could be running on, never in one guessed pool: the scratch launcher
 answered `opencode-go` for every unit whose launcher had gone, which is how runs sat on standard Neuralwatt while its count read
 empty until 2026-11-11.
 */
export type PoolMembership =
  | {
    readonly holds: 'a-pool';
    readonly pool: ProviderPool;
  }
    | {
      readonly holds: 'some-pools';
      readonly pools: readonly ProviderPool[];
    }
    | { readonly holds: 'no-pool' };

/**
 Reads every pool a run of one name could be running on, for a run whose own command line could not be read.

 The name is the only evidence left, and it is the same evidence that decides whether a readable run is in a pool at all: an
 audit runs on another account, a Kimi run on a provider no pool of the DeepSeek kind holds, and any other run could name any
 model and so could be on any pool. Holding a slot in each of them holds the room that run may have taken; guessing one pool
 under-counted the others and is what this replaces.

 @param runName - the run's name

 @param pools - the pools to read, which are the shipped ones unless a caller set their limits

 @returns every pool such a run could hold, or nothing when its name rules every pool out

 @example
 ```ts
 const pools = poolsARunOfThatNameCouldHold({ runName: 'copy-route', },);
 ```
 */
export function poolsARunOfThatNameCouldHold({
  runName,
  pools = PROVIDER_POOLS,
}: {
  readonly runName: string;
  readonly pools?: readonly ProviderPool[];
},): readonly ProviderPool[] {
  if (runName.startsWith(AUDIT_RUN_PREFIX,)) {
    return [];
  }
  if (runName.startsWith(KIMI_RUN_PREFIX,)) {
    return pools.filter(function isTheKimiPool(pool,) {
      return pool.name === NEURALWATT_KIMI_FLEX.name;
    },);
  }
  return pools;
}

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
  return (found === undefined) ? { holds: 'no-pool', } : {
    holds: 'a-pool',
    pool: found,
  };
}

/**
 Reads which pool a run counts against.

 @param runName - the run's name, which says whether it is an audit or a Kimi run

 @param pool - the pool its command line names, when one was read

 @param pools - the pools to read, which are the shipped ones unless a caller set their limits

 @returns the pool it holds a slot in, every pool it could be running on when nothing names one, or that no shared pool holds
 it

 @example
 ```ts
 const membership = membershipOf({ runName: 'kimi-review-1', },);
 ```
 */
export function membershipOf({
  runName,
  pool,
  pools = PROVIDER_POOLS,
}: {
  readonly runName: string;
  readonly pool?: PoolMembership;
  readonly pools?: readonly ProviderPool[];
}): PoolMembership {
  if (runName.startsWith(AUDIT_RUN_PREFIX,)) {
    return { holds: 'no-pool', };
  }
  if (pool !== undefined) {
    return pool;
  }
  /**
   Every pool a run of this name could hold, which is where it holds a slot while nothing names one.
   */
  const couldHold = poolsARunOfThatNameCouldHold({
    runName,
    pools,
  },);
  return (couldHold.length === 0) ? { holds: 'no-pool', } : {
    holds: 'some-pools',
    pools: couldHold,
  };
}
