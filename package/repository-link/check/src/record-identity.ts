/**
 The identity writer, the cut-over step that records what the check enforces: the link path, the exact link text, the
 approved canonical root, the filesystem UUIDs and the phase, plus the repository marker `.git/monochromatic-root-commit`.
 The check reads the file and never writes it; only this module does.

 @module
 */

import { execFile, } from 'node:child_process';
import {
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { promisify, } from 'node:util';
import { checkRepositoryLink, } from './check-link.ts';
import { ROOT_COMMIT_MARKER, } from './constants.ts';
import {
  readIdentity,
  writeIdentity,
} from './identity-file.ts';
import type {
  LinkAnswer,
  LinkIdentity,
  LinkPhase,
  Probe,
} from './link-types.ts';

/**
 Runs a program and waits for it to end.
 */
const execFileAsync = promisify(execFile,);

/**
 Git's own program, never the repository's wrapper.
 */
const GIT = '/usr/bin/git';

/**
 What recording refused or finished with.
 */
export class RecordIdentityError extends Error {
  /**
   Creates the error.

   @param message - what was refused
   @param options - the error options, for a cause
   */
  public constructor(
    message: string,
    options?: ErrorOptions,
  ) {
    super(
      message,
      options,
    );
    this.name = 'RecordIdentityError';
  }
}

/**
 Reads the first commit hash of `main` without the repository's wrapper.

 @param root - the repository root
 
 @returns the hash
 
 @throws RecordIdentityError when Git cannot name it
 */
async function firstCommitOfMain(root: string,): Promise<string> {
  try {
    /**
     What the call returned.
     */
    const result = await execFileAsync(
      GIT,
      [
        '--no-optional-locks',
        '-C',
        root,
        'rev-list',
        '--max-parents=0',
        'main',
      ],
      {
        encoding: 'utf8',
      },
    );
    /**
     The hash of this step.
     */
    const hash = result.stdout
      .split('\n',)[0]
      ?.trim()
      ?? '';
    if (hash === '') {
      throw new RecordIdentityError(`git named no first commit of main in ${root}`,);
    }
    return hash;
  }
  catch (error) {
    if (error instanceof RecordIdentityError) {
      throw error;
    }
    throw new RecordIdentityError(
      `cannot read the first commit of main in ${root}`,
      { cause: error, },
    );
  }
}

/**
 Writes the marker file into `.git`, or confirms the one that is there.

 @param root - the repository root
 
 @param rootCommit - the hash to record
 
 @throws RecordIdentityError when a different marker is already there
 */
async function writeMarker(
  {
    root,
    rootCommit,
  }: {
    readonly root: string;
    readonly rootCommit: string;
  },
): Promise<void> {
  /**
   The path this step works on.
   */
  const path = join(
    root,
    '.git',
    ROOT_COMMIT_MARKER,
  );
  try {
    /**
     The ledger entry of this subvolume, when it has one.
     */
    const existing = (await readFile(
      path,
      'utf8',
    )).trim();
    if (existing !== rootCommit) {
      throw new RecordIdentityError(`${path} already holds ${existing}, not ${rootCommit}; nothing was changed`,);
    }
  }
  catch (error) {
    if (error instanceof RecordIdentityError) {
      throw error;
    }
    if ((error as NodeJS.ErrnoException).code !== 'ENOENT') {
      throw error;
    }
    await writeFile(
      path,
      `${rootCommit}\n`,
    );
  }
}

/**
 Builds the identity of phase `before`: the repository is a real folder on the NVMe.

 @param linkPath - the repository path
 
 @param mountPoint - the 860's mount point
 
 @param probe - the probe
 
 @returns the identity
 
 @throws RecordIdentityError when the path is not a real folder
 */
async function identityBefore(
  {
    linkPath,
    mountPoint,
    probe,
  }: {
    readonly linkPath: string;
    readonly mountPoint: string;
    readonly probe: Probe;
  },
): Promise<LinkIdentity> {
  if ((await probe.lstat(linkPath,)).kind !== 'directory') {
    throw new RecordIdentityError(`${linkPath} is not a real folder, so phase before cannot be recorded`,);
  }
  return {
    linkPath,
    linkText: '',
    canonicalRoot: await probe.realpath(linkPath,),
    mountPoint,
    driveUuid: await probe.isMountPoint(mountPoint,) ? await probe.uuidOf(mountPoint,) : '',
    nvmeUuid: await probe.uuidOf(linkPath,),
    rootCommit: '',
    phase: 'before',
  };
}

/**
 Builds the identity of phase `after`, writing the marker into the repository on the 860.

 @param linkPath - the repository path, now a link
 
 @param mountPoint - the 860's mount point
 
 @param probe - the probe
 
 @param previous - the identity recorded before, whose NVMe UUID is kept
 
 @returns the identity
 
 @throws RecordIdentityError when the path is not a link to a folder on a mounted 860
 */
async function identityAfter(
  {
    linkPath,
    mountPoint,
    probe,
    previous,
  }: {
    readonly linkPath: string;
    readonly mountPoint: string;
    readonly probe: Probe;
    readonly previous: LinkIdentity;
  },
): Promise<LinkIdentity> {
  if ((await probe.lstat(linkPath,)).kind !== 'link') {
    throw new RecordIdentityError(`${linkPath} is not a symbolic link, so phase after cannot be recorded`,);
  }
  /**
   The root of this step.
   */
  const root = await probe.realpath(linkPath,);
  if ((root === '') || (!await probe.isMountPoint(mountPoint,))) {
    throw new RecordIdentityError(`${linkPath} does not resolve while ${mountPoint} is mounted; nothing was recorded`,);
  }
  /**
   The root commit of this step.
   */
  const rootCommit = await firstCommitOfMain(root,);
  await writeMarker({
    root,
    rootCommit,
  },);
  return {
    linkPath,
    linkText: await probe.readlink(linkPath,),
    canonicalRoot: root,
    mountPoint,
    driveUuid: await probe.uuidOf(root,),
    nvmeUuid: previous.nvmeUuid,
    rootCommit,
    phase: 'after',
  };
}

/**
 Records the identity of a phase, then runs the check against it.

 @param phase - `before` or `after`
 
 @param identityFile - where to write the identity
 
 @param linkPath - the repository path
 
 @param mountPoint - the 860's mount point
 
 @param probe - the probe
 
 @returns the identity written and the check's answer for it
 
 @throws RecordIdentityError when the current state does not fit the phase
 */
export async function recordIdentity(
  {
    phase,
    identityFile,
    linkPath,
    mountPoint,
    probe,
  }: {
    readonly phase: LinkPhase;
    readonly identityFile: string;
    readonly linkPath: string;
    readonly mountPoint: string;
    readonly probe: Probe;
  },
): Promise<{
  readonly identity: LinkIdentity;
  readonly answer: LinkAnswer
}> {
  /**
   The previous of this step.
   */
  const previous = await readIdentity(identityFile,);
  /**
   The identity file of the repository link check.
   */
  const identity = phase === 'before'
    ? await identityBefore({
      linkPath,
      mountPoint,
      probe,
    },)
    : await identityAfter({
      linkPath,
      mountPoint,
      probe,
      previous: previous.identity,
    },);
  await writeIdentity({
    path: identityFile,
    identity,
  },);
  /**
   The answer of this step.
   */
  const answer = await checkRepositoryLink({
    read: {
      found: true,
      identity,
    },
    probe,
  },);
  return {
    identity,
    answer,
  };
}
