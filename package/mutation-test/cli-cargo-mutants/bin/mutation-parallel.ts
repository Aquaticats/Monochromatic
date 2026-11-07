#!/usr/bin/env node
/**
 Start one capped systemd user unit per source file; each runs the campaign task through the gate-slot limiter, after
 every unit's image tag has been made from the gate image (`--from-tag`) or found to exist.
 Run without arguments for the options, or see the README.
 */
import { mkdir } from 'node:fs/promises';
import { homedir } from 'node:os';
import { resolve } from 'node:path';

import { parseLauncherOptions } from '../src/options.ts';
import { planUnits } from '../src/plan.ts';
import { formatWaitHint } from '../src/report.ts';
import { startUnit } from '../src/start-units.ts';
import { ensureUnitImages } from '../src/unit-images.ts';

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
 missing dies at once, and the launcher would still exit 0.
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
 Start lines of every unit, in argument order; the units start together and the lines print once all have started.
 */
const startLines = (await Promise.all(plans.map(function startPlan(plan) {
  return startUnit({
    unit: plan.unit,
    systemdRunArguments: plan.systemdRunArguments,
  });
})))
  .map(function lineOf(started) {
    return started.line;
  });
for (const line of startLines)
  console.log(line);
console.log(
  formatWaitHint(plans.map(function unitOf(plan) {
    return plan.unit;
  })),
);
