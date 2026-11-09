/**
 Strips one program wrapper's own words from a command word list.
 
 A wrapper owns words in two ways: options with their values,
 and positional words it consumes before the program,
 as `flock`'s lock file, `ssh`'s host, `taskset`'s mask and `chrt`'s priority.
 Both are consumed here so the program is the first word the wrapper does not own,
 which is why `flock lockfile -c 'cmd'` reads as that command string
 rather than as a program called `-c`.
 
 The scan runs inside a named function expression so the mutable position,
 ownership and mode stay out of the function-body scope,
 where this repository forbids a root `let`.
 
 @module
 */

import type { CommandWrapper, } from './command-wrapper-record.ts';
import {
  readVariableAssignment,
  type WrapperAssignment,
} from './shell-assignment-words.ts';
import { shellQuotedWord, } from './shell-source-words.ts';

//region Constants

/**
 Sentinel returned when an option spelling carries no attached command line.
 */
const NO_ATTACHED_PAYLOAD: unique symbol = Symbol('option spelling carries no attached command line',);

/**
 Sentinel returned when an option spelling carries no attached value.
 */
const NO_ATTACHED_OPTION_VALUE: unique symbol = Symbol('option spelling carries no attached value',);

/**
 Characters that spell a number a wrapper owns: hexadecimal digits.
 */
const NUMBER_DIGITS = '0123456789abcdefABCDEF';

/**
 Separators a cpu list and a mask range use between numbers, as `0,3` and `7-11`.
 */
const NUMBER_SEPARATORS = ',:-';

//endregion Constants

//region Types

/**
 Mutable accumulator one unwrap pass fills.
 */
type WrapperScan = {
  /**
   Command lines collected from every layer.
   */
  readonly payloads: string[];
  /**
   Assignments collected from every layer.
   */
  readonly assignments: WrapperAssignment[];
  /**
   Whether any layer replaced the search path with the platform default.
   */
  usesDefaultSearchPath: boolean;
  /**
   Variable names any layer removes from the environment.
   */
  readonly unsetVariables: string[];
  /**
   Whether any layer hands the program to another host.
   */
  reachesAnotherHost: boolean;
};

//endregion Types

//region Wrapper word scan

/**
 Reads a command line carried by an option spelling attached to its value.
 
 Both spellings real wrappers accept are read:
 the separator form `--split-string=cmd`,
 and the short form `-Scmd` that GNU `env` takes with nothing between.
 
 @param word - option word being inspected
 
 @param options - option names whose attached values carry a command line
 
 @returns carried command line, or {@link NO_ATTACHED_PAYLOAD} when the spelling carries none
 
 @example
 ```ts
 attachedOptionPayload({ word: '--split-string=git commit', options }); // 'git commit'
 attachedOptionPayload({ word: '-Sgit commit', options }); // 'git commit'
 ```
 */
function attachedOptionPayload({
  word,
  options,
}: {
  readonly word: string;
  readonly options: ReadonlySet<string>;
},): string | typeof NO_ATTACHED_PAYLOAD {
  for (const option of options) {
    /**
     Separator between an option name and its attached value.
     */
    const prefix = `${option}=`;
    /**
     Offset where the payload starts, or -1 when the spelling carries none.
     */
    const payloadStart = word.startsWith(prefix,)
      ? prefix.length
      : (option.startsWith('--',) || (!word.startsWith(option,))
        ? (-1)
        : option.length);
    if (payloadStart === (-1))
      continue;

    /**
     Command line after the option spelling.
     */
    const payload = word.slice(payloadStart,);
    return payload.length > 0
      ? payload
      : NO_ATTACHED_PAYLOAD;
  }
  return NO_ATTACHED_PAYLOAD;
}

/**
 Reads the value attached to an option word as `--option=value`.
 
 @param word - option word being inspected
 
 @param options - option names to match
 
 @returns attached value, or `undefined` when the word spells none of the options
 
 @example
 ```ts
 attachedOptionValue({ word: '--unset=PATH', options }); // 'PATH'
 ```
 */
function attachedOptionValue({
  word,
  options,
}: {
  readonly word: string;
  readonly options: ReadonlySet<string>;
},): string | typeof NO_ATTACHED_OPTION_VALUE {
  for (const option of options) {
    /**
     Separator between an option name and its attached value.
     */
    const prefix = `${option}=`;
    if (word.startsWith(prefix,))
      return word.slice(prefix.length,);
  }
  return NO_ATTACHED_OPTION_VALUE;
}

/**
 Reads the command line a split-string option starts.
 
 GNU `env` splits the option's payload and appends the words after it to the
 resulting argument list, so `env -S'git' commit -m x` runs `git commit -m x`.
 The payload and every following word therefore form one command line.
 
 @param payload - command line the option carried
 
 @param trailingWords - words after the option value
 
 @returns one shell source line for the program the wrapper launches
 
 @example
 ```ts
 payloadCommandLine({ payload: 'git', trailingWords: ['commit', '-m', 'x'] });
 // `git 'commit' '-m' 'x'`
 ```
 */
function payloadCommandLine({
  payload,
  trailingWords,
}: {
  readonly payload: string;
  readonly trailingWords: readonly string[];
},): string {
  return [
    payload,
    ...trailingWords.map(function quotedTrailingWord(word,): string {
      return shellQuotedWord(word,);
    },),
  ]
    .join(' ',);
}

/**
 Reports whether one word is the number the wrapper owns before its program.
 
 `taskset` takes an affinity mask and `chrt` a priority before the command,
 so that number is neither an option nor the program name.
 A wrapper owns one such number: after it is consumed,
 the next word is a program and its arguments,
 which may themselves start with hexadecimal letters (`taskset 1 cc`).
 
 @param word - word after the wrapper name, its options, and its earlier positional words
 
 @param wrapper - wrapper specification applied to the words
 
 @param numericWordsConsumed - count of numbers this wrapper already consumed
 
 @returns whether the wrapper owns that number rather than launching it
 
 @example
 ```ts
 numericWordIsWrapperOwned({ word: '0x3', wrapper: tasksetWrapper, numericWordsConsumed: 0 }); // true
 numericWordIsWrapperOwned({ word: 'cc', wrapper: tasksetWrapper, numericWordsConsumed: 1 }); // false
 ```
 */
function numericWordIsWrapperOwned({
  word,
  wrapper,
  numericWordsConsumed,
}: {
  readonly word: string;
  readonly wrapper: CommandWrapper;
  readonly numericWordsConsumed: number;
},): boolean {
  return (wrapper.numericWordBeforeProgram === true)
    && (numericWordsConsumed === 0)
    && isNumberWord(word,);
}

/**
 Reports whether one word spells a number the way a wrapper writes it.
 
 Read by index rather than by pattern,
 so one pass settles it and no input length can make the reading expensive.
 
 @param word - word inspected as a number the wrapper owns
 
 @returns whether the word is hexadecimal digits joined by separators
 
 @example
 ```ts
 isNumberWord('0x3'); // true
 isNumberWord('0,3,7-11'); // true
 isNumberWord('git'); // false
 ```
 */
function isNumberWord(word: string,): boolean {
  /**
   Offset of the first digit, after the prefix of a hexadecimal mask.
   */
  const digitsStart = word.startsWith('0x',) || word.startsWith('0X',)
    ? 2
    : 0;
  return (function readsDigitRuns(): boolean {
    /**
     Digits read since the last separator.
     */
    let runLength = 0;
    for (let index = digitsStart; index < word.length; index += 1) {
      /**
       Character at the inspected offset.
       */
      const character = word[index] ?? '';
      if (NUMBER_DIGITS.includes(character,)) {
        runLength += 1;
        continue;
      }
      if ((!NUMBER_SEPARATORS.includes(character,)) || (runLength === 0))
        return false;
      runLength = 0;
    }
    return runLength > 0;
  })();
}

/**
 Reads the words of the program a wrapper launches, once the scan reaches it.
 
 A wrapper that reparses its remaining words as shell source contributes that
 line instead of a program,
 unless an option switched it to running them as a program and its arguments.
 
 @param words - wrapper name followed by its arguments
 
 @param programStartIndex - offset of the first word of the wrapped program
 
 @param wrapper - wrapper specification applied to those words
 
 @param scan - accumulator receiving the payloads every layer contributes
 
 @param runsProgramArguments - whether an option switched the wrapper to running its words as arguments
 
 @returns words starting at the wrapped program, empty when the wrapper runs a shell source line
 
 @example
 ```ts
 programWordsAfterWrapper({
   words: ['flock', '/tmp/repo.lock', 'git', 'commit'],
   programStartIndex: 2,
   wrapper: flockWrapper,
   scan,
   runsProgramArguments: false,
});
// ['git', 'commit']
 ```
 */
function programWordsAfterWrapper({
  words,
  programStartIndex,
  wrapper,
  scan,
  runsProgramArguments,
}: {
  readonly words: readonly string[];
  readonly programStartIndex: number;
  readonly wrapper: CommandWrapper;
  readonly scan: WrapperScan;
  readonly runsProgramArguments: boolean;
},): readonly string[] {
  /**
   Words from the program onwards, which the wrapper either runs or reparses.
   */
  const programWords = words.slice(programStartIndex,);
  if ((wrapper.joinsRemainderAsShellSource === true) && (!runsProgramArguments)) {
    if (programWords.length > 0)
      scan.payloads
        .push(programWords.join(' ',),);
    return [];
  }
  return programWords;
}

/**
 Drops a wrapper's own options and positional words, keeping the wrapped command words.
 
 A wrapper owns words in two ways: options with their values,
 and positional words it consumes before the program,
 as `flock`'s lock file, `ssh`'s host, `taskset`'s mask and `chrt`'s priority.
 Both are consumed here so the program is the first word the wrapper does not own,
 which is why `flock lockfile -c 'cmd'` reads as that command string
 rather than as a program called `-c`.
 
 The scan runs inside a named function expression so the mutable position,
 ownership and mode stay out of the function-body scope,
 where this repository forbids a root `let`.
 
 @param words - wrapper name followed by its arguments
 
 @param wrapper - wrapper specification applied to those words
 
 @param scan - accumulator receiving the payloads and environment effects of every layer
 
 @returns words starting at the wrapped program
 
 @example
 ```ts
 wrappedCommandWords({
   words: ['sudo', '-u', 'root', 'git', 'commit'],
   wrapper: sudoWrapper,
   scan,
});
// ['git', 'commit']
 ```
 */
function wrappedCommandWords({
  words,
  wrapper,
  scan,
}: {
  readonly words: readonly string[];
  readonly wrapper: CommandWrapper;
  readonly scan: WrapperScan;
},): readonly string[] {
  return (function scanWrapperWords(): readonly string[] {
    /**
     Whether `--` already ended the wrapper's own options.
     */
    let optionsEnded = false;
    /**
     Count of the wrapper's leading positional words already consumed.
     */
    let leadingWordsConsumed = 0;
    /**
     Count of the numbers the wrapper owns that were already consumed.
     */
    let numericWordsConsumed = 0;
    /**
     Whether an option switched the wrapper to running its words as arguments.
     */
    let runsProgramArguments = false;

    for (let index = 1; index < words.length; index += 1) {
      /**
       Argument word inspected as wrapper option, owned word, or wrapped program.
       */
      const word = words[index];
      if (word === undefined)
        break;

      if ((!optionsEnded) && (word === '--')) {
        optionsEnded = true;
        continue;
      }

      if ((!optionsEnded) && word.startsWith('-',)) {
        if (wrapper.defaultSearchPathFlags
          .has(word,)) {
          scan.usesDefaultSearchPath = true;
          continue;
        }
        if (wrapper.unsetVariableOptions
          .has(word,)) {
          /**
           Variable name the wrapper removes from the wrapped program's environment.
           */
          const variable = words[index + 1];
          if (variable !== undefined)
            scan.unsetVariables
              .push(variable,);
          index += 1;
          continue;
        }
        /**
         Variable name attached to an unset option as `--unset=PATH`.
         */
        const attachedUnsetVariable = attachedOptionValue({
          word,
          options: wrapper.unsetVariableOptions,
        },);
        if ((typeof attachedUnsetVariable) !== 'symbol') {
          scan.unsetVariables
            .push(attachedUnsetVariable,);
          continue;
        }
        /**
         Assignment attached to a setenv option as `--setenv=PATH=/usr/bin`.
         */
        const attachedSetenv = attachedOptionValue({
          word,
          options: wrapper.setenvOptions,
        },);
        if (attachedSetenv !== NO_ATTACHED_OPTION_VALUE) {
          /**
           Assignment the attached value spells.
           */
          const assignment = readVariableAssignment(attachedSetenv,);
          if ((typeof assignment) !== 'symbol')
            scan.assignments
              .push(assignment,);
          continue;
        }
        if (wrapper.setenvOptions
          .has(word,)) {
          /**
           Assignment the wrapper applies to the wrapped program's environment.
           */
          const assignment = readVariableAssignment(words[index + 1] ?? '',);
          if ((typeof assignment) !== 'symbol')
            scan.assignments
              .push(assignment,);
          index += 1;
          continue;
        }

        /**
         Whole command string carried by an attached command option spelling.
         */
        const attachedCommand = attachedOptionPayload({
          word,
          options: wrapper.commandStringOptions,
        },);
        if (attachedCommand !== NO_ATTACHED_PAYLOAD) {
          scan.payloads
            .push(attachedCommand,);
          return [];
        }
        if (wrapper.commandStringOptions
          .has(word,)) {
          /**
           Whole command string following a separate command option.
           */
          const command = words[index + 1];
          if (command !== undefined)
            scan.payloads
              .push(command,);
          return [];
        }

        /**
         Command line carried by an attached payload option spelling.
         */
        const attached = attachedOptionPayload({
          word,
          options: wrapper.payloadOptions,
        },);
        if (attached !== NO_ATTACHED_PAYLOAD) {
          scan.payloads
            .push(payloadCommandLine({
              payload: attached,
              trailingWords: words.slice(index + 1,),
            },),);
          return [];
        }
        if (wrapper.payloadOptions
          .has(word,)) {
          /**
           Command line following a separate payload option.
           */
          const payload = words[index + 1];
          if (payload !== undefined) {
            scan.payloads
              .push(payloadCommandLine({
                payload,
                trailingWords: words.slice(index + 2,),
              },),);
          }
          return [];
        }
        if ((wrapper.programArgumentsFlags
          ?.has(word,)) === true)
          runsProgramArguments = true;
        if (wrapper.optionsWithValue
          .has(word,))
          index += 1;
        continue;
      }

      if (numericWordIsWrapperOwned({
        word,
        wrapper,
        numericWordsConsumed,
      },)) {
        numericWordsConsumed += 1;
        continue;
      }
      if (leadingWordsConsumed < wrapper.leadingWordsToSkip) {
        leadingWordsConsumed += 1;
        continue;
      }
      return programWordsAfterWrapper({
        words,
        programStartIndex: index,
        wrapper,
        scan,
        runsProgramArguments,
      },);
    }
    return [];
  })();
}


export type { WrapperScan, };
export { wrappedCommandWords, };
