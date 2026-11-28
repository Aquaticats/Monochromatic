/**
 Show what each value of the `snapdir` property does, which the project's third
 answer of 2026-11-22 asked for before any code relies on it.

 For each of `visible`, `hidden` and `disabled`, as the unprivileged account and
 under the design's own grants, the arm lists the checkout's `.zfs` directory,
 tries to enter a snapshot's directory by name, and reads the host's mount table
 before and after. Then it reads whether the account may set the property back,
 and which values this guest's OpenZFS release and the one the route's guests run
 accept.

 What the guest runs is `snapdir-arm-script-fixture.ts`, which a unit test holds
 without a guest; this file starts that script and judges the log it leaves.

 Nothing is set on the host: the pool is this guest's file-backed one.

 @example
 ```ts
 const pulled = await snapdirMatrix({ chosen });
 ```
 */

import {
  requireMarkers,
} from '../../bin/mvm-guest-arm-log.ts';
import type {
  GuestFacts,
} from '../../bin/mvm-guest-guest-facts.ts';
import {
  guest,
} from '../../bin/mvm-guest-script.ts';



import {
  armTimeoutMilliseconds,
  expectedSnapdirMarkers,
} from './adoption-fixture.ts';
import { snapdirScript, } from './snapdir-arm-script-fixture.ts';

/**
 The log the snapdir arm writes inside the run's log folder.
 */
const snapdirLog = 'guest-snapdir.log';

/**
 Run the property matrix and refuse a log that does not prove it.

 @param chosen - the run's guest and log folder

 @returns the arm's log, as the guest wrote it

 @throws Error naming the marker the pulled log lacks

 @example
 ```ts
 const pulled = await snapdirMatrix({ chosen });
 ```
 */
export async function snapdirMatrix({
  chosen,
}: {
  readonly chosen: GuestFacts;
},): Promise<string> {
  /**
   The matrix: one fresh pool, then each value of the property read as the
   account the design delegates to.
   */
  const script = snapdirScript();
  /**
   The arm's log, written where the run's own logs go.
   */
  const pulled = await guest({
    chosen,
    script,
    log: snapdirLog,
    timeoutMilliseconds: armTimeoutMilliseconds,
  },);
  requireMarkers({
    text: pulled,
    markers: expectedSnapdirMarkers,
    subject: 'the snapdir log',
    because: 'the guest ran the arm without proving it',
  },);
  return pulled;
}
