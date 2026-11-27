/**
 One guest script a guest driver's step runs through `mvm`, with the guest's whole answer kept in one named log.

 The script reaches the guest as a single argument to its one shell, after the guest's own hostname has been checked, so
 a driver that was run outside the guest it names stops at that check instead of running a ZFS command on a person's
 machine. The log is read back and returned: what a guest printed is the evidence the step judges.
 */
import {
  guestCommandLine,
  secondsOf,
} from './mvm-guest-command-line.ts';
import { GuestDriverError, } from './mvm-guest-error.ts';
import {
  readStepLog,
  runStepCommand,
} from './mvm-guest-step-command.ts';

/**
 The `mvm` subcommand that runs one command inside a named guest, through the guest agent.
 */
const EXEC_SUBCOMMAND = 'exec';

/**
 The backend `mvm` is asked for: the guests this package's arms need are `libvirt` domains.
 */
const LIBVIRT_BACKEND = 'libvirt';

/**
 Run one script inside a named guest and answer what the guest wrote.

 The bound the host gives `mvm` is the guest's own bound plus the margin a driver names, so the guest's `timeout` fires
 first and the guest reports the script's own status rather than being killed with its whole command.

 @param logDirectory - the directory the step's logs live in

 @param name - the guest's name, which its own hostname must answer

 @param script - the shell script to run inside the guest

 @param log - the log's name inside the log directory

 @param timeoutMilliseconds - how long the host waits before `mvm` and everything it started are stopped

 @param guestTimeoutMarginSeconds - seconds the guest's own `timeout` fires before the host's bound

 @returns what the guest wrote, which is the whole of its log

 @throws GuestDriverError when `mvm` could not run its command, or when the command exited nonzero

 @example
 ```ts
 const printed = await runGuestScript({
   logDirectory: '/logs',
   name: 'git-policy-lock',
   script: 'zpool list',
   log: 'guest-pool.log',
   timeoutMilliseconds: 1_800_000,
   guestTimeoutMarginSeconds: 60,
 });
 ```
 */
export async function runGuestScript({
  logDirectory,
  name,
  script,
  log,
  timeoutMilliseconds,
  guestTimeoutMarginSeconds,
}: {
  readonly logDirectory: string;
  readonly name: string;
  readonly script: string;
  readonly log: string;
  readonly timeoutMilliseconds: number;
  readonly guestTimeoutMarginSeconds: number;
},): Promise<string> {
  /**
   The status the guest's command exited with, which `mvm` passes on.
   */
  const status = await runStepCommand({
    logDirectory,
    program: 'mvm',
    argumentWords: [
      '--backend',
      LIBVIRT_BACKEND,
      EXEC_SUBCOMMAND,
      name,
      '--',
      guestCommandLine({
        name,
        guestTimeoutSeconds: secondsOf({ milliseconds: timeoutMilliseconds, },) - guestTimeoutMarginSeconds,
        script,
      },),
    ],
    log,
    timeoutMilliseconds,
  },);
  if (status !== 0)
    throw new GuestDriverError(`mvm exec exited ${String(status,)}; see ${log}`,);
  return await readStepLog({
    logDirectory,
    log,
  },);
}
