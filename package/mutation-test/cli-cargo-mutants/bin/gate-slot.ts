#!/usr/bin/env node
/**
 Run one heavy command under a host-wide slot limit, so the campaigns working in parallel do not push the host load
 past what their time limits tolerate. The command inherits stdio and its exit status becomes this one's.
 */
import { mkdir } from 'node:fs/promises';
import { homedir } from 'node:os';
import { setTimeout as sleep } from 'node:timers/promises';

import {
  parseGateSlotArguments,
  runInSlot,
} from '../src/gate-slot.ts';
import { flockProgram } from '../src/constants.ts';
import { runProgram } from '../src/run-program.ts';

/**
 Slot settings and command of this run.
 */
const options = parseGateSlotArguments({
  argv: process.argv
    .slice(2),
  environment: process.env,
  home: homedir(),
});
await mkdir(
  options.lockDirectory,
  { recursive: true },
);
// The command stays in the caller's cgroup: a `systemd-run --scope` wrapper would move it out of the shared CPU cap.
process.exitCode = await runInSlot({
  options,
  runFlock: async function runFlock(flockArguments) {
    return (await runProgram({
      program: flockProgram,
      args: flockArguments,
      inheritStdio: true,
    })).status;
  },
  sleep: async function sleepFor(milliseconds) {
    await sleep(milliseconds);
  },
});
