#!/usr/bin/env node

/**
 Wrapper for `tsc` that filters out diagnostics from known false-positive sources.
 
 Suppressed sources:
 - `node_modules`: JSR packages ship `.ts` source files instead of `.d.ts` declarations.
   TypeScript's resolver prefers `.ts` siblings over `.js` exports,
   and `skipLibCheck` only covers `.d.ts` files.
   This causes `tsc --build` to type-check JSR package source
   under the consumer's tsconfig, producing false positives.
 - Auto-generated typesafe-i18n files (`i18n/i18n-*.ts`): these violate
   `--isolatedDeclarations` and carry "manual changes will be overwritten" headers.
 
 This wrapper:
 1. Runs `tsc` with all provided arguments (defaults to `--build` if none given)
 2. Captures stdout/stderr
 3. Drops diagnostic lines from suppressed sources
 4. Drops continuation lines (indented lines following a dropped diagnostic)
 5. Preserves process failure unless known diagnostic suppression fully accounts for it
 
 See `doc/troubleshooting/typescript.md` section
 "JSR packages ship `.ts` source files that `skipLibCheck` cannot skip"
 for full root cause analysis.
 
 @example
 ```bash
 task-tsc --build
 task-tsc --build --noEmit
 task-tsc --noEmit -p tsconfig.json
 ```
 */

import {
  glob,
  unlink,
} from 'node:fs/promises';

import spawn from 'nano-spawn';
import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';

import {
  buildTscArgs,
  SINGLE_THREADED_ENV,
} from './tsc-args.ts';
import { filterTscOutput, } from './tsc-output-filter.ts';
import { isFullySuppressedCompilerFailure } from './tsc-suppressed-failure.ts';

//region Incremental cache cleanup

/**
 Glob patterns for TypeScript incremental caches that `task-tsc` refreshes before each run.
 */
const BUILD_INFO_GLOBS = [
  'dist/**/*.tsbuildinfo',
  '.cache/typescript/**/*.tsbuildinfo',
] as const;

/**
 Removes all `.tsbuildinfo` files emitted by TypeScript in the current working directory.
 
 `composite: true` implies `incremental: true`, which produces `.tsbuildinfo` caches.
 tsc's `--build` mode has a cache invalidation bug (#2666) where stale `.tsbuildinfo`
 files cause false negatives after dependency updates. Deleting them before each build
 forces a clean check while preserving all other `composite` benefits
 (rootDir defaulting, include enforcement, declaration defaulting).
 
 The root tsconfig redirects build metadata to `.cache/typescript/root.tsbuildinfo`
 so root type checks do not create an ignored `dist/` directory just to hold metadata.
 
 @example
 ```ts
 await removeStaleBuildInfo();
 // All configured build-info cache files in cwd are now deleted
 ```
 */
async function removeStaleBuildInfo(): Promise<void> {
  /**
   Buffered tsbuildinfo paths collected from every configured async glob; unlinked concurrently below.
   */
  const entries = (await Promise.all(
    BUILD_INFO_GLOBS.map(function collectBuildInfo(pattern,): Promise<string[]> {
      return Array.fromAsync(glob(pattern,),);
    },),
  ))
    .flat();
  await Promise.all(entries.map(function unlinkEntry(entry,) {
    return unlink(entry,);
  },),);
}

//endregion Incremental cache cleanup

//region Main execution

/**
 Runs the `task-tsc` command-line wrapper.
 
 Clears stale incremental caches via {@link removeStaleBuildInfo}, runs `tsc`,
 and on failure filters the captured output through {@link filterTscOutput}
 before deciding the exit code.
 
 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  /**
   Raw single-threaded env request inherited from root mise fanout, when present.
   */
  const singleThreadedEnv = process.env[SINGLE_THREADED_ENV];

  /**
   Arguments forwarded to tsc, with wrapper defaults and root-fanout controls applied.
   */
  const tscArgs = buildTscArgs({
    cliArgs: process.argv
      .slice(2,),
    ...((singleThreadedEnv === undefined)
      ? {}
      : { singleThreadedEnv, }),
  },);

  // tsc #2666: stale .tsbuildinfo causes false negatives; clean before each build
  await removeStaleBuildInfo();

  try {
    /**
     Successful spawn result; stdout/stderr are forwarded unfiltered when tsc exits 0.
     */
    const result = await spawn(
      'tsc',
      [...tscArgs,],
    );

    // tsc succeeded (exit 0): pass output through unfiltered
    if (result.stdout
      .length
      > 0)
      process.stdout
        .write(result.stdout,);
    if (result.stderr
      .length
      > 0)
      process.stderr
        .write(result.stderr,);
  }
  catch (error) {
    if (
      (error !== null)
        && ((typeof error) === 'object')
        && ('exitCode' in error)
    ) {
      /* oxlint-disable typescript/no-unsafe-type-assertion -- 'exitCode' in check above narrows to subprocess shape */
      /**
       Subprocess failure narrowed to the shape exposed by the bun/node spawn libraries; carries the streams to filter.
       */
      const subprocessError = error as {
        stdout?: string;
        stderr?: string;
        exitCode?: number;
        signalName?: string;
      };
      /* oxlint-enable typescript/no-unsafe-type-assertion */

      /**
       Filtered stdout payload with low-value tsc diagnostics suppressed; written below when non-empty.
       */
      // Filter stdout (where tsc writes diagnostics)
      const stdoutResult = filterTscOutput(subprocessError.stdout
        ?? '',);
      /**
       Filtered stderr payload with low-value tsc diagnostics suppressed; written below when non-empty.
       */
      // Filter stderr as well in case tsc writes diagnostics there
      const stderrResult = filterTscOutput(subprocessError.stderr
        ?? '',);

      if (stdoutResult.filtered
        .length
        > 0) {
        process.stdout
          .write(stdoutResult.filtered,);
        // Ensure trailing newline for clean terminal output
        if (!stdoutResult.filtered
          .endsWith('\n',))
          process.stdout
            .write('\n',);
      }

      if (stderrResult.filtered
        .length
        > 0) {
        process.stderr
          .write(stderrResult.filtered,);
        if (!stderrResult.filtered
          .endsWith('\n',))
          process.stderr
            .write('\n',);
      }

      // A crash or unexplained failure is not a successful type check merely because it has no TS error header.
      if (!isFullySuppressedCompilerFailure({
        stdout: subprocessError.stdout ?? '',
        stderr: subprocessError.stderr ?? '',
        filteredStdout: stdoutResult.filtered,
        filteredStderr: stderrResult.filtered,
      }))
        process.exitCode = subprocessError.exitCode
          ?? 1;

      if ((subprocessError.signalName
        !== undefined)
        && (subprocessError.signalName
          !== ''))
        {
        console.error(
          `[task-tsc] tsc terminated by signal: ${subprocessError.signalName}`,
        );
        process.exitCode = 1;
      }
    }
    else {
      // Non-subprocess error (e.g. tsc not found)
      console.error(
        `[task-tsc] failed to execute tsc: ${
          caughtValueText(error,)
        }`,
      );
      process.exitCode = 1;
    }
  }
}

if (import.meta.main)
  await main();

//endregion Main execution
