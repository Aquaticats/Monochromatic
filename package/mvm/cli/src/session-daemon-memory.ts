/**
 Reads the machine's total memory for the cap on the session daemon's unit.

 The project set that cap on 2026-11-21 as "half of total memory": every guest
 inside `mvm-libvirt-session.service` together may use half of what the machine
 has, and the other half stays with the rest of the session. Rule `LQM` asks for
 that basis beside the value.

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
 Divisor of the machine's total memory that the unit may use: half.

 The project chose "half of total" for issue 6338 on 2026-11-21
 (the answer sent 2026-11-21T23:07:05-05:00), so guests together get half the
 machine and the session keeps the other half for everything else.
 */
const CAP_DIVISOR = 2;

/**
 Kibibytes in one mebibyte, which is what systemd's `M` suffix means.
 */
const KIB_PER_MIB = 1_024;

/**
 Suffix systemd reads as a power of 1024: `31576M` is 31576 MiB.
 */
const MEMORY_MAX_SUFFIX = 'M';

/**
 Smallest cap mvm gives the unit, in mebibytes.

 Half of a machine whose total memory is under two mebibytes is not a cap any unit
 can run in, and such a machine is not one mvm runs on. The floor keeps the value
 mvm hands systemd a positive one, without claiming what `0` would mean there.
 */
const MINIMUM_MEMORY_MAX_MIB = 1;

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
        `mvm caps libvirt's session daemon unit at half of this machine's total memory, which it reads from ${path}, and could not read it: ${detail}`,
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

 @returns Cap of half that memory, in mebibytes with systemd's `M` suffix

 @example
 ```ts
 memoryMaxArgument({ memTotalKib: 2_097_152 }); // => '1024M'
 ```
 */
export function memoryMaxArgument({ memTotalKib, }: {
  readonly memTotalKib: number;
},): string {
  /**
   Half the machine's memory, in kibibytes.
   */
  const halfKib = memTotalKib / CAP_DIVISOR;
  /**
   Half the machine's memory, rounded down to whole mebibytes.
   */
  const halfMib = Math.floor(halfKib / KIB_PER_MIB,);
  /**
   Whole mebibytes the cap may state, never zero.
   */
  const mib = Math.max(
    MINIMUM_MEMORY_MAX_MIB,
    halfMib,
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
 Reads the machine's total memory and returns the cap half of it gives the
 session daemon's unit.

 @param path - `meminfo` file to read; defaults to {@link MEMINFO_PATH}

 @returns Cap in the form systemd's `MemoryMax=` reads

 @throws {@link SessionDaemonMemoryError} when the file cannot be read or holds no total

 @example
 ```ts
 await sessionDaemonMemoryMax(); // => '31576M' on a machine with 61.7 GiB
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
   Cap that half of the reported total gives.
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
