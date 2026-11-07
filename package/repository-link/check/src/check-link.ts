/**
 The shared check: is the repository link usable right now? Phase `after` checks the link to the 860; phase `before`
 checks the real folder on the NVMe so the tools can be installed first. Every call is a few `lstat` and `stat`
 calls and a read of mountinfo; none creates anything.

 @module
 */

import { join, } from 'node:path';
import { setTimeout as wait, } from 'node:timers/promises';
import {
  DEFAULT_DEADLINE_SECONDS,
  MILLISECONDS_PER_SECOND,
  ROOT_COMMIT_MARKER,
} from './constants.ts';
import type {
  IdentityRead,
} from './identity-file.ts';
import type {
  LinkAnswer,
  LinkIdentity,
  Probe,
} from './link-types.ts';
import { errorNameOf, } from './error-code-of.ts';

/**
 Builds an answer that is not ready.

 @param kind - the answer kind
 
 @param message - the plain message
 
 @returns the answer
 */
function refusal(
  {
    kind,
    message,
  }: {
    readonly kind: LinkAnswer['kind'];
    readonly message: string;
  },
): LinkAnswer {
  return {
    kind,
    ready: false,
    message,
  };
}

/**
 Checks phase `before`: a real folder on the NVMe with a `.git` folder.

 @param identity - the recorded identity
 
 @param probe - the probe
 
 @returns the answer
 */
async function checkBefore(
  {
    identity,
    probe,
  }: {
    readonly identity: LinkIdentity;
    readonly probe: Probe;
  },
): Promise<LinkAnswer> {
  /**
   The entry of this step.
   */
  const entry = await probe.lstat(identity.linkPath,);
  if (entry.kind !== 'directory') {
    return refusal({
      kind: 'not-a-repository',
      message: `${identity.linkPath} is not a real folder (it is ${entry.kind}), but phase before expects the repository `
        + 'there; nothing was written',
    },);
  }
  /**
   The uuid of this step.
   */
  const uuid = await probe.uuidOf(identity.linkPath,);
  if ((uuid !== identity.nvmeUuid) || (identity.nvmeUuid === '')) {
    return refusal({
      kind: 'wrong-filesystem',
      message: `${identity.linkPath} is on filesystem ${uuid || 'unknown'}, the recorded NVMe is `
        + `${identity.nvmeUuid || 'unrecorded'}; nothing was written`,
    },);
  }
  if ((await probe.lstat(join(
    identity.linkPath,
    '.git',
  ),)).kind !== 'directory') {
    return refusal({
      kind: 'not-a-repository',
      message: `${identity.linkPath}/.git is not a folder; nothing was written`,
    },);
  }
  return {
    kind: 'ready-before',
    ready: true,
    message: `the repository at ${identity.linkPath} is ready (before the move)`,
  };
}

/**
 Explains why the link's target is not there: the drive is not mounted, or the folder is missing on a mounted drive.

 @param identity - the recorded identity
 
 @param probe - the probe
 
 @returns the answer
 */
async function explainMissingTarget(
  {
    identity,
    probe,
  }: {
    readonly identity: LinkIdentity;
    readonly probe: Probe;
  },
): Promise<LinkAnswer> {
  if (!await probe.isMountPoint(identity.mountPoint,)) {
    return refusal({
      kind: 'drive-not-mounted',
      message: `the 860 drive is not mounted yet (${identity.mountPoint} is empty), so the repository at `
        + `${identity.linkPath} is not there; unlock it (the mount-860 service does that at login) and run this again`,
    },);
  }
  return refusal({
    kind: 'mounted-folder-missing',
    message: `the 860 drive is mounted but ${identity.canonicalRoot} does not exist; nothing was written`,
  },);
}

/**
 Checks that the resolved root is on the recorded filesystem, is a repository, and carries the recorded marker.

 @param identity - the recorded identity
 
 @param probe - the probe
 
 @param root - the resolved root
 
 @returns the answer
 */
async function checkResolvedRoot(
  {
    identity,
    probe,
    root,
  }: {
    readonly identity: LinkIdentity;
    readonly probe: Probe;
    readonly root: string;
  },
): Promise<LinkAnswer> {
  /**
   The uuid of this step.
   */
  const uuid = await probe.uuidOf(root,);
  if ((uuid !== identity.driveUuid) || (identity.driveUuid === '')) {
    return refusal({
      kind: 'wrong-filesystem',
      message: `${identity.linkPath} resolves to ${root} on filesystem ${uuid || 'unknown'}, but the recorded 860 is `
        + `${identity.driveUuid || 'unrecorded'}; nothing was written`,
    },);
  }
  if (((await probe.lstat(root,)).kind !== 'directory') || ((await probe.lstat(join(
    root,
    '.git',
  ),)).kind !== 'directory')) {
    return refusal({
      kind: 'not-a-repository',
      message: `${root} is not a repository folder with a .git folder; nothing was written`,
    },);
  }
  /**
   The git uuid of this step.
   */
  const gitUuid = await probe.uuidOf(join(
    root,
    '.git',
  ),);
  if (gitUuid !== uuid) {
    return refusal({
      kind: 'wrong-filesystem',
      message: `${root}/.git is on filesystem ${gitUuid || 'unknown'} but ${root} is on ${uuid}; nothing was written`,
    },);
  }
  /**
   The marker this step searches for.
   */
  const marker = (await probe.readText(join(
    root,
    '.git',
    ROOT_COMMIT_MARKER,
  ),)).trim();
  if ((marker !== identity.rootCommit) || (identity.rootCommit === '')) {
    return refusal({
      kind: 'wrong-repository',
      message: `${root}/.git/${ROOT_COMMIT_MARKER} holds ${marker || 'nothing'}, the recorded root commit is `
        + `${identity.rootCommit || 'unrecorded'}; nothing was written`,
    },);
  }
  return {
    kind: 'ready-after',
    ready: true,
    message: `the repository at ${identity.linkPath} is ready (on the 860)`,
  };
}

/**
 Checks phase `after`: a link to the recorded root on the recorded filesystem.

 @param identity - the recorded identity
 
 @param probe - the probe
 
 @returns the answer
 */
async function checkAfter(
  {
    identity,
    probe,
  }: {
    readonly identity: LinkIdentity;
    readonly probe: Probe;
  },
): Promise<LinkAnswer> {
  /**
   The entry of this step.
   */
  const entry = await probe.lstat(identity.linkPath,);
  if (entry.kind === 'directory') {
    return refusal({
      kind: 'real-folder-replaced-link',
      message: `${identity.linkPath} is a real folder, not the link to the 860; nothing was written`,
    },);
  }
  if (entry.kind === 'missing') {
    return refusal({
      kind: 'link-missing',
      message: `${identity.linkPath} does not exist; the link to the 860 is gone; nothing was written`,
    },);
  }
  /**
   The text this step works on.
   */
  const text = entry.kind === 'link' ? await probe.readlink(identity.linkPath,) : '';
  if ((entry.kind !== 'link') || (text !== identity.linkText)) {
    return refusal({
      kind: 'wrong-repository',
      message: `${identity.linkPath} is ${entry.kind === 'link' ? `a link to ${text}` : `a ${entry.kind}`}, the recorded link `
        + `text is ${identity.linkText}; nothing was written`,
    },);
  }
  /**
   The root of this step.
   */
  const root = await probe.realpath(identity.linkPath,);
  if (root === '') {
    return await explainMissingTarget({
      identity,
      probe,
    },);
  }
  if (root !== identity.canonicalRoot) {
    return refusal({
      kind: 'wrong-repository',
      message: `${identity.linkPath} resolves to ${root}, the approved root is ${identity.canonicalRoot}; nothing was written`,
    },);
  }
  return await checkResolvedRoot({
    identity,
    probe,
    root,
  },);
}

/**
 Runs the check without a deadline.

 @param read - the identity file's content
 
 @param probe - the probe
 
 @returns the answer
 */
async function answerWithoutDeadline(
  {
    read,
    probe,
  }: {
    readonly read: IdentityRead;
    readonly probe: Probe;
  },
): Promise<LinkAnswer> {
  if (!read.found) {
    return refusal({
      kind: 'identity-missing',
      message: 'the repository link identity file is missing; the cut-over step writes it with repository-link-record',
    },);
  }
  return read.identity
    .phase
    === 'before'
    ? await checkBefore({
      identity: read.identity,
      probe,
    },)
    : await checkAfter({
      identity: read.identity,
      probe,
    },);
}

/**
 Waits for the deadline; an abort (the check answered first) ends the wait quietly.

 @param seconds - the deadline
 
 @param signal - aborted when the check has answered
 
 @param answer - what to give when the deadline passes
 
 @returns the answer, after the deadline or after the abort (when it no longer matters)
 */
async function waitForDeadline(
  {
    seconds,
    signal,
    answer,
  }: {
    readonly seconds: number;
    readonly signal: AbortSignal;
    readonly answer: LinkAnswer;
  },
): Promise<LinkAnswer> {
  try {
    return await wait(
      seconds * MILLISECONDS_PER_SECOND,
      answer,
      { signal, },
    );
  }
  catch (error) {
    if (errorNameOf(error,) === 'AbortError') {
      return answer;
    }
    throw error;
  }
}

/**
 Checks the repository link and gives up after a deadline: a worker stuck in a stalled `stat` is abandoned and the
 answer is that the drive did not answer.

 @param read - the identity file's content
 
 @param probe - the probe
 
 @param deadlineSeconds - how long to wait for the filesystem
 
 @returns the answer

 @example
 ```ts
 import { checkRepositoryLink, } from '\@monochromatic-dev/repository-link-check';
 
 const result = await checkRepositoryLink({ read, probe, deadlineSeconds, },);
 ```
 */
export async function checkRepositoryLink(
  {
    read,
    probe,
    deadlineSeconds = DEFAULT_DEADLINE_SECONDS,
  }: {
    readonly read: IdentityRead;
    readonly probe: Probe;
    readonly deadlineSeconds?: number;
  },
): Promise<LinkAnswer> {
  /**
   Cancels the deadline timer once the check has answered, so the process is not kept alive by it.
   */
  const controller = new AbortController();
  /**
   The answer given when the deadline passes first.
   */
  const timeoutAnswer = refusal({
    kind: 'drive-did-not-answer',
    message: `the drive did not answer within ${String(deadlineSeconds,)} s, so the state of ${read.identity
      .linkPath} `
      + 'is unknown; nothing was written',
  },);
  /**
   Whichever of the check and the deadline finishes first.
   */
  const answer = await Promise.race([
    answerWithoutDeadline({
      read,
      probe,
    },),
    waitForDeadline({
      seconds: deadlineSeconds,
      signal: controller.signal,
      answer: timeoutAnswer,
    },),
  ],);
  controller.abort();
  return answer;
}
