import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  watcherDiedReport,
  watcherEndUnknownReport,
} from './bypass-watch-failure.ts';
import { BypassRouteError, } from './errors.ts';
import {
  PROCESS_ABSENT,
  readLinuxProcessIdentity,
} from './linux-process-identity.ts';
import type { FailureDelivery, } from './runtime-failure-report.ts';
import { systemIsStopping, } from './system-stop.ts';
import {
  readWatcherRecord,
  type WatcherProcessIdentity,
} from './tunnel-bypass-watcher-sidecar.ts';
import {
  KIND_ROUTE_WATCHER_DIED,
} from './update-policy-notice.ts';

/**
 Milliseconds the sentinel waits after the watcher is gone before it reads the identity record,
 so a stop that kills the watcher and then removes its record is not mistaken for a death.
 */
export const SENTINEL_GRACE_MS = 1_500;

/**
 Milliseconds between two checks that the watcher still runs.
 */
export const SENTINEL_POLL_MS = 500;

/**
 Number of arguments the sentinel takes:
 state path,
 owner id,
 pid,
 and start time of the watcher.
 */
const SENTINEL_ARGUMENT_COUNT = 4;

/**
 What the identity record says about the end of the watcher.
 */
export type SentinelVerdict =
  | { readonly kind: 'expected'; }
    | { readonly kind: 'unexpected'; }
    | {
      readonly kind: 'unknown';
      readonly reason: string
    };

/**
 What the sentinel concluded:
 the verdict on the record,
 or that the whole system was stopping,
 which ends the watcher and the tunnel with everything else and is not reported.
 */
export type SentinelOutcome =
  | SentinelVerdict
    | { readonly kind: 'system stopping'; };

/**
 Facts the sentinel was started with.
 */
export type SentinelTarget = {
  /**
   Persisted bypass state path that keys the identity record.
   */
  readonly statePath: string;

  /**
   Identity of the watcher to wait for.
   */
  readonly identity: WatcherProcessIdentity;
};

/**
 Reads the sentinel's arguments.

 @param argv - Arguments after the script path.

 @returns The target.

 @throws {@link BypassRouteError} when an argument is missing or not a positive pid.

 @example
 ```ts
 parseSentinelArguments({ argv: ['/run/wg-quicker/wg0.json', 'owner', '123', '456'] });
 ```
 */
export function parseSentinelArguments(
  { argv, }: { readonly argv: readonly string[]; },
): SentinelTarget {
  if (argv.length !== SENTINEL_ARGUMENT_COUNT)
    throw new BypassRouteError('Usage: bypass-watch-sentinel <state-path> <owner-id> <pid> <start-time>',);
  /**
   Arguments by position.
   */
  const [
    statePath,
    ownerId,
    pidText,
    startTime,
  ] = argv;
  /**
   Parsed process identifier.
   */
  const pid = Number(pidText,);
  if ((statePath === undefined) || (ownerId === undefined)
    || (startTime === undefined)
    || (!Number.isSafeInteger(pid,))
    || (pid <= 0))
    throw new BypassRouteError(`Invalid sentinel arguments: ${argv.join(' ',)}`,);
  return {
    statePath,
    identity: {
      ownerId,
      pid,
      startTime,
    },
  };
}

/**
 Reports whether the watcher process still runs.
 A process with the same pid and another start time is a different process.

 @param identity - Watcher identity.

 @param read - Reader of the process identity,
 replaceable in tests.

 @returns Whether the same process is alive.

 @example
 ```ts
 await watcherIsAlive({ identity });
 ```
 */
export async function watcherIsAlive(
  {
    identity,
    read = readLinuxProcessIdentity,
  }: {
    readonly identity: WatcherProcessIdentity;
    readonly read?: typeof readLinuxProcessIdentity;
  },
): Promise<boolean> {
  /**
   Procfs identity of the pid now.
   */
  const live = await read({ pid: identity.pid, },);
  return (live !== PROCESS_ABSENT) && (live.startTime === identity.startTime);
}

/**
 Judges the end of the watcher from the identity record it left.

 A record removed by the watcher's own clean exit or by `wg-quicker down` means the end was asked for,
 and so does a record that names another watcher.
 A record that still names this watcher means it was killed,
 ran out of memory,
 or crashed.

 @param statePath - Persisted bypass state path.

 @param identity - Identity of the watcher that ended.

 @param read - Reader of the record,
 replaceable in tests.

 @returns The verdict.

 @example
 ```ts
 await judgeWatcherEnd({ statePath, identity });
 ```
 */
export async function judgeWatcherEnd(
  {
    statePath,
    identity,
    read = readWatcherRecord,
  }: {
    readonly statePath: string;
    readonly identity: WatcherProcessIdentity;
    readonly read?: typeof readWatcherRecord;
  },
): Promise<SentinelVerdict> {
  try {
    /**
     Record now on disk.
     */
    const record = await read({ statePath, },);
    if (record.kind === 'identity') {
      /**
       Identity the record holds,
       which is this watcher's when all three parts match.
       */
      const { identity: recordedIdentity, } = record;
      if ((recordedIdentity.ownerId === identity.ownerId)
        && (recordedIdentity.pid === identity.pid)
        && (recordedIdentity.startTime === identity.startTime))
        return { kind: 'unexpected', };
    }
    // A record that holds no identity is not this watcher's record either,
    // and the exclusion a stop wrote is the identity.
    return { kind: 'expected', };
  }
  catch (error) {
    return {
      kind: 'unknown',
      reason: String(error,),
    };
  }
}

/**
 Waits for the watcher to end,
 judges the end,
 and delivers a report when nobody asked for it.

 @param target - Statepath and identity of the watcher.

 @param deliver - Delivery of failures to the journal and the desktop.

 @param isAlive - Liveness check,
 replaceable in tests.

 @param pause - Wait between checks,
 replaceable in tests.

 @param judge - Judge of the end,
 replaceable in tests.

 @param isSystemStopping - Tells whether the whole system is shutting down,
 replaceable in tests.

 @param graceMs - Milliseconds between the end and the judgement.

 @param pollMs - Milliseconds between liveness checks.

 @returns The outcome:
 the verdict,
 or that the system was stopping and nothing was reported.

 @example
 ```ts
 await runWatcherSentinel({ target, deliver });
 ```
 */
export async function runWatcherSentinel(
  {
    target,
    deliver,
    isAlive = watcherIsAlive,
    pause = wait,
    judge = judgeWatcherEnd,
    isSystemStopping = systemIsStopping,
    graceMs = SENTINEL_GRACE_MS,
    pollMs = SENTINEL_POLL_MS,
  }: {
    readonly target: SentinelTarget;
    readonly deliver: FailureDelivery;
    readonly isAlive?: typeof watcherIsAlive;
    readonly pause?: (milliseconds: number,) => Promise<unknown>;
    readonly judge?: typeof judgeWatcherEnd;
    readonly isSystemStopping?: typeof systemIsStopping;
    readonly graceMs?: number;
    readonly pollMs?: number;
  },
): Promise<SentinelOutcome> {
  // oxlint-disable-next-line eslint/no-await-in-loop -- Each liveness check must finish before the next pause.
  while (await isAlive({ identity: target.identity, },)) {
    // oxlint-disable-next-line eslint/no-await-in-loop -- Bounded poll interval avoids a busy spin.
    await pause(pollMs,);
  }
  await pause(graceMs,);
  /**
   What the record says about the end.
   */
  const verdict = await judge({
    statePath: target.statePath,
    identity: target.identity,
  },);
  if ((verdict.kind !== 'expected') && await isSystemStopping())
    return { kind: 'system stopping', };
  if (verdict.kind === 'unexpected')
    await deliver({
      kind: KIND_ROUTE_WATCHER_DIED,
      line: watcherDiedReport({ pid: target.identity
        .pid, },),
    },);
  if (verdict.kind === 'unknown')
    await deliver({
      kind: KIND_ROUTE_WATCHER_DIED,
      line: watcherEndUnknownReport({
        pid: target.identity
          .pid,
        reason: verdict.reason,
      },),
    },);
  return verdict;
}
