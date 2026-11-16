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

try {
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
}
finally {
  await rm(directory, { force: true, recursive: true, },);
}
