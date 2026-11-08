/**
 Programs that run another program given after their own options.
 
 A guard that reads only the command name misses a mutation hidden behind
 `sudo`, `env`, `timeout`, or a shell script string,
 so each wrapper contributes the option words that belong to it.
 
 A wrapper also changes the environment of the program it launches:
 `command -p` searches the platform default path instead of the caller's,
 `env -i` drops the environment altogether,
 and `env -u` removes a named variable.
 Those effects are reported so a caller can tell whether that program still
 finds a wrapper binary the caller's path exposes.
 
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
   as `timeout` does with its duration.
   */
  readonly leadingWordsToSkip: number;
};

//endregion Types

//region Wrapper table

/**
 Empty option set reused by wrappers that take none.
 */
const NO_OPTIONS: ReadonlySet<string> = new Set();

/**
 Wrappers that agent command lines use to launch another program.
 */
const COMMAND_WRAPPERS: readonly CommandWrapper[] = [
  {
    names: new Set(['env',],),
    optionsWithValue: new Set([
      '-C',
      '-S',
      '-u',
      '--chdir',
      '--split-string',
      '--unset',
    ],),
    payloadOptions: new Set([
      '-S',
      '--split-string',
    ],),
    defaultSearchPathFlags: new Set([
      '--ignore-environment',
      '-i',
    ],),
    unsetVariableOptions: new Set([
      '--unset',
      '-u',
    ],),
    leadingWordsToSkip: 0,
  },
  {
    names: new Set(['command',],),
    optionsWithValue: NO_OPTIONS,
    payloadOptions: NO_OPTIONS,
    defaultSearchPathFlags: new Set(['-p',],),
    unsetVariableOptions: NO_OPTIONS,
    leadingWordsToSkip: 0,
  },
  {
    names: new Set([
      'doas',
      'sudo',
    ],),
    optionsWithValue: new Set([
      '-C',
      '-D',
      '-R',
      '-T',
      '-U',
      '-g',
      '-h',
      '-p',
      '-r',
      '-t',
      '-u',
      '--chdir',
      '--chroot',
      '--close-from',
      '--command-timeout',
      '--group',
      '--host',
      '--other-user',
      '--prompt',
      '--role',
      '--type',
      '--user',
    ],),
    payloadOptions: NO_OPTIONS,
    defaultSearchPathFlags: NO_OPTIONS,
    unsetVariableOptions: NO_OPTIONS,
    leadingWordsToSkip: 0,
  },
  {
    names: new Set([
      'builtin',
      'nohup',
      'setsid',
    ],),
    optionsWithValue: NO_OPTIONS,
    payloadOptions: NO_OPTIONS,
    defaultSearchPathFlags: NO_OPTIONS,
    unsetVariableOptions: NO_OPTIONS,
    leadingWordsToSkip: 0,
  },
  {
    names: new Set(['exec',],),
    optionsWithValue: new Set([
      '-a',
      '--argv0',
    ],),
    payloadOptions: NO_OPTIONS,
    defaultSearchPathFlags: NO_OPTIONS,
    unsetVariableOptions: NO_OPTIONS,
    leadingWordsToSkip: 0,
  },
  {
    names: new Set(['time',],),
    optionsWithValue: new Set([
      '-f',
      '-o',
      '--format',
      '--output',
    ],),
    payloadOptions: NO_OPTIONS,
    defaultSearchPathFlags: NO_OPTIONS,
    unsetVariableOptions: NO_OPTIONS,
    leadingWordsToSkip: 0,
  },
  {
    names: new Set(['nice',],),
    optionsWithValue: new Set([
      '-n',
      '--adjustment',
    ],),
    payloadOptions: NO_OPTIONS,
    defaultSearchPathFlags: NO_OPTIONS,
    unsetVariableOptions: NO_OPTIONS,
    leadingWordsToSkip: 0,
  },
  {
    names: new Set(['ionice',],),
    optionsWithValue: new Set([
      '-c',
      '-n',
      '-p',
      '-P',
      '-u',
      '--class',
      '--classdata',
      '--pgid',
      '--pid',
      '--uid',
    ],),
    payloadOptions: NO_OPTIONS,
    defaultSearchPathFlags: NO_OPTIONS,
    unsetVariableOptions: NO_OPTIONS,
    leadingWordsToSkip: 0,
  },
  {
    names: new Set(['stdbuf',],),
    optionsWithValue: new Set([
      '-e',
      '-i',
      '-o',
      '--error',
      '--input',
      '--output',
    ],),
    payloadOptions: NO_OPTIONS,
    defaultSearchPathFlags: NO_OPTIONS,
    unsetVariableOptions: NO_OPTIONS,
    leadingWordsToSkip: 0,
  },
  {
    names: new Set(['timeout',],),
    optionsWithValue: new Set([
      '-k',
      '-s',
      '--kill-after',
      '--signal',
    ],),
    payloadOptions: NO_OPTIONS,
    defaultSearchPathFlags: NO_OPTIONS,
    unsetVariableOptions: NO_OPTIONS,
    leadingWordsToSkip: 1,
  },
  {
    names: new Set(['xargs',],),
    optionsWithValue: new Set([
      '-E',
      '-I',
      '-L',
      '-P',
      '-a',
      '-d',
      '-n',
      '-s',
      '--arg-file',
      '--delimiter',
      '--eof',
      '--max-args',
      '--max-chars',
      '--max-lines',
      '--max-procs',
      '--process-slot-var',
      '--replace',
    ],),
    payloadOptions: NO_OPTIONS,
    defaultSearchPathFlags: NO_OPTIONS,
    unsetVariableOptions: NO_OPTIONS,
    leadingWordsToSkip: 0,
  },
];

//endregion Wrapper table

export type { CommandWrapper, };

export { COMMAND_WRAPPERS, };
