/**
 One detached guest script whose end the host reads through the guest agent, by
 asking the guest for the last line of the script's log, and whose file it then
 pulls with `mvm pull`.

 The adoption guest's build and its arms run for minutes, and the guest agent has
 lost the status of a long `exec`, so the work runs detached and ends by
 appending `DONE=<status>` to its own log. Asking the guest for that line is what
 this driver does rather than reading the shared folder from the host, and the
 pull afterwards is `mvm`'s own, so the log a person reads is the one `mvm` copied
 out.

 @example
 ```ts
 const pulled = await detachedStep({ chosen, script: 'id', log: 'guest-id.log', timeout: 900_000, finishTimeout: 1_020_000 });
 ```
 */

import {
  type execFile,
  execFile as execFileProgram,
} from 'node:child_process';
import { join, } from 'node:path';
import { promisify, } from 'node:util';

import {
  startDetachedScript,
  waitForDoneLine,
} from '../guest-host/guest-side-scripts.ts';
import type { GuestRun, } from '../guest-host/guest-run.ts';
import { hostCommandTimeoutMilliseconds, } from '../guest-host/guest-host-bounds.ts';
import { host, } from '../guest-host/host-side-commands.ts';
import {
  detachedScriptFiles,
  doneStatusOf,
  readLog,
} from '../guest-host/guest-host-rules-fixture.ts';

/**
 `execFile` with a promise result, typed through Node's own adapter contract,
 because the generic `promisify(execFile)` form trips `strict-void-return`
 (`doc/troubleshooting/oxlint-promisify-void-return.md`).
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 Runs a program without a shell and waits for it to end or for its deadline.
 */
const execFileAsync: typeof execFile.__promisify__ = promisifyExecFile(execFileProgram,);

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
   The host's copy of the log, which the pull below writes.
   */
  const pulledPath = join(
    chosen.logDir,
    log,
  );
  /**
   Whether the script ended inside the host's bound, which the guest's own last
   line answers.
   */
  const finished = await waitForDoneLine({
    answer: function answer(): Promise<string> {
      return lastLineInGuest({
        chosen,
        log,
      },);
    },
    timeout: finishTimeout,
  },);
  await host({
    chosen,
    command: 'mvm',
    args: [
      '--backend',
      'libvirt',
      'pull',
      chosen.name,
      `/mnt/shared/${log}`,
      pulledPath,
    ],
    log: `host-pull-${log}`,
  },);
  /**
   The log as `mvm` copied it out.
   */
  const pulled = await readLog({ path: pulledPath, },);
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

/**
 The last line of one guest script's log, asked of the guest itself.

 A guest that cannot answer, because its agent is busy or the file is not there
 yet, is no line yet: the caller is waiting for a `DONE=` line that has not
 arrived, and refusing here would fail a run whose script is still working.

 @param chosen - the run's guest and log folder

 @param log - the log's name inside the guest's shared folder

 @returns the log's last line as the guest prints it, or empty text

 @example
 ```ts
 await lastLineInGuest({ chosen, log: 'guest-crate-build.log' });
 ```
 */
async function lastLineInGuest({
  chosen,
  log,
}: {
  readonly chosen: GuestRun;
  readonly log: string;
},): Promise<string> {
  try {
    /**
     What the guest printed for the log's last line.
     */
    const answer = await execFileAsync(
      'mvm',
      [
        '--backend',
        'libvirt',
        'exec',
        chosen.name,
        '--',
        'tail',
        '--lines=1',
        `/mnt/shared/${log}`,
      ],
      {
        encoding: 'utf8',
        timeout: hostCommandTimeoutMilliseconds,
      },
    );
    /**
     The line with the answer's own journal noise trimmed away.
     */
    const line = ((typeof answer.stdout) === 'string') ? answer.stdout : '';
    return line.trim();
  }
  catch (caught) {
    // A guest that cannot answer yet is no `DONE=` line yet, so the caught
    // value is not read here; the wait's own bound names the log it gave up on.
    return '';
  }
}
