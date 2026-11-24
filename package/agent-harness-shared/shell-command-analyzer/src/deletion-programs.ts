/**
 The programs whose deletions this repository holds to the literal-path rule.
 
 The rule is about what a deletion does rather than about one program's name:
 `rm`, `rmdir`, `unlink` and `shred` each delete a path their own operands
 name, and `find -delete` deletes the paths its expression matched, which no
 operand ever spells.
 
 Each program keeps the options that take the next word as a value,
 so a value is never read as a path that would be deleted.
 
 @module
 */

import { lastPathSegment, } from './executable-path-words.ts';

//region Constants

/**
 Sentinel returned when a command word names no program that deletes.
 */
const NO_DELETION_PROGRAM: unique symbol = Symbol('word names a program that deletes nothing',);

/**
 Word that ends a program's own options, after which every word is a target.
 */
const OPTIONS_TERMINATOR = '--';

/**
 Character that starts one of a program's own options.
 */
const OPTION_PREFIX = '-';

/**
 Program whose expression can delete the paths it matched.
 */
const FIND_PROGRAM_NAME = 'find';

/**
 Word that asks `find` to delete what it matched.
 */
const FIND_DELETE_ACTION = '-delete';

//endregion Constants

//region Types

/**
 One program whose operands name paths it deletes.
 */
type DeletionProgram = {
  /**
   Executable names that introduce the program.
   */
  readonly names: ReadonlySet<string>;
  /**
   Options whose value is the next word rather than a target.
   */
  readonly optionsWithValue: ReadonlySet<string>;
};

/**
 One word a deletion program would delete, with the spelling it was written in.
 */
type DeletionTarget = {
  /**
   Value after quote removal, which is the path the shell would hand the program.
   */
  readonly value: string;
  /**
   Original shell spelling, which is what shows a computed path.
   */
  readonly sourceText: string;
};

//endregion Types

//region Table

/**
 Programs whose deletions the literal-path rule covers.
 */
const DELETION_PROGRAMS: readonly DeletionProgram[] = [
  {
    names: new Set(['rm',],),
    optionsWithValue: new Set(),
  },
  {
    names: new Set(['rmdir',],),
    optionsWithValue: new Set(),
  },
  {
    names: new Set(['unlink',],),
    optionsWithValue: new Set(),
  },
  {
    names: new Set(['shred',],),
    optionsWithValue: new Set([
      '-n',
      '-s',
      '--iterations',
      '--random-source',
      '--size',
    ],),
  },
];

//endregion Table

//region Reads

/**
 Reads the program one command word names, when it deletes.
 
 @param word - command word of a parsed record or a raw run
 
 @returns the program specification, or {@link NO_DELETION_PROGRAM} for any other word
 
 @example
 ```ts
 deletionProgramOf('/usr/bin/rmdir'); // the rmdir program
 deletionProgramOf('git'); // NO_DELETION_PROGRAM
 ```
 */
function deletionProgramOf(word: string,): DeletionProgram | typeof NO_DELETION_PROGRAM {
  /**
   Name the word runs, read by its last path segment so a path spelling matches too.
   */
  const name = lastPathSegment(word,);
  return DELETION_PROGRAMS
    .find(function programNamed(candidate,): boolean {
      return candidate
        .names
        .has(name,);
    },)
    ?? NO_DELETION_PROGRAM;
}

/**
 Reports whether one word is one of a program's own options rather than a target.
 
 @param word - word the program would read
 
 @returns whether the word is an option, where a lone dash is a target
 
 @example
 ```ts
 isOptionWord('-rf'); // true
 isOptionWord('-'); // false
 ```
 */
function isOptionWord(word: string,): boolean {
  return word.startsWith(OPTION_PREFIX,)
    && (word.length > OPTION_PREFIX.length);
}

/**
 Lists the words one deletion program would delete.
 
 @param words - the program's own words, without its name
 
 @param optionsWithValue - options whose value is the next word
 
 @returns targets in source order, with their original spellings
 
 @example
 ```ts
 deletionTargetsOf({ words: [{ value: '-rf', sourceText: '-rf' }, { value: '/x', sourceText: '/x' }], optionsWithValue: new Set() });
 ```
 */
function deletionTargetsOf({
  words,
  optionsWithValue,
}: {
  readonly words: readonly DeletionTarget[];
  readonly optionsWithValue: ReadonlySet<string>;
},): readonly DeletionTarget[] {
  /**
   Targets read so far.
   */
  const targets: DeletionTarget[] = [];
  /**
   Whether the program's own options have ended.
   */
  let optionsEnded = false;
  for (let index = 0; index < words.length; index += 1) {
    /**
     Word inspected as option, option value, or target.
     */
    const word = words[index];
    if (word === undefined)
      break;
    if (!optionsEnded) {
      if (word.value === OPTIONS_TERMINATOR) {
        optionsEnded = true;
        continue;
      }
      if (isOptionWord(word.value,)) {
        if (optionsWithValue.has(word.value,))
          index += 1;
        continue;
      }
    }
    targets.push(word,);
  }
  return targets;
}

/**
 Reports whether one command asks `find` to delete the paths it matched.
 
 `find -delete` deletes paths no operand ever spells, so the finding stands
 whatever the starting points are written as.
 
 @param words - innermost program words of one command line
 
 @returns whether the command runs `find -delete`
 
 @example
 ```ts
 findDeletesThePathsItFound(['find', '/tmp/x', '-delete']); // true
 findDeletesThePathsItFound(['find', '/tmp/x', '-print']); // false
 ```
 */
function findDeletesThePathsItFound(words: readonly string[],): boolean {
  return (lastPathSegment(words.at(0,)
    ?? '',) === FIND_PROGRAM_NAME)
    && (words.includes(FIND_DELETE_ACTION,));
}

//endregion Reads

export {
  DELETION_PROGRAMS,
  deletionProgramOf,
  deletionTargetsOf,
  FIND_DELETE_ACTION,
  findDeletesThePathsItFound,
  NO_DELETION_PROGRAM,
};
export type { DeletionProgram, };
export type { DeletionTarget, };
