/**
 One detached guest script whose answer the host reads from the guest's shared
 folder: start it, wait for its own `DONE=` line, pull its log into the run's
 folder, and refuse a status that is not zero.

 A guest build and a guest arm run for minutes, and the guest agent has lost the
 status of a long `exec`, so the work runs detached and says how it ended in its
 own log. The answer is read from the host side of the shared folder, which is a
 folder `mvm` mounts in the guest, so a busy agent does not hide a finished
 script.

 @example
 ```ts
 const pulled = await detachedStep({ chosen, script: 'id', log: 'guest-id.log', timeout: 900_000, finishTimeout: 1_020_000 });
 ```
 */

import {
  startDetachedScript,
  waitForDoneLine,
} from '../guest-host/guest-side-scripts.ts';
import type { GuestRun, } from '../guest-host/guest-run.ts';
import {
  detachedScriptFiles,
  doneStatusOf,
} from '../guest-host/guest-host-rules-fixture.ts';
import {
  lastLineWithText,
  pullSharedLog,
  sharedFile,
} from './host-shared-log.ts';

/**
 Start one detached guest script, wait for its own end, and pull its log.

 @param chosen - the run's guest and log folder

 @param script - the shell script to run inside the guest

 @param log - the log's name inside the guest's shared folder

 @param timeout - milliseconds the script may take inside the guest

 @param finishTimeout - milliseconds the host waits for the script to end

 @returns the pulled log, written into the run's log folder

 @throws Error naming the status the script ended with, or the wait that ended

 @example
 ```ts
 const pulled = await detachedStep({ chosen, script: 'id', log: 'guest-id.log', timeout: 900_000, finishTimeout: 1_020_000 });
 ```
 */
export async function detachedStep({
  chosen,
  script,
  log,
  timeout,
  finishTimeout,
}: {
  readonly chosen: GuestRun;
  readonly script: string;
  readonly log: string;
  readonly timeout: number;
  readonly finishTimeout: number;
},): Promise<string> {
  /**
   The step and the launcher the guest runs, with the launcher doing the naming
   check so a wrong guest refuses before the script starts.
   */
  const files = detachedScriptFiles({
    chosen,
    script,
    log,
    timeout,
    hostnameCheckInLaunch: true,
  },);
  await startDetachedScript({
    chosen,
    step: files.step,
    launch: files.launch,
    log,
  },);
  /**
   The host's path to the log the guest appends to.
   */
  const path = sharedFile({
    name: chosen.name,
    file: log,
  },);
  /**
   The log's last line, which is the `DONE=` line once the script has ended.

   @returns the last line of the guest's log, or empty text when it has none
   */
  function answer(): Promise<string> {
    return lastLineWithText({ path, },);
  }

  /**
   Whether the script ended inside the host's bound.
   */
  const finished = await waitForDoneLine({
    answer,
    timeout: finishTimeout,
  },);
  /**
   The log as the guest left it, copied into the run's folder.
   */
  const pulled = await pullSharedLog({
    name: chosen.name,
    file: log,
    logDir: chosen.logDir,
  },);
  if (!finished)
    throw new Error(`${log}: the guest script did not finish inside ${String(finishTimeout)} ms; see ${log}`);
  /**
   The status the script wrote into its own log.
   */
  const status = doneStatusOf({ pulled, },);
  if (!(status.found && (status.status === '0')))
    throw new Error(`${log}: the guest script ended ${status.found ? status.status : 'without a status'}; see ${log}`);
  return pulled;
}
