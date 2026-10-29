#!/usr/bin/env node
/**
 Host-side generator for `e2e/commit-shape-trace.json`.

 Mines commit shapes from real repository history with real Git by absolute path,
 bypassing the cli-git wrapper,
 and writes an anonymized trace
 (no contents,
 paths,
 object IDs,
 messages,
 or author data).
 Run through `mise run //package/git-policy/cli:e2e:concurrent:trace`.

 @module
 */

import { spawn, } from 'node:child_process';
import { once, } from 'node:events';
import { writeFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import { parseArgs, } from 'node:util';
import { text as consumeText, } from 'node:stream/consumers';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  COMMIT_RECORD_SEPARATOR,
  parseCommitShapeLog,
} from './commit-shape-log-fixture.ts';
import {
  anonymizeCommitShapes,
  collectBlobOids,
  parseBlobSizes,
  serializeCommitShapeTrace,
} from './commit-shape-trace-fixture.ts';

//region Setup

/**
 Logger for the trace generator.
 */
const moduleLogger = tagged({ tag: 'commit-shape-trace-generator', },);

/**
 Real Git on the host, never the PATH-shadowing wrapper.
 */
const REAL_GIT = '/usr/bin/git';

/**
 Default number of newest non-merge commits mined.
 */
const DEFAULT_COMMIT_LIMIT = 4_000;

/**
 Committed trace destination beside this module.
 */
const TRACE_PATH = join(
  import.meta.dirname,
  'commit-shape-trace.json',
);

/**
 Error raised when real Git exits non-zero.
 */
class TraceGitError extends Error {
  /**
   Creates an error carrying Git's diagnostic.

   @param message - failed command and stderr

   @example
   ```ts
   throw new TraceGitError('git log failed');
   ```
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'TraceGitError';
  }
}

//endregion Setup

//region Git access

/**
 Runs real Git with configuration that could alter log output pinned off.

 @param repository - repository whose history is read

 @param args - Git arguments after the pinned `-c` options

 @param input - optional standard input

 @returns standard output

 @throws {@link TraceGitError} on non-zero exit

 @example
 ```ts
 await runRealGit({ repository: '/repo', args: ['rev-parse', 'HEAD'] });
 ```
 */
async function runRealGit({
  repository,
  args,
  input,
}: Readonly<{
  repository: string;
  args: readonly string[];
  input?: string;
}>,): Promise<string> {
  /**
   Logger for this invocation.
   */
  const l = tagged({
    tag: runRealGit.name,
    l: moduleLogger,
  },);
  l.debug(`git ${args[0] ?? ''}`,);
  /**
   Child process with captured streams.
   */
  const child = spawn(
    REAL_GIT,
    [
      '-C',
      repository,
      '-c',
      'log.showSignature=false',
      '-c',
      'core.quotePath=false',
      ...args,
    ],
    { stdio: [
      'pipe',
      'pipe',
      'pipe',
    ], },
  );
  child.stdin
    .end(input ?? '',);
  /**
   Stream collection started before settlement.
   */
  const output = Promise.all([
    consumeText(child.stdout,),
    consumeText(child.stderr,),
  ],);
  await once(
    child,
    'close',
  );
  /**
   Settled standard output and diagnostic.
   */
  const [stdout, stderr,] = await output;
  if (child.exitCode !== 0) {
    l.error(`git exited ${String(child.exitCode,)}`,);
    throw new TraceGitError(`${REAL_GIT} ${args.join(' ',)} exited ${String(child.exitCode,)}\n${stderr}`,);
  }
  return stdout;
}

//endregion Git access

//region Generation

/**
 Parses generator options from task arguments.

 @param argv - arguments after the script path

 @returns repository, revision, and commit limit

 @example
 ```ts
 parseGeneratorOptions(['--limit', '200']);
 ```
 */
function parseGeneratorOptions(argv: readonly string[],): Readonly<{
  repository?: string;
  revision: string;
  limit: number;
}> {
  /**
   Parsed option values.
   */
  const { values, } = parseArgs({
    args: [...argv,],
    options: {
      repository: { type: 'string', },
      revision: {
        type: 'string',
        default: 'HEAD',
      },
      limit: {
        type: 'string',
        default: String(DEFAULT_COMMIT_LIMIT,),
      },
    },
  },);
  /**
   Parsed commit limit.
   */
  const limit = Number(values.limit,);
  if ((!Number.isSafeInteger(limit,)) || (limit <= 0))
    throw new RangeError(`--limit must be a positive integer, got ${values.limit}`,);
  return {
    ...(values.repository === undefined ? {} : { repository: values.repository, }),
    revision: values.revision,
    limit,
  };
}

/**
 Mines, anonymizes, and writes the trace.

 @param argv - task arguments

 @example
 ```ts
 await generateCommitShapeTrace(['--repository', '/repo']);
 ```
 */
async function generateCommitShapeTrace(argv: readonly string[],): Promise<void> {
  /**
   Logger for this run.
   */
  const l = tagged({
    tag: generateCommitShapeTrace.name,
    l: moduleLogger,
  },);
  /**
   Parsed options.
   */
  const options = parseGeneratorOptions(argv,);
  /**
   History source, defaulting to the repository containing this package.
   */
  const repository = options.repository
    ?? (await runRealGit({
      repository: import.meta.dirname,
      args: [
        'rev-parse',
        '--show-toplevel',
      ],
    },)).trim();
  l.info(`mining newest ${String(options.limit,)} non-merge commits of ${options.revision}`,);
  /**
   Raw plus numstat log, oldest first.
   */
  const logText = await runRealGit({
    repository,
    args: [
      'log',
      '--no-merges',
      `--max-count=${String(options.limit,)}`,
      '--reverse',
      '-z',
      // `%x1e` is COMMIT_RECORD_SEPARATOR, the record boundary parseCommitShapeLog splits on.
      '--format=%x1ecommit',
      '--raw',
      '--numstat',
      '--no-abbrev',
      '-M',
      '--no-color',
      '--no-ext-diff',
      '--no-textconv',
      options.revision,
      '--',
    ],
  },);
  /**
   Parsed commits.
   */
  const commits = parseCommitShapeLog(logText,);
  /**
   Blobs needing sizes.
   */
  const oids = collectBlobOids(commits,);
  l.info(`parsed ${String(commits.length,)} commits referencing ${String(oids.length,)} blobs`,);
  /**
   Blob sizes from one batch-check.
   */
  const blobSizes = parseBlobSizes(await runRealGit({
    repository,
    args: [
      'cat-file',
      '--batch-check=%(objectname) %(objectsize)',
    ],
    input: `${oids.join('\n',)}\n`,
  },),);
  /**
   Anonymized trace.
   */
  const trace = anonymizeCommitShapes({
    commits,
    blobSizes,
    source: `newest ${String(commits.length,)} non-merge commits of the Monochromatic repository, oldest first`,
  },);
  await writeFile(
    TRACE_PATH,
    serializeCommitShapeTrace(trace,),
  );
  l.info(`wrote ${TRACE_PATH} with ${String(trace.pathCount,)} distinct paths`,);
}

//endregion Generation

await generateCommitShapeTrace(process.argv
  .slice(2,),);
