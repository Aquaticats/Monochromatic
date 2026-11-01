/**
 Coverage-driver scenarios over the file sink and its ancestor search, the
 browser-only factories evaluated under Node (they must decline cleanly),
 the noop sink, and the neutral default sink list.

 @module
 */

import {
  mkdir,
  mkdtemp,
  rm,
  stat,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  dirname,
  join,
} from 'node:path';

import {
  type LogRecord,
  type Sink,
  sinks,
} from '@monochromatic-dev/logger-core/ts';
import {
  createIndexedDbSink,
  createOpfsSink,
} from '@monochromatic-dev/logger-core/ts/browser.ts';
import { createDefaultSinks, } from '@monochromatic-dev/logger-core/ts/default-sinks.neutral.ts';
import {
  _findNodeModulesUp as findNodeModulesUp,
  createFileSink,
} from '@monochromatic-dev/logger-core/ts/node.ts';

import { BOUNDARY_TOKENS, } from './boundary-corpus.ts';

//region Fixtures

/**
 Directory the file sink creates under the nearest `node_modules`.
 */
const LOG_DIR_NAME = '.monochromatic';

/**
 Shape of the throwaway package the file sink runs in: no `node_modules`
 anywhere above (verify declines), an empty `node_modules` (verify
 succeeds), or a `node_modules` whose log directory name is taken by a
 file (verify fails while creating the directory).
 */
type PackageLayout = 'bare' | 'blocked-log-dir' | 'node-modules';

/**
 Entered throwaway package; disposing leaves and removes it.
 */
type ThrowawayPackage = AsyncDisposable & {
  readonly dir: string;
};

/**
 Builds one info record.

 @param message - Message body.

 @returns Record at a fixed timestamp.

 @example
 ```ts
 record({ message: 'x' });
 ```
 */
function record({ message, }: { readonly message: string; },): LogRecord {
  return {
    level: 'info',
    message,
    timestamp: 0,
  };
}

/**
 `stat` stand-in that always reports a missing path, so the ancestor search
 exhausts silently; always throws.
 */
function statAlwaysMissing(): never {
  throw Object.assign(
    new Error('ENOENT',),
    { code: 'ENOENT', },
  );
}

/**
 `stat` stand-in that always reports a permission failure, so the ancestor
 search reports an unexpected fault; always throws.
 */
function statAlwaysDenied(): never {
  throw Object.assign(
    new Error('EACCES',),
    { code: 'EACCES', },
  );
}

/**
 Accepts an ancestor-search fault report without keeping it; the report
 call is the branch under coverage.
 */
function ignoreReport(): void {
  // Reported faults are expected from the denying stand-in.
}

/**
 Writes before verify, verifies twice (the second call shares the memo),
 writes after, and flushes when the sink has a hook.

 @param sink - Sink to walk.

 @example
 ```ts
 await walkSink({ sink: createNoopSink() });
 ```
 */
async function walkSink({ sink, }: { readonly sink: Sink; },): Promise<void> {
  await sink.write(record({ message: 'before verify', },),);
  await Promise.all([
    sink.verify(),
    sink.verify(),
  ],);
  await sink.write(record({ message: 'after verify', },),);
  if (sink.flush !== undefined)
    await sink.flush();
}

/**
 Enters a fresh temporary directory laid out per `layout`; disposing
 returns to the previous working directory and removes it. The file sink
 resolves its log path from `process.cwd()`, so the layout decides which
 verify branch it takes.

 @param layout - Package shape to stage.

 @returns Entered package.

 @example
 ```ts
 await using pkg = await enterThrowawayPackage({ layout: 'node-modules' });
 ```
 */
async function enterThrowawayPackage({ layout, }: { readonly layout: PackageLayout; },): Promise<ThrowawayPackage> {
  /**
   Working directory to restore.
   */
  const original = process.cwd();
  /**
   Fresh temporary root.
   */
  const dir = await mkdtemp(join(
    scratchRoot(),
    'logger-coverage-',
  ),);
  if (layout !== 'bare')
    await mkdir(join(
      dir,
      'node_modules',
    ),);
  if (layout === 'blocked-log-dir')
    await writeFile(
      join(
        dir,
        'node_modules',
        LOG_DIR_NAME,
      ),
      'a file where the log directory must go',
    );
  process.chdir(dir,);
  return {
    dir,
    async [Symbol.asyncDispose](): Promise<void> {
      process.chdir(original,);
      await rm(
        dir,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

//endregion Fixtures

//region Scenarios

/**
 File sink inside this package (an ancestor `node_modules` exists): the
 corpus lands as JSONL.
 */
async function exerciseFileSinkAvailable(): Promise<void> {
  /**
   Sink resolving its log file under this package's `node_modules`.
   */
  const sink = createFileSink();
  await walkSink({ sink, },);
  await Promise.all(BOUNDARY_TOKENS.map(function writeToken(message,): Promise<void> {
    return sink.write(record({ message, },),);
  },),);
}

/**
 File sink from a directory with no `node_modules` anywhere above it:
 verify declines without a fault.
 */
async function exerciseFileSinkUnavailable(): Promise<void> {
  /**
   Bare directory entered for the scope.
   */
  await using _outside = await enterThrowawayPackage({ layout: 'bare', },);
  await walkSink({ sink: createFileSink(), },);
}

/**
 File sink whose log directory cannot be created because a file holds its
 name: verify fails and reports.
 */
async function exerciseFileSinkVerifyFailure(): Promise<void> {
  /**
   Blocked package entered for the scope.
   */
  await using _blocked = await enterThrowawayPackage({ layout: 'blocked-log-dir', },);
  await walkSink({ sink: createFileSink(), },);
}

/**
 File sink whose log directory vanishes after verify: the append fails and
 reports, and the sink stays usable.
 */
async function exerciseFileSinkAppendFailure(): Promise<void> {
  /**
   Package entered for the scope.
   */
  await using pkg = await enterThrowawayPackage({ layout: 'node-modules', },);
  /**
   Sink whose log file lives under the package.
   */
  const sink = createFileSink();
  await sink.verify();
  await rm(
    join(
      pkg.dir,
      'node_modules',
      LOG_DIR_NAME,
    ),
    {
      force: true,
      recursive: true,
    },
  );
  await sink.write(record({ message: 'appended after the directory vanished', },),);
}

/**
 Ancestor search with the real `stat`, a `stat` that always misses, and a
 `stat` that always denies.
 */
async function exerciseAncestorSearch(): Promise<void> {
  await findNodeModulesUp({
    cwd: import.meta.dirname,
    dirname,
    join,
    stat,
  },);
  await findNodeModulesUp({
    cwd: import.meta.dirname,
    dirname,
    join,
    reportError: ignoreReport,
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- The stand-in only throws; the search never reads a result from it.
    stat: statAlwaysMissing as unknown as typeof stat,
  },);
  await findNodeModulesUp({
    cwd: import.meta.dirname,
    dirname,
    join,
    reportError: ignoreReport,
    // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- The stand-in only throws; the search never reads a result from it.
    stat: statAlwaysDenied as unknown as typeof stat,
  },);
}

/**
 Browser-only factories under Node decline at verify and drop writes; the
 noop sink accepts everything; the neutral default list builds.
 */
async function exerciseOtherFactories(): Promise<void> {
  await walkSink({ sink: createIndexedDbSink(), },);
  await walkSink({ sink: createOpfsSink(), },);
  await walkSink({ sink: sinks.createNoopSink(), },);
  createDefaultSinks();
}

//endregion Scenarios

/**
 Runs every Node sink scenario in order.

 @example
 ```ts
 await exerciseNodeSinks();
 ```
 */
export async function exerciseNodeSinks(): Promise<void> {
  await exerciseFileSinkAvailable();
  await exerciseFileSinkUnavailable();
  await exerciseFileSinkVerifyFailure();
  await exerciseFileSinkAppendFailure();
  await exerciseAncestorSearch();
  await exerciseOtherFactories();
}
