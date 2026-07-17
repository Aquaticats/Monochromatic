import { randomUUID, } from 'node:crypto';
import {
  mkdir,
  readFile,
  readdir,
  rename,
  rm,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { WorktreeCopyError, } from './errors.ts';
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
 * Reports unknown value is ordinary object record.
 *
 * @param value - parsed JSON value
 *
 * @returns whether value is non-null object and not array
 *
 * @example
 * ```ts
 * isRecord({});
 * // => true
 * ```
 */
function isRecord(value: unknown,): value is Record<string, unknown> {
  return ((typeof value) === 'object') && (value !== null)
    && (!Array.isArray(value,));
}

/**
 * Reports unknown value is readonly string array.
 *
 * @param value - parsed JSON field
 *
 * @returns whether every array member is string
 *
 * @example
 * ```ts
 * isStringArray(['a']);
 * // => true
 * ```
 */
function isStringArray(value: unknown,): value is readonly string[] {
  return Array.isArray(value,)
    && value.every(function isString(item,): item is string {
      return (typeof item) === 'string';
    },);
}

/**
 * Validates durable JSON record before recovery uses filesystem paths.
 *
 * @param value - parsed unknown JSON value
 *
 * @param path - journal path used in diagnostics
 *
 * @returns validated schema-version-one journal
 *
 * @throws {@link WorktreeCopyError} when record is malformed
 *
 * @example
 * ```ts
 * validateJournal({ value: parsed, path: '/journal.json' });
 * ```
 */
function validateJournal({
  value,
  path,
}: Readonly<{
  value: unknown;
  path: string;
}>,): WorktreeCopyJournal {
  if (isRecord(value,)
    && (value.version === 1)
    && ((value.phase === 'staged') || (value.phase === 'installing'))
    && isStringArray(value.createdEntries,)
    && isStringArray(value.selectedRoots,)
    && ((typeof value.destinationRoot) === 'string')
    && ((typeof value.sourceRoot) === 'string')
    && ((typeof value.stageContainer) === 'string')
    && ((typeof value.stageRoot) === 'string')) {
    return {
      createdEntries: value.createdEntries,
      destinationRoot: value.destinationRoot,
      phase: value.phase,
      selectedRoots: value.selectedRoots,
      sourceRoot: value.sourceRoot,
      stageContainer: value.stageContainer,
      stageRoot: value.stageRoot,
      version: 1,
    };
  }
  throw new WorktreeCopyError(
    `cli-git: worktree-copy journal is corrupt: ${JSON.stringify(path,)}.`,
  );
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
    createdEntries: [...record.createdEntries,],
    selectedRoots: [...record.selectedRoots,],
  };
  try {
    await writeFile(
      temporaryPath,
      `${JSON.stringify(serializedRecord,)}\n`,
      {
        encoding: 'utf8',
        flag: 'wx',
        mode: PRIVATE_FILE_MODE,
      },
    );
    await rename(
      temporaryPath,
      path,
    );
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
  const root = join(
    commonDir,
    JOURNAL_RELATIVE_ROOT,
  );
  await mkdir(
    root,
    { recursive: true, mode: PRIVATE_DIRECTORY_MODE, },
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
    createdEntries: [],
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
  const root = join(
    commonDir,
    JOURNAL_RELATIVE_ROOT,
  );
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
        record: validateJournal({
          value: parsed,
          path,
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
  await rm(
    pending.path,
    { force: true, },
  );
  await rm(
    pending.record.stageContainer,
    { recursive: true, force: true, },
  );
}
