import type { SubprocessError, } from 'nano-spawn';

/**
 * Renders caught cause without retaining mutable foreign object.
 *
 * @param cause - optional underlying failure
 *
 * @returns empty suffix or plain caught message
 *
 * @example
 * ```ts
 * causeSuffix(new Error('failure'));
 * // => ' Cause: failure'
 * ```
 */
function causeSuffix(cause?: unknown,): string {
  if (cause === undefined)
    return '';
  if (Error.isError(cause,))
    return ` Cause: ${cause.message}`;
  return ` Cause: ${String(cause,)}`;
}

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
  public constructor(
    message: string,
    cause?: unknown,
  ) {
    super(`${message}${causeSuffix(cause,)}`,);
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
  /**
   * User-facing copy failure diagnostic.
   */
  public readonly copyFailureMessage: string;

  /**
   * Real Git failed before copy settlement.
   */
  public readonly gitFailed: boolean;

  /**
   * Real-Git numeric exit status when process exited normally.
   */
  public readonly gitFailureExitCode?: number;

  /**
   * Creates combined lifecycle failure without retaining mutable subprocess errors.
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
    gitFailure?: SubprocessError;
  }>,) {
    super(copyFailure.message,);
    this.name = 'ForwardedGitWorktreeCopyError';
    this.copyFailureMessage = copyFailure.message;
    this.gitFailed = gitFailure !== undefined;
    if (gitFailure?.exitCode !== undefined)
      this.gitFailureExitCode = gitFailure.exitCode;
  }
}
