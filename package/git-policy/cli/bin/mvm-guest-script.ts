/**
 One guest script a guest driver's step runs to its end through `mvm`, with the guest's whole answer kept in one named log.

 The script reaches the guest as a single argument to its one shell, after the guest's own hostname has been checked, so
 a driver that was run outside the guest it names stops at that check instead of running a ZFS command on a person's
 machine. The log is read back and returned: what a guest printed is the evidence the step judges.

 The bound the host gives `mvm` is the guest's own bound plus the margin the command line applies, so the guest's
 `timeout` fires first and the guest reports the script's own status rather than being killed with its whole command.
 `bin/mvm-guest-script.ts` answered this question for four drivers and `e2e/guest-host/guest-side-scripts.ts` for ten
 arm files; the surviving answer is `mvm-guest`'s, which starts the command in a process group of its own.
 */
import {
  join,
} from 'node:path';

import {
  guestCommandLine,
} from './mvm-guest-command-line.ts';
import { guestBackend, } from './mvm-guest-backend.ts';
import { GuestDriverError, } from './mvm-guest-error.ts';
import type { GuestFacts, } from './mvm-guest-guest-facts.ts';
import {
  readLog,
  runStepCommand,
} from './mvm-guest-step-command.ts';

/**
 The `mvm` subcommand that runs one command inside a named guest, through the guest agent.
 */
const EXEC_SUBCOMMAND = 'exec';

/**
 Run one script inside a named guest and answer what the guest wrote.

 @param chosen - the run's guest and log folder

 @param script - the shell script to run inside the guest

 @param log - the log's name inside the log directory

 @param timeoutMilliseconds - how long the host waits before `mvm` and everything it started are stopped, which the
 guest's own `timeout` is sized from

 @returns what the guest wrote, which is the whole of its log

 @throws GuestDriverError when `mvm` could not run its command, or when the command exited nonzero

 @example
 ```ts
 const printed = await guest({ chosen, script: 'zpool list', log: 'guest-pool.log', timeoutMilliseconds: 1_800_000 });
 ```
 */
export async function guest({
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
  /**
   The status the guest's command exited with, which `mvm` passes on.
   */
  /**
   The backend this run drives, whose words select it on every `mvm` this step starts.
   */
  const backend = guestBackend();
  /**
   The status the guest's command exited with, which `mvm` passes on.
   */
  const status = await runStepCommand({
    logDirectory: chosen.logDir,
    program: 'mvm',
    argumentWords: [
      ...backend.mvmArgumentWords,
      EXEC_SUBCOMMAND,
      chosen.name,
      '--',
      guestCommandLine({
        name: chosen.name,
        timeoutMilliseconds,
        script,
      },),
    ],
    log,
    timeoutMilliseconds,
  },);
  if (status !== 0)
    throw new GuestDriverError(`mvm exec exited ${String(status,)}; see ${log}`,);
  return await readLog({
    path: join(
      chosen.logDir,
      log,
    ),
  },);
}
