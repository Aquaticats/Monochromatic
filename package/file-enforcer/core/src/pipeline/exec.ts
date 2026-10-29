import spawn from 'nano-spawn';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { l, } from '../logger.ts';
import {
  type Command,
  evaluatePredicate,
  type PlatformCommands,
  type PlatformEntry,
} from '../platform/evaluate-predicate.ts';

//region exec

/**
 Direct invocation form: a single executable plus its arguments.
 */
export type ExecDirectInvocation = {
  readonly cmd: string;
  readonly args?: readonly string[];
};

/**
 Platform-aware invocation form: predicate/command tuples evaluated top-to-bottom.
 */
export type ExecPlatformInvocation = {
  readonly platformCommands: PlatformCommands;
};

/**
 Discriminated invocation accepted by {@link exec}.
 */
export type ExecInvocation = ExecDirectInvocation | ExecPlatformInvocation;

/**
 Runs an external command directly or evaluates platform entries to find a matching command.
 
 Direct form: pass `{ cmd, args? }` to spawn the executable with the given args.
 
 Platform-aware form: pass `{ platformCommands }`; entries are evaluated top-to-bottom
 and the first matching predicate's command is executed. Predicates are direct commands
 (no shell); exit code 0 = match.
 
 @param invocation - Direct invocation `{ cmd, args? }` or platform-aware `{ platformCommands }`
 
 @returns Stdout output as a string
 
 @throws When the command exits with a non-zero code
 
 @throws When no platform predicate matches (platform-aware form only): includes all tested predicates
 
 @example
 ```ts
 // Direct form
 const version = await exec({ cmd: 'git', args: ['--version'] });
 
 // Platform-aware form: first match wins (try mise, then brew, then apt)
 const output = await exec({
   platformCommands: [
     [['mise', '--version'],    ['mise', 'exec', '--', 'git', 'pull']],
     [['brew', '--version'],    ['brew', 'install', 'git']],
     [['apt-get', '--version'], ['apt-get', 'install', 'git']],
   ],
 });
 ```
 */
export async function exec(invocation: ExecInvocation,): Promise<string> {
  if ('platformCommands' in invocation)
    return await execPlatformAware(invocation.platformCommands,);
  return await execDirect({
    cmd: invocation.cmd,
    args: invocation.args
      ?? [],
  },);
}

//endregion exec

//region Direct execution

/**
 Executes a single command directly via `nano-spawn`.
 
 @param cmd - Executable name
 
 @param args - Arguments passed to the command
 
 @returns Stdout output as a string
 */
async function execDirect(
  {
    cmd,
    args,
  }: {
    readonly cmd: string;
    readonly args: readonly string[];
  },
): Promise<string> {
  /**
   Function-scoped logger tagged with the call site for traceable command logs.
   */
  const rl = tagged({
    tag: execDirect.name,
    l,
  },);
  rl.debug(`${cmd} ${args.join(' ',)}`,);
  /**
   Spawn result; only `stdout` is forwarded to the caller.
   */
  const { stdout, } = await spawn(
    cmd,
    [...args,],
  );
  return stdout;
}

//endregion Direct execution

//region Platform-aware execution

/**
 Evaluates all predicates concurrently via {@link evaluatePredicate}, then
 executes the command via {@link execCommand} from the first entry (by
 declaration order) whose predicate succeeded.
 
 @param platformCommands - Ordered `[predicate, command]` tuples
 
 @returns Stdout from the matched command
 
 @throws {@link PlatformMatchError} when no predicate matches
 */
async function execPlatformAware(
  platformCommands: PlatformCommands,
): Promise<string> {
  /**
   Predicate outcomes; aligned positionally with `platformCommands` for first-match lookup.
   */
  const results = await Promise.all(
    platformCommands.map(async function checkPredicate(entry,): Promise<boolean> {
      /**
       Predicate half of the entry tuple; command half is consulted only on match.
       */
      const [predicate,] = entry;
      return await evaluatePredicate(predicate,);
    },),
  );
  /**
   Index of the first successful predicate, or `-1` if no platform matched.
   */
  const matchIndex = results.findIndex(Boolean,);
  if (matchIndex === (-1))
    throw new PlatformMatchError(platformCommands,);
  /**
   Tuple at the winning index; revalidated since `at`-style access could be `undefined`.
   */
  const matched = platformCommands[matchIndex];
  if (!matched)
    throw new PlatformMatchError(platformCommands,);
  /**
   Command half of the matched tuple; predicate half already consumed for ranking.
   */
  const [, cmd,] = matched;
  return await execCommand(cmd,);
}

/**
 Dispatches a {@link Command} to the appropriate execution path.
 
 - **Array, first element is string**: `[cmd, ...args]` → {@link execDirect}
 - **Array, first element is array**: nested {@link PlatformCommands} → {@link execPlatformAware} (recursive)
 
 @param cmd - Command as `[cmd, ...args]` or nested {@link PlatformCommands}
 
 @returns Stdout output as a string
 */
async function execCommand(cmd: Command,): Promise<string> {
  if (isNestedPlatformCommands(cmd,))
    return await execPlatformAware(cmd,);
  /**
   Head/tail split so `execDirect` receives executable name and argv separately.
   */
  const [executable = '', ...args] = cmd as readonly string[];
  return await execDirect({
    cmd: executable,
    args,
  },);
}

/**
 Determines whether a non-string {@link Command} is a nested {@link PlatformCommands}.
 Checks if the first element is an array (predicate tuple) rather than a string (command name).
 
 @param cmd - Array-form command to inspect
 
 @returns `true` if the command is a nested {@link PlatformCommands}
 */
function isNestedPlatformCommands(
  cmd: readonly string[] | PlatformCommands,
): cmd is PlatformCommands {
  return (cmd.length
    > 0) && Array
      .isArray(cmd[0],);
}

//endregion Platform-aware execution

//region Errors

/**
 Thrown when no predicate in a {@link PlatformCommands} list matches the current platform.
 Includes all tested predicates for debuggability.
 */
class PlatformMatchError extends Error {
  /**
   @param platformCommands - Entries that were tested and all failed
   */
  constructor(platformCommands: PlatformCommands,) {
    /**
     Comma-joined predicate render, embedded into the error message for debuggability.
     */
    const predicateList = platformCommands
      .map(function formatOne(entry,): string {
        return formatEntry(entry,);
      },)
      .join(', ',);
    super(
      `No platform predicate matched. Tested: [${predicateList}]`,
    );
    this.name = 'PlatformMatchError';
  }
}

/**
 Formats a single platform entry's predicate for error messages.
 
 @param entry - Platform entry to format
 
 @returns Human-readable predicate representation
 */
function formatEntry(entry: PlatformEntry,): string {
  /**
   Predicate half of the entry tuple; only this half is rendered for error messages.
   */
  const [predicate,] = entry;
  return `[${
    predicate
      .map(function quote(segment,): string {
        return `"${segment}"`;
      },)
      .join(', ',)
  }]`;
}

//endregion Errors
