/**
 Runs the daily update's package checks: Rust crates and TypeScript packages in two pools side by side, each pool
 bounded, every check bounded in time, and a deadline for the whole pass.

 Rationale: `doc/planning/toolchain-always-latest.md` ("Checking every package after the update").

 @module
 */

import {
  RUST_CHECK_PARALLEL,
  TYPESCRIPT_CHECK_PARALLEL,
} from './daily-update-check-bounds.ts';
import {
  NO_ERROR_IN_OUTPUT,
} from './daily-update-first-error.ts';
import { oneAfterAnother, } from './daily-update-one-after-another.ts';
import type {
  CheckEffects,
  CheckKind,
  CheckResult,
  PackageCheck,
} from './daily-update-package-check-types.ts';

/**
 How many checks of each kind run at once.
 */
export const CHECK_PARALLEL: Readonly<Record<CheckKind, number>> = {
  rust: RUST_CHECK_PARALLEL,
  typescript: TYPESCRIPT_CHECK_PARALLEL,
};

/**
 Kinds of package, each with a pool of its own.
 */
export const CHECK_KINDS: readonly CheckKind[] = [
  'rust',
  'typescript',
];

/**
 Runs one check, or skips it when the pass is out of time.

 @param effects - the outside world

 @param check - the check

 @param deadlineAt - clock time after which a check that has not started is skipped

 @param log - writes one journal line

 @returns the result; a check that cannot start is `not-run` with the reason logged

 @example
 ```ts
 const result = await runOneCheck({ effects, check, deadlineAt, log, },);
 ```
 */
export async function runOneCheck({
  effects,
  check,
  deadlineAt,
  log,
}: {
  readonly effects: CheckEffects;
  readonly check: PackageCheck;
  readonly deadlineAt: number;
  readonly log: (line: string,) => void;
},): Promise<CheckResult> {
  if (effects.now() >= deadlineAt) {
    log(`check skipped, the pass ran out of time: ${check.package} ${check.task}`,);
    return {
      check,
      status: 'not-run',
      seconds: 0,
    };
  }
  try {
    /**
     How the command ended.
     */
    const outcome = await effects.runCheck(check,);
    /**
     Status the outcome stands for.
     */
    const status = outcome.timedOut ? 'timed-out' : outcome.exitCode === 0 ? 'passed' : 'failed';
    /**
     First error of a failed check, or the sentinel when it printed none.
     */
    const found = status === 'failed' ? outcome.firstError ?? NO_ERROR_IN_OUTPUT : NO_ERROR_IN_OUTPUT;
    log(`check ${status} in ${outcome.seconds} s: ${check.package} ${check.task}${
      found === NO_ERROR_IN_OUTPUT ? '' : ` (${found.code === '' ? found.message : found.code})`
    }`,);
    return {
      check,
      status,
      seconds: outcome.seconds,
      ...found === NO_ERROR_IN_OUTPUT ? {} : { firstError: found, },
    };
  }
  catch (error) {
    log(`check could not run: ${check.package} ${check.task}: ${String(error,)}`,);
    return {
      check,
      status: 'not-run',
      seconds: 0,
    };
  }
}

/**
 Runs checks with a bounded pool of workers, each taking the next check when its last one ends.

 @param effects - the outside world

 @param checks - checks of one kind, in order

 @param workers - how many run at once

 @param deadlineAt - clock time after which a check that has not started is skipped

 @param log - writes one journal line

 @returns results by position in `checks`

 @example
 ```ts
 const results = await runPool({ effects, checks, workers: 2, deadlineAt, log, },);
 ```
 */
export async function runPool({
  effects,
  checks,
  workers,
  deadlineAt,
  log,
}: {
  readonly effects: CheckEffects;
  readonly checks: readonly PackageCheck[];
  readonly workers: number;
  readonly deadlineAt: number;
  readonly log: (line: string,) => void;
},): Promise<readonly CheckResult[]> {
  /**
   Results by position, filled as checks end.
   */
  const results: CheckResult[] = [];
  /**
   One cursor all workers take their next position from.
   */
  const positions = checks.entries();
  // Every worker reads from the same cursor, so each position is taken once and a worker starts a check only when its last ended.
  await Promise.all(Array.from(
    { length: Math.min(
      workers,
      checks.length,
    ), },
    async function worker() {
      for await (const done of oneAfterAnother({
        items: positions,
        run: async function runAt([position, check,],) {
          return {
            position,
            result: await runOneCheck({
              effects,
              check,
              deadlineAt,
              log,
            },),
          };
        },
      },)) {
        results[done.position] = done.result;
      }
    },
  ),);
  return results;
}

/**
 Runs every check in two pools, one per kind, side by side.

 @param effects - the outside world

 @param checks - the checks

 @param deadlineAt - clock time after which a check that has not started is skipped

 @param log - writes one journal line

 @returns one result per check, in the order of `checks`

 @example
 ```ts
 const results = await runChecks({ effects, checks, deadlineAt: effects.now() + CHECK_PASS_DEADLINE_MS, log, },);
 ```
 */
export async function runChecks({
  effects,
  checks,
  deadlineAt,
  log,
}: {
  readonly effects: CheckEffects;
  readonly checks: readonly PackageCheck[];
  readonly deadlineAt: number;
  readonly log: (line: string,) => void;
},): Promise<readonly CheckResult[]> {
  // The kinds have their own pools and do not depend on each other, so they run side by side.
  /**
   Results per kind.
   */
  const perKind = await Promise.all(CHECK_KINDS.map(function runKind(kind,) {
    return runPool({
      effects,
      checks: checks.filter(function isKind(check,) {
        return check.kind === kind;
      },),
      workers: CHECK_PARALLEL[kind],
      deadlineAt,
      log,
    },);
  },),);
  /**
   Results by key, so the original order can be restored.
   */
  const byKey = new Map(perKind.flat()
    .map(function keyed(result,) {
      return [
        `${result.check
          .package}:${result.check
            .task}`,
        result,
      ] as const;
    },),);
  return checks.flatMap(function inOrder(check,) {
    /**
     The check's result, always present because every check was given to a pool.
     */
    const result = byKey.get(`${check.package}:${check.task}`,);
    return result === undefined ? [] : [result,];
  },);
}
