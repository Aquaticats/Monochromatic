#!/usr/bin/env node
/**
 Run the embedding fuzzer with baked source, seeds and toolchain, retaining corpus and crash evidence.
 */
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';

import { mapInSequence } from '@monochromatic-dev/mutation-test-cli-cargo-mutants/ts';

import {
  run,
  ScannerVerificationError,
  snapshot,
  type Snapshot,
} from '../../cli/bin/container-snapshot.ts';
import { removalOf } from '../../cli/bin/removal-of.ts';

/**
 Cargo arguments of the embedding fuzz run.
 */
const fuzzCommand: readonly string[] = [
  'cargo',
  'fuzz',
  'run',
  '--fuzz-dir',
  '.',
  'fuzz_embedding',
  '--sanitizer',
  'address',
  '--target',
  'x86_64-unknown-linux-gnu',
  '--codegen-units',
  '16',
  'seed/fuzz_embedding',
  '--',
  '-max_total_time=120',
  '-max_len=20000',
  '-timeout=10',
  '-rss_limit_mb=1536',
  '-seed=94',
  '-dict=dictionary/forbidden-strings.dict',
];

/**
 Cargo arguments of the Clippy run over every fuzz sidecar target.
 */
const clippyCommand: readonly string[] = [
  'cargo',
  'clippy',
  '--offline',
  '--locked',
  '--all-targets',
  '--',
  '--deny',
  'warnings',
];

/**
 Directories copied out of the container, with the evidence directory each lands in.
 */
const evidenceDirectories: readonly {
  readonly directory: string;
  readonly destination: string;
}[] = [
  {
    directory: 'seed/fuzz_embedding',
    destination: 'corpus',
  },
  {
    directory: 'artifacts',
    destination: 'artifacts',
  },
];

/**
 The outcome of copying one directory out of the container.
 */
type Copied = {
  readonly directory: string;
  readonly status?: number;
  readonly stderr: string;
};

/**
 Run Clippy over the fuzz sidecar in the snapshot image.

 @param fixture - the Clippy snapshot

 @throws ScannerVerificationError when Clippy exits nonzero

 @example
 ```ts
 await clippyRun(fixture);
 ```
 */
async function clippyRun(fixture: Snapshot): Promise<void> {
  /**
   What the Clippy container left behind.
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
    })}\n`,
  );
  if (result.status !== 0)
    throw new ScannerVerificationError(`Fuzz Clippy exited ${result.status ?? 'without a status'}; inspect ${fixture.evidence}.`);
}

/**
 Copy one directory out of the fuzz container into the evidence directory.

 @param container - the container that ran the fuzzer

 @param fixture - the fuzz snapshot

 @param directory - the directory inside the container, relative to the sidecar

 @param destination - the evidence subdirectory that receives it

 @returns how the copy ended

 @example
 ```ts
 const copied = await copyOut({ container, fixture, directory: 'artifacts', destination: 'artifacts' });
 ```
 */
async function copyOut({
  container,
  fixture,
  directory,
  destination,
}: {
  readonly container: string;
  readonly fixture: Snapshot;
  readonly directory: string;
  readonly destination: string;
}): Promise<Copied> {
  /**
   What the copy left behind.
   */
  const copy = await run({
    command: 'podman',
    args: [
      'cp',
      `${container}:/work/package/forbidden-strings/cli.fuzz/${directory}`,
      join(
        fixture.evidence,
        destination,
      ),
    ],
    capture: true,
    allowFailure: true,
  });
  return {
    directory,
    ...(copy.status === undefined ? {} : { status: copy.status }),
    stderr: copy.stderr,
  };
}

/**
 Run the fuzzer in the snapshot image and keep its corpus and crash evidence.

 @param fixture - the fuzz snapshot

 @throws ScannerVerificationError when the fuzzer exits nonzero or the evidence cannot be retrieved

 @example
 ```ts
 await fuzzRun(fixture);
 ```
 */
async function fuzzRun(fixture: Snapshot): Promise<void> {
  /**
   The container once it exists, and whether its evidence was kept; removed (or reported) when the run ends.
   */
  const state = {
    container: '',
    evidencePreserved: false,
  };
  /**
   Removal of the container when its evidence was kept, or a note that it is retained for recovery.
   */
  await using cleanup = {
    async [Symbol.asyncDispose](): Promise<void> {
      if ((state.container !== '') && state.evidencePreserved)
        await run({
          command: 'podman',
          args: [
            'rm',
            '--force',
            state.container,
          ],
          capture: true,
        });
      else if (state.container !== '')
        console.error(`Fuzz evidence retrieval failed; disposable container retained for recovery: ${state.container}`);
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
  })).stdout
    .trim();
  /**
   The created container.
   */
  const { container } = state;
  /**
   What the fuzz container left behind.
   */
  const result = await run({
    command: 'podman',
    args: [
      'start',
      '--attach',
      container,
    ],
    allowFailure: true,
    transcript: fixture.evidence,
  });
  /**
   The outcome of each directory copy.
   */
  const copies = await mapInSequence({
    items: evidenceDirectories,
    run: function copyOf(item) {
      return copyOut({
        container,
        fixture,
        directory: item.directory,
        destination: item.destination,
      });
    },
  });
  state.evidencePreserved = copies.every(function succeeded(copy) {
    return copy.status === 0;
  });
  await writeFile(
    join(
      fixture.evidence,
      'exit.json',
    ),
    `${JSON.stringify(
      {
        status: result.status ?? null,
        signal: result.signal ?? null,
        copied: Object.fromEntries(copies.map(function recordOf(copy) {
          return [
            copy.directory,
            {
              status: copy.status ?? null,
              stderr: copy.stderr,
            },
          ];
        })),
      },
      null,
      2,
    )}\n`,
  );
  if ((result.status !== 0) || (!state.evidencePreserved))
    throw new ScannerVerificationError(`Embedding fuzzing exited ${result.status ?? 'without a status'}; inspect ${fixture.evidence}.`);
}

/**
 Bound both the fuzz campaign and every input while leaving reproducer evidence intact.

 @throws ScannerVerificationError when an option other than `--clippy` is given, or the run fails

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
  /**
   The only accepted option, if given.
   */
  const [option] = options;
  if ((options.length > 1) || ((options.length === 1) && (option !== '--clippy')))
    throw new ScannerVerificationError('Only --clippy is accepted.');
  /**
   Whether to run Clippy instead of the fuzzer.
   */
  const clippy = option === '--clippy';
  /**
   The snapshot image and its evidence directory.
   */
  const fixture = await snapshot({
    command: clippy ? clippyCommand : fuzzCommand,
    name: clippy ? 'fuzz-clippy' : 'embedding-fuzz',
    toolchain: true,
    ...(clippy ? {} : { tool: 'cargo-fuzz' }),
    fuzzing: true,
  });
  /**
   Removal of the build context when the run ends.
   */
  await using removal = removalOf(fixture.context);
  void removal;
  await (clippy ? clippyRun(fixture) : fuzzRun(fixture));
}

await main();
