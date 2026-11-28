/**
 A guest script a driver starts detached and reads back later, so a build that outlives its command still reports, and
 the log a pull or a copy brings back as the run's evidence.

 The guest's crate build ran for ten minutes and the two compiling jobs of the first runs exhausted the guest's memory,
 where the run lost the command's process and the reason with it. A script started this way runs under `setsid nohup`,
 writes its log where the host can read it, and ends by appending its own status line to that log. The host asks whether
 the script has ended, pulls the log, and reads its status out of the whole log.

 Where the host learns that the script has ended, and how the log reaches it, are the caller's to say:
 {@link lastLineFromTheGuest} asks the guest agent for the log's last line, `e2e/step-22-arms/host-shared-log.ts` reads the
 host's own side of the shared folder because a busy agent then hides nothing, and {@link pullGuestLog} is `mvm`'s own
 copy. Both were answers of the three sets of guest helpers, and the polling, the pull and the judgement of the status
 are the one answer this module holds.

 The guest-name check a detached script carries is `{@link detachedScriptFiles}`'s distinguishing switch: the step 22
 driver's scripts answer it in the step file, the adoption driver's and the start probe's in the launcher, and the
 removal-lineage, settling and cancel-and-settle drivers' scripts carry none.

 The launcher is a command this host waits for, and the status it exits with is read: a launcher that was refused says so
 at once instead of leaving the run to wait out the bound for a log no guest will write.
 */
import { writeFile, } from 'node:fs/promises';
import {
  basename,
  join,
} from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  doneStatusOf,
  detachedScriptFiles,
  endedLineHolds,
  type GuestNameCheck,
} from './mvm-guest-command-line.ts';
import {
  HOST_COMMAND_TIMEOUT_MS,
  LOG_FILE_MODE,
  POLL_INTERVAL_MS,
} from './mvm-guest-bounds.ts';
import { guestBackend, } from './mvm-guest-backend.ts';
import { ensureTheGuestSharedFolder, } from './mvm-guest-shared-folder.ts';
import { GuestDriverError, } from './mvm-guest-error.ts';
import type { GuestFacts, } from './mvm-guest-guest-facts.ts';
import { runGateCommand, } from './native-gate-run.ts';
import {
  readLog,
  runQuietStepCommand,
  runRequiredStepCommand,
  runStepCommand,
} from './mvm-guest-step-command.ts';

/**
 Push one file into the guest's shared folder.

 `mvm push` lands each file under its destination's own name, which is where the launcher looks for the step it starts.

 @param chosen - the run's guest and log folder

 @param from - the file on this host

 @param log - the log's name inside the log directory

 @throws GuestDriverError when the push exited nonzero

 @example
 ```ts
 await pushIntoSharedFolder({ chosen, from: '/logs/build.log.step.sh', log: 'guest-push-build.log' });
 ```
 */
async function pushIntoSharedFolder({
  chosen,
  from,
  log,
}: {
  readonly chosen: GuestFacts;
  readonly from: string;
  readonly log: string;
},): Promise<void> {
  /**
   The backend this run drives, which decides where `mvm push` puts the file.
   */
  const backend = guestBackend();
  // A backend that mounts no share has no folder for the push to land in until this makes one.
  await ensureTheGuestSharedFolder({ chosen, },);
  await runRequiredStepCommand({
    logDirectory: chosen.logDir,
    program: 'mvm',
    argumentWords: [
      ...backend.mvmArgumentWords,
      'push',
      chosen.name,
      from,
      backend.pushDestinationOf(basename(from,),),
    ],
    log,
    timeoutMilliseconds: HOST_COMMAND_TIMEOUT_MS,
  },);
}

/**
 Start one guest script detached from the command that starts it, with its log left in the guest's shared folder.

 @param chosen - the run's guest and log folder

 @param script - the shell script to run inside the guest

 @param log - the log's name inside the guest's shared folder, which also names the two launcher files

 @param timeoutMilliseconds - milliseconds the script may take inside the guest

 @param nameCheck - which of the two files carries the guest's name check, if either

 @throws GuestDriverError when the launcher was refused, or when it never started or was stopped at its bound

 @example
 ```ts
 await startDetachedGuestScript({
   chosen,
   script: 'cargo build --release --tests',
   log: 'build.log',
   timeoutMilliseconds: 2_400_000,
   nameCheck: 'none',
 });
 ```
 */
export async function startDetachedGuestScript({
  chosen,
  script,
  log,
  timeoutMilliseconds,
  nameCheck,
}: {
  readonly chosen: GuestFacts;
  readonly script: string;
  readonly log: string;
  readonly timeoutMilliseconds: number;
  readonly nameCheck: GuestNameCheck;
},): Promise<void> {
  /**
   The step and launcher files this script runs from, one of which carries the guest's name check.
   */
  const files = detachedScriptFiles({
    name: chosen.name,
    script,
    log,
    timeoutMilliseconds,
    nameCheck,
  },);
  /**
   The backend this run drives, which decides the guest's shared folder and how a root command is started.
   */
  const backend = guestBackend();
  /**
   The file holding the script, its bound and the status line it ends with.
   */
  const stepPath = join(
    chosen.logDir,
    `${log}.step.sh`,
  );
  await writeFile(
    stepPath,
    files.step,
    { mode: LOG_FILE_MODE, },
  );
  /**
   The file that starts the step file in a session of its own and returns at once.
   */
  const launchPath = join(
    chosen.logDir,
    `${log}.launch.sh`,
  );
  await writeFile(
    launchPath,
    files.launch,
    { mode: LOG_FILE_MODE, },
  );
  // The two pushes go one after the other: both write the same push log, so a second one started beside the first would
  // truncate the file the first was still writing.
  await pushIntoSharedFolder({
    chosen,
    from: stepPath,
    log: `guest-push-${log}.log`,
  },);
  await pushIntoSharedFolder({
    chosen,
    from: launchPath,
    log: `guest-push-${log}.log`,
  },);
  // The launcher's text ends with `exit 0`, so a status other than zero is the launcher's own refusal: either the guest
  // it names is not the machine the command reached, or `mvm` could not ask the guest at all. What the script went on to
  // do is read from the shared log below, and this refusal is said here rather than after a wait for a log that no guest
  // will write.
  /**
   The status the launcher exited with, which is this host's only news that the step file was started at all.
   */
  const launcherStatus = await runQuietStepCommand({
    program: 'mvm',
    argumentWords: [
      ...backend.mvmArgumentWords,
      'exec',
      chosen.name,
      '--',
      ...backend.rootWords,
      'sh',
      `${backend.guestSharedFolder}/${log}.launch.sh`,
    ],
    timeoutMilliseconds: HOST_COMMAND_TIMEOUT_MS,
  },);
  if (launcherStatus !== 0) {
    /**
     The words that select the backend on a command line, as one word.
     */
    const backendWords = backend.mvmArgumentWords
      .join(' ',);
    /**
     The command as the refusal names it, spelled as a person would type it.
     */
    const refused = `mvm ${backendWords} exec ${chosen.name} -- sh ${backend.guestSharedFolder}/${log}.launch.sh`;
    /**
     The one line a person reads: the command, its status and what that means.
     */
    const said = `${refused} exited ${String(launcherStatus,)}: the launcher was refused, so '${chosen.name}' started none of ${log}.step.sh`;
    throw new GuestDriverError(said,);
  }
}

/**
 The last line of one log inside the guest, as the guest agent prints it.

 A guest that could not be asked answers empty text, which is not a status line: the wait keeps asking until its own
 deadline passes, as the drivers always did.

 @param chosen - the run's guest and log folder

 @param file - the log's name inside the guest's shared folder

 @returns the log's last line, or empty text when the guest answered nothing

 @example
 ```ts
 const line = await lastLineFromTheGuest({ chosen, file: 'build.log' });
 ```
 */
export async function lastLineFromTheGuest({
  chosen,
  file,
}: {
  readonly chosen: GuestFacts;
  readonly file: string;
},): Promise<string> {
  /**
   The backend this run drives, which decides where the guest keeps the log.
   */
  const backend = guestBackend();
  /**
   What the guest answered, which is the log's last line once the script has written one.
   */
  const answer = await runGateCommand({
    program: 'mvm',
    args: [
      ...backend.mvmArgumentWords,
      'exec',
      chosen.name,
      '--',
      'tail',
      '--lines=1',
      `${backend.guestSharedFolder}/${file}`,
    ],
    deadlineMilliseconds: HOST_COMMAND_TIMEOUT_MS,
  },);
  return answer.stdout;
}

/**
 Pull one guest log into the run's own log folder with `mvm`'s own copy, and read it back.

 @param chosen - the run's guest and log folder

 @param file - the log's name inside the guest's shared folder

 @param log - the name of the pull's own host log

 @returns the log's whole text, as `mvm` copied it out

 @throws GuestDriverError when the pull exited nonzero

 @example
 ```ts
 const pulled = await pullGuestLog({ chosen, file: 'adoption-arm.log', log: 'host-pull-adoption.log' });
 ```
 */
export async function pullGuestLog({
  chosen,
  file,
  log,
}: {
  readonly chosen: GuestFacts;
  readonly file: string;
  readonly log: string;
},): Promise<string> {
  /**
   The backend this run drives, which decides where the guest keeps the log.
   */
  const backend = guestBackend();
  /**
   The host's copy of the guest's log, which the pull writes.
   */
  const pulledPath = join(
    chosen.logDir,
    file,
  );
  /**
   The status the pull exited with.
   */
  const status = await runStepCommand({
    logDirectory: chosen.logDir,
    program: 'mvm',
    argumentWords: [
      ...backend.mvmArgumentWords,
      'pull',
      chosen.name,
      `${backend.guestSharedFolder}/${file}`,
      pulledPath,
    ],
    log,
    timeoutMilliseconds: HOST_COMMAND_TIMEOUT_MS,
  },);
  if (status !== 0) {
    /**
     The words that select the backend on a command line, as one word.
     */
    const backendWords = backend.mvmArgumentWords
      .join(' ',);
    /**
     The command as the refusal names it, spelled as a person would type it.
     */
    const refused = `mvm ${backendWords} pull ${chosen.name} ${backend.guestSharedFolder}/${file} ${pulledPath}`;
    /**
     The one line a person reads: the command, its status and where its log is.
     */
    const said = `${refused} exited ${String(status,)}; see ${log}`;
    throw new GuestDriverError(said,);
  }
  return await readLog({ path: pulledPath, },);
}

/**
 How one driver learns that a detached script has ended.

 @param chosen - the run's guest and log folder, which a reader asking the guest agent needs

 @param file - the log's name inside the guest's shared folder

 @returns the log's last line, which is the status line once the script has ended

 @example
 ```ts
 const line = await lastLine({ chosen, file: 'build.log' });
 ```
 */
export type LastLine = (argumentsRead: {
  readonly chosen: GuestFacts;
  readonly file: string;
},) => Promise<string>;

/**
 How one driver brings a detached script's log onto this host.

 @param chosen - the run's guest and log folder

 @param file - the log's name inside the guest's shared folder

 @param log - the name of the pull's own host log

 @returns the log's whole text, as the host now holds it

 @example
 ```ts
 const pulled = await pull({ chosen, file: 'build.log', log: 'host-pull-build.log' });
 ```
 */
export type Pull = (argumentsRead: {
  readonly chosen: GuestFacts;
  readonly file: string;
  readonly log: string;
},) => Promise<string>;

/**
 Whether the log the host can read already ends with the line a finished script writes.

 @param lastLine - how this driver reads the log's last line

 @param file - the log's name inside the guest's shared folder

 @param deadline - the wall-clock time, in milliseconds, past which this stops asking

 @returns true when the log's last line is a status line, false when the deadline passed first

 @example
 ```ts
 const ended = await endedWithin({ lastLine, file: 'build.log', deadline });
 ```
 */
async function endedWithin({
  chosen,
  lastLine,
  file,
  deadline,
}: {
  readonly chosen: GuestFacts;
  readonly lastLine: LastLine;
  readonly file: string;
  readonly deadline: number;
},): Promise<boolean> {
  if (Date.now() >= deadline)
    return false;
  /**
   The log's last line, as the host answered this look.
   */
  const line = await lastLine({
    chosen,
    file,
  },);
  if (endedLineHolds({ text: line, },))
    return true;
  await wait(POLL_INTERVAL_MS,);
  // Each look waits for the one before it, so the next look is asked for here rather than by a loop around both: a loop
  // holding an `await` would read as work that could have run beside itself, and this one cannot.
  return await endedWithin({
    chosen,
    lastLine,
    file,
    deadline,
  },);
}

/**
 Wait for one detached guest script's own status line, pull its log, and refuse anything but a zero status.

 The log is pulled and read before either judgement is made, so the failing run's own output is on this host whatever
 the wait found.

 @param chosen - the run's guest and log folder

 @param log - the log's name inside the guest's shared folder

 @param finishTimeoutMilliseconds - milliseconds the host waits before giving up on the script

 @param lastLine - how this driver learns the script has ended, which asks the guest unless a driver says otherwise

 @param pull - how this driver brings the log onto the host, which is `mvm`'s own copy unless a driver says otherwise

 @returns what the script wrote, which is the whole of its pulled log

 @throws GuestDriverError when the script did not finish inside the bound, or ended with a status other than zero

 @example
 ```ts
 const pulled = await awaitDetachedGuestScript({ chosen, log: 'build.log', finishTimeoutMilliseconds: 2_580_000 });
 ```
 */
export async function awaitDetachedGuestScript({
  chosen,
  log,
  finishTimeoutMilliseconds,
  lastLine,
  pull,
}: {
  readonly chosen: GuestFacts;
  readonly log: string;
  readonly finishTimeoutMilliseconds: number;
  readonly lastLine?: LastLine;
  readonly pull?: Pull;
},): Promise<string> {
  /**
   How this run learns the script has ended, which asks the guest agent unless the driver named its own reader.
   */
  const readLastLine: LastLine = lastLine ?? lastLineFromTheGuest;
  /**
   How this run brings the log onto the host, which is `mvm`'s own copy unless the driver named its own.
   */
  const readPull: Pull = pull ?? pullGuestLog;
  /**
   Whether the script had written its status line before the host stopped asking.
   */
  const finished = await endedWithin({
    chosen,
    lastLine: readLastLine,
    file: log,
    deadline: Date.now() + finishTimeoutMilliseconds,
  },);
  /**
   What the script wrote, which is the whole of its pulled log.
   */
  const pulled = await readPull({
    chosen,
    file: log,
    log: `host-pull-${log}`,
  },);
  if (!finished) {
    throw new GuestDriverError(`${log}: the guest script did not finish inside ${String(finishTimeoutMilliseconds,)} ms; see ${log}`,);
  }
  /**
   The status the script wrote before it ended, absent when the log holds no status line at all.
   */
  const ended = doneStatusOf({ text: pulled, },);
  if (!(ended.found && (ended.status === '0'))) {
    throw new GuestDriverError(
      `${log}: the guest script ended ${ended.found ? ended.status : 'without a status'}; see ${log}`,
    );
  }
  return pulled;
}
