/**
 Thin adb invocation layer over `nano-spawn`. Every higher-level helper routes
 through {@link runAdb} so error mapping (missing binary vs failed command)
 and serial threading live in one place.
 
 @module
 */

import { tagged, } from '@monochromatic-dev/module-logger/ts';
import nanoSpawn, { SubprocessError, } from 'nano-spawn';

import { ADB, } from './constants.ts';
import {
  AdbCommandError,
  AdbNotFoundError,
} from './errors.ts';

/**
 Module-level tagged logger; each function wraps it with its own name.
 */
const l = tagged({ tag: 'adb', },);

/**
 Report whether an error's `cause` is a Node `ENOENT` (no such file) error,
 which is how a missing `adb` binary surfaces. Used to tell "binary not found"
 apart from a non-zero exit, without exposing an optional code value.
 
 @param cause - Error `cause` that may be a Node `errno` object.
 
 @returns `true` when the cause carries `code === 'ENOENT'`.
 */
function causeIsEnoent({ cause, }: { readonly cause: unknown; },): boolean {
  if ((cause === null) || ((typeof cause) !== 'object')
    || (!('code' in cause))) {
    return false;
  }
  /**
   Loosely-typed view of the cause object's optional `code` field.
   */
  const { code, } = cause as { readonly code?: unknown; };
  return code === 'ENOENT';
}

/**
 Run `adb` with `args`, optionally targeting `serial` via `-s`, and return its
 stdout (trailing newline already stripped by nano-spawn).
 
 @param serial - Device to target with `-s`; omit to let adb pick the sole
                 device (callers resolve ambiguity before calling).
 
 @param args - Arguments after the binary, for example
               `['shell', 'pm', 'list', 'packages', '-3']`.
 
 @returns Captured stdout from the invocation.
 
 @throws {@link AdbNotFoundError} when the `adb` binary is missing (`ENOENT`).
 
 @throws {@link AdbCommandError} when adb exits non-zero or fails otherwise.
 
 @example
 ```ts
 const out = await runAdb({ args: ['devices',], },);
 ```
 */
export async function runAdb({
  serial,
  args,
}: {
  readonly serial?: string;
  readonly args: readonly string[];
},): Promise<string> {
  /**
   Tagged logger for this invocation.
   */
  const fl = tagged({
    tag: runAdb.name,
    l,
  },);
  /**
   Full argv, with the `-s <serial>` selector prepended when targeting one device.
   */
  const fullArgs: readonly string[] = serial === undefined ? args : [
    '-s',
    serial,
    ...args,
  ];
  fl.debug(`${ADB} ${fullArgs.join(' ',)}`,);
  try {
    /**
     Captured stdout from the successful invocation.
     */
    const { stdout, } = await nanoSpawn(
      ADB,
      fullArgs,
    );
    return stdout;
  } catch (error) {
    if (error instanceof SubprocessError) {
      if (causeIsEnoent({ cause: error.cause, },)) {
        throw new AdbNotFoundError(
          'Could not find the "adb" command on PATH. Install Android platform-tools and ensure adb is runnable.',
          { cause: error, },
        );
      }
      /**
       Trimmed stderr, used as the failure detail when non-empty.
       */
      const stderr = error.stderr
        .trim();
      /**
       Human-readable failure detail: stderr when present, else the error message.
       */
      const detail = stderr.length > 0 ? stderr : error.message;
      throw new AdbCommandError(
        `${ADB} ${fullArgs.join(' ',)} failed: ${detail}`,
        { cause: error, },
      );
    }
    throw error;
  }
}
