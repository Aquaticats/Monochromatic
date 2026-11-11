import { spawnSync } from 'node:child_process';
import { readFile } from 'node:fs/promises';
import { resolve } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  cargoProgram,
  commitType,
  failureExitCode,
  gitProgram,
  releaseWorkflow,
  staleHeadMessage,
  systemdRunProgram,
  usageExitCode,
} from './constants.ts';
import {
  loadCrateRegistry,
  registryCrateNames,
} from './crate-registry.ts';
import {
  recordedReleaseRunFailed,
  recordedReleaseRunPassed,
} from './fixture/recorded-run-answers.ts';
import { requestFileName } from './push-run.ts';
import {
  createStubFixture,
  type StubFixture,
} from './stub-tools-fixture.ts';

/**
 Path of the release executable.
 */
const releaseBin = resolve(import.meta.dirname, '..', 'bin', 'release.ts');

/**
 Path of the wait executable.
 */
const waitBin = resolve(import.meta.dirname, '..', 'bin', 'wait-publish.ts');

/**
 Path of the dispatch executable.
 */
const dispatchBin = resolve(import.meta.dirname, '..', 'bin', 'dispatch-release.ts');

/**
 Path of the await executable.
 */
const awaitBin = resolve(import.meta.dirname, '..', 'bin', 'await-release.ts');

/**
 Path of the push executable.
 */
const pushBin = resolve(import.meta.dirname, '..', 'bin', 'push-release.ts');

/**
 The package's own registry, which the executables read whatever `--root` names.
 */
const packageRegistry = await loadCrateRegistry({
  packageDirectory: resolve(import.meta.dirname, '..'),
  readText: async function readText(path) {
    return readFile(path, 'utf8');
  },
});

/**
 Crate names as the unknown-crate message lists them, so adding a crate to the registry changes no test.
 */
const listedCrateNames = registryCrateNames(packageRegistry)
  .join(', ');

/**
 The `forbidden-strings` entry of the registry, whose lock directories the registry-based tests derive their calls from.
 */
const scannerEntry = packageRegistry.get('forbidden-strings');
if (scannerEntry === undefined)
  throw new Error('the registry has no forbidden-strings entry');

/**
 Lock directories of the `forbidden-strings` entry, as a constant that stays typed inside functions.
 */
const scannerLockDirectories: readonly string[] = scannerEntry.lockDirectories;

/**
 Note that holds every character a shell treats as syntax.
 */
const hostileNote = `it's "done"; $HOME \`id\` $(id) && touch pwned`;

/**
 Write the fixture repository of a crate at 0.4.9 with a fuzz crate that pins it exactly.

 @param fixture - fixture to fill
 */
async function writeRepository(fixture: StubFixture): Promise<void> {
  await fixture.write({
    path: 'package/demo/cli/Cargo.toml',
    text: '[package]\nname = "demo-cli"\nversion = "0.4.9"\n',
  });
  await fixture.write({
    path: 'package/demo/cli.fuzz/Cargo.toml',
    text: '[package]\nname = "fuzz"\nversion = "0.0.0"\n\n[dependencies]\n'
      + 'demo-cli = { path = "../cli", version = "=0.4.9" }\n',
  });
}

/**
 Write the fixture repository of the `forbidden-strings` registry entry at 0.6.0: the scanner, the wrapper that pins it,
 and the fuzz and benchmark crates that pin it too.

 @param fixture - fixture to fill
 */
async function writeRegistryRepository(fixture: StubFixture): Promise<void> {
  await fixture.write({
    path: 'package/forbidden-strings/cli/Cargo.toml',
    text: '[package]\nname = "forbidden-strings"\nversion = "0.6.0"\n',
  });
  // Each lock directory must exist, because the lock step starts cargo inside it.
  await Promise.all(scannerLockDirectories.map(async function writeLockDirectory(directory) {
    await fixture.write({
      path: `${directory}/Cargo.toml`,
      text: '[package]\nname = "fuzz"\nversion = "0.0.0"\n',
    });
  }));
  await Promise.all([
    'package/git-policy/cli/Cargo.toml',
    'package/forbidden-strings/cli.fuzz/Cargo.toml',
    'package/forbidden-strings/core.bench/Cargo.toml',
  ].map(async function writePin(path) {
    await fixture.write({
      path,
      text: '[package]\nname = "pinning"\nversion = "0.0.0"\n\n[dependencies]\n'
        + 'forbidden-strings = { path = "../cli", version = "=0.6.0" }\n',
    });
  }));
}

/**
 The commit the push fixture started from, the release commit on top of it, and a head the branch moved to.
 */
const pushBase = 'a'.repeat(40);
const pushMoved = 'c'.repeat(40);

/**
 The subject of the push fixture's release commit.
 */
const pushHeadline = `chore(forbidden-strings/cli): release 0.4.10: ${hostileNote}`;

/**
 The files the push fixture's release commit changes.
 */
const pushPaths: readonly string[] = [
  'package/forbidden-strings/cli/Cargo.toml',
  'package/forbidden-strings/cli/Cargo.lock',
];

/**
 The answer of the commit mutation when the branch is no longer where the commit expected it.
 */
const pushStaleAnswer = JSON.stringify({ errors: [{ message: `${staleHeadMessage} "${pushBase}" but it did not.` }] });

/**
 Fill the fixture of a push: the files of the release commit, what git says about it, and the answer of cargo metadata
 for the crate and its lock directory.

 @param fixture - fixture to fill
 */
async function writePushFixture(fixture: StubFixture): Promise<void> {
  await Promise.all(pushPaths.map(async function writePath(path) {
    await fixture.write({ path, text: `contents of ${path}\n` });
  }));
  await fixture.script({ key: 'git.rev-list', stdout: `${'b'.repeat(40)} ${pushBase}\n` });
  await fixture.script({ key: 'git.log', stdout: `${pushHeadline}\n` });
  await fixture.script({ key: 'git.diff-tree', stdout: `${pushPaths.join('\0')}\0` });
  await fixture.script({
    key: 'cargo',
    stdout: JSON.stringify({
      packages: [
        { name: 'forbidden-strings', source: null, manifest_path: `${fixture.directory}/package/forbidden-strings/cli/Cargo.toml` },
        { name: 'dep', source: null, manifest_path: `${fixture.directory}/package/dep/Cargo.toml` },
        { name: 'serde', source: 'registry+https://github.com/rust-lang/crates.io-index', manifest_path: '/cargo/serde/Cargo.toml' },
      ],
      workspace_root: `${fixture.directory}/package/forbidden-strings/cli`,
    }),
  });
}

/**
 Run the push executable against the fixture with the stubs first on the search path.

 @param fixture - fixture to run in
 @param extra - arguments after the fixed options
 @returns the finished child
 */
function runPush({
  fixture,
  extra,
}: {
  readonly fixture: StubFixture;
  readonly extra: readonly string[];
}) {
  return spawnSync('node', [
    pushBin,
    '--root',
    fixture.directory,
    '--crate',
    'forbidden-strings',
    '--repo',
    'o/r',
    '--branch',
    'main',
    '--scratch-dir',
    `${fixture.directory}/scratch`,
    ...extra,
  ], { env: fixture.environment(), encoding: 'utf8' });
}

/**
 Run the release executable against the fixture with the stubs first on the search path.

 @param fixture - fixture to run in
 @param extra - arguments after the fixed options
 @returns the finished child
 */
function runRelease({
  fixture,
  extra,
}: {
  readonly fixture: StubFixture;
  readonly extra: readonly string[];
}) {
  return spawnSync('node', [
    releaseBin,
    '--root',
    fixture.directory,
    '--crate-name',
    'demo-cli',
    '--crate-dir',
    'package/demo/cli',
    '--scope',
    'demo/cli',
    '--pin-file',
    'package/demo/cli.fuzz/Cargo.toml',
    '--check-bin',
    'demo-bin',
    '--lock-dir',
    'package/demo/cli.fuzz',
    '--unit-name',
    'demo-publish-dry',
    ...extra,
  ], { env: fixture.environment(), encoding: 'utf8' });
}

/**
 Run the wait executable against the fixture with the stubs first on the search path.

 @param fixture - fixture to run in
 @param extra - arguments after the fixed options
 @returns the finished child
 */
function runWait({
  fixture,
  extra,
}: {
  readonly fixture: StubFixture;
  readonly extra: readonly string[];
}) {
  return spawnSync('node', [
    waitBin,
    '--root',
    fixture.directory,
    '--crate-name',
    'demo-cli',
    '--attempts',
    '2',
    '--poll-interval',
    '1',
    ...extra,
  ], { env: fixture.environment(), encoding: 'utf8' });
}

/**
 Endpoint that the await command reads the manifest of the fixture crate at one commit from.

 @param sha - commit hash
 @returns the endpoint, as the command passes it to `gh api`
 */
function manifestEndpoint(sha: string): string {
  return `repos/{owner}/{repo}/contents/package/forbidden-strings/cli/Cargo.toml?ref=${sha}`;
}

/**
 Key under which the stubbed `gh` answers the manifest read of one commit: its command, then the endpoint with each
 slash written as an underscore.

 @param sha - commit hash
 @returns the stub key
 */
function manifestStubKey(sha: string): string {
  return `gh.api-${manifestEndpoint(sha).replaceAll('/', '_')}`;
}

/**
 Default branch the stubbed origin names in the dispatch executable tests.
 */
const stubbedDefaultBranch = 'trunk';

/**
 Make the stubbed `git` answer as a checkout whose default branch origin holds completely.

 @param fixture - the stub fixture
 */
async function scriptOriginHoldsDefaultBranch(fixture: StubFixture): Promise<void> {
  await fixture.script({
    key: 'git.ls-remote',
    stdout: `ref: refs/heads/${stubbedDefaultBranch}\tHEAD\n0123456789abcdef0123456789abcdef01234567\tHEAD\n`,
  });
}

/**
 The registry entry of a crate that other crates of the repository depend on, which the dependency tests write as an
 alpha.
 */
const dependencyEntry = packageRegistry.get('forbidden-regex');
if (dependencyEntry === undefined)
  throw new Error('the registry has no forbidden-regex entry');

/**
 Directory of the dependency crate, as a constant that stays typed inside functions.
 */
const dependencyDirectory: string = dependencyEntry.crateDirectory;

/**
 Make the fixture crate `package/demo/cli` depend on `forbidden-regex`, which sits at an alpha version.

 @param fixture - fixture to fill
 */
async function writeAlphaDependency(fixture: StubFixture): Promise<void> {
  await fixture.write({
    path: 'package/demo/cli/Cargo.toml',
    text: '[package]\nname = "demo-cli"\nversion = "0.4.9"\n\n[dependencies]\n'
      + 'forbidden-regex = { path = "../../forbidden-strings/core", version = "=0.1.0-alpha.1" }\n',
  });
  await fixture.write({
    path: `${dependencyDirectory}/Cargo.toml`,
    text: '[package]\nname = "forbidden-regex"\nversion = "0.1.0-alpha.1"\n',
  });
}

await describe({
  name: '',
  children: [
    describe({
      name: 'release executable',
      children: [
        it({
          name: 'refuses a plain-number release whose dependency is at alpha, names the dependency, and changes and starts nothing',
          fn: async () => {
            await using fixture = await createStubFixture();
            await writeRepository(fixture);
            await writeAlphaDependency(fixture);
            const child = runRelease({ fixture, extra: ['0.4.9', '0.5.0', 'n'] });
            expect(child.status).toBe(usageExitCode);
            expect(child.stderr).toContain('forbidden-regex');
            expect(child.stderr).toContain('0.1.0-alpha.1');
            expect(await fixture.read('package/demo/cli/Cargo.toml')).toContain('version = "0.4.9"');
            expect(await fixture.calls()).toEqual([]);
          },
        }),
        it({
          name: 'releases an alpha whose dependency is at alpha, and starts no publish dry run',
          fn: async () => {
            await using fixture = await createStubFixture();
            await writeRepository(fixture);
            await writeAlphaDependency(fixture);
            const child = runRelease({ fixture, extra: ['0.4.9', '0.5.0-alpha.1', 'n'] });
            expect(child.status).toBe(0);
            expect(child.stdout).toContain('RELEASE COMMITTED');
            expect((await fixture.calls()).map(function programOf(call) {
              return call.program;
            })).toEqual([cargoProgram, cargoProgram, gitProgram]);
          },
        }),
        it({
          name: 'bumps the manifests and starts the four programs in order with the hostile note as one argument',
          fn: async () => {
            await using fixture = await createStubFixture();
            await writeRepository(fixture);
            const child = runRelease({ fixture, extra: ['0.4.9', '0.4.10', hostileNote] });
            expect(child.status).toBe(0);
            expect(child.stdout).toContain('RELEASE COMMITTED');
            expect(await fixture.read('package/demo/cli/Cargo.toml')).toContain('version = "0.4.10"');
            expect(await fixture.read('package/demo/cli.fuzz/Cargo.toml')).toContain('version = "=0.4.10"');
            const calls = await fixture.calls();
            expect(calls.map(function programOf(call) {
              return call.program;
            })).toEqual([cargoProgram, cargoProgram, systemdRunProgram, gitProgram]);
            expect(calls[0]?.args).toEqual(['check', '--offline', '--bin', 'demo-bin']);
            expect(calls[0]?.cwd).toBe(`${fixture.directory}/package/demo/cli`);
            expect(calls[1]?.args).toEqual(['update', '--offline', '--workspace']);
            expect(calls[1]?.cwd).toBe(`${fixture.directory}/package/demo/cli.fuzz`);
            expect(calls[3]?.args).toEqual([
              'commit',
              '--message',
              `${commitType}(demo/cli): release 0.4.10: ${hostileNote}`,
              '--',
              'package/demo/cli/Cargo.toml',
              'package/demo/cli/Cargo.lock',
              'package/demo/cli.fuzz/Cargo.toml',
              'package/demo/cli.fuzz/Cargo.lock',
            ]);
            expect(calls[3]?.cwd).toBe(fixture.directory);
          },
        }),
        it({
          name: 'releases a crate of the registry on a runner with cargo only, the network open and no unit',
          fn: async () => {
            await using fixture = await createStubFixture();
            await writeRegistryRepository(fixture);
            const child = spawnSync('node', [
              releaseBin,
              '--root',
              fixture.directory,
              '--crate',
              'forbidden-strings',
              '--host',
              'runner',
              '0.6.0',
              '0.6.1',
              hostileNote,
            ], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.stderr).toBe('');
            expect(child.status).toBe(0);
            expect(child.stdout).toContain('RELEASE COMMITTED');
            expect(await fixture.read('package/forbidden-strings/cli/Cargo.toml')).toContain('version = "0.6.1"');
            expect(await fixture.read('package/git-policy/cli/Cargo.toml')).toContain('version = "=0.6.1"');
            const calls = await fixture.calls();
            const lockCount = scannerLockDirectories.length;
            expect(calls.map(function programOf(call) {
              return call.program;
            })).toEqual(['cargo', ...scannerLockDirectories.map(function cargoOf() {
              return 'cargo';
            }), 'cargo', 'git']);
            expect(calls[0]?.args).toEqual(['check', '--bin', 'forbidden-strings']);
            expect(calls.slice(1, 1 + lockCount).map(function argsOf(call) {
              return call.args;
            })).toEqual(scannerLockDirectories.map(function updateOf() {
              return ['update', '--workspace'];
            }));
            expect(calls.slice(1, 1 + lockCount).map(function cwdOf(call) {
              return call.cwd;
            })).toEqual(scannerLockDirectories.map(function directoryOf(directory) {
              return `${fixture.directory}/${directory}`;
            }));
            expect(calls[1 + lockCount]?.args).toEqual(['publish', '--dry-run', '--locked', '--allow-dirty']);
            expect(calls[2 + lockCount]?.args.slice(0, 3)).toEqual([
              'commit',
              '--message',
              `chore(forbidden-strings/cli): release 0.6.1: ${hostileNote}`,
            ]);
          },
        }),
        it({
          name: 'refuses on a runner a crate that already holds the target and starts no program',
          fn: async () => {
            await using fixture = await createStubFixture();
            await writeRegistryRepository(fixture);
            const child = spawnSync('node', [
              releaseBin,
              '--root',
              fixture.directory,
              '--crate',
              'forbidden-strings',
              '--host',
              'runner',
              '0.5.9',
              '0.6.0',
              'n',
            ], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(usageExitCode);
            expect(child.stderr).toContain('package/forbidden-strings/cli/Cargo.toml already holds 0.6.0');
            expect(await fixture.calls()).toEqual([]);
          },
        }),
        it({
          name: 'refuses a crate the registry does not list, naming the crates it lists',
          fn: async () => {
            await using fixture = await createStubFixture();
            const child = spawnSync('node', [
              releaseBin,
              '--root',
              fixture.directory,
              '--crate',
              'nope',
              '0.1.0',
              '0.1.1',
              'n',
            ], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(usageExitCode);
            expect(child.stderr).toContain(listedCrateNames);
            expect(await fixture.calls()).toEqual([]);
          },
        }),
        it({
          name: 'stops at a failing step, exits 1 and names the step',
          fn: async () => {
            await using fixture = await createStubFixture();
            await writeRepository(fixture);
            await fixture.script({ key: 'cargo', status: 101 });
            const child = runRelease({ fixture, extra: ['0.4.9', '0.4.10', 'n'] });
            expect(child.status).toBe(failureExitCode);
            expect(child.stdout).toContain('FAILED cargo check (101)');
            expect((await fixture.calls()).length).toBe(1);
          },
        }),
        it({
          name: 'refuses a pin that is not the current version before running anything',
          fn: async () => {
            await using fixture = await createStubFixture();
            await writeRepository(fixture);
            const child = runRelease({ fixture, extra: ['0.4.7', '0.4.8', 'n'] });
            expect(child.status).toBe(usageExitCode);
            expect(child.stderr).toContain('"0.4.9"');
            expect(await fixture.read('package/demo/cli/Cargo.toml')).toContain('version = "0.4.9"');
            expect(await fixture.calls()).toEqual([]);
          },
        }),
        it({
          name: 'prints the usage and exits 2 when run without arguments',
          fn: async () => {
            await using fixture = await createStubFixture();
            const child = spawnSync('node', [releaseBin], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(usageExitCode);
            expect(child.stderr).toContain('Usage');
            expect(child.stderr).toContain('--crate-name');
          },
        }),
        it({
          name: 'prints the usage on standard output and exits 0 for --help',
          fn: async () => {
            await using fixture = await createStubFixture();
            const child = spawnSync('node', [releaseBin, '--help'], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(0);
            expect(child.stdout).toContain('Usage');
          },
        }),
        it({
          name: 'reports bad input as a message without a stack trace and exits 2',
          fn: async () => {
            await using fixture = await createStubFixture();
            const child = runRelease({ fixture, extra: ['0.4.9', 'v0.4.10', 'n'] });
            expect(child.status).toBe(usageExitCode);
            expect(child.stderr).toContain('<to>');
            expect(child.stderr).not.toContain('    at ');
          },
        }),
      ],
    }),
    describe({
      name: 'dispatch executable',
      children: [
        it({
          name: 'refuses a plain-number release whose dependency is at alpha before it asks origin or starts gh',
          fn: async () => {
            await using fixture = await createStubFixture();
            await writeRegistryRepository(fixture);
            await fixture.write({
              path: 'package/forbidden-strings/cli/Cargo.toml',
              text: '[package]\nname = "forbidden-strings"\nversion = "0.6.0"\n\n[dependencies]\n'
                + 'forbidden-regex = { path = "../core", version = "=0.1.0-alpha.1" }\n',
            });
            await fixture.write({
              path: `${dependencyDirectory}/Cargo.toml`,
              text: '[package]\nname = "forbidden-regex"\nversion = "0.1.0-alpha.1"\n',
            });
            const child = spawnSync('node', [
              dispatchBin,
              '--root',
              fixture.directory,
              '--crate',
              'forbidden-strings',
              '0.6.0',
              '0.7.0',
              'n',
            ], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(usageExitCode);
            expect(child.stderr).toContain('forbidden-regex');
            expect(child.stderr).toContain('0.1.0-alpha.1');
            expect(await fixture.calls()).toEqual([]);
          },
        }),
        it({
          name: 'dispatches an alpha release whose dependency is at alpha',
          fn: async () => {
            await using fixture = await createStubFixture();
            await writeRegistryRepository(fixture);
            await fixture.write({
              path: 'package/forbidden-strings/cli/Cargo.toml',
              text: '[package]\nname = "forbidden-strings"\nversion = "0.6.0"\n\n[dependencies]\n'
                + 'forbidden-regex = { path = "../core", version = "=0.1.0-alpha.1" }\n',
            });
            await fixture.write({
              path: `${dependencyDirectory}/Cargo.toml`,
              text: '[package]\nname = "forbidden-regex"\nversion = "0.1.0-alpha.1"\n',
            });
            await scriptOriginHoldsDefaultBranch(fixture);
            await fixture.script({
              key: 'gh.workflow-run',
              stdout: 'https://github.com/o/r/actions/runs/43\n',
            });
            const child = spawnSync('node', [
              dispatchBin,
              '--root',
              fixture.directory,
              '--crate',
              'forbidden-strings',
              '0.6.0',
              '0.7.0-alpha.1',
              'n',
            ], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(0);
            expect(child.stdout).toContain('RELEASE STARTED https://github.com/o/r/actions/runs/43');
          },
        }),
        it({
          name: 'starts the release workflow for a registry crate with gh only and prints the run address',
          fn: async () => {
            await using fixture = await createStubFixture();
            await writeRegistryRepository(fixture);
            await scriptOriginHoldsDefaultBranch(fixture);
            await fixture.script({
              key: 'gh.workflow-run',
              stdout: '✓ Created workflow_dispatch event for crate-release.yml at main\nhttps://github.com/o/r/actions/runs/42\n',
            });
            const child = spawnSync('node', [
              dispatchBin,
              '--root',
              fixture.directory,
              '--crate',
              'forbidden-strings',
              '0.6.0',
              '0.6.1',
              hostileNote,
            ], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.stderr).toBe('');
            expect(child.status).toBe(0);
            expect(child.stdout).toContain('RELEASE STARTED https://github.com/o/r/actions/runs/42');
            const calls = await fixture.calls();
            expect(calls.map(function programAndSubcommand(call) {
              return `${call.program} ${call.args[0]}`;
            })).toEqual([
              'git ls-remote',
              'git fetch',
              'git merge-base',
              'gh workflow',
            ]);
            expect(calls.at(-1)).toEqual({
              program: 'gh',
              args: [
                'workflow',
                'run',
                'crate-release.yml',
                '--raw-field=crate=forbidden-strings',
                '--raw-field=from=0.6.0',
                '--raw-field=to=0.6.1',
                `--raw-field=note=${hostileNote}`,
              ],
              cwd: fixture.directory,
            });
          },
        }),
        it({
          name: 'refuses a crate the registry does not list before it starts gh',
          fn: async () => {
            await using fixture = await createStubFixture();
            const child = spawnSync('node', [
              dispatchBin,
              '--root',
              fixture.directory,
              '--crate',
              'nope',
              '0.6.0',
              '0.6.1',
              'n',
            ], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(usageExitCode);
            expect(child.stderr).toContain(listedCrateNames);
            expect(await fixture.calls()).toEqual([]);
          },
        }),
        it({
          name: 'exits 1 and shows what gh said when gh refuses to start the workflow',
          fn: async () => {
            await using fixture = await createStubFixture();
            await writeRegistryRepository(fixture);
            await scriptOriginHoldsDefaultBranch(fixture);
            await fixture.script({ key: 'gh.workflow-run', status: 1 });
            const child = spawnSync('node', [
              dispatchBin,
              '--root',
              fixture.directory,
              '--crate',
              'forbidden-strings',
              '0.6.0',
              '0.6.1',
              'n',
            ], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(failureExitCode);
            expect(child.stdout).toContain(`FAILED ${releaseWorkflow} was not started (gh exited 1)`);
          },
        }),
        it({
          name: 'exits 1 without starting gh when origin lacks a commit of the local default branch',
          fn: async () => {
            await using fixture = await createStubFixture();
            await writeRegistryRepository(fixture);
            await scriptOriginHoldsDefaultBranch(fixture);
            await fixture.script({ key: 'git.merge-base', status: 1 });
            await fixture.script({ key: 'git.rev-list', stdout: '2\n' });
            await fixture.script({ key: 'git.log', stdout: 'abc1234\tNewest unpushed commit\n' });
            const child = spawnSync('node', [
              dispatchBin,
              '--root',
              fixture.directory,
              '--crate',
              'forbidden-strings',
              '0.6.0',
              '0.6.1',
              'n',
            ], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(failureExitCode);
            expect(child.stdout).toContain(`2 commits of local ${stubbedDefaultBranch}`);
            expect(child.stdout).toContain('abc1234 Newest unpushed commit');
            expect(child.stdout).toContain(`git push origin ${stubbedDefaultBranch}`);
            expect((await fixture.calls()).some(function isGh(call) {
              return call.program === 'gh';
            })).toBe(false);
          },
        }),
        it({
          name: 'prints the usage and exits 2 when run without arguments',
          fn: async () => {
            await using fixture = await createStubFixture();
            const child = spawnSync('node', [dispatchBin], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(usageExitCode);
            expect(child.stderr).toContain('Usage: crate-release-dispatch');
          },
        }),
      ],
    }),
    describe({
      name: 'await executable',
      children: [
        it({
          name: 'follows the release run and the publish run it started, then checks crates.io and the release',
          fn: async () => {
            await using fixture = await createStubFixture();
            await fixture.script({ key: 'gh.run-view', stdout: JSON.stringify(recordedReleaseRunPassed) });
            const unrelated = { databaseId: 901, headSha: 'a'.repeat(40), createdAt: '2026-11-11T01:04:00Z' };
            const own = { databaseId: 902, headSha: 'b'.repeat(40), createdAt: '2026-11-11T01:05:00Z' };
            await fixture.script({ key: 'gh.run-list', stdout: JSON.stringify([own, unrelated]) });
            await fixture.script({
              key: manifestStubKey(unrelated.headSha),
              stdout: '[package]\nname = "forbidden-strings"\nversion = "0.6.0"\n',
            });
            await fixture.script({
              key: manifestStubKey(own.headSha),
              stdout: '[package]\nname = "forbidden-strings"\nversion = "0.6.1"\n',
            });
            await fixture.script({ key: 'curl', stdout: '{"version":{"created_at":"2026-11-01T10:30:00+00:00"}}' });
            await fixture.script({
              key: 'gh.release-view',
              stdout: JSON.stringify({
                assets: [
                  { name: 'forbidden-strings-0.6.1-x86_64-unknown-linux-gnu.tar.gz' },
                  { name: 'forbidden-strings-0.6.1-aarch64-unknown-linux-gnu.tar.gz' },
                ],
              }),
            });
            const child = spawnSync('node', [
              awaitBin,
              '--root',
              fixture.directory,
              '--crate',
              'forbidden-strings',
              'https://github.com/o/r/actions/runs/500',
              '0.6.1',
            ], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.stderr).toBe('');
            expect(child.stdout).toContain('PUBLISHED forbidden-strings-v0.6.1');
            expect(child.status).toBe(0);
            const calls = await fixture.calls();
            const summaries = calls.map(function argsOf(call) {
              return call.args.slice(0, call.args[0] === 'api' ? 2 : 3).join(' ');
            });
            const manifestReads = [`api ${manifestEndpoint(unrelated.headSha)}`, `api ${manifestEndpoint(own.headSha)}`];
            expect(summaries.slice(0, 2)).toEqual([
              'run view 500',
              'run list --workflow=cargo-publish.yml',
            ]);
            // The manifests are read at once, so the order in which the two stubs started is not defined.
            expect(summaries.slice(2, 2 + manifestReads.length).toSorted()).toEqual(manifestReads.toSorted());
            expect(summaries.slice(2 + manifestReads.length)).toEqual([
              'run view 902',
              '--silent --show-error --max-time',
              'release view forbidden-strings-v0.6.1',
            ]);
          },
        }),
        it({
          name: 'exits 1 without looking for a publish run when the release run failed, naming the job and the step',
          fn: async () => {
            await using fixture = await createStubFixture();
            await fixture.script({ key: 'gh.run-view', stdout: JSON.stringify(recordedReleaseRunFailed) });
            const child = spawnSync('node', [
              awaitBin,
              '--root',
              fixture.directory,
              '--crate',
              'forbidden-strings',
              '500',
              '0.6.1',
            ], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(failureExitCode);
            expect(child.stdout).toContain('RELEASE RUN FAILED');
            expect(child.stdout).toContain(
              'job release concluded failure at step "Install Node, pnpm and the workspace dependencies"',
            );
            expect(await fixture.calls()).toHaveLength(1);
          },
        }),
        it({
          name: 'refuses a run that is not a run address before starting any program',
          fn: async () => {
            await using fixture = await createStubFixture();
            const child = spawnSync('node', [
              awaitBin,
              '--root',
              fixture.directory,
              '--crate',
              'forbidden-strings',
              'https://github.com/o/r/pull/3',
              '0.6.1',
            ], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(usageExitCode);
            expect(child.stderr).toContain('<run>');
            expect(await fixture.calls()).toEqual([]);
          },
        }),
        it({
          name: 'prints the usage and exits 2 when run without arguments',
          fn: async () => {
            await using fixture = await createStubFixture();
            const child = spawnSync('node', [awaitBin], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(usageExitCode);
            expect(child.stderr).toContain('Usage: crate-release-await');
          },
        }),
      ],
    }),
    describe({
      name: 'push executable',
      children: [
        it({
          name: 'sends the head commit as one signed-by-GitHub commit on its parent, with the whole contents of its files',
          fn: async () => {
            await using fixture = await createStubFixture();
            await writePushFixture(fixture);
            await fixture.script({
              key: 'gh.api-graphql',
              stdout: '{"data":{"createCommitOnBranch":{"commit":{"oid":"cccc","url":"https://github.com/o/r/commit/cccc"}}}}',
            });
            const child = runPush({ fixture, extra: [] });
            expect(child.stderr).toBe('');
            expect(child.stdout).toContain('PUSHED cccc https://github.com/o/r/commit/cccc');
            expect(child.status).toBe(0);
            const request = JSON.parse(await fixture.read(`scratch/${requestFileName}`)) as {
              variables: { input: { expectedHeadOid: string; message: { headline: string }; fileChanges: { additions: { path: string; contents: string }[] } } };
            };
            expect(request.variables.input.expectedHeadOid).toBe(pushBase);
            expect(request.variables.input.message.headline).toBe(pushHeadline);
            expect(request.variables.input.fileChanges.additions).toEqual(pushPaths.map(function additionOf(path) {
              return { path, contents: Buffer.from(`contents of ${path}\n`).toString('base64') };
            }));
            const calls = await fixture.calls();
            expect(calls.at(-1)?.program).toBe('gh');
            expect(calls.at(-1)?.args).toEqual(['api', 'graphql', `--input=${fixture.directory}/scratch/${requestFileName}`]);
          },
        }),
        it({
          name: 'records the closure of the crate and of its lock directory with cargo metadata before it sends',
          fn: async () => {
            await using fixture = await createStubFixture();
            await writePushFixture(fixture);
            await fixture.script({
              key: 'gh.api-graphql',
              stdout: '{"data":{"createCommitOnBranch":{"commit":{"oid":"cccc","url":"https://github.com/o/r/commit/cccc"}}}}',
            });
            runPush({ fixture, extra: [] });
            const calls = await fixture.calls();
            const cargo = calls.filter(function isCargo(call) {
              return call.program === 'cargo';
            });
            expect(cargo.map(function manifestOf(call) {
              return call.args.find(function isManifest(argument) {
                return argument.startsWith('--manifest-path=');
              }) ?? '';
            }).toSorted()).toEqual([
              'package/forbidden-strings/cli/Cargo.toml',
              ...scannerLockDirectories.map(function manifestOf(directory) {
                return `${directory}/Cargo.toml`;
              }),
            ].toSorted().map(function optionOf(manifest) {
              return `--manifest-path=${manifest}`;
            }));
            expect(cargo.every(function inRoot(call) {
              return call.cwd === fixture.directory;
            })).toBe(true);
          },
        }),
        it({
          name: 'pushes nothing and names the path when the branch moved and a path of the crate changed',
          fn: async () => {
            await using fixture = await createStubFixture();
            await writePushFixture(fixture);
            await fixture.script({ key: 'gh.api-graphql', stdout: pushStaleAnswer, status: 1 });
            await fixture.script({
              key: 'gh.api-repos_o_r_git_ref_heads_main',
              stdout: JSON.stringify({ object: { type: 'commit', sha: pushMoved } }),
            });
            const changed = 'package/forbidden-strings/cli/src/lib.rs';
            await fixture.script({
              key: `gh.api-repos_o_r_compare_${pushBase}...${pushMoved}`,
              stdout: JSON.stringify({ status: 'ahead', files: [{ filename: 'doc/a.md' }, { filename: changed }] }),
            });
            const child = runPush({ fixture, extra: [] });
            expect(child.status).toBe(failureExitCode);
            expect(child.stdout).toContain(`PUSH REFUSED: Paths the release depends on changed on main since ${pushBase}: ${changed}`);
            expect(child.stdout).toContain('dispatch the release again');
            expect(child.stdout).toContain(`main now points to ${pushMoved}`);
            const graphql = (await fixture.calls()).filter(function isGraphql(call) {
              return (call.program === 'gh') && (call.args[1] === 'graphql');
            });
            expect(graphql).toHaveLength(1);
          },
        }),
        it({
          name: 'sends again on the new head while nothing the release depends on changes, and gives up at the limit',
          fn: async () => {
            await using fixture = await createStubFixture();
            await writePushFixture(fixture);
            await fixture.script({ key: 'gh.api-graphql', stdout: pushStaleAnswer, status: 1 });
            await fixture.script({
              key: 'gh.api-repos_o_r_git_ref_heads_main',
              stdout: JSON.stringify({ object: { type: 'commit', sha: pushMoved } }),
            });
            await fixture.script({
              key: `gh.api-repos_o_r_compare_${pushBase}...${pushMoved}`,
              stdout: JSON.stringify({ status: 'ahead', files: [{ filename: 'doc/a.md' }] }),
            });
            const attempts = 2;
            const child = runPush({ fixture, extra: ['--max-attempts', String(attempts)] });
            expect(child.status).toBe(failureExitCode);
            expect(child.stdout).toContain(`decision: none of the watched paths changed; send the commit again on ${pushMoved}`);
            expect(child.stdout).toContain(`moved before each of the ${String(attempts)} allowed sends`);
            const graphql = (await fixture.calls()).filter(function isGraphql(call) {
              return (call.program === 'gh') && (call.args[1] === 'graphql');
            });
            expect(graphql).toHaveLength(attempts);
          },
        }),
        it({
          name: 'exits 1 and says nothing was pushed when GitHub refuses the commit for another reason',
          fn: async () => {
            await using fixture = await createStubFixture();
            await writePushFixture(fixture);
            await fixture.script({
              key: 'gh.api-graphql',
              stdout: '{"errors":[{"message":"Resource not accessible by integration"}]}',
              status: 1,
            });
            const child = runPush({ fixture, extra: [] });
            expect(child.status).toBe(failureExitCode);
            expect(child.stdout).toContain('PUSH REFUSED: Resource not accessible by integration');
            expect(child.stdout).toContain('Nothing was pushed');
          },
        }),
        it({
          name: 'refuses a crate the registry does not list before starting any program',
          fn: async () => {
            await using fixture = await createStubFixture();
            const child = spawnSync('node', [
              pushBin,
              '--root',
              fixture.directory,
              '--crate',
              'nope',
              '--repo',
              'o/r',
              '--branch',
              'main',
              '--scratch-dir',
              `${fixture.directory}/scratch`,
            ], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(usageExitCode);
            expect(child.stderr).toContain(listedCrateNames);
            expect(await fixture.calls()).toEqual([]);
          },
        }),
        it({
          name: 'refuses a relative scratch directory before starting any program',
          fn: async () => {
            await using fixture = await createStubFixture();
            const child = spawnSync('node', [
              pushBin,
              '--root',
              fixture.directory,
              '--crate',
              'forbidden-strings',
              '--repo',
              'o/r',
              '--branch',
              'main',
              '--scratch-dir',
              'scratch',
            ], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(usageExitCode);
            expect(child.stderr).toContain('--scratch-dir');
            expect(await fixture.calls()).toEqual([]);
          },
        }),
        it({
          name: 'prints the usage and exits 2 when run without arguments',
          fn: async () => {
            await using fixture = await createStubFixture();
            const child = spawnSync('node', [pushBin], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(usageExitCode);
            expect(child.stderr).toContain('Usage: crate-release-push');
          },
        }),
      ],
    }),
    describe({
      name: 'wait executable',
      children: [
        it({
          name: 'finds the run, reads what it says about itself, checks crates.io and the release, and prints PUBLISHED',
          fn: async () => {
            await using fixture = await createStubFixture();
            await fixture.script({ key: 'gh.run-view', stdout: JSON.stringify(recordedReleaseRunPassed) });
            await fixture.script({
              key: 'gh.run-list',
              stdout: JSON.stringify([{ databaseId: 77, headSha: 'abcdef0123456789abcdef0123456789abcdef01' }]),
            });
            await fixture.script({ key: 'curl', stdout: '{"version":{"created_at":"2026-10-29T12:00:00+00:00"}}' });
            await fixture.script({
              key: 'gh.release-view',
              stdout: JSON.stringify({
                assets: [
                  { name: 'demo-cli-0.4.10-x86_64-unknown-linux-gnu.tar.gz' },
                  { name: 'demo-cli-0.4.10-aarch64-unknown-linux-gnu.tar.gz' },
                ],
              }),
            });
            const child = runWait({ fixture, extra: ['abcdef0', '0.4.10'] });
            expect(child.stdout).toContain('cargo-publish.yml run 77: completed with success');
            expect(child.stdout).toContain('PUBLISHED demo-cli-v0.4.10');
            expect(child.status).toBe(0);
            const calls = await fixture.calls();
            expect(calls.map(function argsOf(call) {
              return call.args.slice(0, 2).join(' ');
            })).toEqual(['run list', 'run view', '--silent --show-error', 'release view']);
            expect(calls[3]?.args[2]).toBe('demo-cli-v0.4.10');
          },
        }),
        it({
          name: 'exits 1 and names the bound when no run appears',
          fn: async () => {
            await using fixture = await createStubFixture();
            await fixture.script({ key: 'gh.run-list', stdout: '[]' });
            const child = runWait({ fixture, extra: ['abcdef0', '0.4.10'] });
            expect(child.status).toBe(failureExitCode);
            expect(child.stdout).toContain('no cargo-publish.yml run for abcdef0 within 2 seconds');
          },
        }),
        it({
          name: 'exits 1 when crates.io does not know the version',
          fn: async () => {
            await using fixture = await createStubFixture();
            await fixture.script({
              key: 'gh.run-list',
              stdout: JSON.stringify([{ databaseId: 77, headSha: 'abcdef0123456789abcdef0123456789abcdef01' }]),
            });
            await fixture.script({ key: 'curl', stdout: '{"errors":[]}' });
            const child = runWait({ fixture, extra: ['abcdef0', '0.4.10'] });
            expect(child.status).toBe(failureExitCode);
            expect(child.stdout).toContain('crates.io 0.4.10: not found');
          },
        }),
        it({
          name: 'refuses a hash that is a query fragment before starting any program',
          fn: async () => {
            await using fixture = await createStubFixture();
            const child = runWait({ fixture, extra: ['") | .x', '0.4.10'] });
            expect(child.status).toBe(usageExitCode);
            expect(child.stderr).toContain('<sha>');
            expect(await fixture.calls()).toEqual([]);
          },
        }),
        it({
          name: 'prints the usage and exits 2 when run without arguments',
          fn: async () => {
            await using fixture = await createStubFixture();
            const child = spawnSync('node', [waitBin], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(usageExitCode);
            expect(child.stderr).toContain('Usage');
          },
        }),
      ],
    }),
  ],
});
