/**
 Coverage-driver scenarios over the console sink: silent and verbose
 modes (environment, argv, and a browser `window`), warn suppression,
 same-level run grouping, the process stderr path for debug runs and both
 of its failure modes, a console method that throws, hosts whose `env`,
 `argv`, or `stderr` throw on access, verify declining for a missing
 method or a missing `queueMicrotask`, and the control-character
 neutralizer over the boundary corpus. Console methods are replaced with
 no-ops for the run so the gate's log stays readable.

 @module
 */

import {
  _neutralizeControlCharacters as neutralizeControlCharacters,
  type LogRecord,
  type Sink,
  sinks,
} from '@monochromatic-dev/logger-core/ts';

import {
  BOUNDARY_TOKENS,
  LEVELS,
} from './boundary-corpus.ts';
import {
  installGlobalValue,
  poisonProcessProperty,
} from './coverage-globals.ts';

//region Environment control

/**
 Sets or removes `MONOCHROMATIC_VERBOSE` for a scope; disposing restores
 the previous value or absence.

 @param value - Value to set; omitted removes the variable.

 @returns Disposable restoring the previous state.

 @example
 ```ts
 using _verbose = verboseOverride({ value: 'true' });
 ```
 */
function verboseOverride({ value, }: { readonly value?: string; },): Disposable {
  /**
   Previous value, absent when the variable was unset.
   */
  const previous: { current?: string; } = {};
  if (process.env
    .MONOCHROMATIC_VERBOSE
    !== undefined)
    previous.current = process.env
      .MONOCHROMATIC_VERBOSE;
  if (value === undefined)
    delete process.env
      .MONOCHROMATIC_VERBOSE;
  else
    process.env
      .MONOCHROMATIC_VERBOSE = value;
  return {
    [Symbol.dispose](): void {
      if (previous.current === undefined)
        delete process.env
          .MONOCHROMATIC_VERBOSE;
      else
        process.env
          .MONOCHROMATIC_VERBOSE = previous.current;
    },
  };
}

/**
 Sets or removes `MONOCHROMATIC_WARN` for a scope; disposing restores the
 previous value or absence.

 @param value - Value to set; omitted removes the variable.

 @returns Disposable restoring the previous state.

 @example
 ```ts
 using _noWarn = warnOverride({ value: 'false' });
 ```
 */
function warnOverride({ value, }: { readonly value?: string; },): Disposable {
  /**
   Previous value, absent when the variable was unset.
   */
  const previous: { current?: string; } = {};
  if (process.env
    .MONOCHROMATIC_WARN
    !== undefined)
    previous.current = process.env
      .MONOCHROMATIC_WARN;
  if (value === undefined)
    delete process.env
      .MONOCHROMATIC_WARN;
  else
    process.env
      .MONOCHROMATIC_WARN = value;
  return {
    [Symbol.dispose](): void {
      if (previous.current === undefined)
        delete process.env
          .MONOCHROMATIC_WARN;
      else
        process.env
          .MONOCHROMATIC_WARN = previous.current;
    },
  };
}

/**
 Appends `--verbose` to argv for a scope; disposing removes it.

 @returns Disposable removing the flag.

 @example
 ```ts
 using _flag = verboseArgv();
 ```
 */
function verboseArgv(): Disposable {
  process.argv
    .push('--verbose',);
  return {
    [Symbol.dispose](): void {
      process.argv
        .pop();
    },
  };
}

//endregion Environment control

//region Console control

/**
 Console methods the sink can call.
 */
const CONSOLE_METHODS = [
  'debug',
  'error',
  'info',
  'trace',
  'warn',
] as const;

/**
 Stands in for a console method during the run.
 */
function ignoreOutput(): void {
  // The gate reads coverage, not output.
}

/**
 Stands in for `process.stderr.write`, reporting success without writing.

 @returns Always `true`, the stream's "accepted" answer.
 */
function acceptQuietly(): boolean {
  return true;
}

/**
 Stands in for a console method that has been torn down.
 */
function explodeConsole(): void {
  throw new Error('console unavailable',);
}

/**
 Stands in for a closed stderr stream.

 @returns Never returns; always throws.
 */
function explodeStderr(): boolean {
  throw new Error('stderr closed',);
}

/**
 Shadows `process.stderr.write` with an own property for a scope, so the
 sink's stderr probe and write see the replacement; disposing removes the
 shadow so the prototype method shows through again.

 @param replacement - Value to expose as `write`; a non-function makes the
 sink's stderr probe report no stderr.

 @returns Disposable removing the shadow.

 @example
 ```ts
 using _closed = replaceStderrWrite({ replacement: explodeStderr });
 ```
 */
function replaceStderrWrite({ replacement, }: { readonly replacement?: unknown; },): Disposable {
  /**
   Own descriptor to restore, absent when `write` came from the prototype.
   */
  const own = Object.getOwnPropertyDescriptor(
    process.stderr,
    'write',
  );
  Object.defineProperty(
    process.stderr,
    'write',
    {
      configurable: true,
      value: replacement,
      writable: true,
    },
  );
  return {
    [Symbol.dispose](): void {
      if (own === undefined)
        Reflect.deleteProperty(
          process.stderr,
          'write',
        );
      else
        Object.defineProperty(
          process.stderr,
          'write',
          own,
        );
    },
  };
}

/**
 Replaces one console method with `value` for a scope; disposing restores
 the original.

 @param method - Console method name.

 @param value - Replacement; a non-function stages a stripped console.

 @returns Disposable restoring the original.

 @example
 ```ts
 using _broken = replaceConsoleMethod({ method: 'info', value: explodeConsole });
 ```
 */
function replaceConsoleMethod({
  method,
  value,
}: {
  readonly method: (typeof CONSOLE_METHODS)[number];
  readonly value?: unknown;
},): Disposable {
  /**
   Original method to restore.
   */
  const original: unknown = console[method];
  Object.defineProperty(
    console,
    method,
    {
      configurable: true,
      value,
      writable: true,
    },
  );
  return {
    [Symbol.dispose](): void {
      Object.defineProperty(
        console,
        method,
        {
          configurable: true,
          value: original,
          writable: true,
        },
      );
    },
  };
}

/**
 Replaces every console method the sink can call with a no-op and
 `process.stderr.write` with a no-op that reports success; disposing
 restores them.

 @returns Disposable restoring the originals.

 @example
 ```ts
 using _quiet = quietConsole();
 ```
 */
function quietConsole(): Disposable {
  /**
   Restorers for every replaced method, plus stderr.
   */
  const restorers = [
    ...CONSOLE_METHODS.map(function silence(method,): Disposable {
      return replaceConsoleMethod({
        method,
        value: ignoreOutput,
      },);
    },),
    replaceStderrWrite({ replacement: acceptQuietly, },),
  ];
  return {
    [Symbol.dispose](): void {
      for (const restorer of restorers.toReversed())
        restorer[Symbol.dispose]();
    },
  };
}

//endregion Console control

//region Records

/**
 One record per boundary token, cycling through every level.

 @returns Records in corpus order.

 @example
 ```ts
 corpusRecords().length; // corpus size
 ```
 */
function corpusRecords(): readonly LogRecord[] {
  return BOUNDARY_TOKENS.map(function toRecord(
    message,
    index,
  ): LogRecord {
    return {
      level: LEVELS[index % LEVELS.length] ?? 'info',
      message,
      timestamp: index,
    };
  },);
}

/**
 Records forming same-level runs, so grouping extends a run instead of
 opening one for every record.

 @returns Two records per level, adjacent.

 @example
 ```ts
 groupedRecords().length; // twice the level count
 ```
 */
function groupedRecords(): readonly LogRecord[] {
  return LEVELS.flatMap(function pair(level,): readonly LogRecord[] {
    return [
      {
        level,
        message: `${level} first`,
        timestamp: 0,
      },
      {
        level,
        message: `${level} second`,
        timestamp: 1,
      },
    ];
  },);
}

/**
 Writes records through a fresh console sink in one synchronous frame,
 then flushes it.

 @param records - Records to write; the corpus when omitted.

 @example
 ```ts
 await writeThrough({});
 ```
 */
async function writeThrough({ records = corpusRecords(), }: { readonly records?: readonly LogRecord[]; },): Promise<void> {
  /**
   Fresh sink so verbose detection runs again under the current
   environment.
   */
  const sink: Sink = sinks.createConsoleSink();
  await sink.verify();
  await Promise.all(records.map(function writeOne(record,): Promise<void> {
    return sink.write(record,);
  },),);
  if (sink.flush !== undefined)
    await sink.flush();
}

//endregion Records

//region Scenarios

/**
 Silent mode with warn suppressed.
 */
async function exerciseSilentMode(): Promise<void> {
  /**
   Verbose flag removed for the scope.
   */
  using _silent = verboseOverride({},);
  /**
   Warn suppression on for the scope.
   */
  using _noWarn = warnOverride({ value: 'false', },);
  await writeThrough({},);
}

/**
 Verbose mode through the environment, warn suppression off, with
 same-level runs so grouping extends a run.
 */
async function exerciseVerboseEnvironment(): Promise<void> {
  /**
   Verbose flag on for the scope.
   */
  using _verbose = verboseOverride({ value: 'true', },);
  /**
   Warn suppression removed for the scope.
   */
  using _warn = warnOverride({},);
  await writeThrough({},);
  await writeThrough({ records: groupedRecords(), },);
}

/**
 Verbose mode through `--verbose` on argv, then through a browser
 `window`, each with the environment flag removed.
 */
async function exerciseVerboseHosts(): Promise<void> {
  /**
   Verbose flag removed for the scope.
   */
  using _silent = verboseOverride({},);
  {
    /**
     Flag appended for the scope.
     */
    using _flag = verboseArgv();
    await writeThrough({},);
  }
  {
    /**
     Browser window staged for the scope.
     */
    using _window = installGlobalValue({
      name: 'window',
      value: {},
    },);
    await writeThrough({},);
  }
}

/**
 A console method that throws, stderr that throws, and stderr that is
 absent, each in verbose mode so debug records take the stderr path.
 */
async function exerciseOutputFailures(): Promise<void> {
  /**
   Verbose flag on for every failure scenario.
   */
  using _verbose = verboseOverride({ value: 'true', },);
  {
    /**
     `console.info` torn down for the scope.
     */
    using _broken = replaceConsoleMethod({
      method: 'info',
      value: explodeConsole,
    },);
    await writeThrough({},);
  }
  {
    /**
     stderr closed for the scope.
     */
    using _closed = replaceStderrWrite({ replacement: explodeStderr, },);
    await writeThrough({},);
  }
  {
    /**
     stderr absent for the scope.
     */
    using _absent = replaceStderrWrite({},);
    await writeThrough({},);
  }
}

/**
 Hosts whose `process.env`, `process.argv`, or `process.stderr` throw on
 access: every probe takes its reporting path and the sink keeps working.
 */
async function exerciseHostileHosts(): Promise<void> {
  {
    /**
     `process.env` poisoned for the scope.
     */
    using _env = poisonProcessProperty({ name: 'env', },);
    await writeThrough({},);
  }
  {
    /**
     `process.argv` poisoned for the scope.
     */
    using _argv = poisonProcessProperty({ name: 'argv', },);
    await writeThrough({},);
  }
  {
    /**
     Verbose flag on so a debug run reaches the stderr probe.
     */
    using _verbose = verboseOverride({ value: 'true', },);
    /**
     `process.stderr` poisoned for the scope.
     */
    using _stderr = poisonProcessProperty({ name: 'stderr', },);
    await writeThrough({},);
  }
}

/**
 Verify declines when the sample console method is not callable and when
 `queueMicrotask` is missing.
 */
async function exerciseVerifyDeclines(): Promise<void> {
  {
    /**
     `console.info` stripped for the scope.
     */
    using _stripped = replaceConsoleMethod({ method: 'info', },);
    await sinks.createConsoleSink()
      .verify();
  }
  {
    /**
     `queueMicrotask` removed for the scope.
     */
    using _noMicrotask = installGlobalValue({ name: 'queueMicrotask', },);
    await sinks.createConsoleSink()
      .verify();
  }
}

//endregion Scenarios

/**
 Runs every console scenario with console output silenced, then the
 neutralizer over the corpus.

 @example
 ```ts
 await exerciseConsoleSink();
 ```
 */
export async function exerciseConsoleSink(): Promise<void> {
  /**
   Console output silenced for every scenario.
   */
  using _quiet = quietConsole();
  await exerciseSilentMode();
  await exerciseVerboseEnvironment();
  await exerciseVerboseHosts();
  await exerciseOutputFailures();
  await exerciseHostileHosts();
  await exerciseVerifyDeclines();
  for (const token of BOUNDARY_TOKENS)
    neutralizeControlCharacters(token,);
}
