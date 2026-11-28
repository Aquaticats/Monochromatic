/**
 Read and pull the logs one detached step 22 script leaves in a guest's shared
 folder.

 This driver reads the guest's shared folder from the host's own file system
 rather than asking the guest agent, which is what the round that measured step
 22 needed: the shared folder is a host folder `mvm` mounts in the guest, so the
 host reads a finished script's log without an agent answer, and a guest whose
 agent is busy still hands its log over
 (`doc/troubleshooting/mvm-push-writes-the-shared-folder-not-the-guest-path.md`).

 @example
 ```ts
 const answer = await lastLineOfSharedFile({ name: chosen.name, file: armLog });
 ```
 */

import {
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { homedir, } from 'node:os';
import { join, } from 'node:path';

import {
  type GuestBackend,
  guestBackend,
} from '../../bin/mvm-guest-backend.ts';

/**
 Refuse a step that reads the host's own side of the shared folder on a backend that mounts none there.

 What this module reads is the folder `mvm` mounts into a libvirt guest, on the host's file system, so a finished script's
 log is read without an agent answer and a guest whose agent is busy still hands its log over
 (`doc/troubleshooting/mvm-push-writes-the-shared-folder-not-the-guest-path.md`). A backend that has to make the folder
 inside the guest keeps it in the guest: reading it from here would put an `ssh` round trip inside a poll that runs for
 the length of a long build, which nothing has measured, so this driver refuses there instead of waiting on a path that
 does not exist.

 @param backend - the backend this run drives

 @param step - the step the run named

 @throws Error naming the step, the backend and the path it cannot read

 @example
 ```ts
 refuseAHostSideRead({ backend: guestBackend(), step: 'run' }); // refuses on ovh
 ```
 */
export function refuseAHostSideRead({
  backend,
  step,
}: {
  readonly backend: GuestBackend;
  readonly step: string;
}): void {
  if (backend.makesTheSharedFolder === undefined)
    return;
  throw new Error(
    `step '${step}' reads the host's own side of the guest's shared folder, and backend '${backend.kind}' keeps that folder inside the guest: this driver waits by reading the folder's file from this host, which only a libvirt guest mounts`,
  );
}

/**
 The host folder a guest's shared folder is mounted from, which needs no guest
 agent to read.

 @param name - the guest's name

 @param file - the file inside the shared folder

 @returns the host path of that file

 @example
 ```ts
 sharedFile({ name: 'zfs-step-22-measurements', file: 'step-22-measurements-arm.log' });
 ```
 */
export function sharedFile({
  name,
  file,
}: {
  readonly name: string;
  readonly file: string;
},): string {
  return join(
    homedir(),
    '.local',
    'share',
    'mvm',
    'vms',
    name,
    'shared',
    file,
  );
}


/**
 Remove one guest log from the host's own side of the shared folder before a detached script starts.

 A script appends its status line to its log, so a log a previous attempt left still ends with that line, and the wait
 for the new script's end reads it as the new script's own end before the guest has even opened the file. Measured
 2026-11-28: the ninth `run` of this driver read the eighth attempt's `DONE=124` five seconds after starting its build,
 so a build that then ran for an hour was never waited for.

 @param name - the guest's name

 @param file - the file inside the shared folder

 @example
 ```ts
 await discardSharedLog({ name: 'zfs-step-22-measurements', file: 'step-22-measurements-arm.log' });
 ```
 */
export async function discardSharedLog({
  name,
  file,
}: {
  readonly name: string;
  readonly file: string;
},): Promise<void> {
  await rm(
    sharedFile({
      name,
      file,
    },),
    { force: true, },
  );
}

/**
 The last line of a file that has any text in it, or empty text.

 A log that ends with its own newline has an empty last element, which is not a
 line, so the answer is the last line with text in it. A file that is not there
 yet, or that the host cannot read while the guest writes it, is no answer
 either: the caller is waiting for a `DONE=` line that has not arrived.

 @param path - the file to read

 @returns its last line with text in it, or empty text

 @example
 ```ts
 await lastLineWithText({ path: sharedFile({ name: 'guest-one', file: 'arm.log' }) });
 ```
 */
export async function lastLineWithText({
  path,
}: {
  readonly path: string;
},): Promise<string> {
  try {
    /**
     The whole file, which a detached script appends one line at a time.
     */
    const text = await readFile(
      path,
      'utf8',
    );
    /**
     The lines that hold text, in the order the file holds them.
     */
    const lines = text
      .split('\n')
      .map(function trimmed(line,) {
        return line.trim();
      },)
      .filter(function hasText(line,) {
        return line.length > 0;
      },);
    return lines.at(-1,) ?? '';
  }
  catch (caught) {
    // A file the guest has not created yet and one it is still writing are the
    // same answer to a host waiting for a `DONE=` line: nothing yet, so the
    // caught value is not read here.
    return '';
  }
}
/**
 Pull one guest log into the run's own folder, and refuse a file that is not
 there rather than reading it as an empty log.

 @param name - the guest's name

 @param file - the file's name inside the guest's shared folder

 @param logDir - the run's log folder, where the copy lands

 @returns the log's whole text, as the copy holds it

 @throws Error when the shared folder does not hold the file

 @example
 ```ts
 const pulled = await pullSharedLog({ name: 'guest-one', file: 'arm.log', logDir: 'target/logs' });
 ```
 */
export async function pullSharedLog({
  name,
  file,
  logDir,
}: {
  readonly name: string;
  readonly file: string;
  readonly logDir: string;
},): Promise<string> {
  /**
   The file's own path inside the guest's shared folder.
   */
  const path = sharedFile({
    name,
    file,
  },);
  /**
   The log as the guest wrote it, which a run that never got that far does not
   have.
   */
  const pulled = await readIfThere({ path, },);
  await writeFile(
    join(
      logDir,
      file,
    ),
    pulled,
    { mode: 0o600, },
  );
  return pulled;
}

/**
 Read a file that may not be there yet.

 A detached guest script writes its log while the host waits, so a read can meet
 a file that has not been created or one the guest is still writing. Either is
 answered with the text read so far, and never with a refusal.

 @param path - the file to read

 @returns its whole text, or empty text when it is not there

 @example
 ```ts
 await readIfThere({ path: '/home/somebody/.local/share/mvm/vms/guest-one/shared/arm.log' });
 ```
 */
async function readIfThere({
  path,
}: {
  readonly path: string;
},): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (caught) {
    // A shared folder that holds no such file yet is the answer this waits
    // past, so the caught value is not read here.
    return '';
  }
}
