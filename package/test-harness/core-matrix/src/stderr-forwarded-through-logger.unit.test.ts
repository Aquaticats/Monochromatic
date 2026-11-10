/**
 The container and host backends hand a successful run's stderr to the caller's logger, so a consumer can capture,
 tag and silence it, and hand the run's stdout back as the return value (issue #1418).

 `runHost` runs this process's own `node` against a scratch script, and `runContainer` runs a scratch `podman` script
 that stands in for the real one earlier on `PATH`, so neither test starts a container.

 @module
 */

import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  chmod,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  delimiter,
  join,
} from 'node:path';

import {
  runContainer,
  runHost,
} from '../dist/final/node/index.mjs';

/**
 Unique prefix avoids collisions with parallel test runs.
 */
const TEST_DIR_PREFIX = 'matrix-stderr-logging-test';

/**
 Script a fixture run executes: one line on each stream, so the test can tell which stream reached which place.
 */
const BOTH_STREAMS_SCRIPT = [
  "process.stdout.write('matrix fixture stdout',);",
  "process.stderr.write('matrix fixture stderr',);",
  '',
].join('\n',);

/**
 Script of a fixture run that writes no stderr, so a quiet run shows what the logger records when nothing is wrong.
 */
const STDOUT_ONLY_SCRIPT = [
  "process.stdout.write('matrix fixture stdout',);",
  '',
].join('\n',);

/**
 Disposable scratch directory that cleans itself up via `Symbol.asyncDispose`.
 */
type ScratchDir = {
  /** Absolute path to the scratch directory */
  readonly path: string;
  [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates a disposable scratch directory for the fixture files.

 @param id - unique suffix for this test's directory

 @returns disposable scratch directory handle
 */
async function createScratchDir(id: string,): Promise<ScratchDir> {
  /**
   Absolute path of the new directory.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    `${TEST_DIR_PREFIX}-${id}-`,
  ),);

  return {
    path,
    [Symbol.asyncDispose]: async function cleanup(): Promise<void> {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Logger that records each message with its level, so a test can check what a backend forwarded.

 @param messages - array that receives `[level] message` entries

 @returns a logger writing into `messages`
 */
function recordingLogger(messages: string[],): Logger {
  return {
    debug: function debug(message: string,): void {
      messages.push(`[debug] ${message}`,);
    },
    error: function error(message: string,): void {
      messages.push(`[error] ${message}`,);
    },
    fatal: function fatal(message: string,): void {
      messages.push(`[fatal] ${message}`,);
    },
    flush: function flush(): Promise<void> {
      return Promise.resolve();
    },
    info: function info(message: string,): void {
      messages.push(`[info] ${message}`,);
    },
    trace: function trace(message: string,): void {
      messages.push(`[trace] ${message}`,);
    },
    warn: function warn(message: string,): void {
      messages.push(`[warn] ${message}`,);
    },
  };
}

/**
 Disposable that puts a directory first on `PATH` until it is disposed.
 */
type PathRestoration = {
  /** Restores `PATH` to the value the process had */
  [Symbol.dispose]: () => void;
};

/**
 Runs one probe with a directory first on `PATH`, and restores the previous value afterwards.

 @param directory - directory the probe has to find its program in

 @param probe - the run to make with that `PATH`

 @returns whatever the probe resolves with
 */
async function withDirectoryFirstOnPath({
  directory,
  probe,
}: {
  readonly directory: string;
  readonly probe: () => Promise<string>;
},): Promise<string> {
  using pathRestoration: PathRestoration = directoryFirstOnPath(directory,);

  return await probe();
}

/**
 Puts `directory` first on `PATH` until the returned handle is disposed.

 @param directory - directory a probe has to find its program in

 @returns handle whose `Symbol.dispose` restores `PATH`
 */
function directoryFirstOnPath(directory: string,): PathRestoration {
  /**
   `PATH` as the process had it, restored when the probe is done.
   */
  const previousPath = process.env.PATH ?? '';

  process.env.PATH = `${directory}${delimiter}${previousPath}`;

  return {
    [Symbol.dispose]: function restorePath(): void {
      process.env.PATH = previousPath;
    },
  };
}

await describe({
  name: 'stderr forwarded through the logger',
  children: [
    it({
      name: 'runContainer hands a run\'s stderr to the logger and returns its stdout',
      fn: async () => {
        await using dir = await createScratchDir('container',);
        /**
         Stand-in for the real program, found on `PATH` before it; the backend reads only the two streams it writes.
         */
        const standInPath = join(
          dir.path,
          'podman',
        );

        await writeFile(
          standInPath,
          `#!/usr/bin/env node\n${BOTH_STREAMS_SCRIPT}`,
          'utf8',
        );
        await chmod(standInPath, 0o755,);

        /**
         What the logger recorded during the run.
         */
        const messages: string[] = [];
        /**
         The run's stdout, as its caller receives it.
         */
        const output = await withDirectoryFirstOnPath({
          directory: dir.path,
          probe: async function runWithPath(): Promise<string> {
            return await runContainer({
              combination: {
                file: '/workspace/test.unit.matrix.test.ts',
                os: 'container:ubuntu',
                user: 'root',
                runtime: 'node',
              },
              monorepoRoot: dir.path,
              l: recordingLogger(messages,),
            },);
          },
        },);

        expect(output.trim(),).toBe('matrix fixture stdout',);
        expect(messages,).toEqual(['[warn] [runContainer] matrix fixture stderr',],);
      },
    },),

    it({
      name: 'runHost hands a run\'s stderr to the logger and returns its stdout',
      fn: async () => {
        await using dir = await createScratchDir('host',);
        /**
         Script the host backend runs through this process's `node`.
         */
        const scriptPath = join(
          dir.path,
          'writes-both-streams.ts',
        );

        await writeFile(
          scriptPath,
          BOTH_STREAMS_SCRIPT,
          'utf8',
        );

        /**
         What the logger recorded during the run.
         */
        const messages: string[] = [];
        /**
         The run's stdout, as its caller receives it.
         */
        const output = await runHost({
          combination: {
            file: scriptPath,
            os: 'host:',
            user: 'root',
            runtime: 'node',
          },
          l: recordingLogger(messages,),
        },);

        expect(output.trim(),).toBe('matrix fixture stdout',);
        expect(messages,).toEqual(['[warn] [runHost] matrix fixture stderr',],);
      },
    },),

    it({
      name: 'records nothing for a run that writes no stderr',
      fn: async () => {
        await using dir = await createScratchDir('quiet',);
        /**
         Script of a run whose only output is its stdout.
         */
        const scriptPath = join(
          dir.path,
          'writes-stdout-only.ts',
        );

        await writeFile(
          scriptPath,
          STDOUT_ONLY_SCRIPT,
          'utf8',
        );

        /**
         What the logger recorded during the run.
         */
        const messages: string[] = [];
        /**
         The run's stdout, as its caller receives it.
         */
        const output = await runHost({
          combination: {
            file: scriptPath,
            os: 'host:',
            user: 'root',
            runtime: 'node',
          },
          l: recordingLogger(messages,),
        },);

        expect(output.trim(),).toBe('matrix fixture stdout',);
        expect(messages,).toEqual([],);
      },
    },),
  ],
},);
