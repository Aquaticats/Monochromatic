/**
 Exercises the deployed `catalog-specifiers-install-policy-cli.ts` and `catalog-specifiers-negative-control.ts` steps
 through subprocesses, as `.github/workflows/catalog-specifiers.yml` starts them.

 The source files are run, not built copies: the workflow runs them right after checkout with the runner's Node and no
 installed dependencies, so the checked-in TypeScript is the artifact that ships. `curl`, `cargo-binstall` and the policy
 CLI are stand-ins; `sha256sum`, `tar` and `git` are the real programs.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  readdir,
  readFile,
  symlink,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  binstallSource,
  createStepFixture,
  policyCliSource,
  type StepFixture,
  type StepRun,
} from './github-actions-step-fixture.ts';

//region Fixtures: a lock, a cargo-binstall archive and the answers that serve it

/**
 The install program under test.
 */
const INSTALL_PROGRAM = join(import.meta.dirname, 'catalog-specifiers-install-policy-cli.ts',);

/**
 The negative-control program under test.
 */
const CONTROL_PROGRAM = join(import.meta.dirname, 'catalog-specifiers-negative-control.ts',);

/**
 Fake secret the runner holds; no output of a step may show it.
 */
const FAKE_SECRET = 'FAKE_TOKEN_123';

/**
 Address the lock gives for the `cargo-binstall` archive, which must start with the pinned release folder.
 */
const BINSTALL_ADDRESS = 'https://github.com/cargo-bins/cargo-binstall/releases/download/v1.2.3/cargo-binstall-x86_64.tgz';

/**
 Version the lock pins for the policy CLI.
 */
const POLICY_CLI_VERSION = '0.6.1-alpha.1';

/**
 Program name of the release after the rename.
 */
const RENAMED_PROGRAM = 'monochromatic-git-policy-cli';

/**
 Program name of a release before the rename.
 */
const EARLIER_PROGRAM = 'cli-git-native';

/**
 What to put in the lock.
 */
type LockRequest = {
  /**
   Address of the archive, or nothing for a lock without the row.
   */
  readonly url?: string;
  /**
   SHA-256 of the archive, or nothing for a lock without it.
   */
  readonly sha256?: string;
  /**
   Version of the policy CLI, or nothing for a lock without it.
   */
  readonly version?: string;
};

/**
 Builds the lock text a request describes.

 @param request - What to put in the lock

 @returns A lock as `toolchain.lock.json` holds it

 @example
 ```ts
 const lock = lockOf({ url: BINSTALL_ADDRESS, },);
 ```
 */
function lockOf({ url, sha256, version, }: LockRequest,): unknown {
  return {
    tools: {
      'cargo-binstall': {
        artifacts: {
          'linux-x64': {
            ...url === undefined ? {} : { url, },
            ...sha256 === undefined ? {} : { sha256, },
          },
        },
      },
      'cargo:monochromatic-git-policy-cli': version === undefined ? {} : { version, },
    },
  };
}

/**
 Sets up a runner whose lock pins a good `cargo-binstall` archive that installs the given program.

 @param fixture - Runner to set up

 @returns The archive's SHA-256

 @example
 ```ts
 const sha256 = await serveBinstall(fixture,);
 ```
 */
async function serveBinstall(fixture: StepFixture,): Promise<string> {
  /**
   The archive, with the stand-in at its root as the real release has it.
   */
  const archive = await fixture.makeArchive({
    name: 'cargo-binstall-x86_64.tgz',
    top: '',
    members: { 'cargo-binstall': binstallSource(), },
  },);
  await fixture.answerCurl([{ urlIncludes: BINSTALL_ADDRESS, status: 200, bodyFile: archive.path, },],);
  return archive.sha256;
}

/**
 Runs the install step with the environment the workflow gives it.

 @param fixture - Runner to run in

 @param extra - More environment variables

 @returns What the step left behind

 @example
 ```ts
 const run = await runInstall({ fixture, extra: { FAKE_BINSTALL_PROGRAM: RENAMED_PROGRAM, }, },);
 ```
 */
async function runInstall(
  {
    fixture,
    extra = {},
  }: {
    readonly fixture: StepFixture;
    readonly extra?: Readonly<Record<string, string>>;
  },
): Promise<StepRun> {
  return await fixture.run({
    command: [
      process.execPath,
      INSTALL_PROGRAM,
    ],
    env: { GITHUB_TOKEN: FAKE_SECRET, ...extra, },
  },);
}

/**
 Runs the negative-control step with the environment the install step announced.

 @param fixture - Runner to run in

 @param extra - More environment variables, such as what the stand-in policy CLI answers

 @returns What the step left behind

 @example
 ```ts
 const run = await runControl({ fixture, extra: { FAKE_CLI_EXIT: '1', }, },);
 ```
 */
async function runControl(
  {
    fixture,
    extra,
  }: {
    readonly fixture: StepFixture;
    readonly extra: Readonly<Record<string, string>>;
  },
): Promise<StepRun> {
  await fixture.writeProgram({ name: RENAMED_PROGRAM, source: policyCliSource(), },);
  return await fixture.run({
    command: [
      process.execPath,
      CONTROL_PROGRAM,
    ],
    env: {
      CLI_PROGRAM: RENAMED_PROGRAM,
      CLI_WORD: RENAMED_PROGRAM,
      CLI_SETTINGS: `${RENAMED_PROGRAM}.config.jsonc`,
      GITHUB_TOKEN: FAKE_SECRET,
      ...extra,
    },
  },);
}

/**
 Checks that the fake secret is in no stream or runner file of a run.

 @param run - What the step left behind

 @example
 ```ts
 expectNoSecret(run,);
 ```
 */
function expectNoSecret(run: StepRun,): void {
  expect(JSON.stringify(run,),).not.toContain(FAKE_SECRET,);
}

/**
 Reads the JSON lines a stand-in recorded.

 @param fixture - Runner whose records to read

 @param name - Record file name

 @returns One parsed value per line

 @example
 ```ts
 const calls = await recordsOf({ fixture, name: 'binstall-calls.jsonl', },);
 ```
 */
async function recordsOf(
  {
    fixture,
    name,
  }: {
    readonly fixture: StepFixture;
    readonly name: string;
  },
): Promise<readonly unknown[]> {
  return (await readFile(join(fixture.records, name,), 'utf8',))
    .split('\n',)
    .filter(function isLine(line: string,) {
      return line !== '';
    },)
    .map(function parse(line: string,) {
      return JSON.parse(line,) as unknown;
    },);
}

//endregion Fixtures: a lock, a cargo-binstall archive and the answers that serve it

await describe({
  name: 'catalog-specifiers steps',
  children: [
    describe({
      name: 'install the pinned git policy CLI',
      children: [
        it({
          name: 'installs a release after the rename, puts it on PATH and announces its three names',
          fn: async function installsRenamed() {
            await using fixture = await createStepFixture();
            await fixture.writeLock(lockOf({
              url: BINSTALL_ADDRESS,
              sha256: await serveBinstall(fixture,),
              version: POLICY_CLI_VERSION,
            },),);
            const run = await runInstall({ fixture, extra: { FAKE_BINSTALL_PROGRAM: RENAMED_PROGRAM, }, },);
            expect(run.status,).toBe(0,);
            expect(run.stdout,).toBe(`${join(fixture.runnerTemp, 'binstall.tgz',)}: OK\n${RENAMED_PROGRAM} 9.9.9-fake\n`,);
            expect(run.githubPath,).toBe(`${join(fixture.runnerTemp, 'cli-git', 'bin',)}\n`,);
            expect(run.githubEnv,).toBe(
              `CLI_PROGRAM=${RENAMED_PROGRAM}\nCLI_WORD=${RENAMED_PROGRAM}\nCLI_SETTINGS=${RENAMED_PROGRAM}.config.jsonc\n`,
            );
            expect(await recordsOf({ fixture, name: 'binstall-calls.jsonl', },),).toEqual([
              {
                args: [
                  '--no-confirm',
                  '--disable-strategies',
                  'compile',
                  '--root',
                  join(fixture.runnerTemp, 'cli-git',),
                  '--version',
                  POLICY_CLI_VERSION,
                  'monochromatic-git-policy-cli',
                ],
                tokenPresent: true,
              },
            ],);
            expectNoSecret(run,);
          },
        },),
        it({
          name: 'installs a release before the rename and copies the renamed settings file for it',
          fn: async function installsEarlier() {
            await using fixture = await createStepFixture();
            await fixture.writeLock(lockOf({
              url: BINSTALL_ADDRESS,
              sha256: await serveBinstall(fixture,),
              version: POLICY_CLI_VERSION,
            },),);
            await writeFile(join(fixture.checkout, `${RENAMED_PROGRAM}.config.jsonc`,), '{ "renamed": true }\n',);
            const run = await runInstall({ fixture, extra: { FAKE_BINSTALL_PROGRAM: EARLIER_PROGRAM, }, },);
            expect(run.status,).toBe(0,);
            expect(run.stdout,).toContain(`${EARLIER_PROGRAM} 9.9.9-fake\n`,);
            expect(run.githubEnv,).toBe(`CLI_PROGRAM=${EARLIER_PROGRAM}\nCLI_WORD=cli-git\nCLI_SETTINGS=cli-git.config.jsonc\n`,);
            expect(await readFile(join(fixture.checkout, 'cli-git.config.jsonc',), 'utf8',),).toBe('{ "renamed": true }\n',);
          },
        },),
        it({
          name: 'leaves an existing settings file of the earlier name alone, and copies nothing without the renamed one',
          fn: async function leavesSettings() {
            await using fixture = await createStepFixture();
            await fixture.writeLock(lockOf({
              url: BINSTALL_ADDRESS,
              sha256: await serveBinstall(fixture,),
              version: POLICY_CLI_VERSION,
            },),);
            const withoutRenamed = await runInstall({ fixture, extra: { FAKE_BINSTALL_PROGRAM: EARLIER_PROGRAM, }, },);
            expect(withoutRenamed.status,).toBe(0,);
            expect(await readdir(fixture.checkout,),).toEqual(['toolchain.lock.json',],);
            await writeFile(join(fixture.checkout, `${RENAMED_PROGRAM}.config.jsonc`,), '{ "renamed": true }\n',);
            await writeFile(join(fixture.checkout, 'cli-git.config.jsonc',), '{ "earlier": true }\n',);
            const withBoth = await runInstall({ fixture, extra: { FAKE_BINSTALL_PROGRAM: EARLIER_PROGRAM, }, },);
            expect(withBoth.status,).toBe(0,);
            expect(await readFile(join(fixture.checkout, 'cli-git.config.jsonc',), 'utf8',),).toBe('{ "earlier": true }\n',);
          },
        },),
        it({
          name: 'ends with status 1 and creates nothing when the earlier settings name is a link that points nowhere',
          fn: async function refusesDanglingLink() {
            await using fixture = await createStepFixture();
            await fixture.writeLock(lockOf({
              url: BINSTALL_ADDRESS,
              sha256: await serveBinstall(fixture,),
              version: POLICY_CLI_VERSION,
            },),);
            await writeFile(join(fixture.checkout, `${RENAMED_PROGRAM}.config.jsonc`,), '{ "renamed": true }\n',);
            await symlink('nowhere', join(fixture.checkout, 'cli-git.config.jsonc',),);
            const run = await runInstall({ fixture, extra: { FAKE_BINSTALL_PROGRAM: EARLIER_PROGRAM, }, },);
            expect(run.status,).toBe(1,);
            expect((await readdir(fixture.checkout,)).toSorted(),).toEqual([
              'cli-git.config.jsonc',
              `${RENAMED_PROGRAM}.config.jsonc`,
              'toolchain.lock.json',
            ],);
          },
        },),
        it({
          name: 'refuses an archive address outside the pinned release folder, or a lock without one',
          fn: async function refusesAddress() {
            await using fixture = await createStepFixture();
            await fixture.writeLock(lockOf({
              url: 'https://example.invalid/cargo-binstall.tgz',
              sha256: '0'.repeat(64,),
              version: POLICY_CLI_VERSION,
            },),);
            const wrong = await runInstall({ fixture, },);
            expect(wrong.status,).toBe(1,);
            expect(wrong.stdout,).toBe(
              `::error::Unexpected cargo-binstall archive in toolchain.lock.json: 'https://example.invalid/cargo-binstall.tgz'\n`,
            );
            await fixture.writeLock(lockOf({ sha256: '0'.repeat(64,), version: POLICY_CLI_VERSION, },),);
            const missing = await runInstall({ fixture, },);
            expect(missing.status,).toBe(1,);
            expect(missing.stdout,).toBe(`::error::Unexpected cargo-binstall archive in toolchain.lock.json: 'null'\n`,);
          },
        },),
        ...[
          '0.6',
          '1.2',
          'v1.2.3',
          '',
          '.1.2.3',
          '1.a.b',
        ].map(function refusesVersion(version: string,) {
          return it({
            name: `refuses the version ${JSON.stringify(version,)}`,
            fn: async function refusesOne() {
              await using fixture = await createStepFixture();
              await fixture.writeLock(lockOf({ url: BINSTALL_ADDRESS, sha256: '0'.repeat(64,), version, },),);
              const run = await runInstall({ fixture, },);
              expect(run.status,).toBe(1,);
              expect(run.stdout,).toBe(`::error::Unexpected git-policy-cli version in toolchain.lock.json: '${version}'\n`,);
            },
          },);
        },),
        ...[
          '0.6.1',
          '1..2.3',
          '12.34.56-rc.1',
        ].map(function acceptsVersion(version: string,) {
          return it({
            name: `accepts the version ${JSON.stringify(version,)}`,
            fn: async function acceptsOne() {
              await using fixture = await createStepFixture();
              await fixture.writeLock(lockOf({
                url: BINSTALL_ADDRESS,
                sha256: await serveBinstall(fixture,),
                version,
              },),);
              const run = await runInstall({ fixture, extra: { FAKE_BINSTALL_PROGRAM: RENAMED_PROGRAM, }, },);
              expect(run.status,).toBe(0,);
            },
          },);
        },),
        it({
          name: 'ends with status 1 when the archive differs from the lock, or the lock has no SHA-256',
          fn: async function sha256Fails() {
            await using fixture = await createStepFixture();
            await serveBinstall(fixture,);
            await fixture.writeLock(lockOf({ url: BINSTALL_ADDRESS, sha256: '1'.repeat(64,), version: POLICY_CLI_VERSION, },),);
            const differs = await runInstall({ fixture, },);
            expect(differs.status,).toBe(1,);
            expect(differs.stdout,).toBe(`${join(fixture.runnerTemp, 'binstall.tgz',)}: FAILED\n`,);
            expect(differs.githubPath,).toBe('',);
            await fixture.writeLock(lockOf({ url: BINSTALL_ADDRESS, version: POLICY_CLI_VERSION, },),);
            const missing = await runInstall({ fixture, },);
            expect(missing.status,).toBe(1,);
            expect(missing.stderr,).toContain('no properly formatted checksum lines found',);
          },
        },),
        it({
          name: 'ends with the status of a failing download, curl status 22 for an HTTP error',
          fn: async function downloadFails() {
            await using fixture = await createStepFixture();
            await fixture.writeLock(lockOf({ url: BINSTALL_ADDRESS, sha256: '0'.repeat(64,), version: POLICY_CLI_VERSION, },),);
            await fixture.answerCurl([{ urlIncludes: BINSTALL_ADDRESS, status: 404, },],);
            const run = await runInstall({ fixture, },);
            expect(run.status,).toBe(22,);
            expect(run.stderr,).toBe('curl: (22) The requested URL returned error: 404\n',);
          },
        },),
        it({
          name: 'ends with the status of a failing cargo-binstall and announces nothing',
          fn: async function binstallFails() {
            await using fixture = await createStepFixture();
            await fixture.writeLock(lockOf({
              url: BINSTALL_ADDRESS,
              sha256: await serveBinstall(fixture,),
              version: POLICY_CLI_VERSION,
            },),);
            const run = await runInstall({ fixture, extra: { FAKE_BINSTALL_EXIT: '76', }, },);
            expect(run.status,).toBe(76,);
            expect(run.githubPath,).toBe('',);
            expect(run.githubEnv,).toBe('',);
          },
        },),
        it({
          name: 'ends with status 2 for a missing lock and 5 for a lock that is not JSON, as jq did',
          fn: async function lockUnreadable() {
            await using fixture = await createStepFixture();
            const missing = await runInstall({ fixture, },);
            expect(missing.status,).toBe(2,);
            await writeFile(join(fixture.checkout, 'toolchain.lock.json',), '{ not json',);
            const invalid = await runInstall({ fixture, },);
            expect(invalid.status,).toBe(5,);
          },
        },),
      ],
    },),
    describe({
      name: 'negative control',
      children: [
        it({
          name: 'accepts a rejection with status 1 and the disallowed-specifier finding, after staging the literal version',
          fn: async function acceptsRejection() {
            await using fixture = await createStepFixture();
            const run = await runControl({
              fixture,
              extra: {
                FAKE_CLI_EXIT: '1',
                FAKE_CLI_STDOUT: 'error mono/catalog-specifiers/disallowed-specifier left-pad\n\n',
              },
            },);
            expect(run.status,).toBe(0,);
            expect(run.stdout,).toBe('The negative control was rejected as expected.\n',);
            /**
             The throwaway repository the check ran in.
             */
            const control = join(fixture.runnerTemp, 'catalog-specifiers-control',);
            expect(await readFile(join(control, `${RENAMED_PROGRAM}.config.jsonc`,), 'utf8',),).toBe(
              '{ "policies": { "mono/catalog-specifiers": "error" } }\n',
            );
            expect(await readFile(join(control, 'package.json',), 'utf8',),).toBe(
              '{ "name": "control", "dependencies": { "left-pad": "1.3.0" } }\n',
            );
            expect(await recordsOf({ fixture, name: 'policy-cli-calls.jsonl', },),).toEqual([
              {
                args: [
                  RENAMED_PROGRAM,
                  'check',
                  '--all',
                  '--policy',
                  'mono/catalog-specifiers',
                ],
                cwd: control,
              },
            ],);
            expect((await readdir(control,)).toSorted(),).toEqual([
              '.git',
              `${RENAMED_PROGRAM}.config.jsonc`,
              'package.json',
            ],);
            expectNoSecret(run,);
          },
        },),
        it({
          name: 'fails when the check rejects without the disallowed-specifier finding',
          fn: async function failsWithoutFinding() {
            await using fixture = await createStepFixture();
            const run = await runControl({ fixture, extra: { FAKE_CLI_EXIT: '1', FAKE_CLI_STDOUT: 'something else\n', }, },);
            expect(run.status,).toBe(1,);
            expect(run.stdout,).toBe(
              '::error::The negative control exited 1 without the disallowed-specifier finding.\nsomething else\n',
            );
          },
        },),
        ...[
          '0',
          '2',
        ].map(function failsForStatus(status: string,) {
          return it({
            name: `fails when the check exits ${status} instead of 1, and prints what it said`,
            fn: async function failsOne() {
              await using fixture = await createStepFixture();
              const run = await runControl({
                fixture,
                extra: {
                  FAKE_CLI_EXIT: status,
                  FAKE_CLI_STDOUT: 'mono/catalog-specifiers/disallowed-specifier\n',
                },
              },);
              expect(run.status,).toBe(1,);
              expect(run.stdout,).toBe(
                `::error::The negative control exited ${status}, expected 1.\nmono/catalog-specifiers/disallowed-specifier\n`,
              );
            },
          },);
        },),
        it({
          name: 'prints an empty line for a check that said nothing',
          fn: async function failsOnSilence() {
            await using fixture = await createStepFixture();
            const run = await runControl({ fixture, extra: { FAKE_CLI_EXIT: '2', }, },);
            expect(run.status,).toBe(1,);
            expect(run.stdout,).toBe('::error::The negative control exited 2, expected 1.\n\n',);
          },
        },),
        it({
          name: 'ends with status 1 when the settings variable is not set',
          fn: async function needsSettings() {
            await using fixture = await createStepFixture();
            const run = await fixture.run({
              command: [
                process.execPath,
                CONTROL_PROGRAM,
              ],
              env: { CLI_PROGRAM: RENAMED_PROGRAM, CLI_WORD: RENAMED_PROGRAM, },
            },);
            expect(run.status,).toBe(1,);
            expect(run.stderr,).toContain('CLI_SETTINGS: unbound variable',);
          },
        },),
      ],
    },),
  ],
},);
