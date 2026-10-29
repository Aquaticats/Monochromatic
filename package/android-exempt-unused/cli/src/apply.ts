/**
 Apply a computed {@link ./changes.ts Changes} set to the device, one app at a
 time, collecting per-app failures instead of aborting the whole run.
 
 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';

import type { Changes, } from './changes.ts';
import {
  type AppOpsMode,
  MODE_DEFAULT,
  MODE_IGNORE,
} from './constants.ts';
import { AdbError, } from './errors.ts';
import { setAutoRevoke, } from './packages.ts';

/**
 Module-level tagged logger; each function wraps it with its own name.
 */
const l = tagged({ tag: 'apply', },);

/**
 One app whose appops write failed, kept so the summary can list it.
 */
export type ChangeFailure = {
  readonly packageName: string;
  readonly mode: AppOpsMode;
  readonly message: string;
};

/**
 One pending appops write: an app plus the mode to set it to. Named so the map
 callbacks below get a contextual {@link AppOpsMode} and `mode` is not widened
 to `string`.
 */
type Operation = {
  readonly packageName: string;
  readonly mode: AppOpsMode;
};

/**
 Apply every change in `changes` sequentially via {@link setAutoRevoke}.
 adb serializes over a single transport, so concurrency buys nothing and a
 steady order makes progress legible. A failed app raising an
 {@link AdbError} is recorded and the run continues.
 
 @param serial - Device to mutate.
 
 @param changes - Exempt and revert partitions to write.
 
 @param onProgress - Invoked before each write with completed count, total,
                     and the app about to change; drives a spinner message.
 
 @returns One ChangeFailure per app whose write failed; empty on full success.
 
 @example
 ```ts
 const failures = await applyChanges({ serial, changes, },);
 if (failures.length === 0) console.log('all applied',);
 ```
 */
export async function applyChanges({
  serial,
  changes,
  onProgress,
}: {
  readonly serial: string;
  readonly changes: Changes;
  readonly onProgress?: (progress: {
    readonly done: number;
    readonly total: number;
    readonly packageName: string;
  },) => void;
},): Promise<readonly ChangeFailure[]> {
  /**
   Tagged logger for this run.
   */
  const fl = tagged({
    tag: applyChanges.name,
    l,
  },);
  /**
   Flattened write list: exempts first, then reverts, each tagged with its mode.
   */
  const operations: readonly Operation[] = [
    ...changes.toExempt
      .map(function toExemptOp(packageName,): Operation {
        return {
          packageName,
          mode: MODE_IGNORE,
        };
      },),
    ...changes.toRevert
      .map(function toRevertOp(packageName,): Operation {
        return {
          packageName,
          mode: MODE_DEFAULT,
        };
      },),
  ];
  /**
   Apps whose write failed; returned to the caller for reporting.
   */
  const failures: ChangeFailure[] = [];
  for (const [index, operation,] of operations.entries()) {
    onProgress?.({
      done: index,
      total: operations.length,
      packageName: operation.packageName,
    },);
    fl.info(`appops set ${operation.packageName} -> ${operation.mode}`,);
    try {
      // oxlint-disable-next-line eslint/no-await-in-loop -- writes are intentionally sequential: adb serializes over one transport, so parallel sets would contend without speedup and would scramble progress output.
      await setAutoRevoke({
        serial,
        packageName: operation.packageName,
        mode: operation.mode,
      },);
    } catch (error) {
      if (error instanceof AdbError) {
        fl.error(`failed to set ${operation.packageName}: ${error.message}`,);
        failures.push({
          packageName: operation.packageName,
          mode: operation.mode,
          message: error.message,
        },);
      } else {
        throw error;
      }
    }
  }
  return failures;
}
