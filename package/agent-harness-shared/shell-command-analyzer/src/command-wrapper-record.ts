/**
 One program that runs another program named in its arguments.
 
 A guard that reads only the command name misses a mutation hidden behind
 `sudo`, `env`, `timeout`, or a shell script string,
 so each wrapper contributes the option words that belong to it.
 
 The record also carries what the wrapper does to the program it launches:
 which search path it searches,
 which variables it removes,
 and whether the program runs on another host at all.
 
 @module
 */

//region Types

/**
 Program that runs another program given after its own options.
 */
type CommandWrapper = {
  /**
   Executable names that introduce the wrapper.
   */
  readonly names: ReadonlySet<string>;
  /**
   Options whose value is the next word and belongs to the wrapper.
   */
  readonly optionsWithValue: ReadonlySet<string>;
  /**
   Options whose value is a command line the wrapper splits and runs.
   */
  readonly payloadOptions: ReadonlySet<string>;
  /**
   Options that run the wrapped program on the platform default search path
   instead of the caller's, as `command -p` and `env -i` do.
   */
  readonly defaultSearchPathFlags: ReadonlySet<string>;
  /**
   Options whose value names a variable the wrapper removes from the wrapped
   program's environment, as `env -u` does.
   */
  readonly unsetVariableOptions: ReadonlySet<string>;
  /**
   Leading non-option words consumed before the wrapped program,
   as `flock` does with its lock file.
   */
  readonly leadingWordsToSkip: number;
  /**
   Whether one word before the program may be a number the wrapper owns,
   as `taskset`'s affinity mask or `chrt`'s priority.
   Without this the mask or priority reads as the program name,
   and the command behind it stays unread.
   */
  readonly numericWordBeforeProgram?: boolean;
  /**
   Whether the words after the wrapper's options are one shell source line
   rather than the program and its arguments,
   as `eval` joins its arguments and `watch` hands them to `sh -c`.
   */
  readonly joinsRemainderAsShellSource?: boolean;
  /**
   Whether the wrapped program runs on another host,
   whose search path and installed wrappers this command text does not carry.
   `ssh` reports the spellings it proves, an absolute path in particular,
   and never inherits the caller's search path decision.
   */
  readonly launchesOnAnotherHost?: boolean;
};

//endregion Types

//region Constants

/**
 Empty option set reused by wrappers that take none.
 */
const NO_OPTIONS: ReadonlySet<string> = new Set();

//endregion Constants

export type { CommandWrapper, };
export { NO_OPTIONS, };
