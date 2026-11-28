/**
 The slot-state guest driver's build of the arm's test target, started detached from the command that starts it.

 The build takes minutes (6m 07s on 2026-11-27), and a guest agent can lose a long command's result
 (`doc/troubleshooting/mvm-guest-agent-forgets-a-long-commands-result.md`), so the script is started under
 `setsid nohup`, appends the line that says how it ended, and the host waits for that line and pulls the log. It lives
 here rather than in the driver because that driver is at the oxlint line budget, and one call to this function is all
 it adds to it.

 @example
 ```ts
 await buildDetached({ chosen, script: 'cargo build --release', log: 'build.log', timeoutMilliseconds: 2_400_000 });
 ```
 */
import {
  awaitDetachedGuestScript,
  startDetachedGuestScript,
} from '../bin/mvm-guest-detached-script.ts';
import type { GuestFacts, } from '../bin/mvm-guest-guest-facts.ts';

/**
 Milliseconds the host waits past the build's own bound, so the guest can write the status line the host reads.
 */
const BUILD_FINISH_MARGIN_MS = 180_000;

/**
 Start one guest script detached and wait for the status line it appends to its own log.

 @param chosen - the run's guest and log folder

 @param script - the shell script the guest runs detached

 @param log - the log's name inside the guest's shared folder

 @param timeoutMilliseconds - milliseconds the script may take inside the guest

 @returns what the script wrote, which is the whole of its pulled log

 @throws Error naming the status the script ended with, or the wait that ended

 @example
 ```ts
 await buildDetached({ chosen, script: 'cargo build --release', log: 'build.log', timeoutMilliseconds: 2_400_000 });
 ```
 */
export async function buildDetached({
  chosen,
  script,
  log,
  timeoutMilliseconds,
}: {
  readonly chosen: GuestFacts;
  readonly script: string;
  readonly log: string;
  readonly timeoutMilliseconds: number;
},): Promise<string> {
  await startDetachedGuestScript({
    chosen,
    script,
    log,
    timeoutMilliseconds,
    nameCheck: 'launcher',
  },);
  return await awaitDetachedGuestScript({
    chosen,
    log,
    finishTimeoutMilliseconds: timeoutMilliseconds + BUILD_FINISH_MARGIN_MS,
  },);
}
