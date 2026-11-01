/**
 Dependency-free Git boundary for the pre-install scope command. @module
 */

import { execFile, } from 'node:child_process';
import { promisify, } from 'node:util';

import { ScopeError, } from './ci-scope-error.ts';

/**
 Bound captured paths without silently truncating a comparison.
 */
const MAX_OUTPUT_BYTES = 16_777_216;
/**
 A stalled Git invocation must fail the scope step rather than wait indefinitely.
 */
const GIT_TIMEOUT_MILLISECONDS = 30_000;

/**
 Specialize Node's overloaded adapter to its declared execFile contract.
 This preserves the ChildProcess-returning input signature instead of selecting a void callback overload.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;
/**
 Native adapter preserves execFile's stdout, stderr, and rejection details.
 */
const executeFile = promisifyExecFile(execFile,);

/**
 Run Git without a shell and preserve every execution failure.

 @param args - Argument vector whose values cannot become shell syntax.

 @returns Complete stdout only after a successful Git invocation.

 @throws {@link ScopeError} When Git fails, stalls, or exceeds the capture limit.

 @example
 ```ts
 const head = await scopeGit(['rev-parse', '--verify', 'HEAD^{commit}']);
 ```
 */
export async function scopeGit(args: readonly string[],): Promise<string> {
  console.log(`Scope Git: ${args.join(' ',)}`,);
  try {
    /**
     Successful command result; execFile rejects failed commands before this assignment.
     */
    const result = await executeFile(
      'git',
      [...args,],
      {
        encoding: 'utf8',
        maxBuffer: MAX_OUTPUT_BYTES,
        timeout: GIT_TIMEOUT_MILLISECONDS,
        killSignal: 'SIGKILL',
      },
    );
    return result.stdout;
  }
  catch (error: unknown) {
    throw new ScopeError(
      `Cannot establish toml-edit scope: git ${args.join(' ',)} failed. No skip result was written.`,
      { cause: error, },
    );
  }
}
