#!/usr/bin/env node

/**
 The "Compute the tag" step of `.github/workflows/playwright-image.yml`: the tag of the image is the first 16 hex digits of
 the SHA-256 of `playwright.Dockerfile`, so a tag names one content. The step writes it as the `value` output and prints it.

 Reads `playwright.Dockerfile` from the current directory and appends to `GITHUB_OUTPUT`. The tag is computed by
 `playwrightImageTag`, which `mise run prepare:playwright` also uses, so both name the same image. Imports only `node:`
 modules and sibling files that do, because the workflow runs it right after checkout, before any dependency is installed.

 @example
 ```bash
 node package/monorepo-task/core-exec/src/playwright-image-tag.ts
 ```
 */

import { readFile, } from 'node:fs/promises';

import { playwrightImageTag, } from './playwright-image-logic.ts';
import {
  appendToRunnerFile,
  runStep,
  say,
  sayOnStandardError,
  StepExitError,
} from './github-actions-step-support.ts';

/**
 File whose content the tag names, relative to the checkout.
 */
const DOCKERFILE_PATH = 'playwright.Dockerfile';

/**
 Computes the tag and publishes it as the step's `value` output.

 @throws {@link StepExitError} With status 1 when the Dockerfile cannot be read, as `sha256sum` ended

 @example
 ```ts
 await computeTag();
 ```
 */
async function computeTag(): Promise<void> {
  /**
   The exact bytes of the Dockerfile.
   */
  const dockerfile = await readDockerfile();
  /**
   First 16 hex digits of its SHA-256.
   */
  const tag = playwrightImageTag(dockerfile,);
  await appendToRunnerFile({
    variable: 'GITHUB_OUTPUT',
    text: `value=${tag}\n`,
  },);
  say(`tag ${tag}`,);
}

/**
 Reads the Dockerfile, ending the step the way `sha256sum` did when it cannot.

 @returns The file's bytes

 @throws {@link StepExitError} With status 1 when the file cannot be read

 @example
 ```ts
 const bytes = await readDockerfile();
 ```
 */
async function readDockerfile(): Promise<Uint8Array> {
  try {
    return await readFile(DOCKERFILE_PATH,);
  }
  catch (error) {
    sayOnStandardError(`sha256sum: ${DOCKERFILE_PATH}: could not be read (${String(error,)})`,);
    throw new StepExitError(1,);
  }
}

await runStep(computeTag,);
