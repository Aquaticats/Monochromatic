import {
  type PendingWorktreeCopyJournal,
  writeJournal,
} from './journal.ts';
import type {
  InstalledWorktreePath,
  WorktreeCopyJournal,
} from './model.ts';

/**
 * Mutable durable transaction state hidden inside one synchronization call.
 *
 * @example
 * ```ts
 * const state: JournalState = { pending };
 * ```
 */
export type JournalState = {
  /**
   * Latest journal path and record.
   */
  pending: PendingWorktreeCopyJournal;
};

/**
 * Persists installation phase before destination mutation.
 *
 * @param state - mutable latest journal state
 *
 * @mutates state - replaces latest pending record after durable phase write
 *
 * @example
 * ```ts
 * await beginInstalling(state);
 * ```
 */
export async function beginInstalling(state: JournalState,): Promise<void> {
  /**
   * Installing record replacing staged phase.
   */
  const record: WorktreeCopyJournal = {
    ...state.pending
      .record,
    phase: 'installing',
  };
  await writeJournal({
    path: state.pending
      .path,
    record,
  },);
  state.pending = {
    path: state.pending
      .path,
    record,
  };
}

/**
 * Persists one selected destination-path intent before filesystem mutation.
 *
 * @param state - mutable latest journal state
 *
 * @param relativePath - repository path about to be installed
 *
 * @mutates state - replaces latest pending record after durable intent write
 *
 * @example
 * ```ts
 * await recordEntryIntent({ state, relativePath: 'cache/data' });
 * ```
 */
export async function recordEntryIntent({
  state,
  relativePath,
}: Readonly<{
  state: JournalState;
  relativePath: string;
}>,): Promise<void> {
  if (state.pending
    .record
    .intendedEntries
    .includes(relativePath,))
    return;
  /**
   * Updated durable installation-intent list.
   */
  const record: WorktreeCopyJournal = {
    ...state.pending
      .record,
    intendedEntries: [
      ...state.pending
        .record
        .intendedEntries,
      relativePath,
    ],
    phase: 'installing',
  };
  await writeJournal({
    path: state.pending
      .path,
    record,
  },);
  state.pending = {
    path: state.pending
      .path,
    record,
  };
}

/**
 * Persists one proven post-creation filesystem identity.
 *
 * @param state - mutable latest journal state
 *
 * @param entry - path and exact identity captured after exclusive creation
 *
 * @mutates state - replaces latest pending record after durable creation write
 *
 * @example
 * ```ts
 * await recordCreatedEntry({ state, entry });
 * ```
 */
export async function recordCreatedEntry({
  state,
  entry,
}: Readonly<{
  state: JournalState;
  entry: InstalledWorktreePath;
}>,): Promise<void> {
  if (state.pending
    .record
    .createdEntries
    .some(function samePath(
    created,
  ): boolean {
    return created.relativePath === entry.relativePath;
  },)) {
    return;
  }
  /**
   * Updated durable proven-creation list.
   */
  const record: WorktreeCopyJournal = {
    ...state.pending
      .record,
    createdEntries: [
      ...state.pending
        .record
        .createdEntries,
      { ...entry, },
    ],
    phase: 'installing',
  };
  await writeJournal({
    path: state.pending
      .path,
    record,
  },);
  state.pending = {
    path: state.pending
      .path,
    record,
  };
}
