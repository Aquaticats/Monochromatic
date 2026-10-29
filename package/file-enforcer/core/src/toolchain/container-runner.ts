/**
 Bounded container-cell execution shared by the package's matrix runners.

 Every cell runs `podman run --memory=2g --cpus=2 --rm` (AGENTS.md RXI, the
 bounds the rule text names) with subprocess output captured under an explicit
 `maxBuffer` (never unbounded: an unbounded capture drove a matrix driver to
 42 GB RSS on 2026-10-26) and a wall-clock timeout per cell.
 */

import { execFile, } from 'node:child_process';
import { promisify, } from 'node:util';

//region Bounds

/**
 Bytes in one kibibyte.
 */
const BYTES_PER_KIBIBYTE = 1_024;

/**
 Bytes in one mebibyte.
 */
const BYTES_PER_MEBIBYTE: number = BYTES_PER_KIBIBYTE * BYTES_PER_KIBIBYTE;

/**
 Capture cap for one cell's combined output, in mebibytes.
 */
const CAPTURE_CAP_MEBIBYTES = 32;

/**
 Wall-clock cap for one cell, in minutes.
 */
const CELL_TIMEOUT_MINUTES = 20;

/**
 Seconds in one minute.
 */
const SECONDS_PER_MINUTE = 60;

/**
 Milliseconds in one second.
 */
const MILLISECONDS_PER_SECOND = 1_000;

/**
 Explicit capture cap for one cell's combined output.
 Overflow fails the cell loudly instead of growing the driver's heap.
 */
export const MAX_CAPTURED_OUTPUT_BYTES: number = CAPTURE_CAP_MEBIBYTES * BYTES_PER_MEBIBYTE;

/**
 Wall-clock cap for one cell, in milliseconds.
 */
export const CELL_TIMEOUT_MS: number = CELL_TIMEOUT_MINUTES
  * SECONDS_PER_MINUTE
  * MILLISECONDS_PER_SECOND;

/**
 Podman resource flags applied to every container cell.
 */
export const PODMAN_BOUNDS: readonly string[] = [
  '--memory=2g',
  '--cpus=2',
] as const;

//endregion Bounds

//region Cell execution

/**
 Outcome of one bounded subprocess run.
 */
export type BoundedRun = {
  readonly stdout: string;
  readonly stderr: string;
  readonly code: number;
};

/**
 Exit code reported when the capture cap, the timeout, or a failed start
 stopped a command without a numeric exit code of its own.
 */
const STOPPED_COMMAND_EXIT_CODE = 1;

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.

 The generic `promisify(execFile)` form trips `strict-void-return`, because
 Node's overload set includes void-returning callbacks.
 See `doc/troubleshooting/oxlint-promisify-void-return.md`.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

/**
 `execFile` with a promise result instead of a callback.
 */
const execFileAsync = promisifyExecFile(execFile,);

/**
 Reads one `execFile` rejection into the outcome shape.

 @param error - Rejection of the promisified `execFile`

 @returns Captured output and exit code; `code` is `1` when the error carries
 no numeric exit code

 @example
 ```ts
 const outcome = readExecFailure(new Error('spawn failed',),);
 ```
 */
function readExecFailure(error: unknown,): BoundedRun {
  if (!Error.isError(error,)) {
    return {
      stdout: '',
      stderr: String(error,),
      code: STOPPED_COMMAND_EXIT_CODE,
    };
  }
  /**
   Captured stdout the failed command produced before it stopped.
   */
  const stdout = ('stdout' in error) && ((typeof error.stdout) === 'string')
    ? error.stdout
    : '';
  /**
   Captured stderr the failed command produced before it stopped.
   */
  const stderr = ('stderr' in error) && ((typeof error.stderr) === 'string')
    ? error.stderr
    : '';
  /**
   Exit code from the error object, or the generic stop code.
   */
  const code = ('code' in error) && ((typeof error.code) === 'number')
    ? error.code
    : STOPPED_COMMAND_EXIT_CODE;
  return {
    stdout,
    stderr: `${stderr}\n${error.message}`,
    code,
  };
}

/**
 Runs one command with an explicit output cap and timeout.

 @param command - Executable to start

 @param args - Arguments passed to the executable

 @param timeoutMs - Wall-clock cap in milliseconds; absent means
 {@link CELL_TIMEOUT_MS}

 @returns Captured output and exit code; `code` is `1` when the capture cap
 or the timeout stopped the command

 @example
 ```ts
 const outcome = await runBounded({ command: 'echo', args: ['hello',], },);
 ```
 */
export async function runBounded(
  {
    command,
    args,
    timeoutMs = CELL_TIMEOUT_MS,
  }: {
    readonly command: string;
    readonly args: readonly string[];
    readonly timeoutMs?: number;
  },
): Promise<BoundedRun> {
  try {
    /**
     Captured output of a command that exited with code 0.
     */
    const completed = await execFileAsync(
      command,
      [...args],
      {
        timeout: timeoutMs,
        maxBuffer: MAX_CAPTURED_OUTPUT_BYTES,
        encoding: 'utf8',
        killSignal: 'SIGKILL',
      },
    );
    return {
      stdout: completed.stdout,
      stderr: completed.stderr,
      code: 0,
    };
  } catch (error) {
    return readExecFailure(error,);
  }
}

/**
 Runs one bounded podman container cell:
 `podman run --memory=2g --cpus=2 --rm` over the mounted worktree.

 The mount skips a SELinux `:Z` relabel (the repository's volume refuses
 label xattrs, `lsetxattr: operation not permitted`) and passes
 `--security-opt label=disable` instead.

 @param image - Fully tagged image reference to run

 @param command - Shell command executed inside the container

 @param monorepoRoot - Worktree root mounted at `/workspace`

 @returns Captured output and exit code of the cell

 @example
 ```ts
 const outcome = await runContainerCell({ image: 'ubuntu:latest', command: 'true', monorepoRoot: '/repo', },);
 ```
 */
export async function runContainerCell(
  {
    image,
    command,
    monorepoRoot,
  }: {
    readonly image: string;
    readonly command: string;
    readonly monorepoRoot: string;
  },
): Promise<BoundedRun> {
  return await runBounded({
    command: 'podman',
    args: [
      'run',
      '--rm',
      ...PODMAN_BOUNDS,
      // label=disable instead of a :Z relabel: the repository sits on a
      // volume where SELinux label xattrs are refused, which aborts the mount
      // setup before any test runs.
      '--security-opt',
      'label=disable',
      '--volume',
      `${monorepoRoot}:/workspace`,
      image,
      'sh',
      '-c',
      command,
    ],
  },);
}

//endregion Cell execution
