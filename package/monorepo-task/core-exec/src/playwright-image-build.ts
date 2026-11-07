#!/usr/bin/env node

/**
 The "Build the image" step of `.github/workflows/playwright-image.yml`: builds `playwright.Dockerfile` with `podman` and
 tags it `<IMAGE>:<TAG>`, labelled with the repository it came from.

 Reads `IMAGE`, `TAG` and `REPOSITORY_URL` from the environment and `playwright.Dockerfile` from the current directory;
 `podman`'s output and exit status are the step's own. Imports only `node:` modules and sibling files that do, because the
 workflow runs it before any dependency is installed.

 @example
 ```bash
 node package/monorepo-task/core-exec/src/playwright-image-build.ts
 ```
 */

import {
  requiredEnvironment,
  runProgramOrEnd,
  runStep,
} from './github-actions-step-support.ts';

/**
 Label that links the package to this repository, so the first push creates it there.
 */
const SOURCE_LABEL = 'org.opencontainers.image.source';

/**
 Label with what the image holds.
 */
const DESCRIPTION_LABEL = 'org.opencontainers.image.description';

/**
 What the image holds, as the package page shows it.
 */
const DESCRIPTION = 'Browsers and system libraries for the Playwright tests of the Monochromatic monorepo (playwright.Dockerfile)';

/**
 Prints the `podman` version, then builds the image.

 @throws {@link StepExitError} With the status of `podman` when it fails

 @example
 ```ts
 await buildImage();
 ```
 */
async function buildImage(): Promise<void> {
  await runProgramOrEnd({
    file: 'podman',
    args: ['--version',],
    cwd: process.cwd(),
  },);
  await runProgramOrEnd({
    file: 'podman',
    args: [
      'build',
      '--file',
      'playwright.Dockerfile',
      '--label',
      `${SOURCE_LABEL}=${requiredEnvironment('REPOSITORY_URL',)}`,
      '--label',
      `${DESCRIPTION_LABEL}=${DESCRIPTION}`,
      '--tag',
      `${requiredEnvironment('IMAGE',)}:${requiredEnvironment('TAG',)}`,
      '.',
    ],
    cwd: process.cwd(),
  },);
}

await runStep(buildImage,);
