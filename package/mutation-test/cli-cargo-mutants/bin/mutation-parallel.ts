#!/usr/bin/env node
/**
 Start one capped systemd user unit per source file; each runs the campaign task through the gate-slot limiter.
 Run without arguments for the options, or see the README.
 */
import { mkdir } from 'node:fs/promises';
import { homedir } from 'node:os';
import { resolve } from 'node:path';

import { systemdRunProgram } from '../src/constants.ts';
import { parseLauncherOptions } from '../src/options.ts';
import { planUnits } from '../src/plan.ts';
import {
  formatStartLine,
  formatWaitHint,
} from '../src/report.ts';
import { runProgram } from '../src/run-program.ts';

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
const startLines = await Promise.all(plans.map(async function startUnit(plan) {
  /**
   Outcome of starting this unit.
   */
  const started = await runProgram({
    program: systemdRunProgram,
    args: plan.systemdRunArguments,
    inheritStdio: false,
  });
  return formatStartLine({
    unit: plan.unit,
    status: started.status,
    stderr: started.stderr,
  });
}));
for (const line of startLines)
  console.log(line);
console.log(
  formatWaitHint(plans.map(function unitOf(plan) {
    return plan.unit;
  })),
);
