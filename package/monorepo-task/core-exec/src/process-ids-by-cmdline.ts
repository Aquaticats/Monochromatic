/**
 Process ids selected by the exact command line their `/proc` entry reports.

 The scan lists the process table first and reads each listed process's
 `/proc/<pid>/cmdline` second. A process can end between those two steps, which
 is ordinary rather than a failure: Linux answers the read with `ESRCH` once the
 task is released and the open with `ENOENT` once the directory is gone, so a
 listed entry reporting either is simply not there to match.

 @module
 */

//region Vanished-process classification

/**
 Operating-system error codes a `/proc` entry reports once the process it named has ended.

 `fs/proc/base.c` returns `ESRCH` from the `cmdline` read when `get_proc_task` finds no
 task, and the directory lookup answers `ENOENT` once the entry is gone. Every other error
 names a real problem with the scan itself, such as `EACCES` under a restrictive `hidepid`,
 so it must reach the caller instead of reading as a process that ended.
 */
const VANISHED_PROCESS_ERROR_CODES: ReadonlySet<string> = new Set([
  'ENOENT',
  'ESRCH',
],);

/**
 Whether an error says the process behind one `/proc` entry ended while it was read.

 Only a real error value carrying one of {@link VANISHED_PROCESS_ERROR_CODES} counts. A
 thrown plain object, or an error with no code, names something the scan cannot explain as
 an ordinary exit, so the caller sees it.

 @param error - Value a command-line read threw

 @returns `true` for the operating system's vanished-process codes

 @example
 ```ts
 isVanishedProcessError(errnoError('ENOENT')); // true
 isVanishedProcessError(errnoError('ESRCH')); // true
 isVanishedProcessError(errnoError('EACCES')); // false
 ```
 
 @internal
 */
function isVanishedProcessError(error: unknown,): boolean {
  return Error.isError(error,)
    && ('code' in error)
    && ((typeof error.code) === 'string')
    && VANISHED_PROCESS_ERROR_CODES.has(error.code,);
}

//endregion Vanished-process classification

//region Table scan

/**
 What one scan reads from the operating system.

 The two reads stay separate because the process table is listed once and every listed
 entry is read afterwards, so only the per-entry read can fail with a vanished process.

 @example
 ```ts
 const reads: ProcessTableReads = {
   entries: readdirSync('/proc'),
   readCmdline: (entry) => readFileSync(`/proc/${entry}/cmdline`, 'utf8'),
 };
 ```
 */
export type ProcessTableReads = {
  /**
   Directory entries of the process table, in the order listed.
   
   Names that are not process ids, such as `self` or `uptime`, are skipped without a read.
   */
  readonly entries: readonly string[];

  /**
   Reads one entry's `/proc/<entry>/cmdline`.
   
   Throws the operating system's error when the process ended before the read.
   
   @param entry - Directory entry naming the process
   
   @returns Command line text, empty for a process with no argument vector
   */
  readonly readCmdline: (entry: string) => string;
};

/**
 Process ids whose `/proc/<pid>/cmdline` equals `expectedCmdline` exactly.

 The comparison is on the whole argument vector, so a command line that merely mentions the
 text never matches. A listed entry whose process ended before its read is skipped, and any
 other read failure is rethrown.

 @param entries - Process table entries, in listing order

 @param expectedCmdline - Exact command line text a process must report to match

 @param readCmdline - Per-entry read of that process's command line

 @returns Matching process ids in listing order

 @throws Whatever a read threw when it does not name a vanished process

 @example
 ```ts
 processIdsWithCmdline({
   entries: ['1', '2'],
   expectedCmdline: 'sleep\u000030\u0000',
   readCmdline: (entry) => readFileSync(`/proc/${entry}/cmdline`, 'utf8'),
 }); // [2] when only process 2 runs `sleep 30`
 ```
 
 @internal
 */
export function processIdsWithCmdline({
  entries,
  expectedCmdline,
  readCmdline,
}: ProcessTableReads & { readonly expectedCmdline: string; },): readonly number[] {
  return entries
    .filter(function isProcessEntry(entry,): boolean {
      return Number.isInteger(Number(entry,),);
    },)
    .filter(function hasExpectedCmdline(entry,): boolean {
      try {
        return readCmdline(entry,) === expectedCmdline;
      }
      catch (error) {
        if (isVanishedProcessError(error,))
          return false;
        throw error;
      }
    },)
    .map(Number,);
}

//endregion Table scan
