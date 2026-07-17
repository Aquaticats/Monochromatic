import { randomUUID, } from 'node:crypto';
import { constants, } from 'node:fs';
import {
  mkdir,
  open,
  readFile,
  readdir,
  rename,
  rm,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';

import { WorktreeCopyError, } from './errors.ts';
import { validateJournalValue, } from './journal-validation.ts';
import type {
  StagedWorktreeSnapshot,
  WorktreeCopyJournal,
} from './model.ts';

/**
 * Worktree-copy journal subtree beneath common Git directory.
 */
const JOURNAL_RELATIVE_ROOT = join(
  'cli-git-worktree-copy',
  'v1',
);

/**
 * Private journal directory mode.
 */
const PRIVATE_DIRECTORY_MODE = 0o700;

/**
 * Private journal file mode.
 */
const PRIVATE_FILE_MODE = 0o600;

/**
 * Journal filename suffix.
 */
const JOURNAL_SUFFIX = '.json';

/**
 * Resolves private journal root beneath canonical common Git directory.
 *
 * @param commonDir - canonical common Git directory
 *
 * @returns private versioned journal root
 *
 * @example
 * ```ts
 * worktreeCopyJournalRoot('/repo/.git');
 * // => '/repo/.git/cli-git-worktree-copy/v1'
 * ```
 */
export function worktreeCopyJournalRoot(commonDir: string,): string {
  return join(
    commonDir,
    JOURNAL_RELATIVE_ROOT,
  );
}

/**
 * Durable journal file paired with validated record.
 *
 * @example
 * ```ts
 * const pending: PendingWorktreeCopyJournal = { path: '/repo/.git/cli-git-worktree-copy/v1/id.json', record };
 * ```
 */
export type PendingWorktreeCopyJournal = Readonly<{
  /**
   * Absolute durable journal path.
   */
  path: string;
  /**
   * Validated journal record.
   */
  record: WorktreeCopyJournal;
}>;

/**
 * Flushes directory metadata where host supports directory handles.
 *
 * @param path - directory containing durable journal mutation
 *
 * @example
 * ```ts
 * await syncDirectory('/repo/.git/cli-git-worktree-copy/v1');
 * ```
 */
async function syncDirectory(path: string,): Promise<void> {
  if (process.platform === 'win32')
    return;
  /**
   * Read-only directory handle used for metadata fsync.
   */
  await using handle = await open(
    path,
    constants.O_RDONLY,
  );
  await handle.sync();
}

/**
 * Atomically writes complete private journal record.
 *
 * @param path - final journal path
 *
 * @param record - complete durable record
 *
 * @example
 * ```ts
 * await writeJournal({ path: '/journal.json', record });
 * ```
 */
export async function writeJournal({
  path,
  record,
}: Readonly<{
  path: string;
  record: WorktreeCopyJournal;
}>,): Promise<void> {
  /**
   * Unique sibling temporary journal path.
   */
  const temporaryPath = `${path}.${randomUUID()}.tmp`;
  /**
   * Plain immutable journal value detached from caller-owned arrays.
   */
  const serializedRecord: WorktreeCopyJournal = {
    ...record,
    intendedEntries: [...record.intendedEntries,],
    selectedRoots: [...record.selectedRoots,],
  };
  try {
    {
      /**
       * Exclusive no-follow temporary journal handle.
       */
      await using handle = await open(
        temporaryPath,
        constants.O_CREAT | constants.O_EXCL | constants.O_WRONLY | constants.O_NOFOLLOW,
        PRIVATE_FILE_MODE,
      );
      await handle.writeFile(
        `${JSON.stringify(serializedRecord,)}\n`,
        'utf8',
      );
      await handle.sync();
    }
    await rename(
      temporaryPath,
      path,
    );
    await syncDirectory(dirname(path,),);
  }
  catch (error: unknown) {
    await rm(
      temporaryPath,
      { force: true, },
    );
    throw new WorktreeCopyError(
      `cli-git: could not persist worktree-copy journal ${JSON.stringify(path,)}.`,
      error,
    );
  }
}

/**
 * Creates staged transaction journal beneath common Git directory.
 *
 * @param commonDir - canonical common Git directory
 *
 * @param destinationRoot - created worktree root
 *
 * @param snapshot - validated private ignored-state snapshot
 *
 * @returns durable journal path and record
 *
 * @example
 * ```ts
 * await createWorktreeCopyJournal({ commonDir: '/repo/.git', destinationRoot: '/wt', snapshot });
 * ```
 */
export async function createWorktreeCopyJournal({
  commonDir,
  destinationRoot,
  snapshot,
}: Readonly<{
  commonDir: string;
  destinationRoot: string;
  snapshot: StagedWorktreeSnapshot;
}>,): Promise<PendingWorktreeCopyJournal> {
  /**
   * Private journal directory.
   */
  const root = worktreeCopyJournalRoot(commonDir,);
  await mkdir(
    root,
    {
      recursive: true,
      mode: PRIVATE_DIRECTORY_MODE,
    },
  );
  /**
   * Unique final journal path.
   */
  const path = join(
    root,
    `${randomUUID()}${JOURNAL_SUFFIX}`,
  );
  /**
   * Initial staged journal record.
   */
  const record: WorktreeCopyJournal = {
    intendedEntries: [],
    destinationRoot,
    phase: 'staged',
    selectedRoots: snapshot.selectedRoots,
    sourceRoot: snapshot.sourceRoot,
    stageContainer: snapshot.stageContainer,
    stageRoot: snapshot.stageRoot,
    version: 1,
  };
  await writeJournal({
    path,
    record,
  },);
  return {
    path,
    record,
  };
}

/**
 * Reads sorted journal filenames or journal-directory absence.
 *
 * @param root - private journal directory
 *
 * @returns sorted JSON journal filenames
 *
 * @example
 * ```ts
 * await readJournalNames('/repo/.git/cli-git-worktree-copy/v1');
 * ```
 */
async function readJournalNames(root: string,): Promise<readonly string[]> {
  try {
    return (await readdir(root,))
      .filter(function journalName(name,): boolean {
        return name.endsWith(JOURNAL_SUFFIX,);
      },)
      .toSorted();
  }
  catch (error: unknown) {
    if (Error.isError(error,)
      && ('code' in error)
      && (error.code === 'ENOENT')) {
      return [];
    }
    throw new WorktreeCopyError(
      `cli-git: could not inspect worktree-copy journals under ${JSON.stringify(root,)}.`,
      error,
    );
  }
}

/**
 * Reads every incomplete durable worktree-copy journal.
 *
 * @param commonDir - canonical common Git directory
 *
 * @returns validated pending journals sorted by filename
 *
 * @example
 * ```ts
 * await readPendingWorktreeCopyJournals('/repo/.git');
 * ```
 */
export async function readPendingWorktreeCopyJournals(
  commonDir: string,
): Promise<readonly PendingWorktreeCopyJournal[]> {
  /**
   * Private journal directory.
   */
  const root = worktreeCopyJournalRoot(commonDir,);
  /**
   * Existing final journal filenames.
   */
  const names = await readJournalNames(root,);
  return Promise.all(names.map(async function readJournal(name,): Promise<PendingWorktreeCopyJournal> {
    /**
     * Absolute final journal path.
     */
    const path = join(
      root,
      name,
    );
    try {
      /**
       * Untrusted parsed journal JSON.
       */
      const parsed: unknown = JSON.parse(await readFile(
        path,
        'utf8',
      ),);
      return {
        path,
        record: validateJournalValue({
          value: parsed,
          path,
          journalRoot: root,
        },),
      };
    }
    catch (error: unknown) {
      if (error instanceof WorktreeCopyError)
        throw error;
      throw new WorktreeCopyError(
        `cli-git: could not read worktree-copy journal ${JSON.stringify(path,)}.`,
        error,
      );
    }
  },),);
}

/**
 * Removes completed journal and private stage.
 *
 * @param pending - completed durable transaction
 *
 * @example
 * ```ts
 * await removeWorktreeCopyJournal(pending);
 * ```
 */
export async function removeWorktreeCopyJournal(
  pending: PendingWorktreeCopyJournal,
): Promise<void> {
  /**
   * Durable completion marker making stage cleanup resumable.
   */
  const completeRecord: WorktreeCopyJournal = {
    ...pending.record,
    phase: 'complete',
  };
  if (pending.record.phase !== 'complete') {
    await writeJournal({
      path: pending.path,
      record: completeRecord,
    },);
  }
  await rm(
    completeRecord.stageContainer,
    {
      recursive: true,
      force: true,
    },
  );
  await rm(
    pending.path,
    { force: true, },
  );
  await syncDirectory(dirname(pending.path,),);
}
