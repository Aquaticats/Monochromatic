#!/usr/bin/env node
/**
 Run one heavy command under a host-wide slot limit, so the campaigns working in parallel do not push the host load
 past what their time limits tolerate. The command inherits stdio and its exit status becomes this one's.
 */
import { mkdir } from 'node:fs/promises';
import { homedir } from 'node:os';

import { parseGateSlotArguments } from '../src/gate-slot.ts';
import { runNicedCommand } from '../src/gate-slot-command.ts';
import { runInSlot } from '../src/gate-slot-run.ts';

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
  runCommand: async function runCommand({ command }) {
    return await runNicedCommand({ command });
  },
});
