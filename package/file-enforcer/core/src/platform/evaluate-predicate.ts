import spawn from 'nano-spawn';

//region Types

/**
 Direct command with arguments, executed as {@link spawn}`(cmd, args)`.
 Exit code 0 = match, non-zero = no match.
 No shell involved; fully cross-platform.
 
 @example
 ```ts
 ['mise', '--version']
 ['notify-send', '--version']
 ```
 */
export type Predicate = readonly string[];

/**
 Command to execute when a predicate matches.
 
 - **Array of strings**: `[cmd, ...args]` tuple.
 - **{@link PlatformCommands}**: nested platform dispatch, evaluated recursively.
 
 Disambiguation at runtime: if the first element of the array is itself an array,
 it is treated as nested {@link PlatformCommands}. Otherwise it is a `[cmd, ...args]` tuple.
 
 @example
 ```ts
 // Command with arguments
 ['notify-send', '--urgency=critical', 'title', 'body']
 
 // Command with no arguments
 ['true']
 
 // Nested platform dispatch
 [
   [['mise', '--version'], ['mise', 'use', 'python']],
   [['apt-get', '--version'], ['apt-get', 'install', '--yes', 'python3']],
 ]
 ```
 */
export type Command = readonly string[] | PlatformCommands;

/**
 Ordered pair of predicate and command.
 The predicate is evaluated first; if it succeeds (exit 0), the command is executed.
 The command may itself be a {@link PlatformCommands} for nested dispatch.
 
 @example
 ```ts
 [['mise', '--version'], ['mise', 'exec', '--', 'git', 'pull']]
 [['brew', '--version'], ['brew', 'install', 'git']]
 ```
 */
export type PlatformEntry = readonly [
  predicate: Predicate,
  cmd: Command,
];

/**
 Ordered list of platform entries.
 Evaluated top-to-bottom; the first entry whose predicate succeeds wins.
 */
export type PlatformCommands = readonly PlatformEntry[];

//endregion Types

//region Evaluation

/**
 Evaluates a predicate by spawning it as a direct command.
 Returns `true` if the command exits with code 0, `false` otherwise.
 
 @param predicate - Command and arguments to evaluate
 
 @returns Whether the predicate succeeded (exit code 0)
 
 @example
 ```ts
 // Check if mise is installed
 await evaluatePredicate(['mise', '--version'])
 
 // Check if a file exists
 await evaluatePredicate(['ls', '/etc/os-release'])
 ```
 */
export async function evaluatePredicate(predicate: Predicate,): Promise<boolean> {
  try {
    /**
     Head/tail split of predicate so `spawn` receives executable plus argv separately.
     */
    const [cmd = '', ...args] = predicate;
    await spawn(
      cmd,
      args,
    );
    return true;
  }
  catch (predicateError: unknown) {
    void predicateError;
    return false;
  }
}

//endregion Evaluation
