/**
 Exercises the deployed `repo-tools-setup.ts` step through a subprocess, as `.github/actions/repo-tools-setup` starts it.

 The source file is run, not a built copy of it: the action runs it right after checkout with the runner's Node and no
 installed dependencies, so the checked-in TypeScript is the artifact that ships. `curl` is a stand-in that answers from a
 scenario; `sha256sum` and `tar` are the real programs, so each check and unpack path really runs.

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
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  createStepFixture,
  releaseProgramSource,
  type StepFixture,
  type StepRun,
} from './github-actions-step-fixture.ts';

//region Fixtures: a lock, release archives and the answers that serve them

/**
 The program under test, which the action runs as `node package/monorepo-task/core-exec/src/repo-tools-setup.ts`.
 */
const PROGRAM = join(import.meta.dirname, 'repo-tools-setup.ts',);

/**
 Repository the fake releases belong to.
 */
const REPOSITORY = 'Example/Repository';

/**
 Fake secret the runner holds; no output of the step may show it.
 */
const FAKE_SECRET = 'FAKE_TOKEN_123';

/**
 Crate behind the `file-enforcer` program.
 */
const FILE_ENFORCER_CRATE = 'monochromatic-file-enforcer-cli';

/**
 Crate behind the `cargo-shadow` program.
 */
const CARGO_SHADOW_CRATE = 'monochromatic-cargo-shadow';

/**
 Version the lock records for every crate.
 */
const CRATE_VERSION = '0.4.2';

/**
 Rust target triple of the x64 runner.
 */
const X64_TARGET = 'x86_64-unknown-linux-gnu';

/**
 Rust target triple of the arm64 runner.
 */
const ARM64_TARGET = 'aarch64-unknown-linux-gnu';

/**
 Address the release assets of a crate are served from.

 @param crate - Crate name

 @returns Address of the release folder

 @example
 ```ts
 const base = releaseBase(FILE_ENFORCER_CRATE,);
 ```
 */
function releaseBase(crate: string,): string {
  return `https://github.com/${REPOSITORY}/releases/download/${crate}-v${CRATE_VERSION}`;
}

/**
 Serves a release of a crate: an archive holding the program, and the release's checksums file.

 @param fixture - Runner to serve from

 @param crate - Crate name

 @param target - Rust target triple

 @param program - Program name inside the archive

 @returns Rules for the stand-in `curl`

 @example
 ```ts
 const rules = await serveRelease({ fixture, crate: FILE_ENFORCER_CRATE, target: X64_TARGET, program: 'file-enforcer', },);
 ```
 */
async function serveRelease(
  {
    fixture,
    crate,
    target,
    program,
  }: {
    readonly fixture: StepFixture;
    readonly crate: string;
    readonly target: string;
    readonly program: string;
  },
): Promise<readonly { urlIncludes: string; status: number; bodyFile?: string; body?: string; }[]> {
  /**
   File name of the archive.
   */
  const name = `${crate}-${CRATE_VERSION}-${target}.tar.gz`;
  /**
   The packed archive.
   */
  const archive = await fixture.makeArchive({
    name,
    top: `${crate}-${CRATE_VERSION}-${target}`,
    members: { [program]: releaseProgramSource(), },
  },);
  return [
    {
      urlIncludes: `${releaseBase(crate,)}/${name}`,
      status: 200,
      bodyFile: archive.path,
    },
    {
      urlIncludes: `${releaseBase(crate,)}/${crate}-${CRATE_VERSION}-sha256sums.txt`,
      status: 200,
      body: `${'0'.repeat(64,)}  some-other-file.tar.gz\n${archive.sha256}  ${name}\n`,
    },
  ];
}

/**
 The lock rows of the three tools for both runner platforms; `mpm-shadow` points at the given archive.

 @param mpmShadowSha256 - SHA-256 recorded for the `mpm-shadow` archive

 @returns A lock as `toolchain.lock.json` holds it

 @example
 ```ts
 await fixture.writeLock(lockWith('0'.repeat(64,),),);
 ```
 */
function lockWith(mpmShadowSha256: string,): unknown {
  return {
    tools: {
      'mpm-shadow': {
        artifacts: {
          'linux-x64': { url: 'https://example.invalid/mpm-shadow-x64.tar.gz', sha256: mpmShadowSha256, },
          'linux-arm64': { url: 'https://example.invalid/mpm-shadow-arm64.tar.gz', sha256: mpmShadowSha256, },
        },
      },
      [`cargo:${FILE_ENFORCER_CRATE}`]: { version: CRATE_VERSION, },
      [`cargo:${CARGO_SHADOW_CRATE}`]: { version: CRATE_VERSION, },
    },
  };
}

/**
 Runs the step with the environment the action gives it.

 @param fixture - Runner to run in

 @param tools - Value of the `tools` input

 @param platform - Value of `RUNNER_PLATFORM`

 @param extra - More environment variables

 @returns What the step left behind

 @example
 ```ts
 const run = await runSetup({ fixture, tools: 'file-enforcer', platform: 'Linux-X64', },);
 ```
 */
async function runSetup(
  {
    fixture,
    tools,
    platform,
    extra = {},
  }: {
    readonly fixture: StepFixture;
    readonly tools: string;
    readonly platform: string;
    readonly extra?: Readonly<Record<string, string>>;
  },
): Promise<StepRun> {
  return await fixture.run({
    command: [
      process.execPath,
      PROGRAM,
    ],
    env: {
      TOOLS: tools,
      RUNNER_PLATFORM: platform,
      GITHUB_REPOSITORY: REPOSITORY,
      GITHUB_TOKEN: FAKE_SECRET,
      ...extra,
    },
  },);
}

/**
 Whether a file of the runner's record folder exists.

 @param records - The record folder

 @param name - File name to look for

 @returns True when the stand-ins wrote that record

 @example
 ```ts
 const called = await hasRecord({ records: fixture.records, name: 'curl-calls.jsonl', },);
 ```
 */
async function hasRecord(
  {
    records,
    name,
  }: {
    readonly records: string;
    readonly name: string;
  },
): Promise<boolean> {
  return (await readdir(records,)).includes(name,);
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

//endregion Fixtures: a lock, release archives and the answers that serve them

await describe({
  name: 'repo-tools-setup step',
  children: [
    it({
      name: 'installs file-enforcer from its release, checks it and puts its folder on PATH',
      fn: async function installsOne() {
        await using fixture = await createStepFixture();
        await fixture.writeLock(
          lockWith('0'.repeat(64,),),
        );
        await fixture.answerCurl(await serveRelease({
          fixture,
          crate: FILE_ENFORCER_CRATE,
          target: X64_TARGET,
          program: 'file-enforcer',
        },),);
        const run = await runSetup({ fixture, tools: 'file-enforcer', platform: 'Linux-X64', },);
        expect(run.status,).toBe(0,);
        // The release check runs in the work folder, so `sha256sum --check` names the archive by its file name.
        expect(run.stdout,).toBe(
          `${FILE_ENFORCER_CRATE}-${CRATE_VERSION}-${X64_TARGET}.tar.gz: OK\nfile-enforcer 1.2.3-fake\n`,
        );
        expect(run.stderr,).toBe('',);
        expect(run.githubPath,).toBe(`${join(fixture.runnerTemp, 'repo-tools-bin',)}\n`,);
        expect(
          await readdir(join(fixture.runnerTemp, 'repo-tools-bin',),),
        ).toEqual(['file-enforcer',],);
        expectNoSecret(run,);
      },
    },),
    it({
      name: 'installs all three tools on an arm64 runner, in the order asked',
      fn: async function installsAll() {
        await using fixture = await createStepFixture();
        /**
         The archive of `mpm-shadow`, whose SHA-256 the lock records.
         */
        const mpm = await fixture.makeArchive({
          name: 'mpm-shadow-arm64.tar.gz',
          top: 'mpm-shadow-0.0.1',
          members: { 'mpm-shadow': releaseProgramSource(), },
        },);
        await fixture.writeLock(lockWith(mpm.sha256,),);
        await fixture.answerCurl([
          {
            urlIncludes: 'https://example.invalid/mpm-shadow-arm64.tar.gz',
            status: 200,
            bodyFile: mpm.path,
          },
          ...await serveRelease({
            fixture,
            crate: FILE_ENFORCER_CRATE,
            target: ARM64_TARGET,
            program: 'file-enforcer',
          },),
          ...await serveRelease({
            fixture,
            crate: CARGO_SHADOW_CRATE,
            target: ARM64_TARGET,
            program: 'cargo-shadow',
          },),
        ],);
        const run = await runSetup({
          fixture,
          tools: 'mpm-shadow file-enforcer cargo-shadow',
          platform: 'Linux-ARM64',
        },);
        expect(run.status,).toBe(0,);
        expect(run.stdout.split('\n',)
          .filter(function isVersion(line: string,) {
            return line.endsWith('1.2.3-fake',);
          },),).toEqual([
            'mpm-shadow 1.2.3-fake',
            'file-enforcer 1.2.3-fake',
            'cargo-shadow 1.2.3-fake',
          ],);
        expect(run.stdout,).toContain(`/mpm-shadow.tar.gz: OK\n`,);
        expect(run.githubPath,).toBe(`${join(fixture.runnerTemp, 'repo-tools-bin',)}\n`,);
      },
    },),
    it({
      name: 'reads the tools input split on blanks, tabs and line breaks',
      fn: async function splitsInput() {
        await using fixture = await createStepFixture();
        await fixture.writeLock(
          lockWith('0'.repeat(64,),),
        );
        await fixture.answerCurl([
          ...await serveRelease({
            fixture,
            crate: FILE_ENFORCER_CRATE,
            target: X64_TARGET,
            program: 'file-enforcer',
          },),
          ...await serveRelease({
            fixture,
            crate: CARGO_SHADOW_CRATE,
            target: X64_TARGET,
            program: 'cargo-shadow',
          },),
        ],);
        const run = await runSetup({
          fixture,
          tools: '  file-enforcer\t\tcargo-shadow\n',
          platform: 'Linux-X64',
        },);
        expect(run.status,).toBe(0,);
        expect(run.stdout,).toContain('file-enforcer 1.2.3-fake\n',);
        expect(run.stdout,).toContain('cargo-shadow 1.2.3-fake\n',);
      },
    },),
    it({
      name: 'installs nothing for an empty tools input but still puts the folder on PATH, without reading the lock',
      fn: async function installsNothing() {
        await using fixture = await createStepFixture();
        const run = await runSetup({ fixture, tools: '', platform: 'Linux-X64', },);
        expect(run.status,).toBe(0,);
        expect(run.stdout,).toBe('',);
        expect(run.githubPath,).toBe(`${join(fixture.runnerTemp, 'repo-tools-bin',)}\n`,);
        expect(await hasRecord({ records: fixture.records, name: 'curl-calls.jsonl', },),).toBe(false,);
      },
    },),
    it({
      name: 'refuses a runner platform that has no prebuilt archive',
      fn: async function refusesPlatform() {
        await using fixture = await createStepFixture();
        const run = await runSetup({ fixture, tools: 'file-enforcer', platform: 'Windows-X64', },);
        expect(run.status,).toBe(1,);
        expect(run.stderr,).toBe(
          '::error::repo-tools-setup: no prebuilt archive for runner Windows-X64. The tools are published for Linux-X64 and Linux-ARM64 only; run the job on one of those runners.\n',
        );
        expect(run.githubPath,).toBe('',);
      },
    },),
    it({
      name: 'stops at an unknown tool after installing the tools before it, and writes no PATH entry',
      fn: async function stopsAtUnknown() {
        await using fixture = await createStepFixture();
        await fixture.writeLock(
          lockWith('0'.repeat(64,),),
        );
        await fixture.answerCurl(await serveRelease({
          fixture,
          crate: FILE_ENFORCER_CRATE,
          target: X64_TARGET,
          program: 'file-enforcer',
        },),);
        const run = await runSetup({ fixture, tools: 'file-enforcer bogus', platform: 'Linux-X64', },);
        expect(run.status,).toBe(1,);
        expect(run.stderr,).toBe(
          `::error::repo-tools-setup: unknown tool 'bogus'. The tools input takes names separated by blanks, from: mpm-shadow, file-enforcer, cargo-shadow, git-policy-cli.\n`,
        );
        expect(run.stdout,).toContain('file-enforcer 1.2.3-fake\n',);
        expect(run.githubPath,).toBe('',);
      },
    },),
    it({
      name: 'reports a lock without the mpm-shadow archive row of this platform',
      fn: async function missingArchiveRow() {
        await using fixture = await createStepFixture();
        await fixture.writeLock({ tools: { 'mpm-shadow': { artifacts: {}, }, }, },);
        const run = await runSetup({ fixture, tools: 'mpm-shadow', platform: 'Linux-X64', },);
        expect(run.status,).toBe(1,);
        expect(run.stderr,).toBe(
          `::error::repo-tools-setup: toolchain.lock.json has no mpm-shadow archive row for linux-x64; run 'mpm-shadow update'\n`,
        );
      },
    },),
    it({
      name: 'reports a lock without the crate row of a release tool',
      fn: async function missingCrateRow() {
        await using fixture = await createStepFixture();
        await fixture.writeLock({ tools: {}, },);
        const run = await runSetup({ fixture, tools: 'cargo-shadow', platform: 'Linux-X64', },);
        expect(run.status,).toBe(1,);
        expect(run.stderr,).toBe(
          `::error::repo-tools-setup: toolchain.lock.json has no cargo:${CARGO_SHADOW_CRATE} row; run 'mpm-shadow update'\n`,
        );
      },
    },),
    it({
      name: 'fails the SHA-256 check of an mpm-shadow archive that differs from the lock',
      fn: async function mpmShadowMismatch() {
        await using fixture = await createStepFixture();
        /**
         An archive other than the one the lock describes.
         */
        const mpm = await fixture.makeArchive({
          name: 'mpm-shadow-x64.tar.gz',
          top: 'mpm-shadow-0.0.1',
          members: { 'mpm-shadow': releaseProgramSource(), },
        },);
        await fixture.writeLock(
          lockWith('1'.repeat(64,),),
        );
        await fixture.answerCurl([{ urlIncludes: 'mpm-shadow-x64.tar.gz', status: 200, bodyFile: mpm.path, },],);
        const run = await runSetup({ fixture, tools: 'mpm-shadow', platform: 'Linux-X64', },);
        expect(run.status,).toBe(1,);
        expect(run.stdout,).toContain('mpm-shadow.tar.gz: FAILED\n',);
        expect(run.githubPath,).toBe('',);
      },
    },),
    it({
      name: 'fails when the release checksums file does not list the archive, or lists another SHA-256',
      fn: async function releaseMismatch() {
        await using fixture = await createStepFixture();
        await fixture.writeLock(
          lockWith('0'.repeat(64,),),
        );
        /**
         A release whose checksums file has no line for the archive.
         */
        const [archiveRule,] = await serveRelease({
          fixture,
          crate: FILE_ENFORCER_CRATE,
          target: X64_TARGET,
          program: 'file-enforcer',
        },);
        const sumsUrl = `${releaseBase(FILE_ENFORCER_CRATE,)}/${FILE_ENFORCER_CRATE}-${CRATE_VERSION}-sha256sums.txt`;
        await fixture.answerCurl([
          archiveRule as { urlIncludes: string; status: number; },
          { urlIncludes: sumsUrl, status: 200, body: `${'0'.repeat(64,)}  another.tar.gz\n`, },
        ],);
        const unlisted = await runSetup({ fixture, tools: 'file-enforcer', platform: 'Linux-X64', },);
        expect(unlisted.status,).toBe(1,);
        expect(unlisted.stderr,).toContain('no properly formatted checksum lines found',);
        await fixture.answerCurl([
          archiveRule as { urlIncludes: string; status: number; },
          {
            urlIncludes: sumsUrl,
            status: 200,
            body: `${'f'.repeat(64,)}  ${FILE_ENFORCER_CRATE}-${CRATE_VERSION}-${X64_TARGET}.tar.gz\n`,
          },
        ],);
        const wrong = await runSetup({ fixture, tools: 'file-enforcer', platform: 'Linux-X64', },);
        expect(wrong.status,).toBe(1,);
        expect(wrong.stdout,).toContain('.tar.gz: FAILED\n',);
      },
    },),
    it({
      name: 'ends with the status of a failing download, curl status 22 for an HTTP error',
      fn: async function downloadFails() {
        await using fixture = await createStepFixture();
        await fixture.writeLock(
          lockWith('0'.repeat(64,),),
        );
        await fixture.answerCurl([{ urlIncludes: releaseBase(FILE_ENFORCER_CRATE,), status: 404, },],);
        const run = await runSetup({ fixture, tools: 'file-enforcer', platform: 'Linux-X64', },);
        expect(run.status,).toBe(22,);
        expect(run.stderr,).toContain('curl: (22) The requested URL returned error: 404\n',);
        expect(run.githubPath,).toBe('',);
      },
    },),
    it({
      name: 'ends with status 1 and a message naming the archive and the missing file when the archive has no member named for the tool',
      fn: async function noMember() {
        await using fixture = await createStepFixture();
        await fixture.writeLock(
          lockWith('0'.repeat(64,),),
        );
        await fixture.answerCurl(await serveRelease({
          fixture,
          crate: FILE_ENFORCER_CRATE,
          target: X64_TARGET,
          program: 'something-else',
        },),);
        const run = await runSetup({ fixture, tools: 'file-enforcer', platform: 'Linux-X64', },);
        expect(run.status,).toBe(1,);
        expect(run.stderr,).toBe(
          '::error::repo-tools-setup: the archive of file-enforcer holds no file named file-enforcer, so nothing was installed. Either the archive could not be read (tar says why above) or the release lays its files out differently from what this action expects; open the release named in toolchain.lock.json and check the file names in it.\n',
        );
      },
    },),
    it({
      name: 'ends with the status of the installed program when its --version fails',
      fn: async function versionFails() {
        await using fixture = await createStepFixture();
        await fixture.writeLock(
          lockWith('0'.repeat(64,),),
        );
        await fixture.answerCurl(await serveRelease({
          fixture,
          crate: FILE_ENFORCER_CRATE,
          target: X64_TARGET,
          program: 'file-enforcer',
        },),);
        const run = await runSetup({
          fixture,
          tools: 'file-enforcer',
          platform: 'Linux-X64',
          extra: { FAKE_TOOL_VERSION_EXIT: '3', },
        },);
        expect(run.status,).toBe(3,);
        expect(run.githubPath,).toBe('',);
      },
    },),
    it({
      name: 'ends with status 5 when the lock is not JSON, as jq did',
      fn: async function lockNotJson() {
        await using fixture = await createStepFixture();
        await writeFile(join(fixture.checkout, 'toolchain.lock.json',), '{ not json',);
        const run = await runSetup({ fixture, tools: 'file-enforcer', platform: 'Linux-X64', },);
        expect(run.status,).toBe(5,);
        expect(run.githubPath,).toBe('',);
        expect(run.stderr,).toContain('::error::toolchain.lock.json is not valid JSON (',);
        expect(run.stderr,).toContain(`Restore the file from git, or regenerate it with 'mpm-shadow update'`,);
      },
    },),
    it({
      name: 'ends with status 2 when the lock is missing, as jq did',
      fn: async function lockMissing() {
        await using fixture = await createStepFixture();
        const run = await runSetup({ fixture, tools: 'file-enforcer', platform: 'Linux-X64', },);
        expect(run.status,).toBe(2,);
        expect(run.stderr,).toContain(`::error::toolchain.lock.json could not be opened from ${fixture.checkout} (`,);
        expect(run.stderr,).toContain('Run the step from the repository root after the checkout step',);
      },
    },),
    it({
      name: 'ends with status 127 and names the program when curl is not on PATH',
      fn: async function curlMissing() {
        await using fixture = await createStepFixture();
        await fixture.writeLock(
          lockWith('0'.repeat(64,),),
        );
        const run = await runSetup({ fixture, tools: 'file-enforcer', platform: 'Linux-X64', extra: { PATH: '/nonexistent', }, },);
        expect(run.status,).toBe(127,);
        expect(run.stderr,).toContain('::error::The program curl could not be started (',);
        expect(run.stderr,).toContain('Install it on the runner or put it on PATH before this step',);
      },
    },),
    it({
      name: 'ends with status 1 and names the variable when the tools input is not set',
      fn: async function toolsUnset() {
        await using fixture = await createStepFixture();
        const run = await fixture.run({
          command: [
            process.execPath,
            PROGRAM,
          ],
          env: { RUNNER_PLATFORM: 'Linux-X64', },
        },);
        expect(run.status,).toBe(1,);
        expect(run.stderr,).toBe(
          '::error::The environment variable TOOLS is not set, and this step cannot run without it. Set it in the env block of the workflow step (or job) that runs this step; GitHub itself provides only the GITHUB_* and RUNNER_* variables.\n',
        );
      },
    },),
    it({
      name: 'installs the git policy CLI from the archive row of the lock, which is a GitHub release and never crates.io',
      fn: async function installsPolicyCli() {
        await using fixture = await createStepFixture();
        /**
         The release archive the lock row names, with the program inside one top folder as a release has it.
         */
        const archive = await fixture.makeArchive({
          name: 'policy-cli-x64.tar.gz',
          top: 'monochromatic-git-policy-cli-0.6.1-x86_64-unknown-linux-gnu',
          members: { 'monochromatic-git-policy-cli': releaseProgramSource(), },
        },);
        await fixture.writeLock({
          tools: {
            'cargo:monochromatic-git-policy-cli': {
              version: '0.6.1',
              artifacts: {
                'linux-x64': { url: 'https://example.invalid/policy-cli-x64.tar.gz', sha256: archive.sha256, },
              },
            },
          },
        },);
        await fixture.answerCurl([
          { urlIncludes: 'https://example.invalid/policy-cli-x64.tar.gz', status: 200, bodyFile: archive.path, },
        ],);
        const run = await runSetup({ fixture, tools: 'git-policy-cli', platform: 'Linux-X64', },);
        expect(run.status,).toBe(0,);
        expect(run.stdout,).toContain('monochromatic-git-policy-cli 1.2.3-fake\n',);
        expect(
          await readdir(join(fixture.runnerTemp, 'repo-tools-bin',),),
        ).toEqual(['monochromatic-git-policy-cli',],);
        expect(run.githubPath,).toBe(`${join(fixture.runnerTemp, 'repo-tools-bin',)}\n`,);
      },
    },),
    it({
      name: 'names the lock row and the fix when the lock has no archive row of this platform for the git policy CLI',
      fn: async function policyCliRowMissing() {
        await using fixture = await createStepFixture();
        await fixture.writeLock({ tools: { 'cargo:monochromatic-git-policy-cli': { version: '0.6.1', artifacts: {}, }, }, },);
        const run = await runSetup({ fixture, tools: 'git-policy-cli', platform: 'Linux-X64', },);
        expect(run.status,).toBe(1,);
        expect(run.stderr,).toBe(
          `::error::repo-tools-setup: toolchain.lock.json has no cargo:monochromatic-git-policy-cli archive row for linux-x64; run 'mpm-shadow update'\n`,
        );
      },
    },),
    it({
      name: 'keeps the secret out of every output and file, and off every command line',
      fn: async function keepsSecret() {
        await using fixture = await createStepFixture();
        await fixture.writeLock(
          lockWith('0'.repeat(64,),),
        );
        await fixture.answerCurl(await serveRelease({
          fixture,
          crate: FILE_ENFORCER_CRATE,
          target: X64_TARGET,
          program: 'file-enforcer',
        },),);
        const run = await runSetup({ fixture, tools: 'file-enforcer', platform: 'Linux-X64', },);
        expectNoSecret(run,);
        expect(await readFile(join(fixture.records, 'curl-calls.jsonl',), 'utf8',),).not.toContain(FAKE_SECRET,);
      },
    },),
  ],
},);
