/**
 Package-level adb operations: list third-party apps, read which are currently
 exempt from auto-revoke, and write a single app's appops mode.
 
 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';

import { runAdb, } from './adb.ts';
import {
  type AppOpsMode,
  AUTO_REVOKE_OP,
  MODE_IGNORE,
} from './constants.ts';
import { AdbCommandError, } from './errors.ts';
import {
  parseExemptedQuery,
  parsePackageList,
} from './parse.ts';

/**
 Module-level tagged logger; each function wraps it with its own name.
 */
const l = tagged({ tag: 'packages', },);

/**
 Number of concurrent `appops get` calls in the fallback path. adb multiplexes
 over one transport, so a modest cap keeps the device responsive without
 issuing hundreds of simultaneous shells.
 */
const GET_CONCURRENCY = 12;

/**
 One app's exempt verdict from the per-app fallback path.
 */
type Verdict = {
  readonly packageName: string;
  readonly exempted: boolean;
};

/**
 List third-party (user-installed) application ids on the device, by
 running `pm list packages -3` via {@link runAdb} and parsing the output
 with {@link parsePackageList}.
 
 @param serial - Device to query.
 
 @returns Validated third-party application ids.
 
 @example
 ```ts
 const apps = await listThirdPartyPackages({ serial: 'ABC123', },);
 ```
 */
export async function listThirdPartyPackages({ serial, }: { readonly serial: string; },): Promise<readonly string[]> {
  /**
   Captured stdout from `pm list packages -3`.
   */
  const stdout = await runAdb({
    serial,
    args: [
      'shell',
      'pm',
      'list',
      'packages',
      '-3',
    ],
  },);
  return parsePackageList({ stdout, },);
}

/**
 Set one app's {@link ./constants.ts AUTO_REVOKE_OP} appops mode by
 invoking `cmd appops set` via {@link runAdb}.
 
 @param serial - Device to mutate.
 
 @param packageName - Application id to change; must already be validated.
 
 @param mode - Target mode: exempt or revert-to-default.
 
 @example
 ```ts
 await setAutoRevoke({ serial, packageName: 'com.example.app', mode: 'ignore', },);
 ```
 */
export async function setAutoRevoke({
  serial,
  packageName,
  mode,
}: {
  readonly serial: string;
  readonly packageName: string;
  readonly mode: AppOpsMode;
},): Promise<void> {
  await runAdb({
    serial,
    args: [
      'shell',
      'cmd',
      'appops',
      'set',
      packageName,
      AUTO_REVOKE_OP,
      mode,
    ],
  },);
}

/**
 Read whether one app is exempt by running `cmd appops get` via
 {@link runAdb} and parsing the output for the op. Used only by the
 fallback path when bulk `query-op` is unavailable; a failing invocation
 surfaces as an {@link AdbCommandError}, treated here as not exempted.
 
 @param serial - Device to query.
 
 @param packageName - Application id to inspect.
 
 @returns `true` when the op reads back as {@link ./constants.ts MODE_IGNORE}.
 */
async function isExemptedViaGet({
  serial,
  packageName,
}: {
  readonly serial: string;
  readonly packageName: string;
},): Promise<boolean> {
  /**
   Tagged logger for this call.
   */
  const fl = tagged({
    tag: isExemptedViaGet.name,
    l,
  },);
  try {
    /**
     Captured stdout from `cmd appops get <pkg> <op>`.
     */
    const stdout = await runAdb({
      serial,
      args: [
        'shell',
        'cmd',
        'appops',
        'get',
        packageName,
        AUTO_REVOKE_OP,
      ],
    },);
    return stdout.includes(`: ${MODE_IGNORE}`,);
  } catch (error) {
    if (error instanceof AdbCommandError) {
      fl.warn(`could not read appops for ${packageName}; treating as not exempted: ${error.message}`,);
      return false;
    }
    throw error;
  }
}

/**
 Determine the exempt set by querying each app individually via
 {@link isExemptedViaGet}, in bounded concurrent batches. Fallback for
 devices whose appops lacks `query-op`.
 
 @param serial - Device to query.
 
 @param packages - Application ids to check.
 
 @returns Subset of `packages` that read back as exempt.
 */
async function getExemptedViaGet({
  serial,
  packages,
}: {
  readonly serial: string;
  readonly packages: readonly string[];
},): Promise<readonly string[]> {
  /**
   Start index of each concurrency-capped batch.
   */
  const batchStarts = Array.from(
    { length: Math.ceil(packages.length / GET_CONCURRENCY,), },
    function atBatchStart(
      _unused,
      index,
    ): number {
      return index * GET_CONCURRENCY;
    },
  );
  /**
   Accumulated exempt application ids across batches.
   */
  const exempted: string[] = [];
  for (const start of batchStarts) {
    /**
     Application ids in this batch.
     */
    const batch = packages.slice(
      start,
      start + GET_CONCURRENCY,
    );
    /**
     Per-app exempt verdicts for this batch.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- batches run sequentially on purpose: each Promise.all caps concurrency at GET_CONCURRENCY so the device is not flooded with simultaneous shells.
    const verdicts = await Promise.all(
      batch.map(async function verdictFor(packageName,): Promise<Verdict> {
        return {
          packageName,
          exempted: await isExemptedViaGet({
            serial,
            packageName,
          },),
        };
      },),
    );
    exempted.push(
      ...verdicts
        .filter(function isExempt(verdict,): boolean {
          return verdict.exempted;
        },)
        .map(function toName(verdict,): string {
          return verdict.packageName;
        },),
    );
  }
  return exempted;
}

/**
 List which of `packages` are currently exempt from auto-revoke.
 
 Primary path is a single bulk `query-op` via {@link runAdb}; if that
 raises an {@link AdbCommandError} (older Android without the subcommand),
 it falls back to per-app `get` via {@link getExemptedViaGet}. Either way
 the result is intersected with `packages` so only in-scope third-party
 apps remain.
 
 @param serial - Device to query.
 
 @param packages - Third-party application ids to consider in scope.
 
 @returns Subset of `packages` currently exempt.
 
 @example
 ```ts
 const exempt = await listExempted({ serial, packages: apps, },);
 ```
 */
export async function listExempted({
  serial,
  packages,
}: {
  readonly serial: string;
  readonly packages: readonly string[];
},): Promise<readonly string[]> {
  /**
   Tagged logger for this call.
   */
  const fl = tagged({
    tag: listExempted.name,
    l,
  },);
  /**
   In-scope application ids, used to filter the query result.
   */
  const inScope: ReadonlySet<string> = new Set(packages,);
  try {
    /**
     Captured stdout from the bulk `query-op` invocation.
     */
    const stdout = await runAdb({
      serial,
      args: [
        'shell',
        'cmd',
        'appops',
        'query-op',
        AUTO_REVOKE_OP,
        MODE_IGNORE,
      ],
    },);
    return parseExemptedQuery({ stdout, },)
      .filter(function inScopeName(name,): boolean {
        return inScope.has(name,);
      },);
  } catch (error) {
    if (error instanceof AdbCommandError) {
      fl.warn(`query-op unavailable; falling back to per-app appops get: ${error.message}`,);
      return getExemptedViaGet({
        serial,
        packages,
      },);
    }
    throw error;
  }
}
