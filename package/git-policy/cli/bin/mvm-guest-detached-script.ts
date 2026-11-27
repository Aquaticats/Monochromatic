/**
 A guest script a driver starts detached and reads back later, so a build that outlives its command still reports.

 The guest's crate build ran for ten minutes and the two compiling jobs of the first runs exhausted the guest's memory,
 where the run lost the command's process and the reason with it. A script started this way runs under `setsid nohup`,
 writes its log where the host can read it, and ends by appending its own status line to that log. The host asks the log
 whether the script has ended, and reads the status out of the whole log once it has.
 */
import { writeFile, } from 'node:fs/promises';
import {
  basename,
  join,
} from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import { runGateCommand, } from './native-gate-run.ts';
import {
  detachedLaunchText,
  detachedStepText,
  endedLineHolds,
  endedStatusText,
  secondsOf,
} from './mvm-guest-command-line.ts';
import { GuestDriverError, } from './mvm-guest-error.ts';
import {
  readStepLog,
  runQuietStepCommand,
  runRequiredStepCommand,
  runStepCommand,
} from './mvm-guest-step-command.ts';

/**
 Milliseconds one `mvm` command of a detached script's start or end may take: a push, the launch, one look at the log,
 and the pull.
 */
const GUEST_COMMAND_TIMEOUT_MS = 120_000;

/**
 Permission bits of the two files a detached script is started from: the account that runs the driver reads them.
 */
const LAUNCHER_FILE_MODE = 0o600;

/**
 The folder inside the guest the host's pushes land in, which the guest mounts as `/mnt/shared`.
 */
const SHARED_FOLDER = '/shared';

/**
 The folder the guest's own commands read the shared folder at.
 */
const GUEST_SHARED_FOLDER = '/mnt/shared';

/**
 Push one file into the guest's shared folder.

 `mvm push` lands each file under its destination's own name, which is where the launcher looks for the step it starts.

 @param name - the guest's name

 @param logDirectory - the directory the push's own log lives in

 @param from - the file on this host

 @param log - the log's name inside the log directory

 @throws GuestDriverError when the push exited nonzero

 @example
 ```ts
 await pushIntoSharedFolder({
   name: 'guest-a',
   logDirectory: '/logs',
   from: '/logs/build.log.step.sh',
   log: 'guest-push-build.log.log',
 });
 ```
 */
async function pushIntoSharedFolder({
  name,
  logDirectory,
  from,
  log,
}: {
  readonly name: string;
  readonly logDirectory: string;
  readonly from: string;
  readonly log: string;
},): Promise<void> {
  await runRequiredStepCommand({
    logDirectory,
    program: 'mvm',
    argumentWords: [
      '--backend',
      'libvirt',
      'push',
      name,
      from,
      `${SHARED_FOLDER}/${basename(from,)}`,
    ],
    log,
    timeoutMilliseconds: GUEST_COMMAND_TIMEOUT_MS,
  },);
}

/**
 Start one guest script detached from the command that starts it, with its log left in the guest's shared folder.

 @param logDirectory - the directory the launcher files and the push logs live in

 @param name - the guest's name, which its own hostname must answer

 @param script - the shell script to run inside the guest

 @param log - the log's name inside the guest's shared folder, which also names the two launcher files

 @param timeoutMilliseconds - milliseconds the script may take inside the guest

 @param guestTimeoutMarginSeconds - seconds the guest's own `timeout` fires before the host's bound

 @throws GuestDriverError when the guest could not be asked to start the script

 @example
 ```ts
 await startDetachedGuestScript({
   logDirectory: '/logs',
   name: 'guest-a',
   script: 'cargo build --release --tests',
   log: 'build.log',
   timeoutMilliseconds: 2_400_000,
   guestTimeoutMarginSeconds: 60,
 });
 ```
 */
export async function startDetachedGuestScript({
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
},): Promise<void> {
  /**
   The guest-side seconds the script's own `timeout` is given.
   */
  const guestTimeoutSeconds = secondsOf({ milliseconds: timeoutMilliseconds, },) - guestTimeoutMarginSeconds;
  /**
   The file holding the script, its bound and the status line it ends with.
   */
  const stepPath = join(
    logDirectory,
    `${log}.step.sh`,
  );
  await writeFile(
    stepPath,
    detachedStepText({
      guestTimeoutSeconds,
      script,
      log,
    },),
    { mode: LAUNCHER_FILE_MODE, },
  );
  /**
   The file that starts the step file in a session of its own and returns at once.
   */
  const launchPath = join(
    logDirectory,
    `${log}.launch.sh`,
  );
  await writeFile(
    launchPath,
    detachedLaunchText({ log, },),
    { mode: LAUNCHER_FILE_MODE, },
  );
  // The two pushes go one after the other: both write the same push log, so a second one started beside the first
  // would truncate the file the first was still writing.
  await pushIntoSharedFolder({
    logDirectory,
    name,
    from: stepPath,
    log: `guest-push-${log}.log`,
  },);
  await pushIntoSharedFolder({
    logDirectory,
    name,
    from: launchPath,
    log: `guest-push-${log}.log`,
  },);
  // The launcher's text ends with `exit 0`, so its own status says only that the guest accepted the file; what the
  // script went on to do is read from the shared log below.
  await runQuietStepCommand({
    program: 'mvm',
    argumentWords: [
      '--backend',
      'libvirt',
      'exec',
      name,
      '--',
      'sh',
      `${GUEST_SHARED_FOLDER}/${log}.launch.sh`,
    ],
    timeoutMilliseconds: GUEST_COMMAND_TIMEOUT_MS,
  },);
}

/**
 The last line of one log inside the guest, as the guest printed it.

 A guest that could not be asked answers empty text, which is not a status line: the wait keeps asking until its own
 deadline passes, as the drivers always did.

 @param name - the guest's name

 @param path - the log's path inside the guest

 @returns the log's last line, or empty text when the guest answered nothing

 @example
 ```ts
 const line = await lastGuestLogLine({ name: 'guest-a', path: '/mnt/shared/build.log' });
 ```
 */
async function lastGuestLogLine({
  name,
  path,
}: {
  readonly name: string;
  readonly path: string;
},): Promise<string> {
  /**
   What the guest answered, which is the log's last line once the script has written one.
   */
  const answer = await runGateCommand({
    program: 'mvm',
    args: [
      '--backend',
      'libvirt',
      'exec',
      name,
      '--',
      'tail',
      '--lines=1',
      path,
    ],
    deadlineMilliseconds: GUEST_COMMAND_TIMEOUT_MS,
  },);
  return answer.stdout;
}

/**
 Whether the guest's log already ends with the line a finished script writes.

 @param name - the guest's name

 @param path - the log's path inside the guest

 @param deadline - the wall-clock time, in milliseconds, past which this stops asking

 @param pollIntervalMilliseconds - how long to wait between two of those questions

 @returns true when the log's last line is a status line, false when the deadline passed first

 @example
 ```ts
 const ended = await endedWithin({
   name: 'guest-a',
   path: '/mnt/shared/build.log',
   deadline,
   pollIntervalMilliseconds: 15_000,
 });
 ```
 */
async function endedWithin({
  name,
  path,
  deadline,
  pollIntervalMilliseconds,
}: {
  readonly name: string;
  readonly path: string;
  readonly deadline: number;
  readonly pollIntervalMilliseconds: number;
},): Promise<boolean> {
  if (Date.now() >= deadline)
    return false;
  /**
   The log's last line, as the guest answered this look.
   */
  const line = await lastGuestLogLine({
    name,
    path,
  },);
  if (endedLineHolds({ text: line, },))
    return true;
  await wait(pollIntervalMilliseconds,);
  // Each look waits for the one before it, so the next look is asked for here rather than by a loop around both: a
  // loop holding an `await` would read as work that could have run beside itself, and this one cannot.
  return await endedWithin({
    name,
    path,
    deadline,
    pollIntervalMilliseconds,
  },);
}

/**
 Wait for one detached guest script's own status line, pull its log, and refuse anything but a zero status.

 The log is pulled and read before either judgement is made, so the failing run's own output is on this host whatever
 the wait found.

 @param logDirectory - the directory the script's log is pulled into

 @param name - the guest's name

 @param log - the log's name inside the guest's shared folder

 @param timeoutMilliseconds - milliseconds the host waits before giving up on the script

 @param pollIntervalMilliseconds - how long to wait between two looks at the log

 @returns what the script wrote, which is the whole of its pulled log

 @throws GuestDriverError when the script did not finish inside the bound, or ended with a status other than zero

 @example
 ```ts
 const pulled = await awaitDetachedGuestScript({
   logDirectory: '/logs',
   name: 'guest-a',
   log: 'build.log',
   timeoutMilliseconds: 2_580_000,
   pollIntervalMilliseconds: 15_000,
 });
 ```
 */
export async function awaitDetachedGuestScript({
  logDirectory,
  name,
  log,
  timeoutMilliseconds,
  pollIntervalMilliseconds,
}: {
  readonly logDirectory: string;
  readonly name: string;
  readonly log: string;
  readonly timeoutMilliseconds: number;
  readonly pollIntervalMilliseconds: number;
},): Promise<string> {
  /**
   The log's path inside the guest, where the step file wrote it.
   */
  const path = `${GUEST_SHARED_FOLDER}/${log}`;
  /**
   The pulled log's own name inside the log directory, which a refusal reports.
   */
  const pulledName = log;
  /**
   Whether the script had written its status line before the host stopped asking.
   */
  const finished = await endedWithin({
    name,
    path,
    deadline: Date.now() + timeoutMilliseconds,
    pollIntervalMilliseconds,
  },);
  /**
   The status the pull exited with.
   */
  const status = await runStepCommand({
    logDirectory,
    program: 'mvm',
    argumentWords: [
      '--backend',
      'libvirt',
      'pull',
      name,
      path,
      join(
        logDirectory,
        pulledName,
      ),
    ],
    log: `host-pull-${log}`,
    timeoutMilliseconds: GUEST_COMMAND_TIMEOUT_MS,
  },);
  if (status !== 0) {
    throw new GuestDriverError(
      `mvm --backend libvirt pull ${name} ${path} ${join(
        logDirectory,
        pulledName,
      )} exited ${String(status,)}; see host-pull-${log}`,
    );
  }
  /**
   What the script wrote, which is the whole of its pulled log.
   */
  const pulled = await readStepLog({
    logDirectory,
    log: pulledName,
  },);
  if (!finished) {
    throw new GuestDriverError(`${log}: the guest script did not finish inside ${String(timeoutMilliseconds,)} ms; see ${log}`,);
  }
  /**
   The status the script wrote before it ended, empty when the log holds no status line at all.
   */
  const endedStatus = endedStatusText({ text: pulled, },);
  if (endedStatus !== '0') {
    throw new GuestDriverError(
      `${log}: the guest script ended ${endedStatus === '' ? 'without a status' : endedStatus}; see ${log}`,
    );
  }
  return pulled;
}
