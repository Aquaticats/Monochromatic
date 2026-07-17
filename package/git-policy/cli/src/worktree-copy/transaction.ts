import { rm, } from 'node:fs/promises';

import { collectEntryManifest, } from './entry-manifest.ts';
import { WorktreeCopyError, } from './errors.ts';
import { installSnapshot, } from './install.ts';
import {
  createWorktreeCopyJournal,
  type PendingWorktreeCopyJournal,
  readPendingWorktreeCopyJournals,
  removeWorktreeCopyJournal,
} from './journal.ts';
import type {
  CreatedWorktree,
  StagedWorktreeSnapshot,
  WorktreeCopySummary,
} from './model.ts';
import { stageIgnoredSnapshot, } from './snapshot.ts';
import {
  beginInstalling,
  type JournalState,
} from './transaction-journal.ts';

/**
 * Reconstructs staged snapshot from validated durable journal.
 *
 * @param journal - pending durable worktree-copy transaction
 *
 * @returns staged payload and deterministic manifest
 *
 * @example
 * ```ts
 * await snapshotFromJournal(pending);
 * ```
 */
async function snapshotFromJournal(
  journal: PendingWorktreeCopyJournal,
): Promise<StagedWorktreeSnapshot> {
  try {
    /**
     * Deterministic entries currently retained in staged payload.
     */
    const entries = await collectEntryManifest({
      root: journal.record
        .stageRoot,
      selectedRoots: journal.record
        .selectedRoots,
      excludedRoots: [],
    },);
    return {
      entries,
      selectedRoots: journal.record
        .selectedRoots,
      sourceRoot: journal.record
        .sourceRoot,
      stageContainer: journal.record
        .stageContainer,
      stageRoot: journal.record
        .stageRoot,
    };
  }
  catch (error: unknown) {
    throw new WorktreeCopyError(
      `cli-git: could not recover staged ignored state at ${JSON.stringify(journal.record
        .stageRoot,)}.`,
      error,
    );
  }
}

/**
 * Completes one staged or interrupted destination installation.
 *
 * @param pending - durable transaction
 *
 * @param snapshot - validated staged payload
 *
 * @returns newly installed selected entry count
 *
 * @example
 * ```ts
 * await completeJournal({ pending, snapshot });
 * ```
 */
async function completeJournal({
  pending,
  snapshot,
}: Readonly<{
  pending: PendingWorktreeCopyJournal;
  snapshot: StagedWorktreeSnapshot;
}>,): Promise<number> {
  /**
   * Mutable current journal record for callbacks.
   */
  const state: JournalState = { pending, };
  await beginInstalling(state,);
  /**
   * Newly installed selected entry count.
   */
  const copiedEntries = await installSnapshot({
    snapshot,
    destinationRoot: pending.record
      .destinationRoot,
    journalState: state,
  },);
  await removeWorktreeCopyJournal(state.pending,);
  return copiedEntries;
}

/**
 * Recovers every durable interrupted worktree-copy transaction.
 *
 * @param commonDir - canonical common Git directory
 *
 * @returns recovered destination count
 *
 * @throws {@link WorktreeCopyError} while retaining conflicting evidence
 *
 * @example
 * ```ts
 * await recoverWorktreeCopyTransactions('/repo/.git');
 * ```
 */
export async function recoverWorktreeCopyTransactions(
  commonDir: string,
): Promise<number> {
  /**
   * Pending journals in deterministic filename order.
   */
  const pending = await readPendingWorktreeCopyJournals(commonDir,);
  for (const journal of pending) {
    /* oxlint-disable no-await-in-loop -- recovery order is deterministic and stops at first retained conflict */
    /**
     * Deterministic staged snapshot reconstructed for current journal.
     */
    const snapshot = await snapshotFromJournal(journal,);
    /* oxlint-enable no-await-in-loop */
    // oxlint-disable-next-line no-await-in-loop -- one journal must settle before later transaction uses same destinations
    await completeJournal({
      pending: journal,
      snapshot,
    },);
  }
  return pending.length;
}

/**
 * Creates transaction journal or removes unowned stage after journal failure.
 *
 * @param commonDir - canonical common Git directory
 *
 * @param destinationRoot - newly created worktree root
 *
 * @param snapshot - validated staged snapshot
 *
 * @returns durable transaction journal
 *
 * @example
 * ```ts
 * await createJournalOrCleanup({ commonDir, destinationRoot, snapshot });
 * ```
 */
async function createJournalOrCleanup({
  commonDir,
  destinationRoot,
  snapshot,
}: Readonly<{
  commonDir: string;
  destinationRoot: string;
  snapshot: StagedWorktreeSnapshot;
}>,): Promise<PendingWorktreeCopyJournal> {
  try {
    return await createWorktreeCopyJournal({
      commonDir,
      destinationRoot,
      snapshot,
    },);
  }
  catch (error: unknown) {
    await rm(
      snapshot.stageContainer,
      {
        recursive: true,
        force: true,
      },
    );
    throw error;
  }
}

/**
 * Synchronizes ignored source state into one newly registered worktree.
 *
 * @param commonDir - canonical common Git directory
 *
 * @param sourceRoot - canonical source worktree
 *
 * @param destinationRoot - canonical created worktree
 *
 * @param registeredRoots - every registered root excluded from recursive copy
 *
 * @param gitPath - absolute real-Git executable
 *
 * @returns newly installed selected entry count
 *
 * @example
 * ```ts
 * await synchronizeCreatedWorktree({ commonDir, sourceRoot, destinationRoot, registeredRoots, gitPath });
 * ```
 */
async function synchronizeCreatedWorktree({
  commonDir,
  sourceRoot,
  destinationRoot,
  registeredRoots,
  gitPath,
}: Readonly<{
  commonDir: string;
  sourceRoot: string;
  destinationRoot: string;
  registeredRoots: readonly string[];
  gitPath: string;
}>,): Promise<number> {
  /**
   * Validated private ignored-state snapshot.
   */
  const snapshot = await stageIgnoredSnapshot({
    sourceRoot,
    destinationRoot,
    registeredRoots,
    gitPath,
  },);
  /**
   * Durable staged transaction journal.
   */
  const pending = await createJournalOrCleanup({
    commonDir,
    destinationRoot,
    snapshot,
  },);
  return completeJournal({
    pending,
    snapshot,
  },);
}

/**
 * Synchronizes ignored source state into every created worktree.
 *
 * @param commonDir - canonical common Git directory
 *
 * @param sourceRoot - canonical source worktree, absent for bare repository
 *
 * @param created - newly registered linked worktrees
 *
 * @param registeredRoots - all registered roots excluded from source recursion
 *
 * @param gitPath - absolute real-Git executable
 *
 * @returns aggregate success summary
 *
 * @example
 * ```ts
 * await synchronizeCreatedWorktrees({ commonDir, sourceRoot: '/repo', created, registeredRoots, gitPath });
 * ```
 */
export async function synchronizeCreatedWorktrees({
  commonDir,
  sourceRoot,
  created,
  registeredRoots,
  gitPath,
}: Readonly<{
  commonDir: string;
  sourceRoot?: string;
  created: readonly CreatedWorktree[];
  registeredRoots: readonly string[];
  gitPath: string;
}>,): Promise<WorktreeCopySummary> {
  if (sourceRoot === undefined) {
    return {
      copiedEntries: 0,
      destinationCount: created.length,
    };
  }
  /**
   * Newly installed selected entry counts per destination.
   */
  const copiedCounts: number[] = [];
  for (const destination of created) {
    // oxlint-disable-next-line no-await-in-loop -- destination transactions remain isolated and deterministic
    copiedCounts.push(await synchronizeCreatedWorktree({
      commonDir,
      sourceRoot,
      destinationRoot: destination.root,
      registeredRoots,
      gitPath,
    },),);
  }
  /**
   * Aggregate newly installed selected entry count.
   */
  const copiedEntries = copiedCounts.reduce(
    function addCount(
      total,
      count,
    ): number {
    return total + count;
  },
    0,
  );
  return {
    copiedEntries,
    destinationCount: created.length,
    sourceRoot,
  };
}
