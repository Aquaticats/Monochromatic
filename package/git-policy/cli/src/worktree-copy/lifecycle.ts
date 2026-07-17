import { tagged, } from '@monochromatic-dev/module-logger/ts';
import nanoSpawn, { SubprocessError, } from 'nano-spawn';

import {
  ForwardedGitWorktreeCopyError,
  WorktreeCopyError,
} from './errors.ts';
import {
  observeWorktreeRepository,
  WORKTREE_COPY_NOT_APPLICABLE,
} from './git-observer.ts';
import { findCreatedWorktrees, } from './git-registry.ts';
import type {
  ForwardedGitExecution,
  WorktreeCopySummary,
} from './model.ts';
import {
  recoverWorktreeCopyTransactions,
  synchronizeCreatedWorktrees,
} from './transaction.ts';

/**
 * Logger root for real-Git worktree-copy lifecycle.
 */
const l = tagged({ tag: 'cli-git', },);

/**
 * Executes real Git while retaining nonzero subprocess result for post-processing.
 *
 * @param args - final transformed Git argv
 *
 * @param gitPath - absolute real-Git executable
 *
 * @returns optional real-Git subprocess failure
 *
 * @example
 * ```ts
 * await executeRealGit({ args: ['status'], gitPath: '/usr/bin/git' });
 * ```
 */
async function executeRealGit({
  args,
  gitPath,
}: Readonly<{
  args: readonly string[];
  gitPath: string;
}>,): Promise<ForwardedGitExecution> {
  try {
    await nanoSpawn(
      gitPath,
      [...args,],
      { stdio: 'inherit', },
    );
    return {};
  }
  catch (error: unknown) {
    if (error instanceof SubprocessError)
      return { failure: error, };
    throw error;
  }
}

/**
 * Renders one successful ignored-state synchronization summary line.
 *
 * @param summary - aggregate copy facts
 *
 * @returns terminal line ending in LF
 *
 * @example
 * ```ts
 * renderSummary({ copiedEntries: 2, destinationCount: 1, sourceRoot: '/repo' });
 * // => cli-git summary line
 * ```
 */
function renderSummary(summary: WorktreeCopySummary,): string {
  /**
   * Human source description for worktree or bare repository.
   */
  const source = summary.sourceRoot === undefined
    ? 'bare repository with an empty source set'
    : JSON.stringify(summary.sourceRoot,);
  /**
   * Destination noun matching exact count.
   */
  const worktreeNoun = summary.destinationCount === 1
    ? 'worktree'
    : 'worktrees';
  /**
   * Entry noun matching exact count.
   */
  const entryNoun = summary.copiedEntries === 1
    ? 'entry'
    : 'entries';
  return `cli-git: copied ${String(summary.copiedEntries,)} ignored filesystem ${entryNoun} from ${source} into ${String(summary.destinationCount,)} new ${worktreeNoun}.\n`;
}

/**
 * Normalizes unknown post-Git failure into typed copy diagnostic.
 *
 * @param error - unknown observation, recovery, or filesystem failure
 *
 * @returns typed worktree-copy failure
 *
 * @example
 * ```ts
 * asWorktreeCopyError(new Error('failure'));
 * ```
 */
function asWorktreeCopyError(error: unknown,): WorktreeCopyError {
  return error instanceof WorktreeCopyError
    ? error
    : new WorktreeCopyError(
      'cli-git: automatic ignored-state worktree copy failed.',
      error,
    );
}

/**
 * Runs final real-Git command and synchronizes ignored state into created worktrees.
 *
 * Outcome-based administrative identity comparison covers ordinary aliases and
 * commands that register linked worktrees before returning nonzero.
 *
 * @param args - final transformed Git argv
 *
 * @param gitPath - absolute real-Git executable
 *
 * @throws {@link SubprocessError} when Git failed but copying succeeded
 *
 * @throws {@link ForwardedGitWorktreeCopyError} when copying failed
 *
 * @example
 * ```ts
 * await runGitWithWorktreeCopy({ args: ['worktree', 'add', '-b', 'topic', '../topic'], gitPath: '/usr/bin/git' });
 * ```
 */
export async function runGitWithWorktreeCopy({
  args,
  gitPath,
}: Readonly<{
  args: readonly string[];
  gitPath: string;
}>,): Promise<void> {
  /**
   * Tagged lifecycle logger.
   */
  const rl = tagged({
    tag: runGitWithWorktreeCopy.name,
    l,
  },);
  /**
   * Effective repository observation before real Git.
   */
  const observation = await observeWorktreeRepository({
    args,
    gitPath,
  },);
  if (observation === WORKTREE_COPY_NOT_APPLICABLE) {
    /**
     * Real-Git execution outside effective repository.
     */
    const execution = await executeRealGit({
      args,
      gitPath,
    },);
    if ('failure' in execution)
      throw execution.failure;
    return;
  }

  /**
   * Recovered interrupted transactions before allowing another Git command.
   */
  const recovered = await recoverWorktreeCopyTransactions(observation.commonDir,);
  if (recovered > 0) {
    process.stderr
      .write(
      `cli-git: recovered ignored-state copies for ${String(recovered,)} worktree transaction${recovered === 1 ? '' : 's'}.\n`,
    );
  }
  /**
   * Real-Git result retained while post-command worktree state settles.
   */
  const execution = await executeRealGit({
    args,
    gitPath,
  },);

  try {
    /**
     * Newly registered worktrees and recursive exclusion roots.
     */
    const {
      created,
      registeredRoots,
    } = await findCreatedWorktrees({
      observation,
      gitPath,
    },);
    if (created.length > 0) {
      /**
       * Aggregate successful synchronization facts.
       */
      const summary = await synchronizeCreatedWorktrees({
        commonDir: observation.commonDir,
        ...(observation.sourceRoot === undefined
          ? {}
          : { sourceRoot: observation.sourceRoot, }),
        created,
        registeredRoots,
        gitPath,
      },);
      process.stderr
        .write(renderSummary(summary,),);
      rl.debug(`synchronized ignored state into ${String(created.length,)} newly registered worktree(s)`,);
    }
  }
  catch (error: unknown) {
    /**
     * User-facing normalized copy failure.
     */
    const copyFailure = asWorktreeCopyError(error,);
    /**
     * Primitive forwarded-Git failure details, when Git failed.
     */
    const gitFailure = 'failure' in execution
      ? (execution.failure
        .exitCode
        === undefined
            ? {}
            : { exitCode: execution.failure
              .exitCode, })
      : undefined;
    throw new ForwardedGitWorktreeCopyError({
      copyFailureMessage: copyFailure.message,
      ...(gitFailure === undefined
        ? {}
        : { gitFailure, }),
    },);
  }

  if ('failure' in execution)
    throw execution.failure;
}
