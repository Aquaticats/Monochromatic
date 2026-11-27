/**
 One detached guest script whose answer the host reads from the guest's shared folder: start it, wait for its own
 `DONE=` line, pull its log into the run's folder, and refuse a status that is not zero.

 A guest build and a guest arm run for minutes, and the guest agent has lost the status of a long `exec`, so the work
 runs detached and says how it ended in its own log. The answer is read from the host side of the shared folder, which
 is a folder `mvm` mounts in the guest, so a busy agent does not hide a finished script.

 This driver's step file carries the guest-name check rather than the launcher, which is the shape it has always sent
 and why `nameCheck` is `step` here and `launcher` in the adoption driver.

 @example
 ```ts
 const pulled = await detachedStep({ chosen, script: 'id', log: 'guest-id.log', timeoutMilliseconds: 900_000, finishTimeoutMilliseconds: 1_020_000 });
 ```
 */

import {
  awaitDetachedGuestScript,
  startDetachedGuestScript,
} from '../../bin/mvm-guest-detached-script.ts';
import type { GuestFacts, } from '../../bin/mvm-guest-guest-facts.ts';
import {
  lastLineWithText,
  pullSharedLog,
  sharedFile,
} from './host-shared-log.ts';

/**
 The last line of a detached script's log, read from the host's own side of the guest's shared folder.

 @param chosen - the run's guest and log folder

 @param file - the log's name inside the guest's shared folder

 @returns the log's last line with text in it, or empty text while the script still works

 @example
 ```ts
 await lastLineOfTheSharedFolder({ chosen, file: 'guest-crate-build.log' });
 ```
 */
async function lastLineOfTheSharedFolder({
  chosen,
  file,
}: {
  readonly chosen: GuestFacts;
  readonly file: string;
},): Promise<string> {
  return await lastLineWithText({
    path: sharedFile({
      name: chosen.name,
      file,
    },),
  },);
}

/**
 The log as the guest left it, copied out of the host's own side of its shared folder into the run's folder.

 @param chosen - the run's guest and log folder

 @param file - the log's name inside the guest's shared folder

 @returns the log's whole text, as the guest left it

 @throws Error when the shared folder does not hold the file

 @example
 ```ts
 await pullFromTheSharedFolder({ chosen, file: 'adoption-arm.log', log: 'host-pull-arm.log' });
 ```
 */
async function pullFromTheSharedFolder({
  chosen,
  file,
}: {
  readonly chosen: GuestFacts;
  readonly file: string;
  readonly log: string;
},): Promise<string> {
  return await pullSharedLog({
    name: chosen.name,
    file,
    logDir: chosen.logDir,
  },);
}

/**
 Start one detached guest script, wait for its own end, and pull its log.

 @param chosen - the run's guest and log folder

 @param script - the shell script to run inside the guest

 @param log - the log's name inside the guest's shared folder

 @param timeoutMilliseconds - milliseconds the script may take inside the guest

 @param finishTimeoutMilliseconds - milliseconds the host waits for the script to end

 @returns the pulled log, written into the run's log folder

 @throws Error naming the status the script ended with, or the wait that ended

 @example
 ```ts
 const pulled = await detachedStep({ chosen, script: 'id', log: 'guest-id.log', timeoutMilliseconds: 900_000, finishTimeoutMilliseconds: 1_020_000 });
 ```
 */
export async function detachedStep({
  chosen,
  script,
  log,
  timeoutMilliseconds,
  finishTimeoutMilliseconds,
}: {
  readonly chosen: GuestFacts;
  readonly script: string;
  readonly log: string;
  readonly timeoutMilliseconds: number;
  readonly finishTimeoutMilliseconds: number;
},): Promise<string> {
  await startDetachedGuestScript({
    chosen,
    script,
    log,
    timeoutMilliseconds,
    nameCheck: 'step',
  },);
  return await awaitDetachedGuestScript({
    chosen,
    log,
    finishTimeoutMilliseconds,
    lastLine: lastLineOfTheSharedFolder,
    pull: pullFromTheSharedFolder,
  },);
}
