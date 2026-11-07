#!/usr/bin/env node

/**
 The "Push the image" step of `.github/workflows/playwright-image.yml`: logs in to `ghcr.io` with the workflow's own token,
 pushes `<IMAGE>:<TAG>`, and publishes the digest `podman` wrote as the `digest` output.

 Reads `IMAGE`, `TAG`, `GITHUB_TOKEN`, `GITHUB_ACTOR_NAME` and `RUNNER_TEMP` from the environment. The token goes to
 `podman login` through its standard input only, never on a command line. Imports only `node:` modules and sibling files
 that do, because the workflow runs it before any dependency is installed.

 @example
 ```bash
 node package/monorepo-task/core-exec/src/playwright-image-push.ts
 ```
 */

import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';

import {
  appendToRunnerFile,
  requiredEnvironment,
  runProgramOrEnd,
  runStep,
  say,
  sayOnStandardError,
  StepExitError,
  withoutTrailingLineBreaks,
} from './github-actions-step-support.ts';

/**
 Start of a well-formed digest.
 */
const DIGEST_PREFIX = 'sha256:';

/**
 Reads the digest file `podman push` wrote, ending the step the way `cat` did when it cannot.

 @param path - The digest file

 @returns Its text without trailing line breaks

 @throws {@link StepExitError} With status 1 when the file cannot be read

 @example
 ```ts
 const digest = await readDigest(path,);
 ```
 */
async function readDigest(path: string,): Promise<string> {
  try {
    return withoutTrailingLineBreaks(await readFile(
      path,
      'utf8',
    ),);
  }
  catch (error) {
    sayOnStandardError(`cat: ${path}: could not be read (${String(error,)})`,);
    throw new StepExitError(1,);
  }
}

/**
 Logs in, pushes and publishes the digest.

 @throws {@link StepExitError} With the status of a failing `podman`, or status 1 when it wrote something that is not a
   digest

 @example
 ```ts
 await pushImage();
 ```
 */
async function pushImage(): Promise<void> {
  /**
   The image to push.
   */
  const reference = `${requiredEnvironment('IMAGE',)}:${requiredEnvironment('TAG',)}`;
  /**
   Where `podman push` writes the digest.
   */
  const digestFile = join(
    requiredEnvironment('RUNNER_TEMP',),
    'playwright-image.digest',
  );
  await runProgramOrEnd({
    file: 'podman',
    args: [
      'login',
      'ghcr.io',
      '--username',
      requiredEnvironment('GITHUB_ACTOR_NAME',),
      '--password-stdin',
    ],
    input: requiredEnvironment('GITHUB_TOKEN',),
    cwd: process.cwd(),
  },);
  await runProgramOrEnd({
    file: 'podman',
    args: [
      'push',
      '--digestfile',
      digestFile,
      reference,
    ],
    cwd: process.cwd(),
  },);
  /**
   What `podman` wrote as the digest.
   */
  const digest = await readDigest(digestFile,);
  if (!digest.startsWith(DIGEST_PREFIX,)) {
    say(`::error title=Unexpected digest::podman wrote '${digest}' as the digest of the pushed image`,);
    throw new StepExitError(1,);
  }
  await appendToRunnerFile({
    variable: 'GITHUB_OUTPUT',
    text: `digest=${digest}\n`,
  },);
  say(`pushed ${reference} as ${digest}`,);
}

await runStep(pushImage,);
