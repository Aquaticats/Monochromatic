import { caughtValueText, } from '@monochromatic-dev/module-caught-value/ts';

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
  return ` Cause: ${caughtValueText(cause,)}`;
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
 *   copyFailureMessage: 'copy failed',
 *   gitFailure: { exitCode: 1 },
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
   * @param copyFailureMessage - ignored-state synchronization diagnostic
   *
   * @param gitFailure - optional primitive real-Git failure details
   */
  public constructor({
    copyFailureMessage,
    gitFailure,
  }: Readonly<{
    copyFailureMessage: string;
    gitFailure?: Readonly<{
      exitCode?: number;
    }>;
  }>,) {
    super(copyFailureMessage,);
    this.name = 'ForwardedGitWorktreeCopyError';
    this.copyFailureMessage = copyFailureMessage;
    this.gitFailed = gitFailure !== undefined;
    if (gitFailure?.exitCode !== undefined)
      this.gitFailureExitCode = gitFailure.exitCode;
  }
}
