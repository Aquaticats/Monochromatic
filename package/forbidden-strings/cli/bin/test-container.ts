#!/usr/bin/env node
/**
 Verify scanner binary and embedding consumers, or all-feature Clippy, in a disposable source snapshot.
 */
import { writeFile } from 'node:fs/promises';
import { join } from 'node:path';

import {
  run,
  ScannerVerificationError,
  snapshot,
} from './container-snapshot.ts';
import { removalOf } from './removal-of.ts';

/**
 Cargo arguments of the all-feature Clippy run.
 */
const clippyCommand: readonly string[] = [
  'cargo',
  'clippy',
  '--offline',
  '--locked',
  '--all-targets',
  '--all-features',
  '--',
  '--deny',
  'warnings',
];

/**
 Cargo arguments of the all-feature test run.
 */
const testCommand: readonly string[] = [
  'cargo',
  'test',
  '--offline',
  '--locked',
  '--all-targets',
  '--all-features',
  '--',
  '--test-threads=1',
];

/**
 Choose the command the snapshot image runs.

 @param option - `--clippy`, `--release`, or nothing for the debug tests

 @returns the Cargo command, with `--release` placed after the subcommand when asked

 @example
 ```ts
 commandOf('--release');
 ```
 */
function commandOf(option?: string): readonly string[] {
  if (option === '--clippy') return clippyCommand;
  if (option !== '--release') return testCommand;
  return [
    ...testCommand.slice(
      0,
      2,
    ),
    '--release',
    ...testCommand.slice(2),
  ];
}

/**
 Label of the evidence directory and the image tag.

 @param option - `--clippy`, `--release`, or nothing for the debug tests

 @returns `clippy`, `release-test` or `test`

 @example
 ```ts
 nameOf('--clippy');
 ```
 */
function nameOf(option?: string): string {
  if (option === '--clippy') return 'clippy';
  return option === '--release' ? 'release-test' : 'test';
}

/**
 Preserve exit evidence on both successful and failed verification.

 @throws ScannerVerificationError when the option is not `--clippy` or `--release`, or the container exits nonzero

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
  if ((options.length > 1) || ((options.length === 1) && (option !== '--clippy')
    && (option !== '--release')))
    throw new ScannerVerificationError('Only --clippy or --release is accepted.');
  /**
   The snapshot image and its evidence directory.
   */
  const fixture = await snapshot({
    command: commandOf(option),
    name: nameOf(option),
    toolchain: option === '--clippy',
  });
  /**
   Removal of the build context when the run ends.
   */
  await using cleanup = removalOf(fixture.context);
  void cleanup;
  /**
   What the verification container left behind.
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
    throw new ScannerVerificationError(`Verification exited ${result.status ?? 'without a status'}; inspect ${fixture.evidence}.`);
}

await main();
