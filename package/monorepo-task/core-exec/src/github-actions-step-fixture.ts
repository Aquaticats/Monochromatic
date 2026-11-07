/**
 A disposable runner for the tests of the workflow step programs, and for the old-versus-new comparison of the shell
 steps they replaced.

 It makes a checkout folder, a runner temporary folder, the four files a step appends to (`GITHUB_OUTPUT`, `GITHUB_ENV`,
 `GITHUB_PATH`, `GITHUB_STEP_SUMMARY`) and a folder of stand-in programs that comes first on `PATH`; it runs any command
 there and returns the status, the two output streams, the four files and the records the stand-ins kept. Nothing in it
 reaches the network, a registry or the developer's own tools, and every secret in it is a fake.

 @module
 */

import { createHash, } from 'node:crypto';
import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  FAKE_BINSTALL_SOURCE,
  FAKE_CURL_SCENARIO_VARIABLE,
  FAKE_CURL_SOURCE,
  FAKE_PODMAN_SOURCE,
  FAKE_POLICY_CLI_SOURCE,
  FAKE_RECORD_DIRECTORY_VARIABLE,
  FAKE_RELEASE_PROGRAM_SOURCE,
} from './github-actions-step-fixture-fake-tools.ts';
import {
  collect,
  readIfPresent,
  writeScript,
} from './github-actions-step-fixture-process.ts';
import type {
  ArchiveRequest,
  CurlRule,
  ProgramFile,
  RunRequest,
  StepFixture,
} from './github-actions-step-fixture-types.ts';

export type {
  ArchiveRequest,
  ArchiveResult,
  CurlRule,
  ProgramFile,
  RunRequest,
  StepFixture,
  StepRun,
} from './github-actions-step-fixture-types.ts';

/**
 Makes a disposable runner with its stand-in `curl`, `podman`, `cargo-binstall` source and policy CLI source ready.

 @returns The runner

 @example
 ```ts
 await using fixture = await createStepFixture();
 await fixture.answerCurl([],);
 ```
 */
export async function createStepFixture(): Promise<StepFixture> {
  /**
   Everything of this fixture lives under one disposable folder.
   */
  const root = await mkdtemp(join(
    scratchRoot(),
    'workflow-step-',
  ),);
  /**
   Where the step runs.
   */
  const checkout = join(
    root,
    'checkout',
  );
  /**
   `RUNNER_TEMP`.
   */
  const runnerTemp = join(
    root,
    'runner-temp',
  );
  /**
   Stand-ins and their records.
   */
  const records = join(
    root,
    'records',
  );
  /**
   Fake release files.
   */
  const served = join(
    root,
    'served',
  );
  /**
   Folder that comes first on `PATH`.
   */
  const programs = join(
    root,
    'programs',
  );
  await Promise.all([
    checkout,
    runnerTemp,
    records,
    served,
    programs,
  ].map(async function make(folder: string,) {
    await mkdir(
      folder,
      { recursive: true, },
    );
  },),);
  /**
   The four files a step appends to.
   */
  const runnerFiles = {
    output: join(
      root,
      'github-output',
    ),
    env: join(
      root,
      'github-env',
    ),
    path: join(
      root,
      'github-path',
    ),
    summary: join(
      root,
      'github-step-summary',
    ),
  };
  /**
   Answers of the stand-in `curl`.
   */
  const scenario = join(
    root,
    'curl-scenario.json',
  );
  await writeScript({
    path: join(
      programs,
      'curl',
    ),
    source: FAKE_CURL_SOURCE,
  },);
  await writeScript({
    path: join(
      programs,
      'podman',
    ),
    source: FAKE_PODMAN_SOURCE,
  },);
  await writeFile(
    scenario,
    JSON.stringify({ rules: [], },),
  );
  return {
    checkout,
    runnerTemp,
    records,
    served,
    root,
    async answerCurl(rules: readonly CurlRule[],) {
      await writeFile(
        scenario,
        JSON.stringify({ rules, },),
      );
    },
    async writeLock(lock: unknown,) {
      await writeFile(
        join(
          checkout,
          'toolchain.lock.json',
        ),
        JSON.stringify(lock,),
      );
    },
    async makeArchive({
      name,
      top,
      members,
    }: ArchiveRequest,) {
      /**
       Folder that is packed.
       */
      const staging = await mkdtemp(join(
        root,
        'staging-',
      ),);
      await mkdir(
        join(
          staging,
          top,
        ),
        { recursive: true, },
      );
      await Promise.all(Object.entries(members,)
        .map(async function put([member, source,]: readonly [
          string,
          string,
        ],) {
          await writeScript({
            path: join(
              staging,
              top,
              member,
            ),
            source,
          },);
        },),);
      /**
       Where the archive goes.
       */
      const path = join(
        served,
        name,
      );
      await collect({
        command: [
          'tar',
          '--create',
          '--gzip',
          '--file',
          path,
          '--directory',
          staging,
          top === '' ? '.' : top,
        ],
        env: { PATH: '/usr/bin:/bin', },
        cwd: root,
      },);
      return {
        path,
        sha256: createHash('sha256',)
          .update(await readFile(path,),)
          .digest('hex',),
      };
    },
    async writeProgram({
      name,
      source,
    }: ProgramFile,) {
      await writeScript({
        path: join(
          programs,
          name,
        ),
        source,
      },);
    },
    async run({
      command,
      env,
      cwd = checkout,
    }: RunRequest,) {
      await Promise.all(Object.values(runnerFiles,)
        .map(async function empty(file: string,) {
          await writeFile(
            file,
            '',
          );
        },),);
      /**
       The step's environment: the runner's variables, the stand-ins first on `PATH`, then the step's own.
       */
      const environment = {
        PATH: `${programs}:/usr/bin:/bin`,
        HOME: root,
        RUNNER_TEMP: runnerTemp,
        GITHUB_OUTPUT: runnerFiles.output,
        GITHUB_ENV: runnerFiles.env,
        GITHUB_PATH: runnerFiles.path,
        GITHUB_STEP_SUMMARY: runnerFiles.summary,
        [FAKE_RECORD_DIRECTORY_VARIABLE]: records,
        [FAKE_CURL_SCENARIO_VARIABLE]: scenario,
        ...env,
      };
      /**
       What the command printed and its status.
       */
      const result = await collect({
        command,
        env: environment,
        cwd,
      },);
      return {
        ...result,
        githubOutput: await readIfPresent(runnerFiles.output,),
        githubEnv: await readIfPresent(runnerFiles.env,),
        githubPath: await readIfPresent(runnerFiles.path,),
        githubSummary: await readIfPresent(runnerFiles.summary,),
      };
    },
    async [Symbol.asyncDispose]() {
      await rm(
        root,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Source of a stand-in `cargo-binstall` program, for a test that packs it into an archive.

 @returns Node script source

 @example
 ```ts
 const members = { 'cargo-binstall': binstallSource(), };
 ```
 */
export function binstallSource(): string {
  return FAKE_BINSTALL_SOURCE;
}

/**
 Source of a program that prints a fixed version, for a test that packs it into a release archive.

 @returns Node script source

 @example
 ```ts
 const members = { 'file-enforcer': releaseProgramSource(), };
 ```
 */
export function releaseProgramSource(): string {
  return FAKE_RELEASE_PROGRAM_SOURCE;
}

/**
 Source of the stand-in git policy CLI.

 @returns Node script source

 @example
 ```ts
 await fixture.writeProgram({ name: 'monochromatic-git-policy-cli', source: policyCliSource(), },);
 ```
 */
export function policyCliSource(): string {
  return FAKE_POLICY_CLI_SOURCE;
}
