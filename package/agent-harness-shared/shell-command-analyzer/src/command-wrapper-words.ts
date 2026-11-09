/**
 Strips program wrappers from a command word list.
 
 `env X=1 sudo timeout 60 git commit` runs one Git command,
 and the guard has to read it as that,
 not as an `env` invocation with arguments.
 
 The stripped layers are reported rather than discarded:
 a caller needs the assignments they carried,
 the command lines they split and run,
 and whether they left the program on a search path other than the caller's.
 
 @module
 */

import {
  COMMAND_WRAPPERS,
  type CommandWrapper,
} from './command-wrappers.ts';
import {
  lastPathSegment,
  looksLikeExecutablePath,
} from './executable-path-words.ts';

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
 Sentinel returned when a word assigns no variable.
 */
const NO_VARIABLE_ASSIGNMENT: unique symbol = Symbol('word assigns no variable',);

//endregion Constants

//region Types

/**
 Assignment word the shell applies before it launches a program.
 */
type WrapperAssignment = {
  /**
   Variable name receiving the value.
   */
  readonly name: string;
  /**
   Value after shell quote removal.
   */
  readonly value: string;
};

/**
 Everything the wrapper layers around one command revealed.
 */
type WrappedCommand = {
  /**
   Words of the innermost program the shell ultimately executes.
   */
  readonly words: readonly string[];
  /**
   Command lines wrappers split and run, outermost layer first.
   */
  readonly payloads: readonly string[];
  /**
   Assignments the wrappers and the shell apply before the program runs.
   */
  readonly assignments: readonly WrapperAssignment[];
  /**
   Whether a wrapper runs the program on the platform default search path
   instead of the caller's.
   */
  readonly usesDefaultSearchPath: boolean;
  /**
   Variable names a wrapper removes from the program's environment.
   */
  readonly unsetVariables: readonly string[];
};

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
};

//endregion Types

//region Shell assignments

/**
 Reads a word as a shell variable assignment prefix.
 
 The parser already separates `X=1 cmd` assignments;
 this reads the same shape among `env` arguments and after `env --`,
 where they are ordinary words.
 
 @param word - command argument word
 
 @returns variable name and value, or `undefined` when the word assigns nothing
 
 @example
 ```ts
 readVariableAssignment('X=1'); // { name: 'X', value: '1' }
 readVariableAssignment('/usr/bin/git'); // undefined
 ```
 */
function readVariableAssignment(word: string,): WrapperAssignment | typeof NO_VARIABLE_ASSIGNMENT {
  /**
   Offset of the first `=`, or -1 when the word carries none.
   */
  const equalsIndex = word.indexOf('=',);
  if (equalsIndex <= 0)
    return NO_VARIABLE_ASSIGNMENT;
  /**
   Text before `=`, which a shell assignment spells as a variable name.
   */
  const name = word.slice(
    0,
    equalsIndex,
  );
  if (name.startsWith('-',) || looksLikeExecutablePath(name,))
    return NO_VARIABLE_ASSIGNMENT;
  return {
    name,
    value: word.slice(equalsIndex + 1,),
  };
}

/**
 Splits the assignment prefixes off the front of a word list.
 
 @param words - command words starting at a possible assignment
 
 @returns remaining words and the assignments dropped from their front
 
 @example
 ```ts
 dropLeadingAssignments(['X=1', 'git', 'commit']);
 // { words: ['git', 'commit'], assignments: [{ name: 'X', value: '1' }] }
 ```
 */
function dropLeadingAssignments(words: readonly string[],): {
  readonly words: readonly string[];
  readonly assignments: readonly WrapperAssignment[];
} {
  /**
   Assignments dropped from the front of the word list.
   */
  const assignments: WrapperAssignment[] = [];
  for (let index = 0; index < words.length; index += 1) {
    /**
     Command word inspected as assignment or program name.
     */
    const word = words[index];
    if (word === undefined)
      break;
    /**
     Variable name and value the word assigns, when it assigns anything.
     */
    const assignment = readVariableAssignment(word,);
    if ((typeof assignment) === 'symbol')
      return {
        words: words.slice(index,),
        assignments,
      };
    assignments.push(assignment,);
  }
  return {
    words: [],
    assignments,
  };
}

//endregion Shell assignments

//region Wrapper stripping

/**
 Reads a command line carried by an option spelling attached to its value.
 
 Both spellings real wrappers accept are read:
 the separator form `--split-string=cmd`,
 and the short form `-Scmd` that GNU `env` takes with nothing between.
 
 @param word - option word being inspected
 
 @param wrapper - wrapper whose payload options are consulted
 
 @returns carried command line, or {@link NO_ATTACHED_PAYLOAD} when the spelling carries none
 
 @example
 ```ts
 attachedPayload({ word: '--split-string=git commit', wrapper }); // 'git commit'
 attachedPayload({ word: '-Sgit commit', wrapper }); // 'git commit'
 ```
 */
function attachedPayload({
  word,
  wrapper,
}: {
  readonly word: string;
  readonly wrapper: CommandWrapper;
},): string | typeof NO_ATTACHED_PAYLOAD {
  for (const option of wrapper.payloadOptions) {
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
 Spells one word so re-reading it as shell source yields the same value.
 
 The payload of a split-string option is shell source, and the words that follow
 it are not, so the two only join into one command line once the trailing words
 are quoted for that grammar.
 
 @param word - argument value after shell quote removal
 
 @returns shell source spelling of that one word
 
 @example
 ```ts
 shellQuotedWord(`it's`); // `'it'\\''s'`
 ```
 */
function shellQuotedWord(word: string,): string {
  /**
   The word with every embedded quote closed, escaped, and reopened.
   */
  const escaped = word.replaceAll(
    `'`,
    `'\\''`,
  );
  return `'${escaped}'`;
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
 Drops a wrapper's own options and leading words, keeping the wrapped command words.
 
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
  for (let index = 1; index < words.length; index += 1) {
    /**
     Argument word inspected as wrapper option or wrapped program.
     */
    const word = words[index];
    if (word === undefined)
      break;
    if (word === '--')
      return words.slice((index + 1) + wrapper.leadingWordsToSkip,);
    if (!word.startsWith('-',))
      return words.slice(index + wrapper.leadingWordsToSkip,);
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
     Command line carried by an attached payload option spelling.
     */
    const attached = attachedPayload({
      word,
      wrapper,
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
    if (wrapper.optionsWithValue
      .has(word,))
      index += 1;
  }
  return [];
}

/**
 Removes every wrapper around the invoked program, keeping the innermost command words.
 
 The loop runs inside a named function expression so the mutable remainder stays out of the
 function-body scope, where this repository forbids a root `let`.
 
 @param words - command name followed by its arguments
 
 @returns innermost command words with everything the stripped wrappers revealed
 
 @example
 ```ts
 unwrapWrappers({ words: ['env', 'X=1', 'sudo', 'git', 'commit'] }).words;
 // ['git', 'commit']
 ```
 */
function unwrapWrappers({ words, }: { readonly words: readonly string[]; },): WrappedCommand {
  /**
   Payloads, assignments and environment effects every stripped layer contributed.
   */
  const scan: WrapperScan = {
    payloads: [],
    assignments: [],
    usesDefaultSearchPath: false,
    unsetVariables: [],
  };

  /**
   Innermost program words after every layer is stripped.
   */
  const unwrapped = (function unwrapLayers(): readonly string[] {
    /**
     Command words of the outermost layer still to strip.
     */
    let remaining = words;
    for (;;) {
      /**
       Words after dropping shell assignment prefixes, and those prefixes.
       */
      const withoutAssignments = dropLeadingAssignments(remaining,);
      scan.assignments
        .push(...withoutAssignments.assignments,);
      /**
       Command word that may name a wrapper.
       */
      const [first,] = withoutAssignments.words;
      if (first === undefined)
        return [];

      /**
       Wrapper specification matching the command word, when it is one.
       */
      const wrapper = COMMAND_WRAPPERS
        .find(function wrapperNamed(candidate,): boolean {
          return candidate
            .names
            .has(lastPathSegment(first,),);
        },);
      if (wrapper === undefined)
        return withoutAssignments.words;

      remaining = wrappedCommandWords({
        words: withoutAssignments.words,
        wrapper,
        scan,
      },);
    }
  })();

  return {
    words: unwrapped,
    payloads: scan.payloads,
    assignments: scan.assignments,
    usesDefaultSearchPath: scan.usesDefaultSearchPath,
    unsetVariables: scan.unsetVariables,
  };
}

//endregion Wrapper stripping

export {
  readVariableAssignment,
  unwrapWrappers,
};
export type {
  WrappedCommand,
  WrapperAssignment,
};
