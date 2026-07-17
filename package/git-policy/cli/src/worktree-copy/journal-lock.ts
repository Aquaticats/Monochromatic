import { randomUUID, } from 'node:crypto';
import { constants, } from 'node:fs';
import {
  chmod,
  mkdir,
  open,
  readFile,
  rename,
  rm,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { wait, } from '@monochromatic-dev/module-async-time/ts';

import {
  PROCESS_IDENTITY_ABSENT,
  resolveProcessBirthIdentity,
} from '../policy-engine/commit-transaction-process-identity.ts';
import { WorktreeCopyError, } from './errors.ts';
import { ensureWorktreeCopyJournalRoot, } from './journal.ts';

/**
 * Delay between bounded worktree-copy lock acquisition attempts.
 */
const LOCK_RETRY_DELAY_MS = 10;

/**
 * Bounded worktree-copy lock acquisition attempts.
 */
const LOCK_RETRY_ATTEMPTS = 100;

/**
 * Inherited environment capability for reentrant same-repository Git calls.
 */
export const WORKTREE_COPY_LEASE_ENV = 'CLI_GIT_WORKTREE_COPY_LEASE';

/**
 * Private lock directory mode.
 */
const PRIVATE_DIRECTORY_MODE = 0o700;

/**
 * Private lock owner file mode.
 */
const PRIVATE_FILE_MODE = 0o600;

/**
 * Another live process owns worktree-copy settlement.
 */
const LOCK_BUSY: unique symbol = Symbol('worktree-copy lock belongs to live process',);

/**
 * Durable lock owner identity.
 */
type LockOwner = Readonly<{
  /**
   * Unguessable capability inherited only by descendants of real Git.
   */
  leaseToken: string;
  /**
   * Operating-system process identifier.
   */
  ownerPid: number;
  /**
   * Process birth identity preventing PID reuse.
   */
  ownerBirthIdentity: string;
  /**
   * Lock schema version.
   */
  schemaVersion: 1;
}>;

/**
 * Ownership-checking lock and descendant reentrancy capability.
 */
export type WorktreeCopyLease = AsyncDisposable & Readonly<{
  /**
   * Capability passed only to real-Git descendants.
   */
  leaseToken: string;
}>;

/**
 * Reads and validates private lock owner metadata.
 *
 * @param ownerPath - exact private owner file
 *
 * @returns validated owner identity
 *
 * @example
 * ```ts
 * await readLockOwner('/repo/.git/cli-git-worktree-copy/v1/settlement.lock/owner.json');
 * ```
 */
async function readLockOwner(ownerPath: string,): Promise<LockOwner> {
  /**
   * Parsed owner metadata behind JSON boundary.
   */
  const value: unknown = JSON.parse(await readFile(
    ownerPath,
    'utf8',
  ),);
  if (((typeof value) !== 'object') || (value === null)
    || (!('schemaVersion' in value))
    || (value.schemaVersion !== 1)
    || (!('leaseToken' in value))
    || ((typeof value.leaseToken) !== 'string')
    || (value.leaseToken
      .length
      === 0)
    || (!('ownerPid' in value))
    || ((typeof value.ownerPid) !== 'number')
    || (!Number.isInteger(value.ownerPid,))
    || (value.ownerPid < 1)
    || (!('ownerBirthIdentity' in value))
    || ((typeof value.ownerBirthIdentity) !== 'string')
    || (value.ownerBirthIdentity
      .length
      === 0)) {
    throw new WorktreeCopyError(
      `cli-git: worktree-copy lock owner is corrupt: ${JSON.stringify(ownerPath,)}.`,
    );
  }
  return {
    leaseToken: value.leaseToken,
    ownerPid: value.ownerPid,
    ownerBirthIdentity: value.ownerBirthIdentity,
    schemaVersion: 1,
  };
}

/**
 * Reads published owner or reports concurrent lock replacement.
 *
 * @param ownerPath - expected published owner file
 *
 * @returns validated owner or busy sentinel
 *
 * @example
 * ```ts
 * await readPublishedOwner('/repo/.git/cli-git-worktree-copy/v1/settlement.lock/owner.json');
 * ```
 */
async function readPublishedOwner(
  ownerPath: string,
): Promise<LockOwner | typeof LOCK_BUSY> {
  try {
    return await readLockOwner(ownerPath,);
  }
  catch (error: unknown) {
    if (Error.isError(error,) && ('code' in error)
      && (error.code === 'ENOENT'))
      return LOCK_BUSY;
    throw error;
  }
}

/**
 * Writes durable owner metadata into unpublished candidate lock directory.
 *
 * @param candidateDirectory - unique candidate lock directory
 *
 * @param owner - current process identity
 *
 * @example
 * ```ts
 * await writeCandidateOwner({ candidateDirectory, owner });
 * ```
 */
async function writeCandidateOwner({
  candidateDirectory,
  owner,
}: Readonly<{
  candidateDirectory: string;
  owner: LockOwner;
}>,): Promise<void> {
  /**
   * Plain owner JSON detached from caller-owned record.
   */
  const ownerJson = JSON.stringify({
    leaseToken: owner.leaseToken,
    ownerBirthIdentity: owner.ownerBirthIdentity,
    ownerPid: owner.ownerPid,
    schemaVersion: owner.schemaVersion,
  },);
  try {
    await mkdir(
      candidateDirectory,
      { mode: PRIVATE_DIRECTORY_MODE, },
    );
    await chmod(
      candidateDirectory,
      PRIVATE_DIRECTORY_MODE,
    );
    {
      /**
       * Exclusive no-follow owner file handle.
       */
      await using handle = await open(
        join(
          candidateDirectory,
          'owner.json',
        ),
        constants.O_CREAT | constants.O_EXCL
          | constants.O_WRONLY
          | constants.O_NOFOLLOW,
        PRIVATE_FILE_MODE,
      );
      await handle.writeFile(
        `${ownerJson}\n`,
        'utf8',
      );
      await handle.sync();
    }
  }
  catch (error: unknown) {
    await rm(
      candidateDirectory,
      {
        recursive: true,
        force: true,
      },
    );
    throw error;
  }
}

/**
 * Reports whether rename failed because published lock already exists.
 *
 * @param error - unknown rename failure
 *
 * @returns whether another lock candidate won publication
 *
 * @example
 * ```ts
 * isExistingLockError(error);
 * ```
 */
function isExistingLockError(error: unknown,): boolean {
  return Error.isError(error,) && ('code' in error)
    && ((error.code === 'EEXIST') || (error.code === 'ENOTEMPTY'));
}

/**
 * Removes stale published lock through ownership-transfer rename.
 *
 * @param lockDirectory - published lock directory
 *
 * @returns busy sentinel so caller retries acquisition
 *
 * @example
 * ```ts
 * await retireStaleLock('/repo/.git/cli-git-worktree-copy/v1/settlement.lock');
 * ```
 */
async function retireStaleLock(
  lockDirectory: string,
): Promise<typeof LOCK_BUSY> {
  /**
   * Unique stale lock path owned only after successful rename.
   */
  const staleDirectory = `${lockDirectory}.${randomUUID()}.stale`;
  try {
    await rename(
      lockDirectory,
      staleDirectory,
    );
  }
  catch (error: unknown) {
    if (isExistingLockError(error,)
      || (Error.isError(error,) && ('code' in error)
        && (error.code === 'ENOENT'))) {
      return LOCK_BUSY;
    }
    throw error;
  }
  await rm(
    staleDirectory,
    {
      recursive: true,
      force: true,
    },
  );
  return LOCK_BUSY;
}

/**
 * Creates ownership-checking disposable for published lock.
 *
 * @param lockDirectory - exact published lock directory
 *
 * @param owner - identity published in lock
 *
 * @returns disposable exclusive lock
 *
 * @example
 * ```ts
 * await ownedLock({ lockDirectory, owner });
 * ```
 */
function ownedLock({
  lockDirectory,
  owner,
}: Readonly<{
  lockDirectory: string;
  owner: LockOwner;
}>,): WorktreeCopyLease {
  return {
    leaseToken: owner.leaseToken,
    async [Symbol.asyncDispose](): Promise<void> {
      /**
       * Live owner immediately before lock removal.
       */
      const liveOwner = await readLockOwner(join(
        lockDirectory,
        'owner.json',
      ),);
      if ((liveOwner.leaseToken !== owner.leaseToken)
        || (liveOwner.ownerPid !== owner.ownerPid)
        || (liveOwner.ownerBirthIdentity !== owner.ownerBirthIdentity)) {
        throw new WorktreeCopyError(
          `cli-git: worktree-copy lock ownership changed: ${JSON.stringify(lockDirectory,)}.`,
        );
      }
      await rm(
        lockDirectory,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 * Attempts one atomic lock publication or stale-owner recovery.
 *
 * @param lockDirectory - exact published lock directory
 *
 * @param owner - current process identity
 *
 * @returns disposable lock or busy sentinel
 *
 * @example
 * ```ts
 * await attemptAcquire({ lockDirectory, owner });
 * ```
 */
async function attemptAcquire({
  lockDirectory,
  owner,
}: Readonly<{
  lockDirectory: string;
  owner: LockOwner;
}>,): Promise<WorktreeCopyLease | typeof LOCK_BUSY> {
  /**
   * Unpublished complete lock candidate.
   */
  const candidateDirectory = `${lockDirectory}.${randomUUID()}.pending`;
  await writeCandidateOwner({
    candidateDirectory,
    owner,
  },);
  try {
    await rename(
      candidateDirectory,
      lockDirectory,
    );
    return ownedLock({
      lockDirectory,
      owner,
    },);
  }
  catch (error: unknown) {
    await rm(
      candidateDirectory,
      {
        recursive: true,
        force: true,
      },
    );
    if (!isExistingLockError(error,))
      throw error;
  }
  /**
   * Identity currently published by competing or stale owner.
   */
  const publishedOwner = await readPublishedOwner(join(
    lockDirectory,
    'owner.json',
  ),);
  if (publishedOwner === LOCK_BUSY)
    return LOCK_BUSY;
  /**
   * Current birth identity for published PID.
   */
  const publishedBirthIdentity = await resolveProcessBirthIdentity(publishedOwner.ownerPid,);
  if ((publishedBirthIdentity !== PROCESS_IDENTITY_ABSENT)
    && (publishedBirthIdentity === publishedOwner.ownerBirthIdentity)) {
    return LOCK_BUSY;
  }
  return retireStaleLock(lockDirectory,);
}

/**
 * Validates inherited reentrancy capability against live repository lock.
 *
 * @param commonDir - canonical common Git directory
 *
 * @param leaseToken - environment capability inherited from parent real Git
 *
 * @returns whether current invocation belongs to active outer settlement
 *
 * @example
 * ```ts
 * await validatesInheritedWorktreeCopyLease({ commonDir: '/repo/.git', leaseToken: process.env.CLI_GIT_WORKTREE_COPY_LEASE });
 * ```
 */
export async function validatesInheritedWorktreeCopyLease({
  commonDir,
  leaseToken,
}: Readonly<{
  commonDir: string;
  leaseToken: string;
}>,): Promise<boolean> {
  if (leaseToken.length === 0)
    return false;
  /**
   * Validated private root expected to contain active outer lock.
   */
  const root = await ensureWorktreeCopyJournalRoot(commonDir,);
  /**
   * Published outer owner or replacement-race sentinel.
   */
  const owner = await readPublishedOwner(join(
    root,
    'settlement.lock',
    'owner.json',
  ),);
  if ((owner === LOCK_BUSY) || (owner.leaseToken !== leaseToken))
    return false;
  /**
   * Current birth identity proving owner PID still names original process.
   */
  const liveBirthIdentity = await resolveProcessBirthIdentity(owner.ownerPid,);
  return (liveBirthIdentity !== PROCESS_IDENTITY_ABSENT)
    && (liveBirthIdentity === owner.ownerBirthIdentity);
}

/**
 * Acquires recoverable exclusive worktree-copy settlement lock.
 *
 * @param commonDir - canonical common Git directory
 *
 * @returns ownership-checking disposable lock
 *
 * @throws {@link WorktreeCopyError} when live owner does not settle in bounded attempts
 *
 * @example
 * ```ts
 * await using lock = await acquireWorktreeCopyLock('/repo/.git');
 * ```
 */
export async function acquireWorktreeCopyLock(
  commonDir: string,
): Promise<WorktreeCopyLease> {
  /**
   * Private journal and lock root.
   */
  const root = await ensureWorktreeCopyJournalRoot(commonDir,);
  /**
   * Current process birth identity.
   */
  const ownerBirthIdentity = await resolveProcessBirthIdentity(process.pid,);
  if (ownerBirthIdentity === PROCESS_IDENTITY_ABSENT) {
    throw new WorktreeCopyError('cli-git: current worktree-copy lock owner identity is unavailable.',);
  }
  /**
   * Complete current lock owner.
   */
  const owner: LockOwner = {
    leaseToken: randomUUID(),
    ownerPid: process.pid,
    ownerBirthIdentity,
    schemaVersion: 1,
  };
  /**
   * Exact repository-wide worktree-copy settlement lock.
   */
  const lockDirectory = join(
    root,
    'settlement.lock',
  );
  for (const _attempt of Array.from({ length: LOCK_RETRY_ATTEMPTS, },)) {
    /* oxlint-disable no-await-in-loop -- lock attempts must remain ordered and bounded */
    /**
     * Current bounded lock-publication result.
     */
    const result = await attemptAcquire({

      lockDirectory,
      owner,
    },);
    /* oxlint-enable no-await-in-loop */
    if (result !== LOCK_BUSY)
      return result;
    // oxlint-disable-next-line no-await-in-loop -- retry delay prevents active-owner spin
    await wait(LOCK_RETRY_DELAY_MS,);
  }
  throw new WorktreeCopyError(
    `cli-git: timed out waiting for active worktree-copy settlement under ${JSON.stringify(commonDir,)}.`,
  );
}
