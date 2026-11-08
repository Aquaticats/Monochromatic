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
 Memory cap of the build: 12 GiB, the `BUILD_MEMORY` that `.github/workflows/music-player-release.yml` leaves on its 4 vCPU,
 16 GB hosted runners for the runner's own processes.
 Adopted, not measured.
 */
const BUILD_MEMORY = '12g';

/**
 CFS period of the build, 100000 microseconds, which is `podman`'s own default period.
 */
const BUILD_CPU_PERIOD_MICROSECONDS = 100_000;

/**
 CPUs of the build: all four vCPUs of a hosted runner, as `BUILD_CPUS` in `music-player-release.yml`.
 */
const BUILD_CPUS = 4;

/**
 Prints the `podman` version, then builds the image.

 The bound is spelled here and not imported from `@monochromatic-dev/container-limit-core`, because the workflow runs
 this file before any dependency is installed; `playwright-image-steps.unit.test.ts` fails when the two differ.
 `podman build` has no disk-weight or process option, so the bound is memory and the CFS quota.

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
      `--memory=${BUILD_MEMORY}`,
      `--cpu-period=${String(BUILD_CPU_PERIOD_MICROSECONDS,)}`,
      `--cpu-quota=${String(BUILD_CPUS * BUILD_CPU_PERIOD_MICROSECONDS,)}`,
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
