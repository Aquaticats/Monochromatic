import {
  readdir,
  readFile,
} from 'node:fs/promises';

import {
  BypassRouteError,
  isErrnoException,
} from './errors.ts';

/**
 Sentinel representing process absent from procfs.
 */
export const PROCESS_ABSENT: unique symbol = Symbol('Linux process is absent',);

/**
 Proc state letter of a process that exited and has not been reaped yet.

 Such a process runs nothing and holds nothing,
 and only its task entry waits for whoever adopted it to collect it.
 */
export const PROCESS_ZOMBIE_STATE = 'Z';

/**
 Procfs identity fields used for liveness and command validation.
 */
export type LinuxProcessIdentity = {
  readonly commandLine: readonly string[];
  readonly startTime: string;
  readonly state: string;
};

/**
 One process still counted in a process group,
 with the state that says whether it can still run.
 */
export type LinuxProcessGroupMember = {
  readonly pid: number;
  readonly state: string;
};

/**
 Zero-based start-time offset after proc stat command field.
 */
const PROC_START_TIME_OFFSET = 19;

/**
 Zero-based process-group offset after proc stat command field.
 The fields there begin with state, which is proc field three, then parent, then group.
 */
const PROC_GROUP_OFFSET = 2;

/**
 Parses proc stat fields after parenthesized command name.
 
 @param pid - Process identifier named in diagnostics.
 
 @param stat - Proc stat text.
 
 @returns Process state and start-time ticks.
 
 @example
 ```ts
 parseProcStat({ pid: 1, stat: '1 (init) S 0 ...' });
 ```
 */
function parseProcStat(
  {
    pid,
    stat,
  }: {
    readonly pid: number;
    readonly stat: string;
  },
): Pick<LinuxProcessIdentity, 'startTime' | 'state'> {
  /**
   End of parenthesized command field.
   */
  const commandEnd = stat.lastIndexOf(')',);
  if (commandEnd === (-1))
    throw new BypassRouteError(`Cannot parse process identity for PID ${String(pid,)}.`,);
  /**
   Fields beginning with process state,
   which is proc field three.
   */
  const fields = stat.slice(commandEnd + 2,)
    .split(' ',);
  /**
   Process state field.
   */
  const [state,] = fields;
  /**
   Start time is proc field twenty-two,
   index nineteen after field three.
   */
  const startTime = fields.at(PROC_START_TIME_OFFSET,);
  if ((state === undefined) || (startTime === undefined))
    throw new BypassRouteError(`Process identity lacks fields for PID ${String(pid,)}.`,);
  return {
    state,
    startTime,
  };
}

/**
 The state and process group one proc stat line names.

 Field three of the line is the state and field five the group,
 both after the parenthesized command name.
 */
type ProcStatGroup = {
  /**
   State letter of the process.
   */
  readonly state: string;

  /**
   Process group the kernel lists the process in.
   */
  readonly group: number;
};

/**
 Parses state and process group from one proc stat line.

 @param pid - Process identifier named in diagnostics.

 @param stat - Proc stat text.

 @returns The state and the process group the line names.

 @throws {@link BypassRouteError} when the line holds neither.

 @example
 ```ts
 parseProcStatGroup({ pid: 1, stat: '1 (init) S 0 1 1 0 -1 ...' });
 ```
 */
function parseProcStatGroup(
  {
    pid,
    stat,
  }: {
    readonly pid: number;
    readonly stat: string;
  },
): ProcStatGroup {
  /**
   End of parenthesized command field.
   */
  const commandEnd = stat.lastIndexOf(')',);
  if (commandEnd === (-1))
    throw new BypassRouteError(`Cannot parse process identity for PID ${String(pid,)}.`,);
  /**
   Fields beginning with process state,
   which is proc field three.
   */
  const fields = stat.slice(commandEnd + 2,)
    .split(' ',);
  /**
   Process state field.
   */
  const [state,] = fields;
  /**
   Process group is proc field five,
   index two after field three.
   */
  const group = fields.at(PROC_GROUP_OFFSET,);
  if ((state === undefined) || (group === undefined))
    throw new BypassRouteError(`Process identity lacks fields for PID ${String(pid,)}.`,);
  return {
    state,
    group: Number(group,),
  };
}

/**
 Reads Linux process identity resistant to PID reuse.
 
 @param pid - Positive process identifier.
 
 @returns Procfs identity or absence when process no longer exists.
 
 @example
 ```ts
 await readLinuxProcessIdentity({ pid: process.pid });
 ```
 */
export async function readLinuxProcessIdentity(
  { pid, }: { readonly pid: number; },
): Promise<LinuxProcessIdentity | typeof PROCESS_ABSENT> {
  if ((!Number.isSafeInteger(pid,)) || (pid <= 0))
    throw new BypassRouteError(`Invalid process identifier: ${String(pid,)}`,);
  try {
    /**
     Proc stat and command line from same bounded observation.
     */
    const [stat, commandLineText,] = await Promise.all([
      readFile(
        `/proc/${String(pid,)}/stat`,
        'utf8',
      ),
      readFile(
        `/proc/${String(pid,)}/cmdline`,
        'utf8',
      ),
    ],);
    /**
     Parsed state and start-time fields.
     */
    const parsed = parseProcStat({
      pid,
      stat,
    },);
    return {
      ...parsed,
      commandLine: commandLineText.split('\0',)
        .filter(function nonemptyArgument(value,): boolean {
          return value !== '';
        },),
    };
  }
  catch (error) {
    if (isErrnoException(error,) && (error.code === 'ENOENT'))
      return PROCESS_ABSENT;
    throw error;
  }
}

/**
 A procfs entry that is not a task of the group asked about:
 it either vanished before its own stat was read,
 or the kernel lists it in another group.
 */
const NOT_IN_GROUP: unique symbol = Symbol('procfs lists it elsewhere',);

/**
 Whether a procfs directory entry names a process.

 Procfs lists one numeric directory per task and named ones for everything else.

 @param name - One directory entry of procfs.

 @returns Whether the entry holds at least one character and every one is a decimal digit.

 @example
 ```ts
 numericEntry('4242');
 ```
 */
function numericEntry(name: string,): boolean {
  if (name === '')
    return false;
  for (let index = 0; index < name.length; index++) {
    /**
     One character of the entry, read as a code unit so a name of any script is never reassembled.
     */
    const character = name.charAt(index,);
    if ((character < '0') || (character > '9'))
      return false;
  }
  return true;
}

/**
 Reads one procfs entry as a member of the group asked about.

 @param name - Numeric procfs directory entry.

 @param pgid - Process group the caller asks about.

 @returns The member, or {@link NOT_IN_GROUP}.

 @throws {@link BypassRouteError} when the stat line holds no state or no group.

 @example
 ```ts
 await memberOf({ name: '4242', pgid: 4242 });
 ```
 */
async function memberOf(
  {
    name,
    pgid,
  }: {
    readonly name: string;
    readonly pgid: number;
  },
): Promise<LinuxProcessGroupMember | typeof NOT_IN_GROUP> {
  /**
   Process identifier the directory entry names.
   */
  const pid = Number(name,);
  try {
    /**
     Stat line of this process, which holds both the state and the group.
     */
    const stat = await readFile(
      `/proc/${String(pid,)}/stat`,
      'utf8',
    );
    /**
     State and group parsed from the same stat line.
     */
    const {
      group,
      state,
    } = parseProcStatGroup({
      pid,
      stat,
    },);
    if (group !== pgid)
      return NOT_IN_GROUP;
    return {
      pid,
      state,
    };
  }
  catch (error) {
    if (isErrnoException(error,) && ((error.code === 'ENOENT') || (error.code === 'ESRCH')))
      return NOT_IN_GROUP;
    throw error;
  }
}

/**
 Reads the state of every process the kernel still counts in one process group.

 `/proc/PID/stat` names the group in field five.
 A member that exited and has not been reaped yet is still counted there,
 and the kernel still answers `kill(-pgid, 0)` for the group,
 so a caller asking whether the group can still run anything has to read the states
 and not only the group's existence.

 @param pgid - Process group identifier.

 @returns One member per process procfs lists, in procfs order.

 @throws {@link BypassRouteError} when a proc stat line holds no state or no group.

 @example
 ```ts
 await readProcessGroupMembers({ pgid: 4242 });
 ```
 */
export async function readProcessGroupMembers(
  { pgid, }: { readonly pgid: number; },
): Promise<readonly LinuxProcessGroupMember[]> {
  if ((!Number.isSafeInteger(pgid,)) || (pgid <= 0))
    throw new BypassRouteError(`Invalid process-group identifier: ${String(pgid,)}`,);
  /**
   Procfs entries, of which the numeric ones name one task each.
   */
  const entries = await readdir('/proc',);
  /**
   Every task of the group,
   with the entries that are not one of them left out.
   */
  const read = await Promise.all(entries
    .filter(function numericName(name,): boolean {
      return numericEntry(name,);
    },)
    .map(function member(name,) {
      return memberOf({
        name,
        pgid,
      },);
    },));
  return read.filter(function present(member,): member is LinuxProcessGroupMember {
    return (typeof member) !== 'symbol';
  },);
}

/**
 Checks process arguments independently from executable install path.
 
 @param identity - Live procfs identity retaining executable argument.
 
 @param expected - Exact arguments following nonempty executable argument.
 
 @returns Whether argument vectors have equal length and values.
 
 @example
 ```ts
 processArgumentsMatch({ identity, expected: ['watcher.mjs'] });
 ```
 */
export function processArgumentsMatch(
  {
    identity,
    expected,
  }: {
    readonly identity: LinuxProcessIdentity;
    readonly expected: readonly string[];
  },
): boolean {
  /**
   Actual vector includes executable argument before compared process arguments.
   */
  const actualLength = identity
    .commandLine
    .length;
  /**
   Expected vector gains one slot for executable argument.
   */
  const expectedLength = expected.length + 1;
  if (actualLength !== expectedLength)
    return false;
  /**
   Nonempty executable argument intentionally independent from runtime install path.
   */
  const [executable,] = identity.commandLine;
  if ((executable === undefined) || (executable === ''))
    return false;
  return expected.every(function sameArgument(
    value,
    index,
  ): boolean {
    return identity.commandLine[index + 1] === value;
  },);
}
