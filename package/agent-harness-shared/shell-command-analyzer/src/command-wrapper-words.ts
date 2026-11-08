/**
 Strips program wrappers from a command word list.
 
 `env X=1 sudo timeout 60 git commit` runs one Git command,
 and the guard has to read it as that,
 not as an `env` invocation with arguments.
 
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

//endregion Constants

//region Shell assignments

/**
 Reports whether a word is a shell variable assignment prefix.
 
 The parser already separates `X=1 cmd` assignments;
 this reads the same shape among `env` arguments and after `env --`,
 where they are ordinary words.
 
 @param word - command argument word
 
 @returns whether the word spells `NAME=value`
 
 @example
 ```ts
 isVariableAssignment('X=1'); // true
 isVariableAssignment('/usr/bin/git'); // false
 ```
 */
function isVariableAssignment(word: string,): boolean {
  /**
   Offset of the first `=`, or -1 when the word carries none.
   */
  const equalsIndex = word.indexOf('=',);
  if (equalsIndex <= 0)
    return false;
  /**
   Text before `=`, which a shell assignment spells as a variable name.
   */
  const name = word.slice(
    0,
    equalsIndex,
  );
  return (!name.startsWith('-',)) && (!looksLikeExecutablePath(name,));
}

/**
 Drops shell assignment prefixes before the invoked program.
 
 @param words - command words starting at a possible assignment
 
 @returns words starting at the first non-assignment word
 
 @example
 ```ts
 dropLeadingAssignments(['X=1', 'git', 'commit']); // ['git', 'commit']
 ```
 */
function dropLeadingAssignments(words: readonly string[],): readonly string[] {
  /**
   Offset of the first word that is not an assignment, or -1 when every word is one.
   */
  const firstCommandWord = words
    .findIndex(function isCommandWord(word,): boolean {
      return !isVariableAssignment(word,);
    },);
  return firstCommandWord === (-1)
    ? []
    : words.slice(firstCommandWord,);
}

//endregion Shell assignments

//region Wrapper stripping

/**
 Reads a command line carried by an attached option spelling such as `--split-string=...`.
 
 @param word - option word being inspected
 
 @param wrapper - wrapper whose payload options are consulted
 
 @returns carried command line, or {@link NO_ATTACHED_PAYLOAD} when the spelling carries none
 
 @example
 ```ts
 attachedPayload({ word: '--split-string=git commit', wrapper });
 // 'git commit'
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
    if (!word.startsWith(prefix,))
      continue;

    /**
     Command line after the separator.
     */
    const payload = word.slice(prefix.length,);
    return payload.length > 0
      ? payload
      : NO_ATTACHED_PAYLOAD;
  }
  return NO_ATTACHED_PAYLOAD;
}

/**
 Drops a wrapper's own options and leading words, keeping the wrapped command words.
 
 @param words - wrapper name followed by its arguments
 
 @param wrapper - wrapper specification applied to those words
 
 @param payloads - collected command lines the wrapper splits and runs
 
 @returns words starting at the wrapped program
 
 @example
 ```ts
 wrappedCommandWords({
   words: ['sudo', '-u', 'root', 'git', 'commit'],
   wrapper: sudoWrapper,
   payloads: [],
});
// ['git', 'commit']
 ```
 */
function wrappedCommandWords({
  words,
  wrapper,
  payloads,
}: {
  readonly words: readonly string[];
  readonly wrapper: CommandWrapper;
  readonly payloads: string[];
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

    /**
     Command line carried by an attached payload option spelling.
     */
    const attached = attachedPayload({
      word,
      wrapper,
    },);
    if (attached !== NO_ATTACHED_PAYLOAD) {
      payloads.push(attached,);
      continue;
    }
    if (wrapper.payloadOptions
      .has(word,)) {
      /**
       Command line following a separate payload option.
       */
      const payload = words[index + 1];
      if (payload !== undefined) {
        payloads.push(payload,);
        index += 1;
      }
      continue;
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
 
 @param payloads - collected command lines wrappers split and run
 
 @returns words starting at the program the shell ultimately executes
 
 @example
 ```ts
 unwrapWrappers({ words: ['env', 'X=1', 'sudo', 'git', 'commit'], payloads: [] });
 // ['git', 'commit']
 ```
 */
function unwrapWrappers({
  words,
  payloads,
}: {
  readonly words: readonly string[];
  readonly payloads: string[];
},): readonly string[] {
  return (function unwrapLayers(): readonly string[] {
    /**
     Command words of the outermost layer still to strip.
     */
    let remaining = words;
    for (;;) {
      /**
       Words after dropping shell assignment prefixes.
       */
      const withoutAssignments = dropLeadingAssignments(remaining,);
      /**
       Command word that may name a wrapper.
       */
      const [first,] = withoutAssignments;
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
        return withoutAssignments;

      remaining = wrappedCommandWords({
        words: withoutAssignments,
        wrapper,
        payloads,
      },);
    }
  })();
}

//endregion Wrapper stripping

export {
  unwrapWrappers,
};
