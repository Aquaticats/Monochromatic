#!/usr/bin/env node

/**
 Command-line entry of `mise run prepare:playwright`: makes the Playwright container image available locally.

 Reads `playwright.Dockerfile` from the current directory (the repository root when mise runs it), pulls
 `ghcr.io/aquaticats/monochromatic-playwright:<tag>` for its content, and builds the file locally only when the pull
 fails (see {@link ensurePlaywrightImage}). Podman's output stays hidden unless a step fails.

 @example
 ```bash
 node package/monorepo-task/core-exec/src/playwright-image.ts
 ```
 */

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import spawn, { SubprocessError, } from 'nano-spawn';
import { readFile, } from 'node:fs/promises';

import {
  ensurePlaywrightImage,
  type PodmanResult,
} from './playwright-image-logic.ts';

/**
 File built when no image can be pulled, relative to the current directory.
 */
const DOCKERFILE_PATH = 'playwright.Dockerfile';

/**
 Runs the real `podman`, capturing its standard output and standard error together.

 @param args - Arguments after `podman`

 @returns The exit status and the combined output; 127 when podman cannot be started

 @example
 ```ts
 const { exitCode, } = await runPodman(['image', 'exists', 'monochromatic-playwright',],);
 ```
 */
async function runPodman(args: readonly string[],): Promise<PodmanResult> {
  try {
    /**
     Result of a successful run.
     */
    const done = await spawn(
      'podman',
      [...args,],
    );
    return {
      exitCode: 0,
      output: done.output,
    };
  }
  catch (error) {
    // A podman that started and exited non-zero reports its status and output.
    if ((error instanceof SubprocessError) && (error.exitCode !== undefined)) {
      return {
        exitCode: error.exitCode,
        output: error.output,
      };
    }
    return {
      exitCode: 127,
      output: `podman could not be started: ${caughtValueText(error,)}`,
    };
  }
}

if (import.meta.main) {
  /**
   Arguments after the entry; the command has none, so any would be silently ignored without this check.
   */
  const extra = process.argv
    .slice(2,);
  if (extra.length > 0)
    throw new Error(`task-playwright-image takes no arguments, but was given: ${extra.join(' ',)}`,);
  // A failure is thrown, so node reports it and exits with its standard status 1.
  await ensurePlaywrightImage({
    dockerfile: await readFile(DOCKERFILE_PATH,),
    dockerfilePath: DOCKERFILE_PATH,
    podman: runPodman,
    log(line,) {
      console.log(`[prepare:playwright] ${line}`,);
    },
  },);
}
