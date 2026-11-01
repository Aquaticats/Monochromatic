/**
 Argument construction helpers for the `task-tsc` wrapper.
 
 Keeps TypeScript 7 parallelism controls isolated from diagnostic filtering so
 the CLI wrapper stays under the repo's per-file line budget.
 
 @example
 ```ts
 buildTscArgs({
   cliArgs: ['--build'],
   singleThreadedEnv: '1',
 });
 // ['--build', '--singleThreaded']
 ```
 */

//region Constants

/**
 Environment variable that asks `task-tsc` to pass TypeScript's `--singleThreaded` flag.
 
 Root mise fanout tasks set this at the outer orchestration boundary so package-local
 `lint:types` tasks do not multiply TypeScript 7 worker pools under mise's own
 package-level parallelism. Direct package-local invocations leave it unset.
 
 @example
 ```toml
 [tasks.lint.env]
 TSC_SINGLE_THREADED = "1"
 ```
 */
export const SINGLE_THREADED_ENV = 'TSC_SINGLE_THREADED';

/**
 TypeScript 7 flag that disables compiler-internal parallelism.
 
 @example
 ```bash
 tsc --singleThreaded --build
 ```
 */
const SINGLE_THREADED_FLAG = '--singleThreaded';

/**
 TypeScript build-mode flags that must remain first in the forwarded argv.
 
 @example
 ```bash
 tsc --build --singleThreaded
 ```
 */
const BUILD_MODE_FLAGS = new Set([
  '--build',
  '-b',
],);

/**
 Environment values that intentionally opt out of single-threaded injection.
 
 @example
 ```bash
 TSC_SINGLE_THREADED=false task-tsc --build
 ```
 */
const DISABLED_SINGLE_THREADED_VALUES = new Set([
  '0',
  'false',
  'no',
  'off',
],);

/**
 Default TypeScript arguments used when `task-tsc` receives no CLI arguments.
 
 @example
 ```bash
 task-tsc
 # forwards: tsc --build
 ```
 */
const DEFAULT_TSC_ARGS = ['--build',] as const;

//endregion Constants

//region Helpers

/**
 Tests whether caller arguments already decide TypeScript's single-threaded mode.
 
 @param args - arguments that will be forwarded to `tsc`
 
 @returns whether `--singleThreaded` already appears in flag or assignment form
 
 @example
 ```ts
 hasExplicitSingleThreadedFlag(['--singleThreaded', '--build']);
 // true
 ```
 */
function hasExplicitSingleThreadedFlag(args: readonly string[],): boolean {
  return args.some(function isSingleThreadedFlag(arg,): boolean {
    return (arg === SINGLE_THREADED_FLAG)
      || arg.startsWith(`${SINGLE_THREADED_FLAG}=`,);
  },);
}

/**
 Tests whether the environment requests TypeScript single-threaded mode.
 
 @param envValue - raw `TSC_SINGLE_THREADED` value
 
 @returns whether the value should cause flag injection
 
 @example
 ```ts
 isSingleThreadedEnvEnabled('1');
 // true
 isSingleThreadedEnvEnabled('false');
 // false
 ```
 */
function isSingleThreadedEnvEnabled(envValue: string,): boolean {
  /**
   Trimmed, case-normalized environment value used for boolean-like opt-out checks.
   */
  const normalizedValue = envValue.trim()
    .toLowerCase();

  if (normalizedValue.length
    === 0)
    return false;

  return !DISABLED_SINGLE_THREADED_VALUES.has(normalizedValue,);
}

/**
 Injects TypeScript's single-threaded flag while preserving build-mode argv rules.
 
 TypeScript requires `--build` to be the first command-line argument, so build-mode
 runs receive `--singleThreaded` immediately after the build flag.
 
 @param args - arguments that will be forwarded to `tsc`
 
 @returns arguments with `--singleThreaded` inserted at a tsc-valid position
 
 @example
 ```ts
 injectSingleThreadedFlag(['--build', '--noEmit']);
 // ['--build', '--singleThreaded', '--noEmit']
 ```
 */
function injectSingleThreadedFlag(args: readonly string[],): readonly string[] {
  /**
   First forwarded argument, or an empty sentinel when no argument exists.
   */
  const firstArg = args[0]
    ?? '';

  if (BUILD_MODE_FLAGS.has(firstArg,)) {
    return [
      firstArg,
      SINGLE_THREADED_FLAG,
      ...args.slice(1,),
    ];
  }

  return [
    SINGLE_THREADED_FLAG,
    ...args,
  ];
}

//endregion Helpers

//region Public API

/**
 Builds arguments forwarded to TypeScript.
 
 Defaults to `--build` when no CLI arguments are present, and injects
 `--singleThreaded` when `TSC_SINGLE_THREADED` is enabled unless the caller
 already supplied the flag explicitly.
 
 @param cliArgs - wrapper CLI arguments after the executable path
 
 @param singleThreadedEnv - raw `TSC_SINGLE_THREADED` value, omitted when unset
 
 @returns final arguments to pass to `tsc`
 
 @example
 ```ts
 buildTscArgs({
   cliArgs: ['--build'],
   singleThreadedEnv: '1',
 });
 // ['--build', '--singleThreaded']
 ```
 
 @internal
 */
export function buildTscArgs({
  cliArgs,
  singleThreadedEnv,
}: {
  readonly cliArgs: readonly string[];
  readonly singleThreadedEnv?: string;
},): readonly string[] {
  /**
   Caller arguments or the wrapper default, copied so returned arrays are independent.
   */
  const baseArgs = cliArgs.length
    > 0
    ? [...cliArgs,]
    : [...DEFAULT_TSC_ARGS,];

  if ((singleThreadedEnv === undefined)
    || (!isSingleThreadedEnvEnabled(singleThreadedEnv,))
    || hasExplicitSingleThreadedFlag(baseArgs,))
    return baseArgs;

  return injectSingleThreadedFlag(baseArgs,);
}

//endregion Public API
