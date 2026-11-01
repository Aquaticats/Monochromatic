/**
 * Line-level diff computation using `git diff --no-index`.
 *
 * Points git directly at the on-disk artifact files in `canary-lint/`,
 * so no temp files are needed.
 */

import spawn from 'nano-spawn';
import {
  findRootCached,
  GIT_REPOSITORY,
} from '@monochromatic-dev/fs-path-core/ts';
import dedent from 'string-dedent';

/**
 * Single line in a diff result
 */
export type DiffLine = {
  readonly type: 'added' | 'removed' | 'unchanged';
  readonly content: string;
};

/**
 * Thrown when `git diff --no-index` fails in a way that is not the expected
 * "files differ" exit. The differ case exits with status 1 carrying the diff
 * text on `stdout`; this error covers the remaining cases (git missing from
 * PATH, spawn failure, git fatal exit) that yield empty `stdout`, wrapping the
 * underlying spawn error via the standard `Error.options.cause` so it is not
 * silently swallowed.
 *
 * @example
 * ```ts
 * try {
 *   await captureGitDiffStdout({ initialPath, fixPath, },);
 * } catch (error) {
 *   if (error instanceof GitDiffError) console.error(error.cause,);
 * }
 * ```
 */
export class GitDiffError extends Error {
  /**
   * Wrap `message` and the optional `cause`.
   *
   * @param message - human-readable description of the git failure
   *
   * @param options - standard `ErrorOptions`; pass `{ cause }` to chain the underlying spawn error
   */
  constructor(
    message: string,
    options?: Readonly<ErrorOptions>,
  ) {
    super(
      message,
      options,
    );
    this.name = 'GitDiffError';
  }
}

/**
 * Computes a line-level diff between two existing files using git.
 *
 * @param initialPath - absolute path to initial pass source file
 *
 * @param fixPath - absolute path to fix pass source file
 *
 * @returns array of diff lines
 *
 * @example
 * ```ts
 * const lines = await computeDiff({ initialPath: '/path/to/initial/canary.ts', fixPath: '/path/to/fix/canary.ts' });
 * // [{ type: 'removed', content: 'const x = 1;' }, { type: 'added', content: 'const x = 2;' }]
 * ```
 */
/**
 * Captures stdout from `git diff --no-index` for the two paths.
 *
 * git exits with status 1 when the files differ (not an error condition); the
 * thrown subprocess error carries the diff text on `stdout`, so we extract it.
 * Any other failure (git missing, spawn failure, or a git fatal exit) yields
 * empty `stdout` and is rethrown as a {@link GitDiffError} rather than swallowed.
 *
 * @param initialPath - absolute path to initial pass source file
 *
 * @param fixPath - absolute path to fix pass source file
 *
 * @returns raw diff stdout
 *
 * @throws GitDiffError when git fails with no `stdout` on the thrown error
 *
 * @example
 * ```ts
 * const raw = await captureGitDiffStdout({ initialPath, fixPath, });
 * // 'diff --git ... \n@@ -1 +1 @@\n-old\n+new\n'
 * ```
 */
async function captureGitDiffStdout({
  initialPath,
  fixPath,
}: {
  readonly initialPath: string;
  readonly fixPath: string;
},): Promise<string> {
  try {
    // The `git` on PATH is the cli-git wrapper, which rejects any git command
    // whose effective cwd is not the repository root. mise runs this package's
    // build task with cwd set to the package directory, so spawn from the
    // resolved repo root instead. The --no-index paths are absolute, so the cwd
    // shift never changes which files git diffs.
    /**
     * Git repository root, resolved to satisfy cli-git's repo-root guard.
     */
    const gitRoot = await findRootCached({ marker: GIT_REPOSITORY, },);
    /**
     * Spawn result holding stdout on the success path.
     */
    const result = await spawn(
      'git',
      [
        'diff',
        '--no-index',
        '--unified=99999',
        '--no-color',
        initialPath,
        fixPath,
      ],
      { cwd: gitRoot, },
    );
    return result.stdout;
  }
  catch (error: unknown) {
    // `git diff --no-index` exits 1 when the files differ; that is the one
    // expected non-zero exit and the only case carrying the diff text on
    // `stdout`. nano-spawn 2.1.0 always attaches a `stdout` property to its
    // SubprocessError (empty string on spawn failure such as git missing, or
    // on git's own fatal exits), so property presence cannot distinguish the
    // differ case: non-empty `stdout` is the real signal that git produced a
    // diff. An empty string would parse into an empty diff and silently hide a
    // genuine failure, so it must throw instead.
    if (((typeof error) === 'object') && (error !== null)
      && ('stdout' in error)
      && ((typeof error.stdout) === 'string')
      && (error.stdout !== ''))
      return error.stdout;
    throw new GitDiffError(
      dedent`
        git diff --no-index produced no diff text (empty stdout on the thrown error)
        Expected the "files differ" exit (status 1 carrying diff text); the likely cause is git missing from PATH, a spawn failure, or a git fatal error
        initial: ${initialPath}
        fix: ${fixPath}
      `,
      { cause: error, },
    );
  }
}

/**
 * Computes a line-level diff between two existing files using git.
 *
 * `git diff --no-index` exits with 1 when files differ (not an error);
 * `--unified=99999` requests enough context lines to include the entire file.
 *
 * @param initialPath - absolute path to initial pass source file
 *
 * @param fixPath - absolute path to fix pass source file
 *
 * @returns array of diff lines
 *
 * @throws GitDiffError when git fails with no `stdout` on the thrown error
 *
 * @example
 * ```ts
 * const lines = await computeDiff({ initialPath: '/path/to/initial/canary.ts', fixPath: '/path/to/fix/canary.ts', });
 * // [{ type: 'removed', content: 'const x = 1;', }, { type: 'added', content: 'const x = 2;', }]
 * ```
 */
export async function computeDiff({
  initialPath,
  fixPath,
}: {
  readonly initialPath: string;
  readonly fixPath: string;
},): Promise<readonly DiffLine[]> {
  /**
   * Raw unified-diff stdout captured from `git diff --no-index`; the differ case carries the diff text, unexpected git failures throw GitDiffError.
   */
  const stdout = await captureGitDiffStdout({
    initialPath,
    fixPath,
  },);

  return parseDiffOutput(stdout,);
}

/**
 * Parses unified diff output into typed diff lines.
 * Skips the diff header lines (everything before the first `@@` hunk marker).
 *
 * @param output - raw unified diff output from git
 *
 * @returns parsed diff lines
 */
function parseDiffOutput(output: string,): readonly DiffLine[] {
  /**
   * Raw output split into lines so each can be classified against the unified-diff prefixes.
   */
  const lines = output.split('\n',);
  /**
   * Accumulator that collects classified diff lines once the parser is inside a hunk.
   */
  const result: DiffLine[] = [];

  /**
   * Whether we have passed the header and reached actual diff content
   */
  let inHunk = false;

  for (const line of lines) {
    if (line.startsWith('@@',)) {
      inHunk = true;
      continue;
    }
    if (!inHunk)
      continue;

    if (line.startsWith('+',)) {
      result.push({
        type: 'added',
        content: line.slice(1,),
      },);
    }
    else if (line.startsWith('-',)) {
      result.push({
        type: 'removed',
        content: line.slice(1,),
      },);
    }
    else if (line.startsWith(' ',)) {
      result.push({
        type: 'unchanged',
        content: line.slice(1,),
      },);
    }
    // Skip "\ No newline at end of file" and empty trailing lines
  }

  return result;
}
