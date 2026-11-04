/**
 Which process a process id names: its start time and boot, and the PID namespace the id is counted in.

 A process id is reused by the kernel, and means something only inside its PID namespace, so a record of an owner that
 names only the id would read as held by whatever unrelated process carries that id later. A process is therefore named by
 its id plus when it started (`/proc/<pid>/stat` field 22, in clock ticks since the boot) plus which boot that tick count
 belongs to (`/proc/sys/kernel/random/boot_id`); the namespace is carried beside it. The birth text is
 `linux:<ticks>@<boot id>`: the first part is the spelling `package/git-policy/cli/src/native/process/process_identity.rs` uses
 (`linux_identity_from_stat`), and the boot suffix keeps a tick count of an earlier boot from matching.

 Three copies of this read exist in the repository (`package/translation-repair/run/src/corpus-run/process-identity.ts`,
 `package/wg/cli-quicker/src/linux-process-identity.ts`, and the Rust one named above). None is a shared package, and each
 ties the read to its own domain's logger or error type, so this package keeps its own small reader.
 */
import {
  readFile,
  readlink,
} from 'node:fs/promises';

import { CargoMutantsToolingError } from './error.ts';
import { isMissing } from './fs-guards.ts';

/**
 Position of the start time among the fields after the command name, counted from zero at the state (field three):
 field twenty-two is nineteen fields after it.
 */
const startTicksIndex = 19;

/**
 Process states of a process that has exited: zombie and dead. A zombie under an init that never reaps keeps its id forever.
 */
const exitedStates: ReadonlySet<string> = new Set([
  'Z',
  'X',
]);

/**
 Where the kernel shows its processes.
 */
export const defaultProcRoot = '/proc';

/**
 Where the kernel names the current boot.
 */
const bootIdPath = '/proc/sys/kernel/random/boot_id';

/**
 Stands for a process that does not exist (any more): no record of it, or it exited. It is a distinct value so that a
 process that cannot be judged is never mistaken for one that is gone.
 */
export const processGone: unique symbol = Symbol('ENOENT or ESRCH on /proc/<pid>/stat');

/**
 Stands for a PID namespace the system does not say, such as that of a process this one may not inspect.
 */
export const namespaceUnknown: unique symbol = Symbol('unreadable namespace link of a process');

/**
 What names one process.
 */
export type ProcessIdentity = {
  /**
   The process id, counted in the namespace.
   */
  readonly pid: number;
  /**
   The birth text: start time and boot.
   */
  readonly birth: string;
  /**
   The PID namespace the id is counted in, such as `pid:[4026531836]`; absent where the system does not say.
   */
  readonly namespace?: string;
};

/**
 Read the code of a system error.

 @param error - caught value

 @returns the code, or the empty text for a value without one

 @example
 ```ts
 errorCodeOf(new Error('x'));
 ```
 */
export function errorCodeOf(error: unknown): string {
  if (!Error.isError(error))
    return '';
  if (!('code' in error))
    return '';
  /**
   The code, which a system error holds as text.
   */
  const { code } = error;
  return (typeof code) === 'string' ? code : '';
}

/**
 Tell whether a failed read says the process is gone: no such record, or the process exited during the read.

 @param error - caught value

 @returns true for `ENOENT` and `ESRCH`

 @example
 ```ts
 recordIsGone(new Error('x'));
 ```
 */
function recordIsGone(error: unknown): boolean {
  return isMissing(error) || (errorCodeOf(error) === 'ESRCH');
}

/**
 Cut the start time out of the text of `/proc/<pid>/stat`.

 The command name is parenthesised and may hold spaces and parentheses, so the fields are counted from after the last
 closing parenthesis.

 @param stat - the text of the record

 @returns the start time in clock ticks, or {@link processGone} when the process is a zombie or dead

 @throws CargoMutantsToolingError when the text has no closing parenthesis or no start time

 @example
 ```ts
 startTicksFromStat('4321 (node) S 1 2 3');
 ```
 */
export function startTicksFromStat(stat: string): string | typeof processGone {
  /**
   End of the command name.
   */
  const commandEnd = stat.lastIndexOf(')');
  if (commandEnd === (-1))
    throw new CargoMutantsToolingError('A Linux process record has no closing parenthesis after the command name, so its start time cannot be read.');
  /**
   Fields from the state on, after the `) ` that ends the command name.
   */
  const fields = stat.slice(commandEnd + 2)
    .trim()
    .split(' ');
  if (exitedStates.has(fields[0] ?? ''))
    return processGone;
  /**
   The start time field.
   */
  const startTicks = fields[startTicksIndex];
  if ((startTicks === undefined) || (startTicks === ''))
    throw new CargoMutantsToolingError('A Linux process record ends before its start time field, so its start time cannot be read.');
  return startTicks;
}

/**
 Read the text of a file for the identity reads.

 @param path - the file

 @returns its text

 @example
 ```ts
 await readProcText('/proc/self/stat');
 ```
 */
async function readProcText(path: string): Promise<string> {
  return await readFile(
    path,
    'utf8',
  );
}

/**
 What a birth text says: the start time and the boot.
 */
export type ParsedBirth = {
  /**
   The start time, in clock ticks since the boot.
   */
  readonly startTicks: string;
  /**
   The boot the ticks count from.
   */
  readonly boot: string;
};

/**
 Stands for a birth text of another spelling than this package writes, which proves nothing about the process.
 */
export const birthNotParsed: unique symbol = Symbol('birth.not-parsed');

/**
 Take a birth text apart.

 @param birth - the text, `linux:<ticks>@<boot id>`

 @returns its parts, or {@link birthNotParsed} for a text of another spelling

 @example
 ```ts
 parseBirth('linux:4242@3f2a');
 ```
 */
export function parseBirth(birth: string): ParsedBirth | typeof birthNotParsed {
  /**
   The prefix every birth text starts with.
   */
  const prefix = 'linux:';
  if (!birth.startsWith(prefix))
    return birthNotParsed;
  /**
   Where the boot id starts: the last `@`, since a boot id holds none.
   */
  const separator = birth.lastIndexOf('@');
  if (separator <= prefix.length)
    return birthNotParsed;
  /**
   The parts.
   */
  const startTicks = birth.slice(
    prefix.length,
    separator,
  );
  /**
   The boot id.
   */
  const boot = birth.slice(separator + 1);
  return ((startTicks === '') || (boot === '')) ? birthNotParsed : {
    startTicks,
    boot,
  };
}

/**
 Read the name of the current boot.

 @returns the boot id

 @throws CargoMutantsToolingError when it cannot be read, so a record could not be told from one of an earlier boot

 @example
 ```ts
 await readBootId();
 ```
 */
export async function readBootId(): Promise<string> {
  try {
    return (await readProcText(bootIdPath)).trim();
  }
  catch (error) {
    throw new CargoMutantsToolingError(`The boot id cannot be read from ${bootIdPath} (${String(error)}), so an owner of an earlier boot could not be told from one of this boot.`);
  }
}

/**
 Read the record of a process.

 @param pid - the process id to look up

 @param readText - reads a file's text

 @param procRoot - the directory that shows the processes

 @returns the text of `/proc/<pid>/stat`, or {@link processGone} when there is no such record or the process exited during
 the read

 @throws CargoMutantsToolingError when the record cannot be read for another reason, such as a refusal

 @example
 ```ts
 await readStat({ pid: 1, readText: readProcText });
 ```
 */
async function readStat({
  pid,
  readText,
  procRoot,
}: {
  readonly pid: number;
  readonly readText: (path: string) => Promise<string>;
  readonly procRoot: string;
}): Promise<string | typeof processGone> {
  /**
   The record of the process.
   */
  const path = `${procRoot}/${String(pid)}/stat`;
  try {
    return await readText(path);
  }
  catch (error) {
    if (recordIsGone(error))
      return processGone;
    throw new CargoMutantsToolingError(`Whether process ${String(pid)} runs cannot be decided: ${path} cannot be read (${String(error)}).`);
  }
}

/**
 Read the birth text of a process.

 @param pid - the process id to look up

 @param readText - reads a file's text; absent to read the real one, given by tests to show a failure the host cannot be
 made to show

 @param procRoot - the directory that shows the processes; a test gives a fake one

 @returns `linux:<start ticks>@<boot id>`, or {@link processGone} when no process has that id, or it exited

 @throws CargoMutantsToolingError when the record cannot be read for a reason other than the process being gone (a refusal,
 for one), or is malformed, because an owner that cannot be judged must stop the caller and never be read as dead

 @example
 ```ts
 await readProcessBirth({ pid: process.pid });
 ```
 */
export async function readProcessBirth({
  pid,
  readText = readProcText,
  procRoot = defaultProcRoot,
}: {
  readonly pid: number;
  readonly readText?: (path: string) => Promise<string>;
  readonly procRoot?: string;
}): Promise<string | typeof processGone> {
  /**
   The record's text.
   */
  const stat = await readStat({
    pid,
    readText,
    procRoot,
  });
  if (stat === processGone)
    return processGone;
  /**
   Its start time.
   */
  const startTicks = startTicksFromStat(stat);
  if (startTicks === processGone)
    return processGone;
  return `linux:${startTicks}@${await readBootId()}`;
}

/**
 Read the PID namespace of a process.

 @param pid - the process id to look up

 @param procRoot - the directory that shows the processes; a test gives a fake one

 @returns the namespace link text, or {@link namespaceUnknown} when the process is gone or its namespace cannot be read

 @example
 ```ts
 await readPidNamespace({ pid: process.pid });
 ```
 */
export async function readPidNamespace({
  pid,
  procRoot = defaultProcRoot,
}: {
  readonly pid: number;
  readonly procRoot?: string;
}): Promise<string | typeof namespaceUnknown> {
  try {
    return await readlink(`${procRoot}/${String(pid)}/ns/pid`);
  }
  catch (error) {
    if (!recordIsGone(error))
      console.error(`The PID namespace of process ${String(pid)} cannot be read: ${String(error)}`);
    return namespaceUnknown;
  }
}

/**
 Gather what names this process.

 @returns its id, birth and namespace

 @throws CargoMutantsToolingError when this process cannot be identified, or the `/proc` it sees describes another PID
 namespace than the one its id is counted in (its own record carries another id), because a record made from it would name
 the wrong process

 @example
 ```ts
 const own = await currentProcessIdentity();
 ```
 */
export async function currentProcessIdentity(): Promise<ProcessIdentity> {
  /**
   What `/proc` reports as this process's id, the first field of its own record.
   */
  const reported = Number((await readProcText('/proc/self/stat')).split(' ')[0]);
  if (reported !== process.pid)
    throw new CargoMutantsToolingError(`The /proc this process sees describes another PID namespace: it reports id ${String(reported)} for the process that has id ${String(process.pid)}.`);
  /**
   Its birth text.
   */
  const birth = await readProcessBirth({ pid: process.pid });
  if (birth === processGone)
    throw new CargoMutantsToolingError(`This process (id ${String(process.pid)}) cannot be identified, so it cannot own a claim.`);
  /**
   The namespace of this process.
   */
  const namespace = await readPidNamespace({ pid: process.pid });
  return {
    pid: process.pid,
    birth,
    ...(namespace === namespaceUnknown ? {} : { namespace }),
  };
}
