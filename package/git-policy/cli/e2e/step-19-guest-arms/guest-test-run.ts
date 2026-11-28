/**
 One library test of this crate, run by its exact name inside the guest, and the environment every arm of this driver
 starts with.

 A test binary that skipped prints `SKIPPED` and exits zero, so the driver reads the log for the libtest line and for the
 arm's own markers rather than trusting the status (`AGENTS.md` rule `TRV`).

 @example
 ```ts
 const pulled = await runOneGuestTest({ chosen, test, environment: [], log: 'guest-units.log' });
 ```
 */

import type {
  GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';
import { GuestDriverError, } from '../../bin/mvm-guest-error.ts';
import { guest, } from '../../bin/mvm-guest-script.ts';

import {
  armTimeoutMilliseconds,
  cratePackage,
  guestVariable,
  openZfsLibrary,
  openZfsPath,
} from './fixture.ts';

/**
 The environment every arm of this driver starts with: the crate's own `PATH`, the OpenZFS userspace the recipe
 installed, and the name of the guest this arm may act in.

 @param chosen - the run's guest, whose name the arm reads

 @returns the `export` lines every arm of this driver starts with

 @example
 ```ts
 armEnvironment({ chosen });
 ```
 */
export function armEnvironment({
  chosen,
}: {
  readonly chosen: GuestFacts;
}): readonly string[] {
  return [
    'set -eu',
    `cd ${cratePackage}`,
    `export PATH=/root/.cargo/bin:${openZfsPath}`,
    `export LD_LIBRARY_PATH=${openZfsLibrary}`,
    `export ${guestVariable}=${chosen.name}`,
  ];
}

/**
 Run one library test by its exact name inside the guest and hand back its own log.

 The status is required, and so is the libtest line naming the test: a filtered run that matched nothing prints
 `ok. 0 passed`, which proves nothing.

 @param chosen - the run's guest and log folder

 @param test - the test's own module path, as libtest prints it

 @param environment - further `export` lines the test reads

 @param log - the log's name inside the run's log folder

 @param timeoutMilliseconds - milliseconds the guest's own command may take

 @returns the whole log the guest wrote

 @throws GuestDriverError when the run exited nonzero or printed no line for the test

 @example
 ```ts
 const pulled = await runOneGuestTest({ chosen, test, environment: [], log: 'guest-units.log' });
 ```
 */
export async function runOneGuestTest({
  chosen,
  test,
  environment,
  log,
  timeoutMilliseconds = armTimeoutMilliseconds,
}: {
  readonly chosen: GuestFacts;
  readonly test: string;
  readonly environment: readonly string[];
  readonly log: string;
  readonly timeoutMilliseconds?: number;
},): Promise<string> {
  /**
   The one test by its exact name, with the libtest line its own log must carry.
   */
  const script = [
    ...armEnvironment({ chosen, }),
    ...environment,
    `if cargo test --release --lib --locked --features test-only-switches -- --exact ${test} --nocapture > /root/${log} 2>&1; then status=0; else status=$?; fi`,
    `tail --lines=80 /root/${log}`,
    'test "$status" = 0',
    `grep --quiet "^test ${test} ... ok" /root/${log}`,
  ].join('\n',);
  // The harness is built by its own step, so this call only runs one test of
  // it.
  // A guest whose agent stopped answering under this machine's memory pressure
  // ends a call like this with the transport and no status,
  // which says nothing about the arm:
  // the run is asked once more, and both logs are kept.
  try {
    return await guest({
      chosen,
      script,
      log,
      timeoutMilliseconds,
    },);
  } catch (error) {
    if (!(error instanceof GuestDriverError))
      throw error;
    return await guest({
      chosen,
      script,
      log: `${log}.again`,
      timeoutMilliseconds,
    },);
  }
}
