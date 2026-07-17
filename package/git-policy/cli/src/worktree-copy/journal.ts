import { randomUUID, } from 'node:crypto';
import {
  constants,
  type Stats,
} from 'node:fs';
import {
  chmod,
  lstat,
  mkdir,
  open,
  readFile,
  readdir,
  realpath,
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
 * Worktree-copy journal parent beneath common Git directory.
 */
const JOURNAL_PARENT_NAME = 'cli-git-worktree-copy';

/**
 * Worktree-copy journal schema directory.
 */
const JOURNAL_VERSION_NAME = 'v1';

/**
 * Private journal directory mode.
 */
const PRIVATE_DIRECTORY_MODE = 0o700;

/**
 * Group and other mode bits forbidden on journal directories.
 */
const NON_PRIVATE_MODE_BITS = 0o077;

/**
 * Private journal root does not exist.
 */
export const JOURNAL_ROOT_ABSENT: unique symbol = Symbol('worktree-copy journal root is absent',);

/**
 * Private stage cleanup path is already absent.
 */
const STAGE_PATH_ABSENT: unique symbol = Symbol('worktree-copy stage path is absent',);

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
    JOURNAL_PARENT_NAME,
    JOURNAL_VERSION_NAME,
  );
}

/**
 * Asserts existing journal directory is canonical and private.
 *
 * @param path - exact journal directory path
 *
 * @throws {@link WorktreeCopyError} when ownership, mode, or identity is unsafe
 *
 * @example
 * ```ts
 * await assertPrivateJournalDirectory('/repo/.git/cli-git-worktree-copy/v1');
 * ```
 */
async function assertPrivateJournalDirectory(path: string,): Promise<void> {
  /**
   * No-follow journal-directory metadata.
   */
  const stats = await lstat(path,);
  /**
   * Effective account owner when platform exposes POSIX identity.
   */
  const effectiveUserId = process.geteuid?.();
  if ((!stats.isDirectory())
    || ((stats.mode & NON_PRIVATE_MODE_BITS) !== 0)
    || ((effectiveUserId !== undefined) && (stats.uid !== effectiveUserId))
    || ((await realpath(path,)) !== path)) {
    throw new WorktreeCopyError(
      `cli-git: worktree-copy journal directory is unsafe: ${JSON.stringify(path,)}.`,
    );
  }
}

/**
 * Asserts journal record is canonical private ordinary file.
 *
 * @param path - exact journal record path
 *
 * @throws {@link WorktreeCopyError} when ownership, mode, links, or identity is unsafe
 *
 * @example
 * ```ts
 * await assertPrivateJournalFile('/repo/.git/cli-git-worktree-copy/v1/id.json');
 * ```
 */
async function assertPrivateJournalFile(path: string,): Promise<void> {
  /**
   * No-follow journal-file metadata.
   */
  const stats = await lstat(path,);
  /**
   * Effective account owner when platform exposes POSIX identity.
   */
  const effectiveUserId = process.geteuid?.();
  if ((!stats.isFile())
    || (stats.nlink !== 1)
    || ((stats.mode & NON_PRIVATE_MODE_BITS) !== 0)
    || ((effectiveUserId !== undefined) && (stats.uid !== effectiveUserId))
    || ((await realpath(path,)) !== path)) {
    throw new WorktreeCopyError(
      `cli-git: worktree-copy journal file is unsafe: ${JSON.stringify(path,)}.`,
    );
  }
}

/**
 * Creates one private journal directory or validates exact existing directory.
 *
 * @param path - exact journal directory path
 *
 * @example
 * ```ts
 * await ensurePrivateJournalDirectory('/repo/.git/cli-git-worktree-copy');
 * ```
 */
async function ensurePrivateJournalDirectory(path: string,): Promise<void> {
  try {
    await mkdir(
      path,
      { mode: PRIVATE_DIRECTORY_MODE, },
    );
  }
  catch (error: unknown) {
    if (!(Error.isError(error,) && ('code' in error) && (error.code === 'EEXIST')))
      throw error;
  }
  await assertPrivateJournalDirectory(path,);
}

/**
 * Creates and validates complete private journal root component by component.
 *
 * @param commonDir - canonical common Git directory
 *
 * @returns validated private journal root
 *
 * @example
 * ```ts
 * await ensureWorktreeCopyJournalRoot('/repo/.git');
 * ```
 */
export async function ensureWorktreeCopyJournalRoot(
  commonDir: string,
): Promise<string> {
  /**
   * Private journal parent.
   */
  const parent = join(
    commonDir,
    JOURNAL_PARENT_NAME,
  );
  await ensurePrivateJournalDirectory(parent,);
  /**
   * Versioned private journal root.
   */
  const root = worktreeCopyJournalRoot(commonDir,);
  await ensurePrivateJournalDirectory(root,);
  return root;
}

/**
 * Validates existing private journal root without creating state.
 *
 * @param commonDir - canonical common Git directory
 *
 * @returns validated root or absence sentinel
 *
 * @example
 * ```ts
 * await existingWorktreeCopyJournalRoot('/repo/.git');
 * ```
 */
export async function existingWorktreeCopyJournalRoot(
  commonDir: string,
): Promise<string | typeof JOURNAL_ROOT_ABSENT> {
  /**
   * Expected private journal parent.
   */
  const parent = join(
    commonDir,
    JOURNAL_PARENT_NAME,
  );
  /**
   * Expected versioned journal root.
   */
  const root = worktreeCopyJournalRoot(commonDir,);
  try {
    await Promise.all([
      assertPrivateJournalDirectory(parent,),
      assertPrivateJournalDirectory(root,),
    ],);
    return root;
  }
  catch (error: unknown) {
    if (Error.isError(error,) && ('code' in error) && (error.code === 'ENOENT'))
      return JOURNAL_ROOT_ABSENT;
    throw error;
  }
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
        constants.O_CREAT | constants.O_EXCL
          | constants.O_WRONLY
          | constants.O_NOFOLLOW,
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
  const root = await ensureWorktreeCopyJournalRoot(commonDir,);
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
  const root = await existingWorktreeCopyJournalRoot(commonDir,);
  if ((typeof root) === 'symbol')
    return [];
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
      await assertPrivateJournalFile(path,);
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
 * Reads no-follow stage metadata or completed-cleanup absence.
 *
 * @param path - private stage path
 *
 * @returns no-follow metadata or stage-path absence sentinel
 *
 * @example
 * ```ts
 * await lstatStageOrAbsent('/worktrees/.cli-git-worktree-copy-id');
 * ```
 */
async function lstatStageOrAbsent(
  path: string,
): Promise<Readonly<Stats> | typeof STAGE_PATH_ABSENT> {
  try {
    return await lstat(path,);
  }
  catch (error: unknown) {
    if (Error.isError(error,) && ('code' in error) && (error.code === 'ENOENT'))
      return STAGE_PATH_ABSENT;
    throw error;
  }
}

/**
 * Restores owner traversal and mutation on staged directories before removal.
 *
 * Source directory modes can intentionally omit owner write permission.
 * The private stage is transaction-owned,
 * so cleanup may make only its directories private and writable before removal.
 *
 * @param stageContainer - validated private stage root
 *
 * @example
 * ```ts
 * await prepareStageRemoval('/worktrees/.cli-git-worktree-copy-id');
 * ```
 */
async function prepareStageRemoval(stageContainer: string,): Promise<void> {
  /**
   * Pending no-follow private directories.
   */
  const pending: string[] = [stageContainer,];
  while (pending.length > 0) {
    /**
     * Current private directory candidate.
     */
    const directory = pending.pop();
    if (directory === undefined)
      throw new WorktreeCopyError('cli-git: private stage cleanup lost pending directory.',);
    /**
     * Current no-follow metadata or completed-cleanup absence.
     */
    // oxlint-disable-next-line no-await-in-loop -- no-follow cleanup walk remains bounded by private stage
    const stats = await lstatStageOrAbsent(directory,);
    if ((typeof stats) === 'symbol')
      continue;
    if (!stats.isDirectory())
      continue;
    // oxlint-disable-next-line no-await-in-loop -- owner mode restoration is required before child enumeration
    await chmod(
      directory,
      PRIVATE_DIRECTORY_MODE,
    );
    // oxlint-disable-next-line no-await-in-loop -- child discovery follows restored private directory mode
    const entries = await readdir(
      directory,
      { withFileTypes: true, },
    );
    entries.filter(function childDirectory(entry,): boolean {
      return entry.isDirectory();
    },)
      .forEach(function queueDirectory(entry,): void {
        pending.push(join(
          directory,
          entry.name,
        ),);
      },);
  }
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
  if (pending.record
    .phase
    !== 'complete') {
    await writeJournal({
      path: pending.path,
      record: completeRecord,
    },);
  }
  await prepareStageRemoval(completeRecord.stageContainer,);
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
