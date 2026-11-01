import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { runDeliveryCommandWithin, } from './runtime-failure-report.ts';

/**
 Module logger for the service steps of the sentinel launch.
 */
const l = tagged({ tag: 'bypass-watch-sentinel-service', },);

/**
 Milliseconds `systemd-run` and `systemctl` may take before they are ended.
 */
const SYSTEMD_COMMAND_LIMIT_MS = 10_000;

/**
 The first program of the service,
 which the system manager may execute where it may not execute the sentinel's program itself.
 On an SELinux host the system manager (`init_t`) may not execute files under a home directory,
 and the toolchain `node` sits in one,
 but it may execute `/usr/bin/env`,
 after which the service runs in `unconfined_service_t`, which may.
 See `doc/troubleshooting/selinux-denies-system-service-exec-of-home-files.md`.
 */
export const SERVICE_TRAMPOLINE = '/usr/bin/env';

/**
 Tells whether `/usr/bin/env` would read a program path as a variable assignment.
 `env` does that for any first argument that contains `=`,
 and it has no option that prevents it,
 so such a path cannot be started behind `env`.

 @param path - The program path.

 @returns Whether the path contains an equals sign.

 @example
 ```ts
 pathBreaksEnv({ path: '/opt/a=b/node' }); // true
 ```
 */
export function pathBreaksEnv({ path, }: { readonly path: string; },): boolean {
  return path.includes('=',);
}

/**
 Tells whether the active state `systemctl` printed for a unit means no process of it runs:
 a unit that ended,
 failed,
 or was collected and is no longer loaded (which prints `inactive`).
 Any other state,
 and any text that is not a state,
 means it may still run.

 @param printed - What `systemctl show --property=ActiveState --value` printed.

 @returns Whether the unit is gone.

 @example
 ```ts
 unitStateMeansGone({ printed: 'inactive\n' }); // true
 ```
 */
export function unitStateMeansGone({ printed, }: { readonly printed: string; },): boolean {
  /**
   The state without surrounding whitespace.
   */
  const state = printed.trim();
  return (state === 'inactive') || (state === 'failed');
}

/**
 Starts the sentinel as a transient service of the system manager.

 @param args - Arguments after `systemd-run`.

 @throws When `systemd-run` is missing or the manager refuses the unit.

 @example
 ```ts
 await startServiceSentinel({ args: ['--unit=x', '--', '/usr/bin/true'] });
 ```
 */
export async function startServiceSentinel({ args, }: { readonly args: readonly string[]; },): Promise<void> {
  await runDeliveryCommandWithin({
    call: {
      command: 'systemd-run',
      args,
    },
    timeoutMs: SYSTEMD_COMMAND_LIMIT_MS,
  },);
}

/**
 Runs `systemctl stop` for a unit and says what happened,
 without throwing:
 a unit that already ended and was collected is no longer loaded,
 which `stop` reports as a failure although nothing runs.

 @param unit - Name of the unit.

 @returns What `systemctl stop` did,
 for the reason when the unit cannot be shown to be gone.

 @example
 ```ts
 await stopUnit({ unit: 'wg-quicker-sentinel-4242-9001' });
 ```
 */
async function stopUnit({ unit, }: { readonly unit: string; },): Promise<string> {
  try {
    await runDeliveryCommandWithin({
      call: {
        command: 'systemctl',
        args: [
          'stop',
          unit,
        ],
      },
      timeoutMs: SYSTEMD_COMMAND_LIMIT_MS,
    },);
    return 'systemctl stop succeeded';
  }
  catch (error) {
    l.debug(`stopping ${unit} did not succeed (it may already be gone): ${String(error,)}`,);
    return String(error,);
  }
}

/**
 Reads the active state of a unit.

 @param unit - Name of the unit.

 @param stopSaid - What `systemctl stop` did,
 for the reason when the state cannot be read.

 @returns What `systemctl show --property=ActiveState --value` printed.

 @throws When `systemctl` cannot start, fails or runs past the limit.

 @example
 ```ts
 await readUnitState({ unit: 'wg-quicker-sentinel-4242-9001', stopSaid: 'systemctl stop succeeded' });
 ```
 */
async function readUnitState(
  {
    unit,
    stopSaid,
  }: {
    readonly unit: string;
    readonly stopSaid: string;
  },
): Promise<string> {
  try {
    return await runDeliveryCommandWithin({
      call: {
        command: 'systemctl',
        args: [
          'show',
          '--property=ActiveState',
          '--value',
          unit,
        ],
      },
      timeoutMs: SYSTEMD_COMMAND_LIMIT_MS,
    },);
  }
  catch (error) {
    throw new Error(
      `its state could not be read: ${String(error,)} (${stopSaid})`,
      { cause: error, },
    );
  }
}

/**
 Stops the transient service of a sentinel and checks that no process of it is left.
 The check runs whatever `systemctl stop` said.

 @param unit - Name of the unit.

 @throws When the unit cannot be shown to be gone:
 its state cannot be read, or it is still active, activating or deactivating.

 @example
 ```ts
 await stopServiceSentinel({ unit: 'wg-quicker-sentinel-4242-9001' });
 ```
 */
export async function stopServiceSentinel({ unit, }: { readonly unit: string; },): Promise<void> {
  /**
   What `systemctl stop` did.
   */
  const stopSaid = await stopUnit({ unit, },);
  /**
   The state of the unit now.
   */
  const printed = await readUnitState({
    unit,
    stopSaid,
  },);
  if (!unitStateMeansGone({ printed, },))
    throw new Error(`the unit is ${printed.trim()} (${stopSaid})`,);
}
