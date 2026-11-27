/**
 What the host does to a guest's own scripts: run one to its end with its output
 in a log, start a long one detached so a lost answer cannot lose its work, and
 wait for the `DONE=` line it appends when it ends.

 A guest build and a guest arm run for minutes, and the guest agent has lost the
 status of a long `exec` (`doc/troubleshooting/mvm-guest-agent-forgets-a-long-commands-result.md`).
 The detached shape is therefore the one a long step uses: the script runs under
 `setsid nohup` from a file, writes its log where the host can read it, and ends
 by appending `DONE=<status>` to that log, which the host then refuses a nonzero
 status from. The step and the launcher are files, not one command line: `mvm
 exec` hands its argument to one shell, and a detached job written inline nested
 that shell inside another, so the ampersand ended up inside the inner quoting
 and the host waited for the whole script instead of its start
 (`doc/troubleshooting/mvm-push-writes-the-shared-folder-not-the-guest-path.md`).

 @example
 ```ts
 const { step, launch } = detachedScriptFiles({ chosen, script, log, timeout, hostnameCheckInLaunch: true });
 await startDetachedScript({ chosen, step, launch, log });
 ```
 */

import { writeFile, } from 'node:fs/promises';
import {
  basename,
  join,
} from 'node:path';

import { wait, } from '@monochromatic-dev/async-time-core/ts';

import {
  guestCommandTimeoutSeconds,
  hostCommandTimeoutMilliseconds,
  pollIntervalMilliseconds,
} from './guest-host-bounds.ts';
import type { GuestRun, } from './guest-run.ts';
import { host, } from './host-side-commands.ts';
import { readLog, } from './log-markers.ts';
import { runIgnoring, } from './logged-command.ts';
import { shellQuote, } from './shell-quote.ts';

/**
 The guest folder every detached script and its log live in, which `mvm push`
 writes and the host can read for a guest that has no agent answer.
 */
export const sharedFolder = '/mnt/shared';

/**
 The two files one detached guest script runs from.
 */
export type DetachedScriptFiles = {
  /**
   The bounded script itself, which runs detached inside the guest.
   */
  readonly step: string;
  /**
   The launcher the host starts, which returns as soon as the script is detached.
   */
  readonly launch: string;
};

/**
 Run one guest script to its end with its whole output in its own log.

 @param chosen - the run's guest and log folder

 @param script - the shell script to run inside the guest

 @param log - the log file's name in the log folder

 @param timeout - milliseconds the script may take inside the guest

 @returns what the guest wrote, read back from the log

 @throws Error naming the guest's status

 @example
 ```ts
 const written = await guest({ chosen, script: 'id', log: 'guest-id.log', timeout: 900_000 });
 ```
 */
export async function guest({
  chosen,
  script,
  log,
  timeout,
}: {
  readonly chosen: GuestRun;
  readonly script: string;
  readonly log: string;
  readonly timeout: number;
},): Promise<string> {
  /**
   The one command the guest's shell reads: this guest, and the script under the
   guest's own `timeout`.
   */
  const command = `test "$(hostname)" = ${chosen.name} && timeout ${String(guestCommandTimeoutSeconds({ timeoutMilliseconds: timeout, },))} sh -ec ${shellQuote(script,)}`;
  await host({
    chosen,
    command: 'mvm',
    args: [
      '--backend',
      'libvirt',
      'exec',
      chosen.name,
      '--',
      command,
    ],
    log,
  },);
  return readLog({ path: join(
    chosen.logDir,
    log,
  ), },);
}

/**
 The step and launcher files one detached guest script runs from.

 The launcher is what the host starts, and the step is what it runs detached:
 both are pushed into the guest's shared folder under their own names, which is
 where the launcher looks for the step.

 @param chosen - the run's guest and log folder

 @param script - the shell script to run inside the guest

 @param log - the log's name inside the shared folder

 @param timeout - milliseconds the script may take inside the guest

 @param hostnameCheckInLaunch - whether the launcher refuses a guest that is not
   this one, rather than the step naming the check

 @returns the two files' whole text

 @example
 ```ts
 const files = detachedScriptFiles({ chosen, script: 'id', log: 'guest-id.log', timeout: 900_000, hostnameCheckInLaunch: true });
 ```
 */
export function detachedScriptFiles({
  chosen,
  script,
  log,
  timeout,
  hostnameCheckInLaunch,
}: {
  readonly chosen: GuestRun;
  readonly script: string;
  readonly log: string;
  readonly timeout: number;
  readonly hostnameCheckInLaunch: boolean;
},): DetachedScriptFiles {
  /**
   The check that the command reached the guest it names, in whichever file
   carries it.
   */
  const hostnameCheck = `test "$(hostname)" = ${chosen.name}`;
  /**
   How the detached script puts its own status into its log, which is what the
   host waits for.
   */
  const doneLine = `echo "DONE=$status" >> ${sharedFolder}/${log}`;
  /**
   The bounded script, which is what runs detached.
   */
  const stepLines = [
    ...(hostnameCheckInLaunch ? [] : [hostnameCheck,]),
    `timeout ${String(guestCommandTimeoutSeconds({ timeoutMilliseconds: timeout, },))} sh -ec ${shellQuote(script,)} > ${sharedFolder}/${log} 2>&1`,
    'status=$?',
    doneLine,
    '',
  ];
  /**
   The launcher, which is what the host starts and which returns at once.
   */
  const launchLines = [
    ...(hostnameCheckInLaunch ? [`${hostnameCheck} || exit 1`,] : []),
    `setsid nohup sh ${sharedFolder}/${log}.step.sh < /dev/null > /dev/null 2>&1 &`,
    'exit 0',
    '',
  ];
  return {
    step: stepLines.join('\n',),
    launch: launchLines.join('\n',),
  };
}

/**
 Start one detached guest script, which cannot lose its answer.

 @param chosen - the run's guest and log folder

 @param step - the step file's whole text

 @param launch - the launcher file's whole text

 @param log - the log's name inside the shared folder, which names both files

 @throws Error naming the command that refused

 @example
 ```ts
 await startDetachedScript({ chosen, step, launch, log: 'guest-crate-build.log' });
 ```
 */
export async function startDetachedScript({
  chosen,
  step,
  launch,
  log,
}: {
  readonly chosen: GuestRun;
  readonly step: string;
  readonly launch: string;
  readonly log: string;
},): Promise<void> {
  /**
   The step file inside the run's log folder, pushed into the guest's shared one.
   */
  const stepPath = join(
    chosen.logDir,
    `${log}.step.sh`,
  );
  /**
   The launcher file inside the run's log folder.
   */
  const launchPath = join(
    chosen.logDir,
    `${log}.launch.sh`,
  );
  await writeFile(
    stepPath,
    step,
    { mode: 0o600, },
  );
  await writeFile(
    launchPath,
    launch,
    { mode: 0o600, },
  );
  await pushFileToGuest({
    chosen,
    from: stepPath,
    log,
  },);
  await pushFileToGuest({
    chosen,
    from: launchPath,
    log,
  },);
  await runIgnoring({
    subject: 'mvm exec',
    command: 'mvm',
    args: [
      '--backend',
      'libvirt',
      'exec',
      chosen.name,
      '--',
      'sh',
      `${sharedFolder}/${log}.launch.sh`,
    ],
    timeout: hostCommandTimeoutMilliseconds,
  },);
}

/**
 Wait for one detached script's own `DONE=` line.

 @param answer - the last line of the script's log, as the guest or the host
   folder gives it; a failure to read counts as no line yet

 @param timeout - milliseconds the host waits before giving up

 @returns whether the script ended inside the bound

 @example
 ```ts
 const finished = await waitForDoneLine({ answer, timeout: 5_400_000 });
 ```
 */
export function waitForDoneLine({
  answer,
  timeout,
}: {
  readonly answer: () => Promise<string>;
  readonly timeout: number;
},): Promise<boolean> {
  /**
   When the host stops waiting, which is the bound a build that hangs meets.
   */
  const deadline = Date.now() + timeout;
  return pollForTheDoneLine({
    answer,
    deadline,
  },);
}

/**
 Ask for a detached script's last line again until it says it ended or the bound
 passes.

 The wait is one recursion rather than a loop because each look is awaited in
 turn: nothing about one look can run beside another, and a loop that awaits is
 a promise chain this package reads as one step nobody can follow.

 @param answer - the last line of the script's log

 @param deadline - when the host stops asking, in milliseconds since the epoch

 @returns whether the script ended inside the bound

 @example
 ```ts
 await pollForTheDoneLine({ answer, deadline });
 ```
 */
async function pollForTheDoneLine({
  answer,
  deadline,
}: {
  readonly answer: () => Promise<string>;
  readonly deadline: number;
},): Promise<boolean> {
  if (Date.now() >= deadline) return false;
  if ((await answer()).startsWith('DONE=')) return true;
  await wait(pollIntervalMilliseconds,);
  return pollForTheDoneLine({
    answer,
    deadline,
  },);
}

/**
 Push one host file into a guest's shared folder under its own name, which is
 where a launched script looks for it.

 @param chosen - the run's guest and log folder

 @param from - the file on the host

 @param log - the detached script's log name, which names this push's own log

 @throws Error naming the command that refused

 @example
 ```ts
 await pushFileToGuest({ chosen, from: join(logDir, 'guest-build.log.step.sh'), log: 'guest-build.log' });
 ```
 */
async function pushFileToGuest({
  chosen,
  from,
  log,
}: {
  readonly chosen: GuestRun;
  readonly from: string;
  readonly log: string;
},): Promise<void> {
  await host({
    chosen,
    command: 'mvm',
    args: [
      '--backend',
      'libvirt',
      'push',
      chosen.name,
      from,
      `/shared/${basename(from,)}`,
    ],
    log: `guest-push-${log}.log`,
  },);
}
