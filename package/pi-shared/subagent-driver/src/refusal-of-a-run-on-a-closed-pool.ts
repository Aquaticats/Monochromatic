/**
 The refusal of a run whose own provider and model name a pool that takes no new run, which both commands that start a run answer
 before they start anything.

 A pool is closed when the owner set it aside, or when a caller closed it for one command with `PI_RUNS_POOL_LIMITS`, and a closed
 pool takes no new run at all. The refusal reads the table with the limits that variable sets, so a pool reopened through it takes
 runs again. A provider and model that belong to no pool are not refused: a stand-in and an audit's account are outside every pool.

 @module
 */

import {
  CLOSED_POOL_LIMIT,
  POOL_LIMITS_VARIABLE,
  type ProviderPool,
  limitsIn,
  poolNamed,
  poolOfThatNameIn,
  poolTakesNoNewRun,
  poolsWithTheseLimits,
} from './pool-limits.ts';

/**
 The status both commands exit with when the pool a run names takes no new run, which is a status of its own so that a caller
 reads a refused pool apart from a run that started and failed.

 The pool gate answers `3` for no room and `4` for a run already going, and every other failure ends at `1`.
 */
export const CLOSED_POOL_EXIT_STATUS = 5;

/**
 A run whose provider and model name a pool that takes no new run, refused before anything is made.
 */
export class ClosedPoolRefusalError extends Error {
  /**
   The name printed with the message.
   */
  override readonly name = 'ClosedPoolRefusalError';
}

/**
 Says what a run on a closed pool is refused with: the pool it named, the reason that pool takes no new run, the pools a run that
 names no model of its own may take instead, in the table's order, and the variable that gives a pool room again.

 @param provider - the provider the run names

 @param model - the model the run names

 @param pool - the pool they share, as the table with the caller's limits holds it

 @param pools - the table with the caller's limits, which is where the pools to name instead come from

 @param namesItsOwnPool - whether the command line named the provider and model itself, or a command's own default was read in their
 place

 @returns the refusal, one line per thing a reader needs

 @example
 ```ts
 const text = refusalTextOf({ provider: 'neuralwatt', model: 'deepseek-v4.1-flash', pool: NEURALWATT_STANDARD, pools: PROVIDER_POOLS, namesItsOwnPool: true, },);
 ```
 */
function refusalTextOf({
  provider,
  model,
  pool,
  pools,
  namesItsOwnPool,
}: {
  readonly provider: string;
  readonly model: string;
  readonly pool: ProviderPool;
  readonly pools: readonly ProviderPool[];
  readonly namesItsOwnPool: boolean;
},): string {
  /**
   The owner's closing of the pool, when it carries one.
   */
  const { closed, } = pool;
  /**
   Why the pool takes no new run: the reason its `closed` field gives, or its limit when nothing else was said.
   */
  const reason = (closed === undefined)
    ? `its limit is ${String(CLOSED_POOL_LIMIT,)}`
    : closed.reason;
  /**
   What a run that names this pool is refused with, one line per thing the reader needs.
   */
  const lines: string[] = [
    `--provider=${provider} --model=${model} names the ${pool.name} pool, which takes no new run: ${reason}.`,
  ];
  if (!namesItsOwnPool) {
    lines.push(
      'This command line named no provider, so the run would have taken this command\'s default provider and model, which name that pool. Start a run that names none through start-pi-run-when-a-pool-has-room, which counts the runs going and gives the run a pool with room.',
    );
  }
  lines.push(
    `A pool takes new runs again when ${POOL_LIMITS_VARIABLE} gives it a limit above zero, as ${POOL_LIMITS_VARIABLE}=${pool.name}=<n> does for one command.`,
  );
  /**
   The open pools a run that names no model of its own may take, in the table's order.
   */
  const openDefaults = pools.filter(function takesDefaultRunsAndIsOpen(candidate,) {
    return candidate.takesDefaultRuns && (!poolTakesNoNewRun({ pool: candidate, },));
  },);
  if (openDefaults.length === 0) {
    lines.push('No pool takes a run that names no model of its own now.',);
  }
  else {
    lines.push('The open pools a run that names no model of its own may take, in the table\'s order:',);
    for (const open of openDefaults) {
      lines.push(`  --provider=${open.provider} --model=${open.model} (${open.name})`,);
    }
  }
  return lines.join('\n',);
}

/**
 Refuses a run whose own provider and model name a pool that takes no new run.

 Nothing is made and nothing is written before this answers: a run refused here leaves no worktree, no run folder and no slot spent
 on a pool the owner closed. The pool is read from the table with the limits `PI_RUNS_POOL_LIMITS` sets applied, so a pool
 reopened that way takes runs again, and a provider and model that belong to no pool are left to run.

 The setting is read only for a run that names a pool at all, and then every pair of it is read or none is: a run outside every
 pool is refused by nothing a broken setting says, and a run inside one is refused rather than counted against a table the caller
 may have meant to change. A caller who meant to close one pool with a pair this command cannot read has said nothing it can act
 on.

 @param provider - the provider the run names

 @param model - the model the run names

 @param environment - the environment the command runs with, whose `PI_RUNS_POOL_LIMITS` sets the pools' limits

 @param namesItsOwnPool - whether the command line named the provider and model itself, or a command's own default was read in their
 place

 @throws ClosedPoolRefusalError when the provider and model name a pool that takes no new run

 @throws PoolLimitsError when the run names a pool and `PI_RUNS_POOL_LIMITS` names no pool or holds anything but a whole number,
 which the caller answers by naming the setting and its fault and starting nothing

 @example
 ```ts
 refuseARunOnAClosedPool({ provider: 'neuralwatt', model: 'deepseek-v4.1-flash', environment: process.env, },);
 ```
 */
export function refuseARunOnAClosedPool({
  provider,
  model,
  environment,
  namesItsOwnPool = true,
}: {
  readonly provider: string;
  readonly model: string;
  readonly environment: Readonly<NodeJS.ProcessEnv>;
  readonly namesItsOwnPool?: boolean;
},): void {
  /**
   The pool this run's provider and model share, or that no pool holds them.
   */
  const membership = poolNamed({
    provider,
    model,
  },);
  // A run outside every pool reads no limit at all: the setting can neither close the pool it names nor reopen one, so a value
  // this command cannot read refuses nothing here.
  if (membership.holds !== 'a-pool') {
    return;
  }
  /**
   The pools with the limits this command's caller set, which is where a pool reopened through the variable takes runs again.
   */
  const pools = poolsWithTheseLimits({
    limits: limitsIn({ listed: environment[POOL_LIMITS_VARIABLE] ?? '', },),
  },);
  /**
   The pool the run names, as the table with the caller's limits holds it.
   */
  const pool = poolOfThatNameIn({
    pool: membership.pool,
    pools,
  },);
  if (!poolTakesNoNewRun({ pool, },)) {
    return;
  }
  throw new ClosedPoolRefusalError(refusalTextOf({
    provider,
    model,
    pool,
    pools,
    namesItsOwnPool,
  },),);
}
