#!/usr/bin/env node
/**
 Start one capped systemd user unit per source file; each runs the campaign task through the gate-slot limiter, after
 every unit's image tag has been made from the gate image (`--from-tag`) or found to exist.
 Exits nonzero when a unit did not start (the others keep running) or the launch was refused; see the README.
 Run without arguments for the options, or see the README.
 */
import { mkdir } from 'node:fs/promises';
import { homedir } from 'node:os';
import { resolve } from 'node:path';

import { launchUnits } from '../src/launch-units.ts';
import { runLauncherMain } from '../src/launcher-main.ts';
import { parseLauncherOptions } from '../src/options.ts';
import { planUnits } from '../src/plan.ts';
import { ensureUnitImages } from '../src/unit-images.ts';

/**
 Run the launch.

 @returns the exit status: 0 when every unit started

 @throws CargoMutantsToolingError when an option is wrong or an image tag is missing, before any unit starts
 */
async function main(): Promise<number> {
  /**
   Options of this launch, validated before anything starts.
   */
  const options = parseLauncherOptions({
    argv: process.argv
      .slice(2),
    environment: process.env,
    home: homedir(),
    packageDirectory: resolve(
      import.meta.dirname,
      '..',
    ),
  });
  /**
   Units to start, planned before anything starts so a bad name stops the whole launch.
   */
  const plans = planUnits(options);
  /**
   Lines about the unit image tags, made from the gate image or checked, before anything starts: a unit whose image is
   missing dies at once.
   */
  const imageLines = await ensureUnitImages({
    options,
    plans,
    ...(process.env
      .XDG_RUNTIME_DIR
      === undefined ? {} : { runtimeDirectory: process.env
        .XDG_RUNTIME_DIR, }),
  });
  for (const line of imageLines)
    console.log(line);
  await mkdir(
    options.logDirectory,
    { recursive: true },
  );
  await mkdir(
    options.temporaryDirectory,
    { recursive: true },
  );
  /**
   How the units started, which decides the exit status.
   */
  const launch = await launchUnits({ plans });
  for (const line of launch.lines)
    console.log(line);
  return launch.exitCode;
}

process.exitCode = await runLauncherMain({ main });
