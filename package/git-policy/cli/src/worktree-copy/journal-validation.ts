import type { Stats, } from 'node:fs';
import {
  lstat,
  readFile,
  realpath,
} from 'node:fs/promises';
import {
  basename,
  dirname,
  isAbsolute,
  join,
  resolve,
} from 'node:path';

import { WorktreeCopyError, } from './errors.ts';
import { assertSafeRepositoryPath, } from './ignored-paths.ts';
import type {
  InstalledWorktreePath,
  WorktreeCopyJournal,
} from './model.ts';
import { STAGE_PREFIX, } from './snapshot.ts';

/**
 * Git-file prefix introducing linked-worktree administrative path.
 */
const GITDIR_PREFIX = 'gitdir: ';

/**
 * Group and other permission bits forbidden on private stage directories.
 */
const NON_PRIVATE_MODE_BITS = 0o077;

/**
 * Filesystem path is absent.
 */
const PATH_ABSENT: unique symbol = Symbol('journal path is absent',);

/**
 * Reports whether unknown JSON value is a non-null object record.
 *
 * @param value - untrusted parsed JSON value
 *
 * @returns whether value supports named field inspection
 *
 * @example
 * ```ts
 * isRecord({ version: 1 });
 * // => true
 * ```
 */
function isRecord(value: unknown,): value is Readonly<Record<string, unknown>> {
  return ((typeof value) === 'object') && (value !== null);
}

/**
 * Reports whether unknown JSON value is an array containing only strings.
 *
 * @param value - untrusted parsed JSON field
 *
 * @returns whether every array member is a string
 *
 * @example
 * ```ts
 * isStringArray(['cache']);
 * // => true
 * ```
 */
function isStringArray(value: unknown,): value is readonly string[] {
  return Array.isArray(value,)
    && value.every(function stringMember(item,): item is string {
      return (typeof item) === 'string';
    },);
}

/**
 * Reports whether string is canonical nonnegative decimal identity.
 *
 * @param value - untrusted device or inode text
 *
 * @returns whether text has canonical decimal form
 *
 * @example
 * ```ts
 * isDecimalIdentity('42');
 * // => true
 * ```
 */
function isDecimalIdentity(value: unknown,): value is string {
  if (((typeof value) !== 'string') || (value === '')
    || ((value.length > 1) && value.startsWith('0',))) {
    return false;
  }
  for (let index = 0; index < value.length; index += 1) {
    /**
     * Current decimal candidate character.
     */
    const character = value.charAt(index,);
    if ((character < '0') || (character > '9'))
      return false;
  }
  return true;
}

/**
 * Reports whether unknown value is durable created-entry identity.
 *
 * @param value - untrusted created-entry record
 *
 * @returns whether every required primitive field is valid
 *
 * @example
 * ```ts
 * isInstalledEntry({ device: '1', inode: '2', relativePath: 'cache', selected: true });
 * // => true
 * ```
 */
function isInstalledEntry(value: unknown,): value is InstalledWorktreePath {
  return isRecord(value,)
    && isDecimalIdentity(value.device,)
    && isDecimalIdentity(value.inode,)
    && ((typeof value.relativePath) === 'string')
    && ((typeof value.selected) === 'boolean');
}

/**
 * Reports whether unknown value is array of durable created-entry identities.
 *
 * @param value - untrusted created-entry array
 *
 * @returns whether every entry has valid primitive fields
 *
 * @example
 * ```ts
 * isInstalledEntryArray([]);
 * // => true
 * ```
 */
function isInstalledEntryArray(
  value: unknown,
): value is readonly InstalledWorktreePath[] {
  return Array.isArray(value,)
    && value.every(isInstalledEntry,);
}

/**
 * Asserts absolute path is lexically canonical.
 *
 * @param value - untrusted journal path
 *
 * @param field - journal field name for diagnostics
 *
 * @throws {@link WorktreeCopyError} when path is relative or noncanonical
 *
 * @example
 * ```ts
 * assertCanonicalAbsolutePath({ value: '/repo/cache', field: 'sourceRoot' });
 * ```
 */
function assertCanonicalAbsolutePath({
  value,
  field,
}: Readonly<{
  value: string;
  field: string;
}>,): void {
  if ((!isAbsolute(value,)) || (resolve(value,) !== value)) {
    throw new WorktreeCopyError(
      `cli-git: worktree-copy journal has unsafe ${field}: ${JSON.stringify(value,)}.`,
    );
  }
}

/**
 * Validates durable JSON record before recovery uses filesystem paths.
 *
 * @param value - parsed unknown JSON value
 *
 * @param path - journal path used in diagnostics
 *
 * @param journalRoot - exact private journal directory
 *
 * @returns validated schema-version-one journal
 *
 * @throws {@link WorktreeCopyError} when record or path relation is malformed
 *
 * @example
 * ```ts
 * validateJournalValue({ value: parsed, path: '/repo/.git/cli-git-worktree-copy/v1/id.json', journalRoot });
 * ```
 */
export function validateJournalValue({
  value,
  path,
  journalRoot,
}: Readonly<{
  value: unknown;
  path: string;
  journalRoot: string;
}>,): WorktreeCopyJournal {
  if (!(isRecord(value,)
    && (value.version === 1)
    && ((value.phase === 'staged') || (value.phase === 'installing')
      || (value.phase === 'complete'))
    && isInstalledEntryArray(value.createdEntries,)
    && isStringArray(value.intendedEntries,)
    && isStringArray(value.selectedRoots,)
    && ((typeof value.destinationRoot) === 'string')
    && ((typeof value.sourceRoot) === 'string')
    && ((typeof value.stageContainer) === 'string')
    && ((typeof value.stageRoot) === 'string'))) {
    throw new WorktreeCopyError(
      `cli-git: worktree-copy journal is corrupt: ${JSON.stringify(path,)}.`,
    );
  }
  /**
   * Plain immutable journal detached from untrusted parsed object.
   */
  const record: WorktreeCopyJournal = {
    createdEntries: value.createdEntries.map(function createdEntry(
      entry,
    ): InstalledWorktreePath {
      return { ...entry, };
    },),
    destinationRoot: value.destinationRoot,
    intendedEntries: [...value.intendedEntries,],
    phase: value.phase,
    selectedRoots: [...value.selectedRoots,],
    sourceRoot: value.sourceRoot,
    stageContainer: value.stageContainer,
    stageRoot: value.stageRoot,
    version: 1,
  };
  if (dirname(path,) !== journalRoot) {
    throw new WorktreeCopyError(
      `cli-git: worktree-copy journal escaped private root: ${JSON.stringify(path,)}.`,
    );
  }
  assertCanonicalAbsolutePath({
    value: record.destinationRoot,
    field: 'destinationRoot',
  },);
  assertCanonicalAbsolutePath({
    value: record.sourceRoot,
    field: 'sourceRoot',
  },);
  assertCanonicalAbsolutePath({
    value: record.stageContainer,
    field: 'stageContainer',
  },);
  assertCanonicalAbsolutePath({
    value: record.stageRoot,
    field: 'stageRoot',
  },);
  if ((dirname(record.stageContainer,) !== dirname(record.destinationRoot,))
    || (!basename(record.stageContainer,)
      .startsWith(STAGE_PREFIX,))
    || (record.stageRoot !== join(
      record.stageContainer,
      'payload',
    ))) {
    throw new WorktreeCopyError(
      `cli-git: worktree-copy journal has unsafe private stage relation: ${JSON.stringify(path,)}.`,
    );
  }
  record.selectedRoots.forEach(assertSafeRepositoryPath,);
  record.intendedEntries.forEach(assertSafeRepositoryPath,);
  record.createdEntries.forEach(function safeCreatedEntry(entry,): void {
    assertSafeRepositoryPath(entry.relativePath,);
  },);
  /**
   * Created paths used to reject duplicate ownership claims.
   */
  const createdPaths = record.createdEntries.map(function createdPath(entry,): string {
    return entry.relativePath;
  },);
  if ((new Set(record.selectedRoots,).size !== record.selectedRoots.length)
    || (new Set(record.intendedEntries,).size !== record.intendedEntries.length)
    || (new Set(createdPaths,).size !== createdPaths.length)) {
    throw new WorktreeCopyError(
      `cli-git: worktree-copy journal contains duplicate repository paths: ${JSON.stringify(path,)}.`,
    );
  }
  return record;
}

/**
 * Reads no-follow metadata or absence for journal path validation.
 *
 * @param path - exact filesystem path
 *
 * @returns no-follow metadata or absence sentinel
 *
 * @example
 * ```ts
 * await lstatOrAbsent('/private/stage');
 * ```
 */
async function lstatOrAbsent(path: string,): Promise<Readonly<Stats> | typeof PATH_ABSENT> {
  try {
    return await lstat(path,);
  }
  catch (error: unknown) {
    if (Error.isError(error,) && ('code' in error)
      && (error.code === 'ENOENT'))
      return PATH_ABSENT;
    throw error;
  }
}

/**
 * Asserts one private stage directory is canonical, owned, and inaccessible to peers.
 *
 * @param path - exact stage directory
 *
 * @throws {@link WorktreeCopyError} when stage identity is unsafe
 *
 * @example
 * ```ts
 * await assertPrivateDirectory('/worktrees/.cli-git-worktree-copy-id');
 * ```
 */
async function assertPrivateDirectory(path: string,): Promise<void> {
  /**
   * No-follow private-directory metadata.
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
      `cli-git: worktree-copy private stage is unsafe: ${JSON.stringify(path,)}.`,
    );
  }
}

/**
 * Asserts journal destination remains registered under expected common directory.
 *
 * @param commonDir - canonical common Git directory
 *
 * @param destinationRoot - canonical linked-worktree root
 *
 * @throws {@link WorktreeCopyError} when destination registration changed
 *
 * @example
 * ```ts
 * await assertRegisteredDestination({ commonDir: '/repo/.git', destinationRoot: '/worktrees/topic' });
 * ```
 */
async function assertRegisteredDestination({
  commonDir,
  destinationRoot,
}: Readonly<{
  commonDir: string;
  destinationRoot: string;
}>,): Promise<void> {
  if ((await realpath(destinationRoot,)) !== destinationRoot) {
    throw new WorktreeCopyError(
      `cli-git: worktree-copy destination identity changed: ${JSON.stringify(destinationRoot,)}.`,
    );
  }
  /**
   * Linked-worktree Git-file pointer.
   */
  const pointer = (await readFile(
    join(
      destinationRoot,
      '.git',
    ),
    'utf8',
  ))
    .trimEnd();
  if (!pointer.startsWith(GITDIR_PREFIX,)) {
    throw new WorktreeCopyError(
      `cli-git: worktree-copy destination is not a linked worktree: ${JSON.stringify(destinationRoot,)}.`,
    );
  }
  /**
   * Canonical linked administrative directory.
   */
  const adminPath = await realpath(resolve(
    destinationRoot,
    pointer.slice(GITDIR_PREFIX.length,),
  ),);
  /**
   * Canonical expected linked administrative parent.
   */
  const expectedAdminRoot = await realpath(join(
    commonDir,
    'worktrees',
  ),);
  if (dirname(adminPath,) !== expectedAdminRoot) {
    throw new WorktreeCopyError(
      `cli-git: worktree-copy destination registration changed: ${JSON.stringify(destinationRoot,)}.`,
    );
  }
}

/**
 * Validates live identities before recovery reads, installs, or removes stage state.
 *
 * @param commonDir - canonical common Git directory
 *
 * @param record - schema-validated journal record
 *
 * @throws {@link WorktreeCopyError} when private stage or destination changed
 *
 * @example
 * ```ts
 * await validateJournalFilesystem({ commonDir: '/repo/.git', record });
 * ```
 */
export async function validateJournalFilesystem({
  commonDir,
  record,
}: Readonly<{
  commonDir: string;
  record: WorktreeCopyJournal;
}>,): Promise<void> {
  try {
    await assertRegisteredDestination({
      commonDir,
      destinationRoot: record.destinationRoot,
    },);
    /**
     * Private stage-container metadata or completed-cleanup absence.
     */
    const containerStats = await lstatOrAbsent(record.stageContainer,);
    if ((typeof containerStats) === 'symbol') {
      if (record.phase === 'complete')
        return;
      throw new WorktreeCopyError(
        `cli-git: incomplete worktree-copy stage is missing: ${JSON.stringify(record.stageContainer,)}.`,
      );
    }
    await assertPrivateDirectory(record.stageContainer,);
    /**
     * Private payload metadata or partial completed cleanup absence.
     */
    const stageStats = await lstatOrAbsent(record.stageRoot,);
    if ((typeof stageStats) === 'symbol') {
      if (record.phase === 'complete')
        return;
      throw new WorktreeCopyError(
        `cli-git: incomplete worktree-copy payload is missing: ${JSON.stringify(record.stageRoot,)}.`,
      );
    }
    await assertPrivateDirectory(record.stageRoot,);
  }
  catch (error: unknown) {
    if (error instanceof WorktreeCopyError)
      throw error;
    throw new WorktreeCopyError(
      `cli-git: could not validate worktree-copy recovery state for ${JSON.stringify(record.destinationRoot,)}.`,
      error,
    );
  }
}
