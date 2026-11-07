#!/usr/bin/env node
/**
 `music-player-desktop-e2e`: runs the keyboard end-to-end suite of the desktop music player against its release
 binary. Run it from the desktop app directory, or name that directory with `--app-dir`.
 
 The run is headless: nothing appears on the desktop. A person who wants to watch passes `--visible` (or sets
 `E2E_VISIBLE=1`), which opens a nested window in the current graphical session. Importing this module runs nothing; the
 run starts only when the file is the entry point.
 
 @example
 ```bash
 music-player-desktop-e2e
 music-player-desktop-e2e --app-dir package/music-player/desktop-app
 E2E_ONLY=seek music-player-desktop-e2e
 music-player-desktop-e2e --visible
 ```
 
 @packageDocumentation
 */

import { homedir, } from 'node:os';
import { resolve, } from 'node:path';
import { parseArgs, } from 'node:util';

import {
  logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';

import { installedCompositorPath, } from './compositor-path.ts';
import {
  loadRunConfig,
  overridesFromEnvironment,
} from './config.ts';
import { printErrorLine, } from './report.ts';
import { runE2e, } from './run.ts';

/**
 Raised after the report when at least one check failed, so the command exits with a non-zero status.
 
 @example
 ```ts
 throw new ChecksFailedError('FAILED focus/ring-visible: the ring is off',);
 ```
 */
export class ChecksFailedError extends Error {
  /**
   Names the error so the exit message shows its kind.
   
   @param message - one line per failed check
   */
  constructor(message: string,) {
    super(message,);
    this.name = 'ChecksFailedError';
  }
}

/**
 What the command line asks for.
 */
export type CommandLine = {
  /**
   The directory of the desktop app: the `--app-dir` value, else the current directory.
   */
  readonly appDir: string;
  /**
   True when `--visible` was given: a person asked for a nested window on the desktop.
   */
  readonly visible: boolean;
};

/**
 Reads the command line.
 
 @param args - arguments after the script name
 
 @returns the directory of the desktop app and whether a visible window was asked for
 
 @throws TypeError when an unknown flag is given
 
 @example
 ```ts
 parseCommandLine(['--app-dir', 'package/music-player/desktop-app',],); // { appDir: '/abs/...', visible: false }
 ```
 */
export function parseCommandLine(args: readonly string[],): CommandLine {
  /**
   Parsed flags.
   */
  const { values, } = parseArgs({
    args: [...args,],
    options: {
      'app-dir': { type: 'string', },
      visible: { type: 'boolean', },
    },
    strict: true,
  },);
  return {
    appDir: resolve(values['app-dir'] ?? process.cwd(),),
    visible: values.visible === true,
  };
}

/**
 Runs the suite and turns failed checks into a failing exit.
 
 @param args - arguments after the script name
 
 @throws ChecksFailedError when at least one check failed
 
 @example
 ```ts
 await main(process.argv.slice(2,),);
 ```
 */
export async function main(args: readonly string[],): Promise<void> {
  /**
   Tagged logger naming the entry point.
   */
  const rl = tagged({
    tag: main.name,
    l: logger,
  },);

  /**
   The directory of the desktop app.
   */
  const {
    appDir,
    visible,
  } = parseCommandLine(args,);
  rl.info(`running against ${appDir}`,);

  /**
   Values the user set in the environment, and the visible window if the flag asked for it.
   */
  const overrides = {
    ...overridesFromEnvironment(),
    ...visible ? { visible: true, } : {},
  };
  /**
   The compositor to start: `E2E_COMPOSITOR`, else the one mise has installed.
   */
  const compositorPath = overrides.compositorPath ?? await installedCompositorPath({ cwd: appDir, },);
  rl.info(`compositor ${compositorPath}`,);

  /**
   The outcomes and the counts.
   */
  const {
    outcomes,
    totals,
  } = await runE2e({
    config: loadRunConfig({
      appDir,
      home: homedir(),
      overrides,
      defaultCompositorPath: compositorPath,
    },),
  },);
  if (totals.failed === 0)
    return;

  /**
   One line per failed check.
   */
  const failures = outcomes
    .filter(function didFail(outcome,): boolean {
      return !outcome.ok;
    },)
    .map(function describeFailure(outcome,): string {
      return `FAILED ${outcome.id}: ${outcome.detail}${
        outcome.evidence === undefined ? '' : ` (evidence: ${outcome.evidence})`
      }`;
    },);
  for (const line of failures)
    printErrorLine(line,);
  throw new ChecksFailedError(`${totals.failed} checks failed`,);
}

if (import.meta.main)
  await main(process.argv
    .slice(2,),);
