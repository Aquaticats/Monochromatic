/**
 Exercises the deployed `catalog-specifiers-negative-control.ts` step through a subprocess, as
 `.github/workflows/catalog-specifiers.yml` starts it.

 The source file is run, not a built copy: the workflow runs it right after checkout with the runner's Node and no installed
 dependencies, so the checked-in TypeScript is the artifact that ships. The policy CLI is a stand-in; `git` is the real
 program.

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
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  createStepFixture,
  policyCliSource,
  type StepFixture,
  type StepRun,
} from './github-actions-step-fixture.ts';

//region Fixtures: the stand-in policy CLI and the step environment

/**
 The negative-control program under test.
 */
const CONTROL_PROGRAM = join(import.meta.dirname, 'catalog-specifiers-negative-control.ts',);

/**
 Fake secret the runner holds; no output of the step may show it.
 */
const FAKE_SECRET = 'FAKE_TOKEN_123';

/**
 Program name of the installed release.
 */
const RENAMED_PROGRAM = 'monochromatic-git-policy-cli';

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

//endregion Fixtures: the stand-in policy CLI and the step environment

await describe({
  name: 'catalog-specifiers negative control',
  children: [
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
          name: 'ends with status 1 and names the variable when the settings variable is not set',
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
            expect(run.stderr,).toContain('The environment variable CLI_SETTINGS is not set',);
          },
        },),
      ],
    },),
  ],
},);
