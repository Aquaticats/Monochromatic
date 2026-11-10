/**
 Shared guardrail helpers for loops that wait on a process found by name.

 @module
 */

import { analyzeShellCommand, } from './analyzer.ts';
import { unwrapWrappers, } from './command-wrapper-words.ts';
import { lastPathSegment, } from './executable-path-words.ts';
import { rawCommandWordRuns, } from './raw-command-word-runs.ts';
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
 Shell words that stand in front of a command without naming the program it runs.

 A command the parser rejected is read as raw word runs,
 which keep `until`, `while`, `do` and `!` as words of the run they open,
 so those words are dropped before the program word is read.
 */
const SHELL_WORDS_BEFORE_A_COMMAND: ReadonlySet<string> = new Set([
  '!',
  'do',
  'elif',
  'else',
  'if',
  'then',
  'until',
  'while',
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
  'When the shell command analyzer cannot parse the command, any `pgrep` or `pidof` in it is refused,',
  'because the loop around that lookup could not be read.',
  'The Bash tool runs your command inside a wrapper shell whose command line contains your command text,',
  'so `pgrep --full <pattern>` matches that wrapper and the loop never sees the process exit.',
  'Wait on a PID instead, under a bound: `timeout 600 tail --pid=<pid> --follow /dev/null`.',
  'Take the PID from the launch (`$!`, or `systemctl --user show --property=MainPID --value <unit>`).',
  'For long runs start a named unit with `systemd-run --user --wait --collect --unit=<name>',
  '--property=MemoryMax=<size> --property=CPUQuota=<percent> --property=TasksMax=<count> --property=IOWeight=10 <command>`',
  '(all four limits, each sized to the command, and never an IOWeight above 10),',
  'or use Monitor on the PID and emit only lines you would act on.',
]
  .join(' ',);

//endregion Constants

//region Predicates

/**
 Collects `-c` script strings passed to shells named in a command's words.

 @param words - command name followed by its arguments, with wrappers already stripped

 @returns script strings that the launched shells execute

 @example
 ```ts
 shellScriptPayloads(['bash', '-c', 'until ! pgrep x; do sleep 1; done']);
 ```
 */
function shellScriptPayloads(words: readonly string[],): string[] {
  return words
    .flatMap(function payloadAfterShellDashC(
      word,
      index,
    ): string[] {
      if (!SHELL_NAMES.has(lastPathSegment(word,),))
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
 Drops the shell words that stand in front of the command a word run holds.

 @param words - words of one run, as the raw reader reports them

 @returns the run's words from the first one that can name a program

 @example
 ```ts
 dropShellWordsBeforeCommand(['until', '!', 'pgrep', '-f', 'x']); // ['pgrep', '-f', 'x']
 ```
 */
function dropShellWordsBeforeCommand(words: readonly string[],): readonly string[] {
  /**
   Position of the first word that can name a program.
   */
  const programIndex = words
    .findIndex(function wordNamesAProgram(word,): boolean {
      return !SHELL_WORDS_BEFORE_A_COMMAND.has(word,);
    },);
  return (programIndex === (-1))
    ? []
    : words.slice(programIndex,);
}

/**
 Checks whether a command word names a process lookup, whatever path spells it.

 @param word - command word from parsed syntax or from a raw word run

 @returns whether the word's last path segment is `pgrep` or `pidof`

 @example
 ```ts
 wordLooksUpProcessByName('/usr/bin/pgrep'); // true
 ```
 */
function wordLooksUpProcessByName(word: string,): boolean {
  return PROCESS_NAME_LOOKUP_COMMANDS.has(lastPathSegment(word,),);
}

/**
 Checks whether shell source the parser rejected holds a process lookup command.

 Reads raw words only, so it serves both an unparsable command and the command lines
 wrappers such as `eval` join into shell source. Loop membership cannot be read from
 these runs, so finding the lookup command is answer enough: the guard refuses what it
 cannot read rather than treating the unreadable loop as proof there is none.

 @param source - shell source text to read

 @returns whether any command position in the text names `pgrep` or `pidof`

 @example
 ```ts
 rawTextWaitsOnProcessName({ source: 'until ! pgrep x; do "', }); // true
 ```
 */
function rawTextWaitsOnProcessName({ source, }: { readonly source: string; },): boolean {
  /**
   Command lines still to read, the outer source text first.
   */
  const pending: string[] = [source,];
  /**
   Command lines already read, so a wrapper repeating its own text ends the walk.
   */
  const read = new Set<string>();

  while (pending.length > 0) {
    /**
     Command line popped from the work list.
     */
    const current = pending.pop();
    if (current === undefined)
      break;
    if (read.has(current,))
      continue;
    read.add(current,);

    for (const words of rawCommandWordRuns({ source: current, },)) {
      /**
       Innermost program words of this run, with shell keywords and wrapper layers stripped.
       */
      const wrapped = unwrapWrappers({
        words: dropShellWordsBeforeCommand(words,),
      },);
      /**
       Word that names the program this run would run.
       */
      const [program,] = wrapped.words;
      if ((program !== undefined) && wordLooksUpProcessByName(program,))
        return true;
      pending.push(
        ...wrapped.payloads,
        ...shellScriptPayloads(wrapped.words,),
      );
    }
  }

  return false;
}

/**
 Checks whether a shell command calls `pgrep` or `pidof` inside a `while` or `until` loop.

 Uses the shared `unbash` analyzer so quoted prose and escaped text are classified by shell grammar.
 Loop membership covers both the condition list and the body,
 so `until ! pgrep -f x; do sleep 1; done` and `while true; do pgrep -f x || break; done` both match.
 Straight-line `pgrep` calls outside any loop stay allowed.
 A command the parser rejects is read from its raw words instead,
 and any `pgrep` or `pidof` it spells in a command position is refused,
 because the loop around it could not be read.

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
    return rawTextWaitsOnProcessName({ source: command, },);

  return analysis.commands
    .some(function commandLooksUpProcessByNameInLoop(info,): boolean {
      if (wordLooksUpProcessByName(info.name,)
        && (info.context
          .conditionLoopDepth
          > 0))
        return true;

      // `timeout 60 bash -c '<script>'` and similar wrappers hide the loop inside a string argument.
      return shellScriptPayloads([
        info.name,
        ...info.args,
      ],)
        .some(waitsOnProcessName,);
    },);
}

//endregion Predicates

export {
  PROCESS_NAME_WAIT_BAN_REASON,
  waitsOnProcessName,
};
