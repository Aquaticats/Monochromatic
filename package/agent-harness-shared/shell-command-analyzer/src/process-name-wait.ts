/**
 Shared guardrail helpers for loops that wait on a process found by name.

 @module
 */

import { analyzeShellCommand, } from './analyzer.ts';
import { unwrapWrappers, } from './command-wrapper-words.ts';
import { lastPathSegment, } from './executable-path-words.ts';
import { rawCommandWordRuns, } from './raw-command-word-runs.ts';
import {
  shellScriptPayloads,
  wordLaunchesShellScript,
} from './shell-script-payloads.ts';
import type { ShellCommandInfo, } from './types.ts';

//region Constants

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
 Sentinel returned when one raw word run holds no word it would run as a program.
 */
const NO_PROGRAM_WORD: unique symbol = Symbol('a run of raw shell words began no command',);

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
 Finds the first redirection operator in one raw word.

 @param word - word from a raw word run

 @returns offset of the first `<` or `>`, or -1 when the word holds no operator

 @example
 ```ts
 redirectOperatorAt('pgrep</dev/null'); // 5
 ```
 */
function redirectOperatorAt(word: string,): number {
  for (let offset = 0; offset < word.length; offset += 1) {
    /**
     Character at the current offset.
     */
    const character = word[offset];
    if ((character === '<')
      || (character === '>'))
      return offset;
  }
  return -1;
}

/**
 Checks whether one word holds nothing but digits, which a redirection writes before its operator.

 @param word - word read from a raw word run

 @returns whether the word spells a file descriptor number

 @example
 ```ts
 isFileDescriptorNumber('2'); // true
 ```
 */
function isFileDescriptorNumber(word: string,): boolean {
  for (const character of word) {
    if ((character < '0')
      || (character > '9'))
      return false;
  }
  return word.length > 0;
}

/**
 Checks whether one word holds only the characters a redirection operator owns.

 @param word - text from the first operator character to the end of the word

 @returns whether the text is an operator with no target attached to it

 @example
 ```ts
 isRedirectionOperatorText('>>'); // true
 isRedirectionOperatorText('>/dev/null'); // false
 ```
 */
function isRedirectionOperatorText(word: string,): boolean {
  for (const character of word) {
    if ((character !== '<')
      && (character !== '>'))
      return false;
  }
  return word.length > 0;
}

/**
 Reads the word one raw run would run as its program, stepping over redirections in front of it.

 A redirection may stand before the program or be attached to it,
 so `while >/dev/null pgrep -f x; do` and `while pgrep</dev/null -f x; do`
 would otherwise read the redirection as the program and hide the lookup.
 A bare operator takes the word after it as its target, so that word is stepped over too.
 A redirection that names a descriptor (`>&2`) or already carries its target (`&>/dev/null`)
 hangs on its own word and hides nothing.

 @param words - words of one raw run, after the shell words in front of its command are dropped

 @returns word the run would run as its program, or {@link NO_PROGRAM_WORD} when it holds none

 @example
 ```ts
 programWordOfRun(['>/dev/null', 'pgrep', '-f', 'x']); // 'pgrep'
 ```
 */
function programWordOfRun(words: readonly string[],): string | typeof NO_PROGRAM_WORD {
  return (function readProgramWord(): string | typeof NO_PROGRAM_WORD {
    /**
     Position of the word being read.
     */
    let index = 0;
    while (index < words.length) {
      /**
       Word being read, then counted as read.
       */
      const word = words[index];
      index += 1;
      if (word === undefined)
        return NO_PROGRAM_WORD;

      /**
       Offset of the redirection operator, or -1 when the word carries none.
       */
      const operatorAt = redirectOperatorAt(word,);
      if (operatorAt === (-1))
        return word;

      /**
       Word text before the operator, with the `&` of `&>file` left out of it.
       */
      const operatorPrefix = word.slice(
        0,
        operatorAt,
      );
      const beforeOperator = operatorPrefix.startsWith('&',)
        ? operatorPrefix.slice(1,)
        : operatorPrefix;
      if ((beforeOperator !== '')
        && (!isFileDescriptorNumber(beforeOperator,)))
        return beforeOperator;

      if (isRedirectionOperatorText(
        word.slice(operatorAt,),
      ))
        index += 1;
    }
    return NO_PROGRAM_WORD;
  })();
}

/**
 Collects the script strings one raw run's program hands to a shell.

 Only the program word launches a shell in a raw run,
 and the run is already unwrapped,
 so `echo bash -c 'pgrep x'` hands over no script and is not refused.
 The script option itself is read by {@link shellScriptPayloads}.

 @param program - word the run would run as its program

 @param words - innermost program words of the run

 @returns script strings the program itself hands to a shell

 @example
 ```ts
 programShellScripts({ program: 'bash', words: ['bash', '-c', 'pgrep x'], }); // ['pgrep x']
 ```
 */
function programShellScripts({
  program,
  words,
}: {
  readonly program: string | typeof NO_PROGRAM_WORD;
  readonly words: readonly string[];
},): readonly string[] {
  return (((typeof program) === 'string')
    && (wordLaunchesShellScript(program,)))
    ? shellScriptPayloads({ words, },)
    : [];
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
      const program = programWordOfRun(wrapped.words,);
      if (((typeof program) === 'string')
        && (wordLooksUpProcessByName(program,)))
        return true;
      pending.push(
        ...wrapped.payloads,
        ...programShellScripts({
          program,
          words: wrapped.words,
        },),
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
      return shellScriptPayloads({
        words: [
          info.name,
          ...info.args,
        ],
      },)
        .some(waitsOnProcessName,);
    },);
}

//endregion Predicates

export {
  PROCESS_NAME_WAIT_BAN_REASON,
  waitsOnProcessName,
};
