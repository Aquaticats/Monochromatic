import { spawnSync } from 'node:child_process';
import { resolve } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type StubFixture,
  withStubFixture,
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
    '--lock-task',
    '//package/demo/cli.fuzz:lock:members',
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
            await withStubFixture(async function body(fixture) {
              await writeRepository(fixture);
              const child = runRelease({ fixture, extra: ['0.4.9', '0.4.10', hostileNote] });
              expect(child.status).toBe(0);
              expect(child.stdout).toContain('RELEASE COMMITTED');
              expect(await fixture.read('package/demo/cli/Cargo.toml')).toContain('version = "0.4.10"');
              expect(await fixture.read('package/demo/cli.fuzz/Cargo.toml')).toContain('version = "=0.4.10"');
              const calls = await fixture.calls();
              expect(calls.map(function programOf(call) {
                return call.program;
              })).toEqual(['cargo', 'mise', 'systemd-run', 'git']);
              expect(calls[0]?.args).toEqual(['check', '--offline', '--bin', 'demo-bin']);
              expect(calls[0]?.cwd).toBe(`${fixture.directory}/package/demo/cli`);
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
            });
          },
        }),
        it({
          name: 'stops at a failing step, exits 1 and names the step',
          fn: async () => {
            await withStubFixture(async function body(fixture) {
              await writeRepository(fixture);
              await fixture.script({ key: 'cargo', status: 101 });
              const child = runRelease({ fixture, extra: ['0.4.9', '0.4.10', 'n'] });
              expect(child.status).toBe(1);
              expect(child.stdout).toContain('FAILED cargo check (101)');
              expect((await fixture.calls()).length).toBe(1);
            });
          },
        }),
        it({
          name: 'refuses a pin that is not the current version before running anything',
          fn: async () => {
            await withStubFixture(async function body(fixture) {
              await writeRepository(fixture);
              const child = runRelease({ fixture, extra: ['0.4.7', '0.4.8', 'n'] });
              expect(child.status).toBe(2);
              expect(child.stderr).toContain('"0.4.9"');
              expect(await fixture.read('package/demo/cli/Cargo.toml')).toContain('version = "0.4.9"');
              expect(await fixture.calls()).toEqual([]);
            });
          },
        }),
        it({
          name: 'prints the usage and exits 2 when run without arguments',
          fn: async () => {
            await withStubFixture(async function body(fixture) {
              const child = spawnSync('node', [releaseBin], { env: fixture.environment(), encoding: 'utf8' });
              expect(child.status).toBe(2);
              expect(child.stderr).toContain('Usage');
              expect(child.stderr).toContain('--crate-name');
            });
          },
        }),
        it({
          name: 'prints the usage on standard output and exits 0 for --help',
          fn: async () => {
            await withStubFixture(async function body(fixture) {
              const child = spawnSync('node', [releaseBin, '--help'], { env: fixture.environment(), encoding: 'utf8' });
              expect(child.status).toBe(0);
              expect(child.stdout).toContain('Usage');
            });
          },
        }),
        it({
          name: 'reports bad input as a message without a stack trace and exits 2',
          fn: async () => {
            await withStubFixture(async function body(fixture) {
              const child = runRelease({ fixture, extra: ['0.4.9', 'v0.4.10', 'n'] });
              expect(child.status).toBe(2);
              expect(child.stderr).toContain('<to>');
              expect(child.stderr).not.toContain('    at ');
            });
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
            await withStubFixture(async function body(fixture) {
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
            });
          },
        }),
        it({
          name: 'exits 1 and names the bound when no run appears',
          fn: async () => {
            await withStubFixture(async function body(fixture) {
              await fixture.script({ key: 'gh.run-list', stdout: '[]' });
              const child = runWait({ fixture, extra: ['abcdef0', '0.4.10'] });
              expect(child.status).toBe(1);
              expect(child.stdout).toContain('no cargo-publish.yml run for abcdef0 within 2 seconds');
            });
          },
        }),
        it({
          name: 'exits 1 when crates.io does not know the version',
          fn: async () => {
            await withStubFixture(async function body(fixture) {
              await fixture.script({
                key: 'gh.run-list',
                stdout: JSON.stringify([{ databaseId: 77, headSha: 'abcdef0123456789abcdef0123456789abcdef01' }]),
              });
              await fixture.script({ key: 'curl', stdout: '{"errors":[]}' });
              const child = runWait({ fixture, extra: ['abcdef0', '0.4.10'] });
              expect(child.status).toBe(1);
              expect(child.stdout).toContain('crates.io 0.4.10: not found');
            });
          },
        }),
        it({
          name: 'refuses a hash that is a query fragment before starting any program',
          fn: async () => {
            await withStubFixture(async function body(fixture) {
              const child = runWait({ fixture, extra: ['") | .x', '0.4.10'] });
              expect(child.status).toBe(2);
              expect(child.stderr).toContain('<sha>');
              expect(await fixture.calls()).toEqual([]);
            });
          },
        }),
        it({
          name: 'prints the usage and exits 2 when run without arguments',
          fn: async () => {
            await withStubFixture(async function body(fixture) {
              const child = spawnSync('node', [waitBin], { env: fixture.environment(), encoding: 'utf8' });
              expect(child.status).toBe(2);
              expect(child.stderr).toContain('Usage');
            });
          },
        }),
      ],
    }),
  ],
});
