#!/usr/bin/env node

/**
 `watch-restart` CLI entrypoint.
 
 TODO: deprecate Optique
 Parses command-line flags via optique, maps them onto
 {@link StartWatchRestartOptions}, then hands control to
 {@link startWatchRestart}. On SIGINT or SIGTERM the orchestrator's
 `stop()` runs and the process exits.
 
 Importing this module is side-effect-free; the program body only
 runs when this file is executed as the entrypoint (gated by
 `import.meta.main`), so unit tests can import {@link parseArgs} /
 {@link argsToOptions} without launching a real watch loop.
 
 @example
 ```bash
 watch-restart -w src/server -- node src/server/index.ts
 watch-restart -w src --ext .ts --exclude '*.test.ts' -- npm run dev
 watch-restart -w src --no-initial; npm test
 ```
 */

// TODO: deprecate Optique
import { object, } from '@optique/core/constructs';
// TODO: deprecate Optique
import {
  multiple,
  optional,
} from '@optique/core/modifiers';
// TODO: deprecate Optique
import {
  argument,
  option,
} from '@optique/core/primitives';
// TODO: deprecate Optique
import {
  integer,
  string,
} from '@optique/core/valueparser';
// TODO: deprecate Optique
import { runSync, } from '@optique/run';
import {
  cliEventToInternal,
  compileRegex,
  FLAG_UNSET,
  parseKillSignal,
  parseTypeToken,
  resolveBoolPair,
  splitCommas,
} from './cli-helpers.ts';
import { installShutdownHandler, } from './cli-shutdown.ts';
import type { ParsedArgs, } from './cli-types.ts';
import {
  startWatchRestart,
  type StartWatchRestartOptions,
} from './start.ts';
import type {
  WatchEntityType,
  WatchEventKind,
} from './types.ts';

/**
 TODO: deprecate Optique
 Module-internal optique parser. Built once at module load. Not
 exported because spelling its inferred type without leaking
 optique-internal generics is impractical; consumers go through
 {@link parseArgs} which exposes the {@link ParsedArgs} shape.
 */
const parser = object({
  watch: multiple(option(
    '-w',
    '--watch',
    string(),
  ),),
  include: multiple(option(
    '-i',
    '--include',
    string(),
  ),),
  exclude: multiple(option(
    '-e',
    '--exclude',
    string(),
  ),),
  includeRegex: multiple(option(
    '--include-regex',
    string(),
  ),),
  excludeRegex: multiple(option(
    '--exclude-regex',
    string(),
  ),),
  ext: multiple(option(
    '--ext',
    string(),
  ),),
  type: multiple(option(
    '--type',
    string(),
  ),),
  events: optional(option(
    '--events',
    string(),
  ),),
  hidden: option('--hidden',),
  noHidden: option('--no-hidden',),
  followSymlinks: option('--follow-symlinks',),
  noFollowSymlinks: option('--no-follow-symlinks',),
  gitignore: option('--gitignore',),
  noGitignore: option('--no-gitignore',),
  ignoreFile: multiple(option(
    '--ignore-file',
    string(),
  ),),
  depth: optional(option(
    '--depth',
    integer(),
  ),),
  poll: optional(option(
    '--poll',
    integer(),
  ),),
  noContentChanged: option('--no-content-changed',),
  maxHashSize: optional(option(
    '--max-hash-size',
    integer(),
  ),),
  debounce: optional(option(
    '--debounce',
    integer(),
  ),),
  stopTimeout: optional(option(
    '--stop-timeout',
    integer(),
  ),),
  noInitial: option('--no-initial',),
  clear: option('--clear',),
  noClear: option('--no-clear',),
  signal: optional(option(
    '--signal',
    string(),
  ),),
  processGroup: option('--process-group',),
  noProcessGroup: option('--no-process-group',),
  rest: multiple(
    argument(string(),),
  ),
},);

/**
 TODO: deprecate Optique
 Runs the optique parser against a synthetic argv and returns the
 {@link ParsedArgs} shape.
 
 `onExit` / `stdout` / `stderr` are overridable so tests can capture
 help text and trap parse-error exits; production callers omit them
 to inherit the optique defaults (`process.exit`, `process.stdout`).
 
 @param options - argv to parse plus optional output / exit hooks
 
 @returns parsed args matching {@link ParsedArgs}
 
 @mutates options through https://github.com/dahlia/optique/blob/b8d39082fdeb37bb16c68b2dc2396d4c9c45b1d5/packages/run/src/run.ts runSync output and exit callback capabilities
 
 @example
 ```ts
 const args = parseArgs({ argv: process.argv.slice(2,), },);
 ```
 */
export function parseArgs(
  options: {
    readonly argv: readonly string[];
    readonly onExit?: (code: number,) => never;
    readonly stdout?: (text: string,) => void;
    readonly stderr?: (text: string,) => void;
  },
): ParsedArgs {
  return runSync(
    parser,
    {
      programName: 'watch-restart',
      args: options.argv,
      help: 'option',
      ...(options.onExit
        === undefined ? {} : { onExit: options.onExit, }),
      ...(options.stdout
        === undefined ? {} : { stdout: options.stdout, }),
      ...(options.stderr
        === undefined ? {} : { stderr: options.stderr, }),
    },
  );
}

/**
 Maps the parsed CLI args onto a {@link StartWatchRestartOptions}.
 
 Pure transformation: no I/O, no global state, no defaults beyond what
 the orchestrator itself applies. `--no-content-changed` becomes
 `contentChanged: false`; absence stays absent (the orchestrator's
 default-true kicks in). Same for `--no-initial`. Pair flags
 (`--hidden`/`--no-hidden`, etc.) collapse to a tri-state via
 {@link resolveBoolPair}; both-passed is a usage error.
 
 The first positional after `--` becomes `command`; the remainder
 becomes `args`. An empty `rest` is a CLI usage error and throws so
 the user sees the cause instead of an opaque "spawn EINVAL" later.
 
 `--ext`, `--events`, and `--type` accept comma-lists; each is split
 and trimmed before mapping. `--include-regex` / `--exclude-regex`
 compile to {@link RegExp} (invalid patterns throw a `SyntaxError`).
 `--signal` validates against {@link parseKillSignal}'s allowed set.
 
 @param args - shape returned by {@link parseArgs}
 
 @returns options object handed straight to {@link startWatchRestart}
 
 @throws Error when no positional command is given after `--`, or when
 any token / regex / signal name fails its respective validator
 
 @example
 ```ts
 const args = parseArgs({ argv: ['-w', 'src', '--', 'node',], },);
 const options = argsToOptions(args,);
 ```
 */
export function argsToOptions(args: ParsedArgs,): StartWatchRestartOptions {
  if (args.rest
    .length
    === 0) {
    throw new Error(
      'No command supplied after "--"; usage: watch-restart -w <dir>... -- <cmd> [<args>...]',
    );
  }
  /**
   Positional split: first non-option after `--` is the command; the rest is its argv.
   */
  const [command, ...commandArgs] = args.rest;
  if ((command === undefined) || (command === '')) {
    throw new Error(
      'Empty command after "--"; usage: watch-restart -w <dir>... -- <cmd> [<args>...]',
    );
  }

  /**
   Flattened, comma-split extension list.
   */
  const extensions: readonly string[] = args.ext
    .flatMap(
    function flattenExt(raw,): string[] {
      return splitCommas(raw,);
    },
    );
  /**
   Flattened, comma-split type list mapped to internal entity types.
   */
  const types: readonly WatchEntityType[] = args
    .type
    .flatMap(
    function flattenType(raw,): string[] {
      return splitCommas(raw,);
    },
    )
    .map(function mapTypeToken(token,): WatchEntityType {
      return parseTypeToken(token,);
    },);
  /**
   Flattened, comma-split event kind list; translated from
   CLI-facing `create`/`delete` to internal `add`/`unlink`.
   Inferred `readonly WatchEventKind[] | undefined` (no annotation, so no
   nullish-union type node); `undefined` when `--events` was not passed.
   */
  const events = args.events
    === undefined
    ? undefined
    : splitCommas(args.events,)
      .map(function mapEventToken(token,): WatchEventKind {
        return cliEventToInternal(token,);
      },);
  /**
   Compiled include regex list; throws here if any pattern is invalid.
   */
  const includeRegex: readonly RegExp[] = args.includeRegex
    .map(
    function mapIncludeRegex(pattern,): RegExp {
      return compileRegex(pattern,);
    },
    );
  /**
   Compiled exclude regex list; throws here if any pattern is invalid.
   */
  const excludeRegex: readonly RegExp[] = args.excludeRegex
    .map(
    function mapExcludeRegex(pattern,): RegExp {
      return compileRegex(pattern,);
    },
    );
  /**
   Tri-state hidden toggle; both-passed throws inside resolveBoolPair.
   */
  const hidden = resolveBoolPair({
    positive: args.hidden,
    negative: args.noHidden,
    flag: 'hidden',
  },);
  /**
   Tri-state symlink-follow toggle.
   */
  const followSymlinks = resolveBoolPair({
    positive: args.followSymlinks,
    negative: args.noFollowSymlinks,
    flag: 'follow-symlinks',
  },);
  /**
   Tri-state gitignore toggle.
   */
  const gitignore = resolveBoolPair({
    positive: args.gitignore,
    negative: args.noGitignore,
    flag: 'gitignore',
  },);
  /**
   Tri-state terminal-clear toggle.
   */
  const clear = resolveBoolPair({
    positive: args.clear,
    negative: args.noClear,
    flag: 'clear',
  },);
  /**
   Tri-state process-group toggle.
   */
  const processGroup = resolveBoolPair({
    positive: args.processGroup,
    negative: args.noProcessGroup,
    flag: 'process-group',
  },);
  /**
   Validated kill signal (or `undefined` when --signal was not passed).
   */
  const killSignal = args.signal
    === undefined
    ? undefined
    : parseKillSignal(args.signal,);

  return {
    paths: args.watch,
    command,
    ...(commandArgs.length
      > 0 ? { args: commandArgs, } : {}),
    ...(args.include
      .length
      > 0 ? { include: args.include, } : {}),
    ...(args.exclude
      .length
      > 0 ? { exclude: args.exclude, } : {}),
    ...(includeRegex.length
      > 0 ? { includeRegex, } : {}),
    ...(excludeRegex.length
      > 0 ? { excludeRegex, } : {}),
    ...(extensions.length
      > 0 ? { extensions, } : {}),
    ...(types.length
      > 0 ? { types, } : {}),
    ...(events === undefined ? {} : { events, }),
    ...(hidden === FLAG_UNSET ? {} : { hidden, }),
    ...(followSymlinks === FLAG_UNSET ? {} : { followSymlinks, }),
    ...(gitignore === FLAG_UNSET ? {} : { gitignore, }),
    ...(args.ignoreFile
      .length
      > 0 ? { ignoreFiles: args.ignoreFile, } : {}),
    ...(args.depth
      === undefined ? {} : { depth: args.depth, }),
    ...(args.poll
      === undefined ? {} : { poll: args.poll, }),
    ...(args.noContentChanged ? { contentChanged: false, } : {}),
    ...(args.maxHashSize
      === undefined ? {} : { maxHashSize: args.maxHashSize, }),
    ...(args.debounce
      === undefined ? {} : { debounce: args.debounce, }),
    ...(args.stopTimeout
      === undefined
      ? {}
      : { stopTimeout: args.stopTimeout, }),
    ...(args.noInitial ? { initial: false, } : {}),
    ...(clear === FLAG_UNSET ? {} : { clear, }),
    ...(killSignal === undefined ? {} : { killSignal, }),
    ...(processGroup === FLAG_UNSET ? {} : { processGroup, }),
  };
}

if (import.meta.main) {
  /**
   Parsed argv from `process.argv.slice(2)`.
   */
  const args = parseArgs({ argv: process.argv
    .slice(2,), },);
  /**
   Mapped options handed to the orchestrator.
   */
  const options = argsToOptions(args,);
  /**
   Live handle; both signals route through it during shutdown.
   */
  const handle = await startWatchRestart(options,);
  installShutdownHandler({
    signal: 'SIGINT',
    handle,
  },);
  installShutdownHandler({
    signal: 'SIGTERM',
    handle,
  },);
}
