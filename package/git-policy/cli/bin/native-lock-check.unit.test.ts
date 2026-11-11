import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  firstErrorLine,
  metadataArguments,
  recordedManifestDirectories,
} from './native-lock-check.ts';
import { runGateCommand } from './native-gate-run.ts';
import { createGateScratchDirectory } from './stand-in-gate-programs-for-tests.ts';

/**
 Path of the command under test, which runs as TypeScript.
 */
const lockCheckPath = join(
  import.meta.dirname,
  'native-lock-check.ts',
);

/**
 Milliseconds one run of the command under test is given.
 */
const testDeadlineMilliseconds = 60_000;

/**
 Environment variable that names the directory whose locked check the stand-in `cargo` fails.
 */
const failingDirectoryVariable = 'GATE_STAND_IN_CARGO_FAILS';

/**
 The stand-in `cargo`: answers every resolution with an empty package list, and fails the one whose directory the fixture
 named, saying what cargo says for a floor the lock cannot meet.
 */
const standInCargo = `#!/usr/bin/env node
const path = require('node:path');
const args = process.argv.slice(2);
const failing = process.env.${failingDirectoryVariable};
if ((failing !== undefined) && (path.basename(process.cwd()) === failing) && args.includes('--locked')) {
  process.stderr.write('error: failed to select a version for the requirement \`pin = "^2"\`\\n');
  process.exit(101);
}
process.stdout.write('{"packages":[]}\\n');
`;

/**
 A checkout holding the directories a test checks, with the stand-in `cargo` first on `PATH`.

 A scratch directory removes itself when the test's scope ends.
 */
async function createFixture({
  parent,
  directories,
  failsIn,
}: {
  readonly parent: string;
  readonly directories: readonly string[];
  readonly failsIn?: string;
}): Promise<{ readonly checkout: string; readonly environment: Readonly<Record<string, string>>; }> {
  /**
   The checkout the run reads its manifests from.
   */
  const checkout = join(
    parent,
    'repo',
  );
  /**
   The directory that goes first on `PATH`.
   */
  const programs = join(
    parent,
    'path',
  );
  await mkdir(
    programs,
    { recursive: true, },
  );
  await writeFile(
    join(programs, 'cargo'),
    standInCargo,
    { mode: 0o700, },
  );
  await Promise.all(directories.map(async function make(directory) {
    await mkdir(
      join(checkout, directory),
      { recursive: true, },
    );
  }));
  return {
    checkout,
    environment: {
      ...process.env,
      PATH: `${programs}:${process.env.PATH ?? ''}`,
      ...(failsIn === undefined ? {} : { [failingDirectoryVariable]: failsIn }),
    } as Readonly<Record<string, string>>,
  };
}

/**
 Run the command under test.

 @param checkout - the checkout to read

 @param args - arguments after the two the fixture always gives

 @param environment - the environment, with the stand-in first on `PATH`

 @returns how the command ended, with its output
 */
async function lockCheck({
  checkout,
  args,
  environment,
}: {
  readonly checkout: string;
  readonly args: readonly string[];
  readonly environment: Readonly<Record<string, string>>;
}) {
  return await runGateCommand({
    program: process.execPath,
    args: [
      lockCheckPath,
      '--checkout',
      checkout,
      ...args,
    ],
    deadlineMilliseconds: testDeadlineMilliseconds,
    environment,
  });
}

await describe({
  name: 'native:lock:check, which asks cargo about the manifests the gate builds',
  children: [
    it({
      name: 'reads the first error line of cargo\'s output, cut to the width a report keeps',
      fn: async () => {
        expect(firstErrorLine({ stderr: 'warning: something\n  error: failed to select a version\n' }),)
          .toBe('error: failed to select a version',);
        expect(firstErrorLine({ stderr: 'no error here\n' }),).toBe('',);
        expect(firstErrorLine({ stderr: `error: ${'x'.repeat(400)}`, }).length,).toBe(110,);
      },
    }),
    it({
      name: 'asks for a locked offline resolution only when the check must answer from the lockfile',
      fn: async () => {
        expect(metadataArguments({ locked: true, }),).toEqual([
          'metadata',
          '--locked',
          '--offline',
          '--format-version',
          '1',
        ],);
        expect(metadataArguments({ locked: false, }),).toEqual([
          'metadata',
          '--offline',
          '--format-version',
          '1',
        ],);
      },
    }),
    it({
      name: 'prints one line per manifest and exits zero when every locked resolution passed',
      fn: async () => {
        /**
         A checkout holding the directories the run checks.
         */
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-lock-check-', });
        const fixture = await createFixture({
          parent: scratch.path,
          directories: ['a', 'b',],
        });
        /**
         How the command ended.
         */
        const printed = await lockCheck({
          checkout: fixture.checkout,
          args: ['a', 'b',],
          environment: fixture.environment,
        });
        expect(printed.status,).toBe(0,);
        expect(printed.stdout.trim()
          .split('\n'),).toEqual(['0 a ', '0 b',],);
      },
    }),
    it({
      name: 'names the manifest and cargo\'s own error when a lock cannot meet a floor, and exits nonzero',
      fn: async () => {
        /**
         A checkout whose second manifest the stand-in cargo refuses.
         */
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-lock-check-', });
        const fixture = await createFixture({
          parent: scratch.path,
          directories: ['a', 'b',],
          failsIn: 'b',
        });
        /**
         How the command ended.
         */
        const printed = await lockCheck({
          checkout: fixture.checkout,
          args: ['a', 'b',],
          environment: fixture.environment,
        });
        expect(printed.status,).toBe(1,);
        expect(printed.stdout,).toContain('101 b error: failed to select a version for the requirement',);
        expect(printed.stdout,).toContain('0 a',);
      },
    }),
    it({
      name: 'checks the manifests the gate builds when the caller names none',
      fn: async () => {
        /**
         A checkout holding every recorded manifest directory.
         */
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-lock-check-', });
        const fixture = await createFixture({
          parent: scratch.path,
          directories: recordedManifestDirectories,
        });
        /**
         How the command ended.
         */
        const printed = await lockCheck({
          checkout: fixture.checkout,
          args: [],
          environment: fixture.environment,
        });
        expect(printed.status,).toBe(0,);
        expect(printed.stdout.trim()
          .split('\n'),).toHaveLength(recordedManifestDirectories.length,);
        for (const directory of recordedManifestDirectories)
          expect(printed.stdout,).toContain(`0 ${directory}`,);
      },
    }),
    it({
      name: 're-resolves the locked entries first when the caller asks, and prints both answers',
      fn: async () => {
        /**
         A checkout whose second manifest the stand-in cargo refuses.
         */
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-lock-check-', });
        const fixture = await createFixture({
          parent: scratch.path,
          directories: ['a', 'b',],
          failsIn: 'b',
        });
        /**
         How the command ended.
         */
        const printed = await lockCheck({
          checkout: fixture.checkout,
          args: [
            '--refresh',
            'a',
            'b',
          ],
          environment: fixture.environment,
        });
        expect(printed.status,).toBe(1,);
        expect(printed.stdout,).toContain('refresh=0 locked=0 a',);
        expect(printed.stdout,).toContain('refresh=0 locked=101 b',);
      },
    }),
    it({
      name: 'refuses an option it does not know instead of reading it as a manifest directory',
      fn: async () => {
        /**
         A checkout holding one manifest directory.
         */
        await using scratch = await createGateScratchDirectory({ prefix: 'git-policy-lock-check-', });
        const fixture = await createFixture({
          parent: scratch.path,
          directories: ['a',],
        });
        /**
         How the refused command ended.
         */
        const printed = await lockCheck({
          checkout: fixture.checkout,
          args: ['--locks', 'a',],
          environment: fixture.environment,
        });
        expect(printed.status,).toBe(1,);
        expect(printed.stderr,).toContain('Unknown option',);
        expect(printed.stderr,).toContain('--checkout',);
      },
    }),
  ],
});
