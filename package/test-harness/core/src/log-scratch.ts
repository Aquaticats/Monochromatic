/**
 Per-run log scratch redirection for module-test processes.

 `redirectLogScratch` points `MONOCHROMATIC_LOG_DIRECTORY` at a fresh scratch
 directory under the injected home before any logger sink resolves its
 destination, so a test run leaves zero log files in any checkout while a
 failed run keeps its scratch logs beside other agent scratch.

 `index.ts` calls it as the entry's first statement. The logger resolves its
 destination lazily on the first record, so that timing still lands every
 record in scratch. No `node:` import appears here because the
 platform-neutral build must load in browsers, where the function does
 nothing at all.

 @module
 */

/**
 Name of the environment variable that redirects the file sink destination.
 */
const LOG_DIRECTORY_VARIABLE = 'MONOCHROMATIC_LOG_DIRECTORY';

/**
 Scratch path segments under the injected home where per-run log directories
 live, matching where agent scratch is expected to accumulate.
 */
const SCRATCH_SEGMENTS = [
  'temp',
  'agent',
  'module-test-logs',
] as const;

/**
 Name of one run's scratch directory, stamped with run start time and process
 id so directories from separate runs never collide.

 @returns Directory name unique to this run

 @example
 ```ts
 const name = runScratchName();
 ```
 */
function runScratchName(): string {
  /**
   Run start time, with every character an ISO stamp reserves for a
   separator replaced so the name stays one path segment.
   */
  const stamp = new Date()
    .toISOString()
    .replaceAll(
    ':',
    '-',
    );
  return `${stamp}-${process.pid}`;
}

/**
 Redirects the file sink destination to per-run scratch, doing nothing when
 the process already chose a destination or runs outside Node.

 @example
 ```ts
 redirectLogScratch();
 ```
 */
export function redirectLogScratch(): void {
  if ((typeof process) === 'undefined')
    return;
  /**
   Live environment object the logger reads for its destination.
   */
  const { env, } = process;
  if (env === undefined)
    return;
  if (env[LOG_DIRECTORY_VARIABLE] !== undefined)
    return;
  /**
   Injected home of the current user, the only root scratch may live under.
   */
  const {
    HOME,
    USERPROFILE,
  } = env;
  /**
   Home directory scratch lives under, preferring the POSIX spelling.
   */
  const home = HOME
    ?? USERPROFILE;
  if (home === undefined)
    return;
  env[LOG_DIRECTORY_VARIABLE] = [
    home,
    ...SCRATCH_SEGMENTS,
    runScratchName(),
  ]
    .join('/',);
}
