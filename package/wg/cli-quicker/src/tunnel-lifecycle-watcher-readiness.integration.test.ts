/**
 Verifies that a route watcher whose cold start is slow is still accepted by `up`.
 The readiness wait is a measured bound on the registration event, not a count of short sleeps,
 so a watcher that registers in a second and a half is not killed halfway through registering.

 The scenario starts the real watcher through `startBypassWatcher`,
 which asks the system manager for the sentinel service when one is reachable,
 so this test runs in a container without a system manager and refuses to run elsewhere.

 @module
 */

import assert from 'node:assert/strict';
import { execFileSync, } from 'node:child_process';
import { existsSync, } from 'node:fs';
import {
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

/**
 Bound of the whole scenario, in milliseconds.
 */
const SCENARIO_BOUND_MILLISECONDS = 120_000;

/**
 What the timeout scenario observed, as it printed it.
 */
type TimeoutObserved = {
  /**
   Whether the start accepted the watcher it started.
   */
  readonly accepted: boolean;
  /**
   How long the start took, in milliseconds.
   */
  readonly elapsedMilliseconds: number;
  /**
   What the start threw, absent when it accepted the watcher.
   */
  readonly failure?: string;
  /**
   Whether a watcher record was left behind.
   */
  readonly recordLeft: boolean;
};

/**
 What the scenario observed, as it printed it.
 */
type Observed = {
  /**
   Whether the start accepted the watcher it started.
   */
  readonly registered: boolean;
  /**
   How long the start took, in milliseconds.
   */
  readonly elapsedMilliseconds: number;
  /**
   What the start threw, absent when it accepted the watcher.
   */
  readonly failure?: string;
};

if (existsSync('/run/systemd/private')) {
  throw new Error(
    'This test starts the sentinel through a watcher, which asks the system manager for a unit when one is reachable; run it in a container without a system manager.',
  );
}

/**
Scratch directory of this test, which the scenario keeps its runtime state in.
 */
const directory = await mkdtemp(join(scratchRoot(), 'wg-watcher-readiness-',),);
await using scratch = {
  async [Symbol.asyncDispose](): Promise<void> {
    await rm(directory, { force: true, recursive: true, },);
  },
};
/**
 What the scenario printed on its last line.
 */
const stdout = execFileSync(
  process.execPath,
  [
    new URL('tunnel-lifecycle-watcher-readiness.scenario.ts', import.meta.url,).pathname,
    directory,
  ],
  {
    timeout: SCENARIO_BOUND_MILLISECONDS,
    encoding: 'utf8',
  },
);
/**
 The observation, parsed from the scenario's verdict line.
 */
const observed = JSON.parse(
  stdout.trim()
    .split('\n',)
    .at(-1,) ?? '{}',
) as Observed;
assert.equal(
  observed.registered,
  true,
  `a healthy watcher whose startup took ${String(observed.elapsedMilliseconds,)} ms was killed before it registered: ${String(observed.failure,)}`,
);

/**
 What the timeout scenario printed on its last line.
 */
const timeoutStdout = execFileSync(
  process.execPath,
  [
    new URL('tunnel-lifecycle-watcher-timeout.scenario.ts', import.meta.url,).pathname,
    join(directory, 'timeout',),
  ],
  {
    timeout: SCENARIO_BOUND_MILLISECONDS,
    encoding: 'utf8',
  },
);
/**
 The observation, parsed from the timeout scenario's verdict line.
 */
const timedOut = JSON.parse(
  timeoutStdout.trim()
    .split('\n',)
    .at(-1,) ?? '{}',
) as TimeoutObserved;
assert.equal(
  timedOut.accepted,
  false,
  `a watcher that never registered was accepted after ${String(timedOut.elapsedMilliseconds,)} ms`,
);
// The deadline is what ends the wait, so the refusal comes at about ten seconds and not when the watcher finally starts.
assert.ok(
  timedOut.elapsedMilliseconds < 20_000,
  `the start waited ${String(timedOut.elapsedMilliseconds,)} ms for a watcher that never registered`,
);
assert.ok(
  timedOut.failure?.includes('did not register process identity',) === true,
  `the refusal does not say the watcher never registered: ${String(timedOut.failure,)}`,
);
// A group whose members have all exited is gone for every purpose the program has, so the failure may not claim it survived a kill.
assert.ok(
  (!(timedOut.failure?.includes('survived SIGKILL',) ?? false)),
  `the refusal claims a group survived SIGKILL: ${timedOut.failure ?? ''}`,
);
