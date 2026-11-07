#!/usr/bin/env node

/**
 The negative-control step of `.github/workflows/catalog-specifiers.yml`: a check that cannot fail proves nothing, so this
 stages one literal dependency in a throwaway repository and requires the policy check to exit 1 with the
 `disallowed-specifier` finding, so that an exit 2 configuration failure cannot pass here.

 Reads `RUNNER_TEMP`, and `CLI_PROGRAM`, `CLI_WORD` and `CLI_SETTINGS` as the workflow's job `env` sets them.
 Imports only `node:` modules and sibling files that do, because the workflow runs it before any dependency is installed.

 @example
 ```bash
 node package/monorepo-task/core-exec/src/catalog-specifiers-negative-control.ts
 ```
 */

import {
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  requiredEnvironment,
  runProgram,
  runProgramOrEnd,
  runStep,
  say,
  StepExitError,
  withoutTrailingLineBreaks,
} from './github-actions-step-support.ts';

/**
 Exit status the policy check must end with for a staged literal version.
 */
const REJECTED_STATUS = 1;

/**
 Finding the rejection must name.
 */
const EXPECTED_FINDING = 'mono/catalog-specifiers/disallowed-specifier';

/**
 Runs the policy check over a throwaway repository with one literal dependency staged and requires its rejection.

 @throws {@link StepExitError} With status 1 when the check does not end with status 1 and the expected finding, or with
   the status of a failing `git`

 @example
 ```ts
 await runNegativeControl();
 ```
 */
async function runNegativeControl(): Promise<void> {
  /**
   The throwaway repository.
   */
  const control = join(
    requiredEnvironment('RUNNER_TEMP',),
    'catalog-specifiers-control',
  );
  await mkdir(
    control,
    { recursive: true, },
  );
  await runProgramOrEnd({
    file: 'git',
    args: [
      'init',
      '--quiet',
    ],
    cwd: control,
  },);
  /**
   Settings file name of the installed release.
   */
  const settings = requiredEnvironment('CLI_SETTINGS',);
  await writeFile(
    join(
      control,
      settings,
    ),
    '{ "policies": { "mono/catalog-specifiers": "error" } }\n',
  );
  await writeFile(
    join(
      control,
      'package.json',
    ),
    '{ "name": "control", "dependencies": { "left-pad": "1.3.0" } }\n',
  );
  await runProgramOrEnd({
    file: 'git',
    args: [
      'add',
      '--',
      settings,
      'package.json',
    ],
    cwd: control,
  },);
  /**
   What the check printed and how it ended; its standard error shows in the step log.
   */
  const check = await runProgram({
    file: requiredEnvironment('CLI_PROGRAM',),
    args: [
      requiredEnvironment('CLI_WORD',),
      'check',
      '--all',
      '--policy',
      'mono/catalog-specifiers',
    ],
    captureStdout: true,
    cwd: control,
  },);
  /**
   The printed text without its final line breaks, as `$(...)` captures it.
   */
  const output = withoutTrailingLineBreaks(check.stdout,);
  if (check.status !== REJECTED_STATUS) {
    say(`::error::The negative control exited ${String(check.status,)}, expected ${String(REJECTED_STATUS,)}.`,);
    say(output,);
    throw new StepExitError(1,);
  }
  if (!output.includes(EXPECTED_FINDING,)) {
    say('::error::The negative control exited 1 without the disallowed-specifier finding.',);
    say(output,);
    throw new StepExitError(1,);
  }
  say('The negative control was rejected as expected.',);
}

await runStep(runNegativeControl,);
