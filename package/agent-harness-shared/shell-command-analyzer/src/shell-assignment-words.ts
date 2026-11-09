/**
 Reads the shell variable assignments a command line carries.
 
 The parser already separates `X=1 cmd` assignments from the command,
 but a wrapper keeps them among its own arguments (`env`, `sudo`, `systemd-run -E`)
 and a bare assignment statement spells them as a command of its own,
 so those words are read here by the same rule.
 
 @module
 */

import { looksLikeExecutablePath, } from './executable-path-words.ts';

//region Constants

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

export {
  dropLeadingAssignments,
  readVariableAssignment,
};
export type { WrapperAssignment, };
