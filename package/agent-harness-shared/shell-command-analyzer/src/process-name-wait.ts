/**
 Shared guardrail helpers for loops that wait on a process found by name.

 @module
 */

import { analyzeShellCommand, } from './analyzer.ts';
import type { ShellCommandInfo, } from './types.ts';

//region Constants

/**
 Shell binaries whose `-c` argument is a script string.
 */
const SHELL_NAMES: ReadonlySet<string> = new Set([
  'bash',
  'sh',
  'zsh',
  'dash',
],);

/**
 Commands that look processes up by name or command line.
 */
const PROCESS_NAME_LOOKUP_COMMANDS: ReadonlySet<string> = new Set([
  'pgrep',
  'pidof',
],);

/**
 Refusal reason used when a `while` or `until` loop looks processes up by name.

 @example
 ```ts
 return PROCESS_NAME_WAIT_BAN_REASON;
 ```
 */
const PROCESS_NAME_WAIT_BAN_REASON: string = [
  'Blocked: a `while` or `until` loop that calls `pgrep` or `pidof` is banned in this repo.',
  'The Bash tool runs your command inside a wrapper shell whose command line contains your command text,',
  'so `pgrep --full <pattern>` matches that wrapper and the loop never sees the process exit.',
  'Wait on a PID instead, under a bound: `timeout 600 tail --pid=<pid> --follow /dev/null`.',
  'Take the PID from the launch (`$!`, or `systemctl --user show --property=MainPID --value <unit>`).',
  'For long runs start a named unit with `systemd-run --user --wait --collect --unit=<name> <command>`,',
  'or use Monitor on the PID and emit only lines you would act on.',
]
  .join(' ',);

//endregion Constants

//region Predicates

/**
 Collects `-c` script strings passed to shells named in a command or its wrapper arguments.

 @param info - parsed command whose name or arguments may launch a shell

 @returns script strings that the launched shells execute

 @example
 ```ts
 shellScriptPayloads(info); // ['until ! pgrep x; do sleep 1; done']
 ```
 */
function shellScriptPayloads(info: ShellCommandInfo,): string[] {
  /**
   Command name followed by its arguments, so a wrapper's inner command is scanned like a direct one.
   */
  const words = [
    info.name,
    ...info.args,
  ];

  return words
    .flatMap(function payloadAfterShellDashC(
      word,
      index,
    ): string[] {
      if (!SHELL_NAMES.has(word,))
        return [];

      /**
       Arguments after the shell name.
       */
      const rest = words.slice(index + 1,);
      /**
       Position of the `-c` flag within `rest`.
       */
      const dashCIndex = rest.indexOf('-c',);
      /**
       Script string following the `-c` flag.
       */
      const payload = rest[dashCIndex + 1];

      return ((dashCIndex !== (-1)) && ((typeof payload) === 'string'))
        ? [payload,]
        : [];
    },);
}

/**
 Checks whether a shell command calls `pgrep` or `pidof` inside a `while` or `until` loop.

 Uses the shared `unbash` analyzer so quoted prose and escaped text are classified by shell grammar.
 Loop membership covers both the condition list and the body,
 so `until ! pgrep -f x; do sleep 1; done` and `while true; do pgrep -f x || break; done` both match.
 Straight-line `pgrep` calls outside any loop stay allowed.

 @param command - shell command from agent harness Bash tool input

 @returns whether command looks processes up by name from within a repeating loop

 @example
 ```ts
 waitsOnProcessName('until ! pgrep -f build; do sleep 1; done'); // true
 waitsOnProcessName('pgrep -af build'); // false
 ```
 */
function waitsOnProcessName(command: string,): boolean {
  /**
   Parsed shell command analysis.
   */
  const analysis = analyzeShellCommand(command,);
  if (!analysis.parsed)
    return false;

  return analysis.commands
    .some(function commandLooksUpProcessByNameInLoop(info,): boolean {
      if (PROCESS_NAME_LOOKUP_COMMANDS.has(info.name,)
        && (info.context
          .conditionLoopDepth
          > 0))
        return true;

      // `timeout 60 bash -c '<script>'` and similar wrappers hide the loop inside a string argument.
      return shellScriptPayloads(info,)
        .some(waitsOnProcessName,);
    },);
}
//endregion Predicates

export {
  PROCESS_NAME_WAIT_BAN_REASON,
  waitsOnProcessName,
};
