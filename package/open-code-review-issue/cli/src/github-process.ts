/**
 Fixed-deadline subprocess boundary for GitHub CLI.
 
 @module
 */

import spawn, {
  SubprocessError,
  type Result,
} from 'nano-spawn';

/**
 Fixed child deadline in milliseconds.
 */
const GITHUB_CHILD_DEADLINE_MS = 60_000;

/**
 Captured successful child result used by GitHub boundary.
 */
export type BoundedProcessResult = Pick<
  Result,
  'stdout' | 'stderr' | 'durationMs'
  >;

/**
 Reports forceful fixed-deadline termination.
 */
export class GitHubProcessTimeoutError extends Error {
  /**
   Partial standard output captured before termination.
   */
  public readonly stdout: string;

  /**
   Partial standard error captured before termination.
   */
  public readonly stderr: string;

  /**
   Creates deadline termination failure.
   
   @param stdout - Partial captured standard output.
   
   @param stderr - Partial captured standard error.
   
   @example
   ```ts
   const error = new GitHubProcessTimeoutError({ stdout: '', stderr: '' });
   ```
   */
  public constructor({
    stdout,
    stderr,
  }: {
    readonly stdout: string;
    readonly stderr: string;
  },) {
    super('GitHub CLI child exceeded the fixed one-minute deadline',);
    this.name = 'GitHubProcessTimeoutError';
    this.stdout = stdout;
    this.stderr = stderr;
  }
}

/**
 Reports child launch, exit, or signal failure with captured output.
 */
export class GitHubProcessError extends Error {
  /**
   Captured standard output.
   */
  public readonly stdout: string;

  /**
   Captured standard error.
   */
  public readonly stderr: string;

  /**
   Numeric child exit code when available.
   */
  public readonly exitCode?: number;

  /**
   Creates process failure.
   
   @param message - Safe process diagnostic.
   
   @param stdout - Captured standard output.
   
   @param stderr - Captured standard error.
   
   @param exitCode - Numeric child exit code when available.
   
   @example
   ```ts
   const error = new GitHubProcessError({ message: 'gh failed', stdout: '', stderr: '' });
   ```
   */
  public constructor({
    message,
    stdout,
    stderr,
    exitCode,
  }: {
    readonly message: string;
    readonly stdout: string;
    readonly stderr: string;
    readonly exitCode?: number;
  },) {
    super(message,);
    this.name = 'GitHubProcessError';
    this.stdout = stdout;
    this.stderr = stderr;
    if (exitCode !== undefined) {
      this.exitCode = exitCode;
    }
  }
}

/**
 Converts nano-spawn failure to adapter-owned process error.
 
 @param error - Unknown rejection from process boundary.
 
 @param file - Executable name used only in safe diagnostic.
 
 @throws {@link GitHubProcessTimeoutError} for dedicated deadline abort.
 
 @throws {@link GitHubProcessError} for every other child failure.
 
 @example
 ```ts
 throwProcessFailure({ error, file: 'gh' });
 ```
 */
function throwProcessFailure({
  error,
  file,
}: {
  readonly error: unknown;
  readonly file: string;
},): never {
  if (!(error instanceof SubprocessError)) {
    throw new GitHubProcessError({
      message: `failed to execute ${file}: ${String(error,)}`,
      stdout: '',
      stderr: '',
    },);
  }
  if (error.isCanceled) {
    throw new GitHubProcessTimeoutError({
      stdout: error.stdout,
      stderr: error.stderr,
    },);
  }
  throw new GitHubProcessError({
    message: `GitHub CLI process failed: ${error.message}`,
    stdout: error.stdout,
    stderr: error.stderr,
    ...(error.exitCode === undefined ? {} : { exitCode: error.exitCode, }),
  },);
}

/**
 Exact subprocess request accepted by bounded runner.
 */
export type BoundedProcessRequest = {
  readonly file: string;
  readonly arguments: readonly string[];
  readonly cwd: string;
};

/**
 Injectable bounded process function for consumer-boundary tests.
 */
export type BoundedProcessRunner = (
  request: BoundedProcessRequest,
) => Promise<BoundedProcessResult>;

/**
 Runs one child with ignored stdin, captured output, and forceful deadline.
 
 @param file - Executable path or command name.
 
 @param arguments - Exact argument vector without shell interpolation.
 
 @param cwd - Explicit child working directory.
 
 @returns Captured successful result.
 
 @throws {@link GitHubProcessTimeoutError} after fixed one-minute deadline.
 
 @throws {@link GitHubProcessError} for launch or nonzero child failure.
 
 @example
 ```ts
 await runBoundedProcess({ file: 'gh', arguments: ['--version'], cwd: process.cwd() });
 ```
 */
export async function runBoundedProcess({
  file,
  arguments: commandArguments,
  cwd,
}: BoundedProcessRequest,): Promise<BoundedProcessResult> {
  try {
    return await spawn(
      file,
      commandArguments,
      {
        cwd,
        stdin: 'ignore',
        stdout: 'pipe',
        stderr: 'pipe',
        env: {
          GH_PROMPT_DISABLED: '1',
        },
        signal: AbortSignal.timeout(GITHUB_CHILD_DEADLINE_MS,),
        killSignal: 'SIGKILL',
        windowsHide: true,
      },
    );
  }
  catch (error: unknown) {
    return throwProcessFailure({
      error,
      file,
    });
  }
}
