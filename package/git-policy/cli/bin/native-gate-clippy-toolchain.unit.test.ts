import {
  chmod,
  mkdir,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  clippyToolchainPrograms,
  gateClippyToolchain,
  missingClippyToolchainPrograms,
  rustupToolchainSysroot,
} from './native-gate-clippy-toolchain.ts';
import { isMissingPathError } from './native-gate-paths.ts';
import { NativeVerificationError } from './native-verification-process.ts';
import { createGateScratchDirectory } from './stand-in-gate-programs-for-tests.ts';

/**
 The toolchain the fixture's lock records, which this machine is not expected to have installed.
 */
const lockedToolchain = 'nightly-2026-11-16';

/**
 The toolchain the fixture's caller environment names, which must never be the one the gate mounts.
 */
const environmentToolchain = 'nightly';

/**
 A stand-in `rustup` that records what it was asked and answers `which` with the `rustc` the fixture made.

 `GATE_STAND_IN_RUSTUP_FAIL=1` makes it refuse the way rustup does for a toolchain it does not have, so the refusal is
 exercised without depending on what this machine has installed.
 */
const standInRustupProgram = `#!/usr/bin/env node
const { appendFileSync } = require('node:fs');
appendFileSync(process.env.GATE_STAND_IN_RUSTUP_LOG, JSON.stringify(process.argv.slice(2)) + '\\n');
if (process.env.GATE_STAND_IN_RUSTUP_FAIL === '1') {
  process.stderr.write("error: toolchain 'nightly-2026-11-16-x86_64-unknown-linux-gnu' is not installed\\n");
  process.exit(1);
}
process.stdout.write(process.env.GATE_STAND_IN_RUSTC + '\\n');
`;

/**
 The text of a file the stand-in appended to, empty while it was never asked.

 @param path - file to read
 @returns its text, or empty text when it is not there
 */
async function recordedText(path: string,): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    if (isMissingPathError({ error, }))
      return '';
    throw error;
  }
}

/**
 One fixture repository with a lock, a nested directory to start the search in, and an installed-looking toolchain.

 Every program the Clippy step needs is written into the toolchain's `bin` directory, so a test that exercises the
 whole path removes only the program it means to be missing.
 */
type ClippyFixture = AsyncDisposable & {
  /**
   Directory the lock search starts in, three levels below the repository root.
   */
  readonly inside: string;
  /**
   The lock the gate reads, so a test can remove it and exercise a repository with none.
   */
  readonly lockFile: string;
  /**
   The installed-looking toolchain directory the gate should mount.
   */
  readonly sysroot: string;
  /**
   Environment that puts the stand-in `rustup` first on `PATH` and names another toolchain for the caller.
   */
  readonly environment: NodeJS.ProcessEnv;
  /**
   What the stand-in `rustup` was asked, one argument array per call.
   */
  readonly calls: () => Promise<readonly (readonly string[])[]>;
};

/**
 Make the fixture.

 @param prefix - the name this test's scratch directory starts with

 @returns the fixture; declare it with `await using`
 */
async function createClippyFixture({ prefix }: { readonly prefix: string; }): Promise<ClippyFixture> {
  /**
   The scratch directory this fixture owns, disposed with it.
   */
  const scratch = await createGateScratchDirectory({ prefix, },);
  /**
   The fixture repository's root, which holds the lock.
   */
  const repository = join(
    scratch.path,
    'repository',
  );
  /**
   Directory deep in the repository, where the lock search starts.
   */
  const inside = join(
    repository,
    'package',
    'git-policy',
    'cli',
  );
  await mkdir(
    inside,
    { recursive: true, },
  );
  /**
   The lock the gate reads.
   */
  const lockFile = join(
    repository,
    'toolchain.lock.json',
  );
  await writeFile(
    lockFile,
    JSON.stringify({
      tools: {
        rust: {
          version: lockedToolchain,
        },
      },
    },),
  );
  /**
   The installed-looking toolchain directory, named the way rustup names a toolchain for this host.
   */
  const sysroot = join(
    scratch.path,
    'rustup',
    'toolchains',
    `${lockedToolchain}-x86_64-unknown-linux-gnu`,
  );
  await mkdir(
    join(
      sysroot,
      'bin',
    ),
    { recursive: true, },
  );
  await Promise.all(clippyToolchainPrograms.map(function written(program) {
    return writeFile(
      join(
        sysroot,
        'bin',
        program,
      ),
      '',
      { mode: 0o700, },
    );
  },),);
  /**
   The directory that goes first on `PATH` and holds the stand-in `rustup`.
   */
  const programs = join(
    scratch.path,
    'path',
  );
  await mkdir(
    programs,
    { recursive: true, },
  );
  await writeFile(
    join(
      programs,
      'rustup',
    ),
    standInRustupProgram,
    { mode: 0o700, },
  );
  /**
   The file the stand-in `rustup` appends its argument arrays to.
   */
  const log = join(
    scratch.path,
    'rustup.jsonl',
  );
  /**
   The path the stand-in prints as the toolchain's `rustc`.
   */
  const rustc = join(
    sysroot,
    'bin',
    'rustc',
  );
  return {
    inside,
    lockFile,
    sysroot,
    environment: {
      ...process.env,
      PATH: `${programs}:${process.env.PATH
        ?? ''}`,
      GATE_STAND_IN_RUSTUP_LOG: log,
      GATE_STAND_IN_RUSTC: rustc,
      RUSTUP_TOOLCHAIN: environmentToolchain,
    },
    async calls(): Promise<readonly (readonly string[])[]> {
      /**
       What the stand-in wrote, absent until it is first asked.
       */
      const text = await recordedText(log,);
      return text.split('\n')
        .filter(function hasText(line) {
          return line !== '';
        })
        .map(function argumentsOf(line): readonly string[] {
          return JSON.parse(line) as readonly string[];
        },);
    },
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        scratch.path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

await describe({
  name: 'the Rust toolchain the gate\'s Clippy step mounts',
  children: [
    it({
      name: 'takes the repository\'s locked toolchain and its sysroot, never the toolchain the caller\'s environment names',
      fn: async () => {
        await using fixture = await createClippyFixture({ prefix: 'git-policy-clippy-toolchain-', },);
        /**
         What the gate resolved.
         */
        const resolved = await gateClippyToolchain({
          startDirectory: fixture.inside,
          environment: fixture.environment,
        },);
        expect(resolved,).toEqual({
          toolchain: lockedToolchain,
          sysroot: fixture.sysroot,
        },);
        expect(await fixture.calls(),).toEqual([[
          'which',
          '--toolchain',
          lockedToolchain,
          'rustc',
        ],],);
        expect(fixture.environment.RUSTUP_TOOLCHAIN,).toBe(environmentToolchain,);
      },
    }),
    it({
      name: 'reads the sysroot as the parent of the bin directory holding the rustc rustup named',
      fn: async () => {
        await using fixture = await createClippyFixture({ prefix: 'git-policy-clippy-sysroot-', },);
        /**
         The sysroot the resolution answered.
         */
        const sysroot = await rustupToolchainSysroot({
          toolchain: lockedToolchain,
          environment: fixture.environment,
        },);
        expect(sysroot,).toBe(fixture.sysroot,);
      },
    }),
    it({
      name: 'names the Clippy component and its install command when the toolchain\'s bin directory holds no cargo-clippy',
      fn: async () => {
        await using fixture = await createClippyFixture({ prefix: 'git-policy-clippy-missing-', },);
        await rm(
          join(
            fixture.sysroot,
            'bin',
            'cargo-clippy',
          ),
        );
        /**
         The refusal, made once and judged by every part it must say.
         */
        const refused = gateClippyToolchain({
          startDirectory: fixture.inside,
          environment: fixture.environment,
        },);
        await expect(refused,).rejects.toThrow(NativeVerificationError,);
        await expect(refused,).rejects.toThrow('holds no runnable cargo-clippy',);
        await expect(refused,).rejects.toThrow('the toolchain\'s Clippy component is not installed',);
        await expect(refused,).rejects.toThrow(`rustup component add clippy --toolchain ${lockedToolchain}`,);
      },
    }),
    it({
      name: 'refuses a cargo-clippy that is there but is not a program PATH would run',
      fn: async () => {
        await using fixture = await createClippyFixture({ prefix: 'git-policy-clippy-not-executable-', },);
        await chmod(
          join(
            fixture.sysroot,
            'bin',
            'cargo-clippy',
          ),
          0o600,
        );
        /**
         The refusal for a Clippy that exists without an execute bit.
         */
        const refused = gateClippyToolchain({
          startDirectory: fixture.inside,
          environment: fixture.environment,
        },);
        await expect(refused,).rejects.toThrow(NativeVerificationError,);
        await expect(refused,).rejects.toThrow('holds no runnable cargo-clippy',);
      },
    }),
    it({
      name: 'reports every program a broken toolchain lacks, in the order the Clippy step needs them',
      fn: async () => {
        await using fixture = await createClippyFixture({ prefix: 'git-policy-clippy-programs-', },);
        await rm(
          join(
            fixture.sysroot,
            'bin',
            'clippy-driver',
          ),
        );
        await rm(
          join(
            fixture.sysroot,
            'bin',
            'cargo',
          ),
        );
        expect(await missingClippyToolchainPrograms({ sysroot: fixture.sysroot, },),).toEqual(['cargo', 'clippy-driver',],);
      },
    }),
    it({
      name: 'names the locked toolchain and the install task when rustup has no such toolchain',
      fn: async () => {
        await using fixture = await createClippyFixture({ prefix: 'git-policy-clippy-refused-', },);
        /**
         The refusal that names the toolchain rustup does not have.
         */
        const refused = gateClippyToolchain({
          startDirectory: fixture.inside,
          environment: {
            ...fixture.environment,
            GATE_STAND_IN_RUSTUP_FAIL: '1',
          },
        },);
        await expect(refused,).rejects.toThrow(NativeVerificationError,);
        await expect(refused,).rejects.toThrow(lockedToolchain,);
        await expect(refused,).rejects.toThrow('mise run toolchain:install',);
      },
    }),
    it({
      name: 'refuses when no lock is above the directory the gate started in',
      fn: async () => {
        await using fixture = await createClippyFixture({ prefix: 'git-policy-clippy-no-lock-', },);
        await rm(fixture.lockFile,);
        /**
         The refusal for a directory with no lock above it.
         */
        const refused = gateClippyToolchain({
          startDirectory: fixture.inside,
          environment: fixture.environment,
        },);
        await expect(refused,).rejects.toThrow('no toolchain.lock.json',);
      },
    }),
  ],
},);
