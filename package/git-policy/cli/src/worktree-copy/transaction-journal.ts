import {
  type PendingWorktreeCopyJournal,
  writeJournal,
} from './journal.ts';
import type { WorktreeCopyJournal, } from './model.ts';

/**
 * Mutable durable transaction state hidden inside one synchronization call.
 *
 * @example
 * ```ts
 * const state: JournalState = { pending };
 * ```
 */
export type JournalState = {
  /** Latest journal path and record. */
  pending: PendingWorktreeCopyJournal;
};

/**
 * Persists installation phase before destination mutation.
 *
 * @param state - mutable latest journal state
 *
 * @returns nothing after journal update
 *
 * @example
 * ```ts
 * await beginInstalling(state);
 * ```
 */
export async function beginInstalling(state: JournalState,): Promise<void> {
  /** Installing record replacing staged phase. */
  const record: WorktreeCopyJournal = {
    ...state.pending.record,
    phase: 'installing',
  };
  await writeJournal({ path: state.pending.path, record, },);
  state.pending = { path: state.pending.path, record, };
}

/**
 * Records one selected entry after destination creation.
 *
 * @param state - mutable latest journal state
 *
 * @param relativePath - newly installed repository path
 *
 * @returns nothing after durable update
 *
 * @example
 * ```ts
 * await recordCreatedEntry({ state, relativePath: 'cache/data' });
 * ```
 */
export async function recordCreatedEntry({
  state,
  relativePath,
}: Readonly<{
  state: JournalState;
  relativePath: string;
}>,): Promise<void> {
  /** Updated durable creation list. */
  const record: WorktreeCopyJournal = {
    ...state.pending.record,
    createdEntries: [
      ...state.pending.record.createdEntries,
      relativePath,
    ],
    phase: 'installing',
  };
  await writeJournal({ path: state.pending.path, record, },);
  state.pending = { path: state.pending.path, record, };
}
