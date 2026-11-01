import { readFile, } from 'node:fs/promises';

import { BypassRouteError, } from './errors.ts';

/**
 Sentinel representing process absent from procfs.
 */
export const PROCESS_ABSENT: unique symbol = Symbol('Linux process is absent',);

/**
 Procfs identity fields used for liveness and command validation.
 */
export type LinuxProcessIdentity = {
  readonly commandLine: readonly string[];
  readonly startTime: string;
  readonly state: string;
};

/**
 Zero-based start-time offset after proc stat command field.
 */
const PROC_START_TIME_OFFSET = 19;

/**
 Narrows caught value to Node filesystem error.
 
 @param error - Caught value.
 
 @returns Whether value carries error code.
 
 @example
 ```ts
 isErrnoException({ code: 'ENOENT' }); // true
 ```
 */
function isErrnoException(error: unknown,): error is NodeJS.ErrnoException {
  return ((typeof error) === 'object')
    && (error !== null)
    && ('code' in error);
}

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
