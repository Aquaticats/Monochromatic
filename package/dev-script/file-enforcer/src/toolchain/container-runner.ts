/**
 Bounded container-cell execution shared by the package's matrix runners.

 Every cell runs `podman run --memory=2g --cpus=2 --rm` (AGENTS.md RXI, the
 bounds the rule text names) with subprocess output captured under an explicit
 `maxBuffer` (never unbounded: an unbounded capture drove a matrix driver to
 42 GB RSS on 2026-10-26) and a wall-clock timeout per cell.
 */

import { execFile, } from 'node:child_process';

//region Bounds

/**
 Explicit capture cap for one cell's combined output.
 Overflow fails the cell loudly instead of growing the driver's heap.
 */
export const MAX_CAPTURED_OUTPUT_BYTES: number = 32 * 1024 * 1024;

/**
 Wall-clock cap for one cell, in milliseconds.
 */
export const CELL_TIMEOUT_MS: number = 20 * 60 * 1000;

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
 Runs one command with an explicit output cap and timeout.
 
 @param options - Command, arguments, and optional timeout override
 
 @returns Captured output and exit code; `code` is `1` when the capture cap
 or the timeout stopped the command
 */
export async function runBounded(
  {
    command,
    args,
    timeoutMs = CELL_TIMEOUT_MS,
  }: {
    readonly command: string;
    readonly args: readonly string[];
    readonly timeoutMs?: number | undefined;
  },
): Promise<BoundedRun> {
  return new Promise<BoundedRun>(
    (resolveRun,) => {
      execFile(
        command,
        [...args],
        {
          timeout: timeoutMs,
          maxBuffer: MAX_CAPTURED_OUTPUT_BYTES,
          encoding: 'utf8',
          killSignal: 'SIGKILL',
        },
        (error, stdout, stderr,) => {
          /**
           Exit code from the error object, or 0 when the command succeeded.
           */
          const code = error === null
            ? 0
            : typeof (error as { code?: unknown; })
              .code === 'number'
              ? (error as { code: number; })
                .code
              : 1;
          resolveRun({
            stdout,
            stderr: error === null
              ? stderr
              : `${
                stderr
              }\n${
                (error as Error)
                  .message
              }`,
            code,
          });
        },
      );
    },
  );
}

/**
 Runs one bounded podman container cell:
 `podman run --memory=2g --cpus=2 --rm` over the mounted worktree.
 
 The mount skips a SELinux `:Z` relabel (the repository's volume refuses
 label xattrs, `lsetxattr: operation not permitted`) and passes
 `--security-opt label=disable` instead.
 
 @param options - Image, in-container command, and worktree root to mount
 
 @returns Captured output and exit code of the cell
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
  return runBounded({
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
