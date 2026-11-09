/**
 Reads whether a command runs on a search path that still finds the Git wrapper.
 
 The wrapper this repository installs is reached through `PATH`.
 A command reaches the real Git binary instead when it assigns `PATH` a folder
 holding that binary, exports such an assignment in an earlier statement,
 removes `PATH` (`env -u PATH`), drops the environment (`env -i`),
 or searches the platform default path (`command -p`).
 
 Nothing here evaluates shell state: a value such as `$PATH` or `$(which git)`
 names no readable folder and never matches.
 
 @module
 */

import {
  delimiter,
} from 'node:path';

import {
  readVariableAssignment,
  type WrappedCommand,
  type WrapperAssignment,
} from './command-wrapper-words.ts';
import type { ShellCommandInfo } from './types.ts';

//region Constants

/**
 Shell variable that selects the executable search path.
 */
const SEARCH_PATH_VARIABLE = 'PATH';

/**
 Builtins whose arguments assign variables for the commands that follow them.
 
 `local` is absent on purpose: it assigns inside one function body,
 and a function body is analyzed as its own command list.
 */
const ASSIGNMENT_BUILTIN_NAMES: ReadonlySet<string> = new Set([
  'declare',
  'export',
  'readonly',
  'typeset',
],);

/**
 Command name the analyzer reports for a statement that assigns variables and runs nothing.
 */
const NO_COMMAND_NAME = '';

//endregion Constants

//region Directory comparison

/**
 Reads one directory spelling for comparison.
 
 @param path - directory as a shell word or as a resolved executable folder
 
 @returns spelling without trailing separators, keeping a root separator
 
 @example
 ```ts
 comparableDirectory('/usr/bin/'); // '/usr/bin'
 comparableDirectory('/'); // '/'
 ```
 */
function comparableDirectory(path: string,): string {
  /**
   Characters of the spelling, so the trailing separators can be dropped by index.
   */
  const characters = path.split('',);
  /**
   Offset of the last character that is not a path separator, or -1 when the spelling is separators only.
   */
  const lastKeptIndex = characters.findLastIndex(function isDirectoryCharacter(character,): boolean {
    return (character !== '/') && (character !== '\\');
  },);
  if (lastKeptIndex === (-1))
    return path;
  return characters
    .slice(
    0,
    lastKeptIndex + 1,
    )
    .join('',);
}

/**
 Reads a folder list in the comparison form search-path entries use.
 
 @param directories - folders as resolved executable folders
 
 @returns comparable folder spellings
 
 @example
 ```ts
 comparableDirectories(['/usr/bin/', '/opt/git/bin']); // Set { '/usr/bin', '/opt/git/bin' }
 ```
 */
function comparableDirectories(directories: readonly string[],): ReadonlySet<string> {
  return new Set(directories
    .map(function comparableSpelling(directory,): string {
      return comparableDirectory(directory,);
    },),);
}

/**
 Reports whether an assigned search path value names a folder that holds the real Git.
 
 Entries the shell only resolves at run time (`$PATH`, `~/bin`) name no readable
 folder here and never match: the guard reads command text, not shell state.
 
 @param value - PATH value as the command spelled it
 
 @param realGitDirectories - comparable folders that hold the real Git binary
 
 @returns whether any literal entry names one of those folders
 
 @example
 ```ts
 searchPathNamesRealGitDirectory({ value: '/usr/bin:/bin', realGitDirectories });
 ```
 */
function searchPathNamesRealGitDirectory({
  value,
  realGitDirectories,
}: {
  readonly value: string;
  readonly realGitDirectories: ReadonlySet<string>;
},): boolean {
  return value
    .split(delimiter,)
    .some(function namesRealGitFolder(entry,): boolean {
      /**
       An empty entry names the working directory, which no command text settles.
       */
      if (entry.length === 0)
        return false;
      return realGitDirectories.has(comparableDirectory(entry,),);
    },);
}

/**
 Reads assignments a command record carries, wherever it spells them.
 
 @param info - parsed command record
 
 @param wrapperAssignments - assignments the stripped wrapper layers carried
 
 @returns every assignment the record applies before its program runs
 
 @example
 ```ts
 commandAssignments({ info, wrapperAssignments: [] });
 ```
 */
function commandAssignments({
  info,
  wrapperAssignments,
}: {
  readonly info: ShellCommandInfo;
  readonly wrapperAssignments: readonly WrapperAssignment[];
},): readonly WrapperAssignment[] {
  return [
    ...info.envAssignments,
    ...wrapperAssignments,
  ];
}

/**
 Reports whether one command record sets the search path to a folder holding the real Git.
 
 A record covers its own program only, except for the two forms the shell keeps for
 later statements: a bare assignment statement (`PATH=/usr/bin`) and the assignment
 builtins (`export PATH=/usr/bin`).
 
 @param info - parsed command record
 
 @param wrapperAssignments - assignments the stripped wrapper layers carried
 
 @param realGitDirectories - comparable folders that hold the real Git binary
 
 @returns whether the record reaches the real Git through the search path
 
 @example
 ```ts
 recordReassignsSearchPath({ info, wrapperAssignments: [], realGitDirectories });
 ```
 */
function recordReassignsSearchPath({
  info,
  wrapperAssignments,
  realGitDirectories,
}: {
  readonly info: ShellCommandInfo;
  readonly wrapperAssignments: readonly WrapperAssignment[];
  readonly realGitDirectories: ReadonlySet<string>;
},): boolean {
  /**
   Assignments the record spells as its own arguments, as `export PATH=/usr/bin` does.
   */
  const builtinAssignments = ASSIGNMENT_BUILTIN_NAMES
    .has(info.name,)
    ? info.args
      .flatMap(function builtinAssignment(word,): readonly WrapperAssignment[] {
      /**
       Variable name and value the argument assigns, when it assigns anything.
       */
        const assignment = readVariableAssignment(word,);
        return (typeof assignment) === 'symbol'
          ? []
          : [assignment,];
      },)
    : [];

  return [
    ...commandAssignments({
      info,
      wrapperAssignments,
    },),
    ...builtinAssignments,
  ]
    .some(function assignsRealGitFolder(assignment,): boolean {
      if (assignment.name !== SEARCH_PATH_VARIABLE)
        return false;
      return searchPathNamesRealGitDirectory({
        value: assignment.value,
        realGitDirectories,
      },);
    },);
}

/**
 Reports whether one command record leaves its search-path assignment for later statements.
 
 @param info - parsed command record
 
 @param realGitDirectories - comparable folders that hold the real Git binary
 
 @returns whether later commands in the same script run with that search path
 
 @example
 ```ts
 statementReassignsSearchPath({ info, realGitDirectories });
 ```
 */
function statementReassignsSearchPath({
  info,
  realGitDirectories,
}: {
  readonly info: ShellCommandInfo;
  readonly realGitDirectories: ReadonlySet<string>;
},): boolean {
  /**
   Whether the record is a bare assignment statement rather than a command prefix.
   */
  const assignsWithoutCommand = info.name === NO_COMMAND_NAME;
  if ((!assignsWithoutCommand) && (!ASSIGNMENT_BUILTIN_NAMES.has(info.name,)))
    return false;
  return recordReassignsSearchPath({
    info,
    wrapperAssignments: [],
    realGitDirectories,
  },);
}

/**
 Reports whether one command runs on a search path that cannot find the wrapper.
 
 Three facts settle it: an enclosing assignment already moved the search path,
 a wrapper runs the program on the platform default path or with `PATH` removed,
 or this record assigns the search path itself.
 
 @param info - parsed command record
 
 @param wrapped - innermost program words and what the stripped wrappers revealed
 
 @param realGitDirectories - comparable folders that hold the real Git binary
 
 @param searchPathExcludesWrapper - whether the search path already excluded the wrapper before this record
 
 @returns whether the program this record runs searches a path without the wrapper
 
 @example
 ```ts
 commandExcludesWrapper({ info, wrapped, realGitDirectories, searchPathExcludesWrapper: false });
 ```
 */
function commandExcludesWrapper({
  info,
  wrapped,
  realGitDirectories,
  searchPathExcludesWrapper,
}: {
  readonly info: ShellCommandInfo;
  readonly wrapped: WrappedCommand;
  readonly realGitDirectories: ReadonlySet<string>;
  readonly searchPathExcludesWrapper: boolean;
},): boolean {
  if (searchPathExcludesWrapper)
    return true;
  if (wrapped.usesDefaultSearchPath)
    return true;
  if (wrapped.unsetVariables
    .includes(SEARCH_PATH_VARIABLE,))
    return true;
  return recordReassignsSearchPath({
    info,
    wrapperAssignments: wrapped.assignments,
    realGitDirectories,
  },);
}

//endregion Directory comparison

export {
  comparableDirectories,
  commandExcludesWrapper,
  statementReassignsSearchPath,
};
