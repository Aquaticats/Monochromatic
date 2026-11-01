import { spawnSync } from 'node:child_process';
import { resolve } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

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
  // The lock directory must exist, because the lock step starts cargo inside it.
  await fixture.write({
    path: 'package/git-policy/cli.fuzz/Cargo.toml',
    text: '[package]\nname = "fuzz"\nversion = "0.0.0"\n',
  });
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

await describe({
  name: '',
  children: [
    describe({
      name: 'release executable',
      children: [
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
            })).toEqual(['cargo', 'cargo', 'systemd-run', 'git']);
            expect(calls[0]?.args).toEqual(['check', '--offline', '--bin', 'demo-bin']);
            expect(calls[0]?.cwd).toBe(`${fixture.directory}/package/demo/cli`);
            expect(calls[1]?.args).toEqual(['update', '--offline', '--workspace']);
            expect(calls[1]?.cwd).toBe(`${fixture.directory}/package/demo/cli.fuzz`);
            expect(calls[3]?.args).toEqual([
              'commit',
              '--message',
              `chore(demo/cli): release 0.4.10: ${hostileNote}`,
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
            expect(calls.map(function programOf(call) {
              return call.program;
            })).toEqual(['cargo', 'cargo', 'cargo', 'git']);
            expect(calls[0]?.args).toEqual(['check', '--bin', 'forbidden-strings']);
            expect(calls[1]?.args).toEqual(['update', '--workspace']);
            expect(calls[2]?.args).toEqual(['publish', '--dry-run', '--locked', '--allow-dirty']);
            expect(calls[1]?.cwd).toBe(`${fixture.directory}/package/git-policy/cli.fuzz`);
            expect(calls[3]?.args.slice(0, 3)).toEqual([
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
            expect(child.status).toBe(2);
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
            expect(child.status).toBe(2);
            expect(child.stderr).toContain('forbidden-strings, monochromatic-git-policy-cli');
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
            expect(child.status).toBe(1);
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
            expect(child.status).toBe(2);
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
            expect(child.status).toBe(2);
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
            expect(child.status).toBe(2);
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
          name: 'starts the release workflow for a registry crate with gh only and prints the run address',
          fn: async () => {
            await using fixture = await createStubFixture();
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
            expect(calls).toHaveLength(1);
            expect(calls[0]).toEqual({
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
            expect(child.status).toBe(2);
            expect(child.stderr).toContain('forbidden-strings, monochromatic-git-policy-cli');
            expect(await fixture.calls()).toEqual([]);
          },
        }),
        it({
          name: 'exits 1 and shows what gh said when gh refuses to start the workflow',
          fn: async () => {
            await using fixture = await createStubFixture();
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
            expect(child.status).toBe(1);
            expect(child.stdout).toContain('FAILED crate-release.yml was not started (gh exited 1)');
          },
        }),
        it({
          name: 'prints the usage and exits 2 when run without arguments',
          fn: async () => {
            await using fixture = await createStubFixture();
            const child = spawnSync('node', [dispatchBin], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(2);
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
            await fixture.script({ key: 'gh.run-view', stdout: '{"createdAt":"2026-11-01T10:00:00Z"}' });
            await fixture.script({
              key: 'gh.run-list',
              stdout: JSON.stringify([{ databaseId: 902, headSha: 'b'.repeat(40), createdAt: '2026-11-01T10:05:00Z' }]),
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
            expect(child.stdout).toContain('PUBLISHED');
            expect(child.status).toBe(0);
            const calls = await fixture.calls();
            expect(calls.map(function argsOf(call) {
              return call.args.slice(0, 3).join(' ');
            })).toEqual([
              'run watch 500',
              'run view 500',
              'run list --workflow=cargo-publish.yml',
              'run watch 902',
              '--silent --show-error --max-time',
              'release view forbidden-strings-v0.6.1',
            ]);
          },
        }),
        it({
          name: 'exits 1 without looking for a publish run when the release run failed',
          fn: async () => {
            await using fixture = await createStubFixture();
            await fixture.script({ key: 'gh.run-watch', status: 1 });
            const child = spawnSync('node', [
              awaitBin,
              '--root',
              fixture.directory,
              '--crate',
              'forbidden-strings',
              '500',
              '0.6.1',
            ], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(1);
            expect(child.stdout).toContain('RELEASE RUN FAILED');
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
            expect(child.status).toBe(2);
            expect(child.stderr).toContain('<run>');
            expect(await fixture.calls()).toEqual([]);
          },
        }),
        it({
          name: 'prints the usage and exits 2 when run without arguments',
          fn: async () => {
            await using fixture = await createStubFixture();
            const child = spawnSync('node', [awaitBin], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(2);
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
            await fixture.write({ path: 'package/demo/cli/Cargo.toml', text: '[package]\nversion = "0.4.10"\n' });
            await fixture.write({ path: 'package/demo/cli/Cargo.lock', text: '# lock\n' });
            await fixture.script({ key: 'git.rev-list', stdout: `${'b'.repeat(40)} ${'a'.repeat(40)}\n` });
            await fixture.script({ key: 'git.log', stdout: `chore(demo/cli): release 0.4.10: ${hostileNote}\n` });
            await fixture.script({ key: 'git.diff-tree', stdout: 'package/demo/cli/Cargo.toml\0package/demo/cli/Cargo.lock\0' });
            await fixture.script({
              key: 'gh.api-graphql',
              stdout: '{"data":{"createCommitOnBranch":{"commit":{"oid":"cccc","url":"https://github.com/o/r/commit/cccc"}}}}',
            });
            const child = spawnSync('node', [
              pushBin,
              '--root',
              fixture.directory,
              '--repo',
              'o/r',
              '--branch',
              'main',
              '--scratch-dir',
              `${fixture.directory}/scratch`,
            ], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.stderr).toBe('');
            expect(child.stdout).toContain('PUSHED cccc https://github.com/o/r/commit/cccc');
            expect(child.status).toBe(0);
            const request = JSON.parse(await fixture.read('scratch/crate-release-commit-request.json')) as {
              variables: { input: { expectedHeadOid: string; message: { headline: string }; fileChanges: { additions: { path: string; contents: string }[] } } };
            };
            expect(request.variables.input.expectedHeadOid).toBe('a'.repeat(40));
            expect(request.variables.input.message.headline).toBe(`chore(demo/cli): release 0.4.10: ${hostileNote}`);
            expect(request.variables.input.fileChanges.additions).toEqual([
              { path: 'package/demo/cli/Cargo.toml', contents: Buffer.from('[package]\nversion = "0.4.10"\n').toString('base64') },
              { path: 'package/demo/cli/Cargo.lock', contents: Buffer.from('# lock\n').toString('base64') },
            ]);
            const calls = await fixture.calls();
            expect(calls.at(-1)?.program).toBe('gh');
            expect(calls.at(-1)?.args).toEqual(['api', 'graphql', `--input=${fixture.directory}/scratch/crate-release-commit-request.json`]);
          },
        }),
        it({
          name: 'exits 1 and says nothing was pushed when GitHub refuses the commit',
          fn: async () => {
            await using fixture = await createStubFixture();
            await fixture.write({ path: 'package/demo/cli/Cargo.toml', text: 'x' });
            await fixture.script({ key: 'git.rev-list', stdout: `${'b'.repeat(40)} ${'a'.repeat(40)}\n` });
            await fixture.script({ key: 'git.log', stdout: 'subject\n' });
            await fixture.script({ key: 'git.diff-tree', stdout: 'package/demo/cli/Cargo.toml\0' });
            await fixture.script({
              key: 'gh.api-graphql',
              stdout: '{"errors":[{"message":"Expected branch to point to aaa but it did not."}]}',
              status: 1,
            });
            const child = spawnSync('node', [
              pushBin,
              '--root',
              fixture.directory,
              '--repo',
              'o/r',
              '--branch',
              'main',
              '--scratch-dir',
              `${fixture.directory}/scratch`,
            ], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(1);
            expect(child.stdout).toContain('PUSH REFUSED: Expected branch to point to aaa but it did not.');
            expect(child.stdout).toContain('nothing was pushed');
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
              '--repo',
              'o/r',
              '--branch',
              'main',
              '--scratch-dir',
              'scratch',
            ], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(2);
            expect(child.stderr).toContain('--scratch-dir');
            expect(await fixture.calls()).toEqual([]);
          },
        }),
        it({
          name: 'prints the usage and exits 2 when run without arguments',
          fn: async () => {
            await using fixture = await createStubFixture();
            const child = spawnSync('node', [pushBin], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(2);
            expect(child.stderr).toContain('Usage: crate-release-push');
          },
        }),
      ],
    }),
    describe({
      name: 'wait executable',
      children: [
        it({
          name: 'finds the run, watches it, checks crates.io and the release, and prints PUBLISHED',
          fn: async () => {
            await using fixture = await createStubFixture();
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
            expect(child.stdout).toContain('run 77');
            expect(child.stdout).toContain('PUBLISHED');
            expect(child.status).toBe(0);
            const calls = await fixture.calls();
            expect(calls.map(function argsOf(call) {
              return call.args.slice(0, 2).join(' ');
            })).toEqual(['run list', 'run watch', '--silent --show-error', 'release view']);
            expect(calls[3]?.args[2]).toBe('demo-cli-v0.4.10');
          },
        }),
        it({
          name: 'exits 1 and names the bound when no run appears',
          fn: async () => {
            await using fixture = await createStubFixture();
            await fixture.script({ key: 'gh.run-list', stdout: '[]' });
            const child = runWait({ fixture, extra: ['abcdef0', '0.4.10'] });
            expect(child.status).toBe(1);
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
            expect(child.status).toBe(1);
            expect(child.stdout).toContain('crates.io 0.4.10: not found');
          },
        }),
        it({
          name: 'refuses a hash that is a query fragment before starting any program',
          fn: async () => {
            await using fixture = await createStubFixture();
            const child = runWait({ fixture, extra: ['") | .x', '0.4.10'] });
            expect(child.status).toBe(2);
            expect(child.stderr).toContain('<sha>');
            expect(await fixture.calls()).toEqual([]);
          },
        }),
        it({
          name: 'prints the usage and exits 2 when run without arguments',
          fn: async () => {
            await using fixture = await createStubFixture();
            const child = spawnSync('node', [waitBin], { env: fixture.environment(), encoding: 'utf8' });
            expect(child.status).toBe(2);
            expect(child.stderr).toContain('Usage');
          },
        }),
      ],
    }),
  ],
});
