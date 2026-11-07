/**
 `openRepositoryRoot()` for programs this repository controls: it runs the full check once, opens the resolved root as a
 directory descriptor, and returns it, so later operations use descriptor-relative calls or `/proc/self/fd/<n>/...`
 and never look the link up again. A held descriptor also makes an ordinary `umount` of the 860 refuse.

 @module
 */

import {
  type FileHandle,
  open,
} from 'node:fs/promises';
import { constants, } from 'node:fs';
import { checkRepositoryLink, } from './check-link.ts';
import {
  identityPathFor,
  readIdentity,
} from './identity-file.ts';
import type {
  LinkAnswer,
  Probe,
} from './link-types.ts';
import { realProbe, } from './real-probe.ts';

/**
 The refusal an `openRepositoryRoot` caller gets when the check is not ready.
 */
export class RepositoryLinkNotReadyError extends Error {
  /**
   The check's answer that caused the refusal.
   */
  public readonly answer: LinkAnswer;

  /**
   Creates the refusal.

   @param answer - the check's answer
   */
  public constructor(answer: LinkAnswer,) {
    super(answer.message,);
    this.name = 'RepositoryLinkNotReadyError';
    this.answer = answer;
  }
}

/**
 The opened repository root.
 */
export type OpenedRepositoryRoot = {
  /**
   The directory handle; close it when done.
   */
  readonly handle: FileHandle;

  /**
   A path that names the root through the held descriptor, valid while the handle is open.
   */
  readonly descriptorPath: string;

  /**
   The resolved root at the time of the check.
   */
  readonly root: string;
};

/**
 Checks the link, then opens the resolved root as a directory descriptor.

 @param identityFile - the identity file; the default is the one under the home folder
 
 @param home - the home folder, for a disposable home in tests
 
 @param probe - the probe; the default reads the running system
 
 @returns the opened root
 
 @throws RepositoryLinkNotReadyError when the check is not ready, carrying its plain message

 @example
 ```ts
 import { openRepositoryRoot, } from '\@monochromatic-dev/repository-link-check';
 
 const result = await openRepositoryRoot({ identityFile, home, probe, },);
 ```
 */
export async function openRepositoryRoot(
  {
    identityFile = '',
    home = '',
    probe = realProbe,
  }: {
    readonly identityFile?: string;
    readonly home?: string;
    readonly probe?: Probe;
  } = {},
): Promise<OpenedRepositoryRoot> {
  /**
   The value read.
   */
  const read = await readIdentity(identityFile === '' ? identityPathFor(home === '' ? (process.env
    .HOME
    ?? '') : home,) : identityFile,);
  /**
   The answer of this step.
   */
  const answer = await checkRepositoryLink({
    read,
    probe,
  },);
  if (!answer.ready) {
    throw new RepositoryLinkNotReadyError(answer,);
  }
  /**
   The root of this step.
   */
  const root = await probe.realpath(read.identity
    .linkPath,);
  /**
   The handle of this step.
   */
  const handle = await open(
    root,
    constants.O_RDONLY | constants.O_DIRECTORY,
  );
  return {
    handle,
    descriptorPath: `/proc/self/fd/${String(handle.fd,)}`,
    root,
  };
}
