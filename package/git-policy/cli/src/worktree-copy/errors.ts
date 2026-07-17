import type { SubprocessError, } from 'nano-spawn';

/**
 * Filesystem or repository failure while synchronizing ignored worktree state.
 *
 * @example
 * ```ts
 * throw new WorktreeCopyError('cli-git: could not copy ignored state.');
 * ```
 */
export class WorktreeCopyError extends Error {
  /**
   * Creates typed worktree-copy failure.
   *
   * @param message - complete user-facing diagnostic
   *
   * @param cause - underlying filesystem or Git failure
   */
  public constructor(message: string, cause?: unknown,) {
    super(message, { cause, },);
    this.name = 'WorktreeCopyError';
  }
}

/**
 * Combined failure preserving real Git status after ignored-state copying also failed.
 *
 * @example
 * ```ts
 * throw new ForwardedGitWorktreeCopyError({
 *   copyFailure: new WorktreeCopyError('copy failed'),
 *   gitFailure,
 * });
 * ```
 */
export class ForwardedGitWorktreeCopyError extends Error {
  /** Worktree copy failure requiring an explicit diagnostic. */
  public readonly copyFailure: WorktreeCopyError;

  /** Real-Git failure whose exit status remains authoritative. */
  public readonly gitFailure: SubprocessError | undefined;

  /**
   * Creates combined lifecycle failure without discarding real-Git status.
   *
   * @param copyFailure - ignored-state synchronization failure
   *
   * @param gitFailure - optional real-Git subprocess failure
   */
  public constructor({
    copyFailure,
    gitFailure,
  }: Readonly<{
    copyFailure: WorktreeCopyError;
    gitFailure: SubprocessError | undefined;
  }>,) {
    super(copyFailure.message, { cause: copyFailure, },);
    this.name = 'ForwardedGitWorktreeCopyError';
    this.copyFailure = copyFailure;
    this.gitFailure = gitFailure;
  }
}
