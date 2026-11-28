/**
 Reads the machine's total memory for the cap on the session daemon's unit.

 The project set that cap on 2026-11-28 as three quarters of total memory,
 replacing its choice of 2026-11-21 for issue 6338 ("half of total"): every
 guest inside `mvm-libvirt-session.service` together may use three quarters of
 what the machine has, and everything outside the unit gets the rest. Rule `LQM`
 asks for that basis beside the value.

 The value is read from `/proc/meminfo`'s `MemTotal` when mvm starts the unit,
 so it follows the machine rather than a number written into mvm.

 @module
 */

import { readFile, } from 'node:fs/promises';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 Logger root for mvm after removing the package log shim.

 @example
 ```ts
 const rl = tagged({ tag: someFunction.name, l, },);
 ```
 */
const l = tagged({ tag: 'mvm', },);

//region Names

/**
 File on Linux that reports the machine's memory, in kibibytes.
 */
export const MEMINFO_PATH = '/proc/meminfo';

/**
 Start of the line in that file whose value is the machine's total memory.
 */
const MEM_TOTAL_KEY = 'MemTotal:';

/**
 Unit the value on that line carries.
 */
const MEM_TOTAL_UNIT = 'kB';

/**
 Position of the number within the fields after the key, once the padding is trimmed.
 */
const MEM_TOTAL_VALUE_INDEX = 0;

/**
 Position of the unit within those fields.
 */
const MEM_TOTAL_UNIT_INDEX = 1;

/**
 Separator between those fields, after the padding the key left is trimmed.
 */
const MEMINFO_FIELD_SEPARATOR = ' ';

/**
 Numerator of the fraction of the machine's total memory the unit may use.

 The owner answered "Three quarters, six guests" on 2026-11-28, which replaced
 the project's choice of 2026-11-21 for issue 6338 ("half of total"); both are
 recorded in
 `doc/decision/owner-answers-of-2026-11-28-guests-swap-drives-removal-and-releases.md`,
 "The unit's cap". Guests together may use three quarters of the machine and
 everything outside the unit gets the rest, so host builds, the gate (its unit
 may take 20 GiB) and the desktop share 15.4 GiB swap instead. Rule `LQM` asks
 for that basis beside the value.
 */
export const CAP_NUMERATOR = 3;

/**
 Denominator of that fraction, kept whole so the cap is whole-number arithmetic
 on kibibytes rather than a divisor with a repeating decimal.
 */
export const CAP_DENOMINATOR = 4;

/**
 Kibibytes in one mebibyte, which is what systemd's `M` suffix means.
 */
const KIB_PER_MIB = 1_024;

/**
 Suffix systemd reads as a power of 1024: `1M` is one mebibyte.
 */
const MEMORY_MAX_SUFFIX = 'M';

/**
 Smallest cap mvm gives the unit, in mebibytes.

 A machine small enough that the fraction rounds down to no whole mebibyte is
 not one mvm runs on. The floor keeps the value mvm hands systemd a positive
 one, without claiming what `0` would mean there.
 */
export const MINIMUM_MEMORY_MAX_MIB = 1;

//endregion Names

//region Error

/**
 The machine's total memory could not be read, so mvm cannot size the unit's
 memory cap.

 @example
 ```ts
 try {
   await sessionDaemonMemoryMax();
 }
 catch (error) {
   if (error instanceof SessionDaemonMemoryError) console.error(error.path);
 }
 ```
 */
export class SessionDaemonMemoryError extends Error {
  /**
   File mvm read for the machine's total memory.
   */
  readonly path: string;

  /**
   @param cause - Read failure, when one explains the text

   @param detail - What was wrong with the file's content

   @param path - File mvm read
   */
  constructor({
    cause,
    detail,
    path,
  }: {
    readonly cause?: unknown;
    readonly detail: string;
    readonly path: string;
  },) {
    super(
      [
        `mvm caps libvirt's session daemon unit at three quarters of this machine's total memory, which it reads from ${path}, and could not read it: ${detail}`,
        'mvm needs that number to set the unit\'s MemoryMax; there is nothing to configure, because the machine reports it.',
      ].join('\n',),
      { cause, },
    );
    this.name = 'SessionDaemonMemoryError';
    this.path = path;
  }
}

//endregion Error

//region Reading

/**
 Reads the number of kibibytes from the `MemTotal` line of a `meminfo` text.

 @param path - File the text came from, named when the text holds no total

 @param text - Content of that file

 @returns Total memory in kibibytes

 @throws {@link SessionDaemonMemoryError} when the text holds no `MemTotal` line

 @example
 ```ts
 parseMemTotalKib({ path: '/proc/meminfo', text: 'MemTotal:       1000 kB\n' }); // => 1000
 ```
 */
export function parseMemTotalKib({
  path,
  text,
}: {
  readonly path: string;
  readonly text: string;
},): number {
  /**
   Fields of the first line that reports a total, when the text holds one.
   */
  const totalFields = text
    .split('\n',)
    .find(function isTotal(line,) {
      return line.startsWith(MEM_TOTAL_KEY,);
    },)
    ?.slice(MEM_TOTAL_KEY.length,)
    .trim()
    .split(MEMINFO_FIELD_SEPARATOR,);
  /**
   Number of kibibytes that line reports, or `NaN` when it does not hold one.
   */
  const kib = Number(totalFields?.[MEM_TOTAL_VALUE_INDEX],);
  /**
   Whether the line carries the unit mvm expects, which rules out a differently formatted file.
   */
  const hasUnit = totalFields?.[MEM_TOTAL_UNIT_INDEX]
    === MEM_TOTAL_UNIT;
  if ((!Number.isFinite(kib,)) || (!hasUnit)) {
    throw new SessionDaemonMemoryError({
      detail: `it has no \`${MEM_TOTAL_KEY}\` line holding a number of ${MEM_TOTAL_UNIT}`,
      path,
    },);
  }
  return kib;
}

/**
 Renders the memory cap for a machine whose total memory is `memTotalKib`, in
 the form systemd's `MemoryMax=` reads.

 @param memTotalKib - Machine's total memory in kibibytes

 @returns Cap of that fraction of the memory, in mebibytes with systemd's `M` suffix

 @example
 ```ts
 memoryMaxArgument({ memTotalKib: 2_097_152 }); // => '1536M'
 ```
 */
export function memoryMaxArgument({ memTotalKib, }: {
  readonly memTotalKib: number;
},): string {
  /**
   The fraction of the machine's memory, in kibibytes.
   */
  const capKib = (memTotalKib * CAP_NUMERATOR) / CAP_DENOMINATOR;
  /**
   That fraction of the machine's memory, rounded down to whole mebibytes.
   */
  const capMib = Math.floor(capKib / KIB_PER_MIB,);
  /**
   Whole mebibytes the cap may state, never zero.
   */
  const mib = Math.max(
    MINIMUM_MEMORY_MAX_MIB,
    capMib,
  );
  return `${String(mib,)}${MEMORY_MAX_SUFFIX}`;
}

/**
 Reads a `meminfo` file's text, reporting an unreadable one with the file it
 names.

 @param path - File to read

 @returns Content of the file

 @throws {@link SessionDaemonMemoryError} when the file cannot be read

 @example
 ```ts
 await readMeminfoText({ path: '/proc/meminfo' });
 ```
 */
async function readMeminfoText({ path, }: { readonly path: string; },): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    throw new SessionDaemonMemoryError({
      cause: error,
      detail: 'the file could not be read',
      path,
    },);
  }
}

/**
 Reads the machine's total memory and returns the cap the decided fraction of
 it gives the session daemon's unit.

 @param path - `meminfo` file to read; defaults to {@link MEMINFO_PATH}

 @returns Cap in the form systemd's `MemoryMax=` reads

 @throws {@link SessionDaemonMemoryError} when the file cannot be read or holds no total

 @example
 ```ts
 await sessionDaemonMemoryMax(); // => '47364M' on this 61.7 GiB machine
 ```
 */
export async function sessionDaemonMemoryMax({ path = MEMINFO_PATH, }: {
  readonly path?: string;
} = {},): Promise<string> {
  /**
   Logger scoped to the read so the value is attributable.
   */
  const rl = tagged({
    tag: sessionDaemonMemoryMax.name,
    l,
  },);
  /**
   Content of the file, read once for both the total and its unit.
   */
  const text = await readMeminfoText({ path, },);
  /**
   Cap that fraction of the reported total gives.
   */
  const cap = memoryMaxArgument({
    memTotalKib: parseMemTotalKib({
      path,
      text,
    },),
  },);
  rl.debug(`the machine's total memory gives the session daemon unit MemoryMax=${cap}`,);
  return cap;
}

//endregion Reading
