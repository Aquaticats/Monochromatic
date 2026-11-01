import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { runDeliveryCommandWithin, } from './runtime-failure-report.ts';

/**
 Module logger for the stopping-system check.
 */
const l = tagged({ tag: 'system-stop', },);

/**
 Milliseconds `systemctl` may take before it is ended.
 */
const PROBE_LIMIT_MS = 5_000;

/**
 Arguments of `systemctl` that print the state of `shutdown.target` alone and exit with zero for any state.
 */
const SHOW_ARGUMENTS = [
  'show',
  '--property=ActiveState',
  '--value',
  'shutdown.target',
] as const;

/**
 Whether the state `systemctl` printed for `shutdown.target` means a shutdown is under way.

 @param printed - What `systemctl show --value` printed.

 @returns Whether the target is active or activating.

 @example
 ```ts
 shutdownTargetMeansStopping({ printed: 'active\n' }); // true
 ```
 */
export function shutdownTargetMeansStopping({ printed, }: { readonly printed: string; },): boolean {
  /**
   The state without surrounding whitespace.
   */
  const state = printed.trim();
  return (state === 'active') || (state === 'activating');
}

/**
 Reads the state of `shutdown.target` through `systemctl` with a cleared environment and a time limit.

 @returns What `systemctl` printed.

 @throws When `systemctl` cannot start, fails or runs past the limit.

 @example
 ```ts
 await readShutdownTargetState();
 ```
 */
export function readShutdownTargetState(): Promise<string> {
  return runDeliveryCommandWithin({
    call: {
      command: 'systemctl',
      args: SHOW_ARGUMENTS,
    },
    timeoutMs: PROBE_LIMIT_MS,
  },);
}

/**
 Tells whether the whole system is shutting down or rebooting.
 The route-watcher sentinel runs as a service of the system manager and outlives the user session of the watcher.
 At a shutdown or a reboot the watcher is ended with everything else:
 the state in `/run` and the tunnel go with it,
 so nothing is left unenforced and the death is not a policy failure.
 A logout while the system keeps running is different:
 the tunnel stays up with no watcher,
 so the update traffic of later sessions is no longer exempted,
 and that death is reported.
 The system counts as stopping only when `shutdown.target` is active or becoming active.
 `/run/nologin` is deliberately not a signal:
 `man 8 shutdown` documents that with a time argument it is created five minutes before the system goes down,
 when a real death should still be reported,
 and a stale or administrator-made file would hide every death.
 A probe that cannot tell is treated as a system that keeps running,
 so a failing probe never hides a death.

 @param readState - Reader of the state of `shutdown.target`,
 replaceable in tests.

 @returns Whether the system is stopping.
 A probe that cannot tell gives false.

 @example
 ```ts
 await systemIsStopping();
 ```
 */
export async function systemIsStopping(
  { readState = readShutdownTargetState, }: { readonly readState?: typeof readShutdownTargetState; } = {},
): Promise<boolean> {
  try {
    return shutdownTargetMeansStopping({ printed: await readState(), },);
  }
  catch (error) {
    l.error(`systemctl could not tell whether the system is stopping (${String(error,)}), so it is judged to keep running`,);
    return false;
  }
}
