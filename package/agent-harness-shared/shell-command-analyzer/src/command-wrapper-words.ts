/**
 Strips program wrappers from a command word list.
 
 `env X=1 sudo timeout 60 git commit` runs one Git command,
 and the guard has to read it as that,
 not as an `env` invocation with arguments.
 
 The stripped layers are reported rather than discarded:
 a caller needs the assignments they carried,
 the command lines they split and run,
 and whether they left the program on a search path other than the caller's.
 
 A wrapper that hands the program to another host reports that too,
 because the caller's search path and its installed wrappers never reach it.
 
 Programs that run a command named among their own arguments (`find -exec`)
 contribute each of those command lines as well.
 
 @module
 */

import { COMMAND_WRAPPERS, } from './command-wrappers.ts';
import { lastPathSegment, } from './executable-path-words.ts';
import { nestedCommandLines, } from './nested-command-lines.ts';
import {
  dropLeadingAssignments,
  type WrapperAssignment,
} from './shell-assignment-words.ts';
import {
  type WrapperScan,
  wrappedCommandWords,
} from './wrapper-word-scan.ts';

//region Types

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
  /**
   Whether a wrapper runs the program on another host,
   whose search path this command text does not carry.
   */
  readonly reachesAnotherHost: boolean;
};

//endregion Types

//region Wrapper stripping

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
    reachesAnotherHost: false,
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

      if (wrapper.launchesOnAnotherHost === true)
        scan.reachesAnotherHost = true;

      remaining = wrappedCommandWords({
        words: withoutAssignments.words,
        wrapper,
        scan,
      },);
    }
  })();

  /**
   Command lines the innermost program runs from among its own arguments,
   as `find -exec` does.
   */
  const nestedLines = nestedCommandLines({ words: unwrapped, },);

  return {
    words: unwrapped,
    payloads: [
      ...scan.payloads,
      ...nestedLines,
    ],
    assignments: scan.assignments,
    usesDefaultSearchPath: scan.usesDefaultSearchPath,
    unsetVariables: scan.unsetVariables,
    reachesAnotherHost: scan.reachesAnotherHost,
  };
}

//endregion Wrapper stripping

export { unwrapWrappers, };
export type { WrappedCommand, };
