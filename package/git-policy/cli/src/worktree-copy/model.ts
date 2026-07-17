import type { SubprocessError, } from 'nano-spawn';

/**
 * Supported copied filesystem entry kinds.
 *
 * @example
 * ```ts
 * const kind: WorktreeCopyEntryKind = 'file';
 * ```
 */
export type WorktreeCopyEntryKind = 'directory' | 'file' | 'symlink';

/**
 * Stable filesystem entry retained in a staged ignored-state snapshot.
 *
 * @example
 * ```ts
 * const entry: WorktreeCopyEntry = {
 *   kind: 'file',
 *   mode: 0o644,
 *   relativePath: 'node_modules/example/index.js',
 * };
 * ```
 */
export type WorktreeCopyEntry = Readonly<{
  /** Entry kind used for copying and collision comparison. */
  kind: WorktreeCopyEntryKind;
  /** Permission bits retained for files and directories. */
  mode: number;
  /** Slash-separated path relative to source and destination worktree roots. */
  relativePath: string;
}>;

/**
 * Effective repository state captured before forwarding to real Git.
 *
 * @example
 * ```ts
 * const observation: WorktreeCopyObservation = {
 *   adminRoot: '/repo/.git/worktrees',
 *   beforeAdminIds: new Set(),
 *   commonDir: '/repo/.git',
 *   effectiveCwd: '/repo',
 *   sourceRoot: '/repo',
 * };
 * ```
 */
export type WorktreeCopyObservation = Readonly<{
  /** Common-Git-directory location containing linked-worktree administration. */
  adminRoot: string;
  /** Linked-worktree administrative identities present before real Git ran. */
  beforeAdminIds: ReadonlySet<string>;
  /** Canonical common Git directory for journal ownership. */
  commonDir: string;
  /** Effective working directory after pre-subcommand `-C` chaining. */
  effectiveCwd: string;
  /** Canonical source worktree root, absent for bare repositories. */
  sourceRoot: string | undefined;
}>;

/**
 * One linked worktree registered while real Git was running.
 *
 * @example
 * ```ts
 * const created: CreatedWorktree = {
 *   adminId: 'topic',
 *   root: '/worktrees/topic',
 * };
 * ```
 */
export type CreatedWorktree = Readonly<{
  /** Stable linked-worktree administrative directory basename. */
  adminId: string;
  /** Canonical created worktree root. */
  root: string;
}>;

/**
 * Validated ignored-state snapshot staged beside one destination worktree.
 *
 * @example
 * ```ts
 * const snapshot: StagedWorktreeSnapshot = {
 *   entries: [],
 *   selectedRoots: [],
 *   sourceRoot: '/repo',
 *   stageContainer: '/worktrees/.cli-git-worktree-copy-abc',
 *   stageRoot: '/worktrees/.cli-git-worktree-copy-abc/payload',
 * };
 * ```
 */
export type StagedWorktreeSnapshot = Readonly<{
  /** Complete staged entries sorted parent before child by repository path. */
  entries: readonly WorktreeCopyEntry[];
  /** Git-selected ignored roots represented by staged entries. */
  selectedRoots: readonly string[];
  /** Source worktree root used for final-equivalence validation. */
  sourceRoot: string;
  /** Private sibling directory owning payload and journal scratch files. */
  stageContainer: string;
  /** Private payload root on destination filesystem. */
  stageRoot: string;
}>;

/**
 * Successful ignored-state synchronization facts rendered as one summary line.
 *
 * @example
 * ```ts
 * const summary: WorktreeCopySummary = {
 *   copiedEntries: 2,
 *   destinationCount: 1,
 *   sourceRoot: '/repo',
 * };
 * ```
 */
export type WorktreeCopySummary = Readonly<{
  /** Filesystem entries newly installed across every destination. */
  copiedEntries: number;
  /** Newly registered worktrees receiving source state. */
  destinationCount: number;
  /** Source worktree root, absent for bare repositories. */
  sourceRoot: string | undefined;
}>;

/**
 * Settled real-Git execution result retained while post-command copying runs.
 *
 * @example
 * ```ts
 * const execution: ForwardedGitExecution = { failure: undefined };
 * ```
 */
export type ForwardedGitExecution = Readonly<{
  /** Real-Git subprocess failure, absent after exit zero. */
  failure: SubprocessError | undefined;
}>;

/**
 * Durable transaction phase for one destination snapshot.
 *
 * @example
 * ```ts
 * const phase: WorktreeCopyJournalPhase = 'installing';
 * ```
 */
export type WorktreeCopyJournalPhase = 'installing' | 'staged';

/**
 * Durable recovery record for one destination worktree copy.
 *
 * @example
 * ```ts
 * const journal: WorktreeCopyJournal = {
 *   createdEntries: [],
 *   destinationRoot: '/worktrees/topic',
 *   phase: 'staged',
 *   sourceRoot: '/repo',
 *   stageContainer: '/worktrees/.cli-git-worktree-copy-abc',
 *   stageRoot: '/worktrees/.cli-git-worktree-copy-abc/payload',
 *   version: 1,
 * };
 * ```
 */
export type WorktreeCopyJournal = Readonly<{
  /** Paths installed by cli-git before current journal write. */
  createdEntries: readonly string[];
  /** Created worktree receiving ignored state. */
  destinationRoot: string;
  /** Current durable transaction phase. */
  phase: WorktreeCopyJournalPhase;
  /** Source worktree supplying ignored state. */
  sourceRoot: string;
  /** Private sibling directory containing journal-owned temporary state. */
  stageContainer: string;
  /** Validated snapshot payload within stage container. */
  stageRoot: string;
  /** Journal schema version. */
  version: 1;
}>;
