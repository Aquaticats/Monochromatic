#!/usr/bin/env node
/**
 Mutate the embedding owners inside a mount-free source snapshot; preserve every outcome and diff.

 The scanner bakes its sources into one image that runs the campaign itself, so it keeps its own snapshot flow; the
 mutant kinds no campaign tries come from the shared package.
 */
import {
  rm,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import { exclusionArguments } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

import {
  run,
  ScannerVerificationError,
  snapshot,
  type Snapshot,
} from './container-snapshot.ts';

/**
 One optional scope: a name for the evidence and the scanner files it mutates.
 */
type Scope = {
  readonly name: string;
  readonly files: readonly string[];
};

/**
 Each optional scope narrows only the mutated files; every scope runs the same baseline and complete consumer suite.
 */
const scopeTable: Readonly<Record<string, Scope>> = {
  '--startup': {
    name: 'startup-mutation',
    files: ['main.rs'],
  },
  // Pathname normalization, native prefix counting and component masking, mutated without the other embedding owners.
  '--pathname': {
    name: 'pathname-mutation',
    files: [
      'path_scan.rs',
      'path_name_bytes.rs',
    ],
  },
};

/**
 Files mutated when no scope is given.
 */
const allFiles: readonly string[] = [
  'scanner.rs',
  'load_request.rs',
  'scan_finding.rs',
  'frx_scan.rs',
  'path_scan.rs',
  'path_name_bytes.rs',
  'frx_load.rs',
  'process_boundary.rs',
  'main.rs',
  'runtime_cache/mod.rs',
  'runtime_cache/warning.rs',
];

/**
 Describe the removal of a build context when a scope ends.

 @param directory - the build context
 
 @returns an object whose disposal deletes the directory
 
 @example
 ```ts
 await using cleanup = removalOf(fixture.context);
 ```
 */
function removalOf(directory: string): AsyncDisposable {
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        directory,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Print the mutants a campaign with the same arguments would test; `stdout.log` then holds one mutant name per line.

 @param fixture - the snapshot whose image lists its mutants
 
 @throws ScannerVerificationError when the listing exits nonzero
 
 @example
 ```ts
 await listMutants(fixture);
 ```
 */
async function listMutants(fixture: Snapshot): Promise<void> {
  /**
   Disposable build context, deleted when the listing ends.
   */
  await using cleanup = removalOf(fixture.context);
  void cleanup;
  /**
   What the listing container left behind.
   */
  const result = await run({
    command: 'podman',
    args: [
      'run',
      '--rm',
      ...fixture.limits,
      fixture.image,
    ],
    allowFailure: true,
    transcript: fixture.evidence,
  });
  await writeFile(
    join(
      fixture.evidence,
      'exit.json',
    ),
    `${JSON.stringify({
      status: result.status ?? null,
      signal: result.signal ?? null,
      listed: true,
    })}\n`,
  );
  if (result.status !== 0)
    throw new ScannerVerificationError(`Mutant listing exited ${result.status ?? 'no status'}; inspect ${fixture.evidence}.`);
}

/**
 Scoped mutations retain the baseline, all policy branches and the complete scanner consumer suite.

 @throws ScannerVerificationError when the options are not at most one scope, optionally with `--list`, or the campaign fails
 
 @example
 ```ts
 await main();
 ```
 */
async function main(): Promise<void> {
  /**
   Arguments after the script path.
   */
  const options = process.argv
    .slice(2);
  // `--list` combines with any scope: it appends cargo-mutants' own `--list` to the unchanged campaign arguments.
  /**
   The `--list` arguments, of which at most one is accepted.
   */
  const listing = options.filter(function isList(option) {
    return option === '--list';
  });
  /**
   Arguments that are not `--list`, which must be at most one scope.
   */
  const chosen = options.filter(function isScope(option) {
    return option !== '--list';
  });
  if ((listing.length > 1) || (chosen.length > 1)
    || ((chosen.length === 1) && (!Object.hasOwn(
      scopeTable,
      chosen[0] ?? '',
    ))))
    throw new ScannerVerificationError(`Accepted: at most one of ${Object.keys(scopeTable)
      .join(', ')}, optionally with --list.`);
  /**
   The chosen scope, or none for the whole scanner.
   */
  const scope = chosen.length === 1 ? scopeTable[chosen[0] ?? ''] : undefined;
  /**
   Whether only the mutants are listed.
   */
  const list = listing.length === 1;
  /**
   The cargo-mutants command the snapshot image runs.
   */
  const command: string[] = [
    'cargo',
    'mutants',
    '--in-place',
    '--all-features',
    '--baseline',
    'run',
    '--no-config',
    '--no-shuffle',
    '--colors=never',
    '--build-timeout',
    '300',
    '--timeout',
    '120',
    '--output',
    '/work/mutation-report',
    '--cargo-arg=--offline',
    '--cargo-arg=--locked',
    '--cargo-test-arg=--',
    '--cargo-test-arg=--test-threads=1',
    '--cargo-test-arg=--skip=rule::frx::compile_tests::builtin_ported_all_compile',
    '--cargo-test-arg=--skip=rule::frx::compile_tests::append_ported_compiles_end_to_end',
    ...exclusionArguments([]),
    ...(scope?.files ?? allFiles).flatMap(function fileArguments(file) {
      return [
        '--file',
        `src/${file}`
      ];
    }),
    ...(list ? ['--list'] : []),
  ];
  /**
   Name of the scope for the evidence directory.
   */
  const name = scope ? scope.name : 'mutation';
  /**
   The snapshot image and its evidence directory.
   */
  const fixture = await snapshot({
    command,
    name: list ? `${name}-list` : name,
    tool: 'cargo-mutants',
  });
  if (list) {
    await listMutants(fixture);
    return;
  }
  /**
   The container once it exists, and whether its report was kept; removed (or reported) with the build context when the
   campaign ends.
   */
  const state = {
    container: '',
    reportPreserved: false,
  };
  /**
   Removal of the container (when its report was kept) and the build context.
   */
  await using cleanup = {
    async [Symbol.asyncDispose](): Promise<void> {
      if ((state.container !== '') && state.reportPreserved)
        await run({
          command: 'podman',
          args: [
            'rm',
            '--force',
            state.container
          ],
          capture: true,
        });
      else if (state.container !== '')
        console.error(`Report retrieval failed; disposable container retained for recovery: ${state.container}`);
      await rm(
        fixture.context,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
  void cleanup;
  state.container = (await run({
    command: 'podman',
    args: [
      'create',
      ...fixture.limits,
      fixture.image,
    ],
    capture: true,
  }))
    .stdout
    .trim();
  /**
   The container created from the snapshot image.
   */
  const {container} = state;
  /**
   Outcome of the campaign run.
   */
  const result = await run({
    command: 'podman',
    args: [
      'start',
      '--attach',
      container
    ],
    allowFailure: true,
    transcript: fixture.evidence,
  });
  /**
   Outcome of copying the report out of the container.
   */
  const report = await run({
    command: 'podman',
    args: [
      'cp',
      `${container}:/work/mutation-report/.`,
      fixture.evidence
    ],
    capture: true,
    allowFailure: true,
  });
  state.reportPreserved = report.status === 0;
  await writeFile(
    join(
      fixture.evidence,
      'exit.json',
    ),
    `${JSON.stringify(
      {
        status: result.status ?? null,
        signal: result.signal ?? null,
        reportCopied: report.status === 0,
        reportCopyError: report.stderr,
      },
      null,
      2,
    )}\n`,
  );
  if ((result.status !== 0) || (report.status !== 0))
    throw new ScannerVerificationError(`Mutation exited ${result.status ?? 'no status'}; report-copy status ${report.status ?? 'no status'}; inspect ${fixture.evidence}.`);
}

await main();
