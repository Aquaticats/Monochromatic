/**
 What:
  Sums an `strace` record of one traced commit: wall time, the calls that force data to disk, bytes moved, files made and
  renamed, processes started, and where the call time went by call and by path.
 Why:
  A paired timing says that a wrapped commit took longer than the native one; these counts say which work the wrapper added,
  and they do not depend on how fast the disk happened to be, so the same work shows in them on a quiet host. The record is
  made by `straceArguments` in `native-perf-git.ts`: one line per call, with the process, the time the call began, the path
  behind every file descriptor and the time the call took.

 In TS you'd write (pseudocode):
 ```ts
 const summary = summarizeTrace(await readFile('commit-service-0-wrapped.strace', 'utf8'));
 summary.syncs.count; // fsync, fdatasync, syncfs, sync and sync_file_range calls
 ```
 */

/**
 Calls that ask the kernel to make data durable before they return.
 */
export const SYNC_CALLS: ReadonlySet<string> = new Set([
  'fsync',
  'fdatasync',
  'syncfs',
  'sync',
  'sync_file_range',
]);

/**
 Calls that return how many bytes they read.
 */
const READ_CALLS: ReadonlySet<string> = new Set([
  'read',
  'pread64',
  'readv',
  'preadv',
  'recvfrom',
  'recvmsg',
]);

/**
 Calls that return how many bytes they wrote.
 */
const WRITE_CALLS: ReadonlySet<string> = new Set([
  'write',
  'pwrite64',
  'writev',
  'pwritev',
  'sendto',
  'sendmsg',
  'copy_file_range',
  'sendfile',
]);

/**
 Calls that move a name.
 */
const RENAME_CALLS: ReadonlySet<string> = new Set([
  'rename',
  'renameat',
  'renameat2',
]);

/**
 Calls that remove a name.
 */
const REMOVE_CALLS: ReadonlySet<string> = new Set([
  'unlink',
  'unlinkat',
  'rmdir',
]);

/**
 Calls whose first quoted argument is the path they act on.
 */
const PATH_CALLS: ReadonlySet<string> = new Set([
  'openat',
  'open',
  'newfstatat',
  'stat',
  'lstat',
  'statx',
  'access',
  'faccessat2',
  'readlink',
  'readlinkat',
  'mkdir',
  'mkdirat',
  'execve',
  ...RENAME_CALLS,
  ...REMOVE_CALLS,
]);

/**
 How many characters of a started program's arguments a summary keeps; a display length, no behavior depends on it.
 */
const COMMAND_LIMIT = 150;

/**
 One call that was parsed from the record.
 */
export type TracedCall = {
  readonly pid: number;
  readonly time: number;
  readonly name: string;
  readonly args: string;
  readonly result: number | undefined;
  readonly duration: number;
};

/**
 Count and total call time of a group of calls.
 */
export type Group = {
  readonly count: number;
  readonly seconds: number;
};

/**
 What one traced command did.
 */
export type TraceSummary = {
  /**
   Seconds from the first call to the last.
   */
  readonly wallSeconds: number;
  /**
   Calls that force data to disk, by call name.
   */
  readonly syncs: Readonly<Record<string, Group>>;
  /**
   Paths that were synced, by path, with the time those calls took.
   */
  readonly syncedPaths: Readonly<Record<string, Group>>;
  readonly bytesRead: number;
  readonly bytesWritten: number;
  /**
   Files opened for creation that the call made.
   */
  readonly filesCreated: number;
  readonly renames: number;
  readonly removals: number;
  /**
   Programs that started, by program name, as `execve` calls that succeeded.
   */
  readonly processes: Readonly<Record<string, number>>;
  /**
   Total call time by call name.
   */
  readonly byCall: Readonly<Record<string, Group>>;
  /**
   Total call time by path, for the calls that name or carry one.
   */
  readonly byPath: Readonly<Record<string, Group>>;
  /**
   Bytes written by path.
   */
  readonly writtenByPath: Readonly<Record<string, number>>;
  /**
   Bytes read by path.
   */
  readonly readByPath: Readonly<Record<string, number>>;
  /**
   Every program that started, in order, as seconds after the first call and its first arguments.
   */
  readonly commands: readonly string[];
};

/**
 The start of a line: process, Unix time, and the rest.
 */
const LINE = /^(\d+)\s+(\d+(?:\.\d+)?)\s+(.*)$/v;

/**
 A call that finished on its line.
 */
const COMPLETE = /^(\w+)\((.*)\)\s+=\s+(-?\d+|\?)(?:<[^>]*>)?(?:\s.*?)?\s+<(\d+\.\d+)>$/v;

/**
 The first half of a call that another process interrupted.
 */
const UNFINISHED = /^(\w+)\((.*?)\s*<unfinished \.\.\.>$/v;

/**
 The second half of an interrupted call.
 */
const RESUMED = /^<\.\.\. (\w+) resumed>(.*)\)\s+=\s+(-?\d+|\?)(?:<[^>]*>)?(?:\s.*?)?\s+<(\d+\.\d+)>$/v;

/**
 Parse a result into a number.

 @param text - the text after the equals sign

 @returns the number, or undefined for `?`
 */
function resultOf(text: string): number | undefined {
  return text === '?' ? undefined : Number(text);
}

/**
 Parse the lines of an `strace` record into calls, joining a call that was interrupted with its resumption.

 @param text - the record

 @returns the calls that finished, in record order
 */
export function parseCalls(text: string): readonly TracedCall[] {
  /**
   Start of each interrupted call, by process.
   */
  const pending = new Map<number, {
    readonly name: string;
    readonly args: string;
  }>();
  /**
   The calls found.
   */
  const calls: TracedCall[] = [];
  for (const line of text.split('\n')) {
    /**
     The line split into process, time and the rest.
     */
    const head = LINE.exec(line);
    if (head === null) {
      continue;
    }
    const [
      ,
      pidText = '',
      timeText = '',
      rest = '',
    ] = head;
    /**
     Process of the line.
     */
    const pid = Number(pidText);
    /**
     Match of a call that finished on its line.
     */
    const complete = COMPLETE.exec(rest);
    if (complete !== null) {
      calls.push({
        pid,
        time: Number(timeText),
        name: complete[1] ?? '',
        args: complete[2] ?? '',
        result: resultOf(complete[3] ?? '?'),
        duration: Number(complete[4]),
      });
      continue;
    }
    /**
     Match of the start of an interrupted call.
     */
    const unfinished = UNFINISHED.exec(rest);
    if (unfinished !== null) {
      pending.set(
        pid,
        {
          name: unfinished[1] ?? '',
          args: unfinished[2] ?? '',
        },
      );
      continue;
    }
    /**
     Match of the end of an interrupted call.
     */
    const resumed = RESUMED.exec(rest);
    if (resumed !== null) {
      /**
       What the call began with.
       */
      const began = pending.get(pid);
      pending.delete(pid);
      calls.push({
        pid,
        time: Number(timeText),
        name: resumed[1] ?? '',
        args: `${began?.args ?? ''}${resumed[2] ?? ''}`,
        result: resultOf(resumed[3] ?? '?'),
        duration: Number(resumed[4]),
      });
    }
  }
  return calls;
}

/**
 The path a call acts on: the path decoded behind its first file descriptor, or its first quoted argument.

 @param call - the call

 @returns the path, or undefined when the call names none
 */
export function pathOf(call: TracedCall): string | undefined {
  /**
   A first argument that is a descriptor with its path, as `5</repo/.git/index>`.
   */
  const descriptor = /^(?:AT_FDCWD|\d+)<([^>]*)>/v.exec(call.args);
  if ((descriptor !== null) && !PATH_CALLS.has(call.name)) {
    return descriptor[1];
  }
  if (PATH_CALLS.has(call.name)) {
    /**
     The first quoted argument.
     */
    const quoted = /"((?:[^"\\]|\\.)*)"/v.exec(call.args);
    return quoted?.[1];
  }
  return descriptor?.[1];
}

/**
 Add one call's time to a table of groups.

 @param table - table to change, owned by the caller while it sums

 @param key - which group

 @param seconds - call time
 */
function add(
  table: Map<string, { count: number; seconds: number; }>,
  key: string,
  seconds: number,
): void {
  /**
   The group so far.
   */
  const group = table.get(key) ?? {
    count: 0,
    seconds: 0,
  };
  group.count += 1;
  group.seconds += seconds;
  table.set(
    key,
    group,
  );
}

/**
 Sum what one traced command did.

 @param text - the `strace` record

 @returns the summary

 @example
 ```ts
 summarizeTrace(await readFile(path, 'utf8')).syncs;
 ```
 */
export function summarizeTrace(text: string): TraceSummary {
  /**
   Calls of the record.
   */
  const calls = parseCalls(text);
  /**
   Tables the loop fills; they are frozen into plain records when it ends.
   */
  const syncs = new Map<string, { count: number; seconds: number; }>();
  const syncedPaths = new Map<string, { count: number; seconds: number; }>();
  const byCall = new Map<string, { count: number; seconds: number; }>();
  const byPath = new Map<string, { count: number; seconds: number; }>();
  const processes = new Map<string, number>();
  const writtenByPath = new Map<string, number>();
  const readByPath = new Map<string, number>();
  const commands: string[] = [];
  let bytesRead = 0;
  let bytesWritten = 0;
  let filesCreated = 0;
  let renames = 0;
  let removals = 0;
  for (const call of calls) {
    add(
      byCall,
      call.name,
      call.duration,
    );
    /**
     Path the call acts on.
     */
    const path = pathOf(call);
    if (path !== undefined) {
      add(
        byPath,
        path,
        call.duration,
      );
    }
    if (SYNC_CALLS.has(call.name)) {
      add(
        syncs,
        call.name,
        call.duration,
      );
      add(
        syncedPaths,
        path ?? '(none)',
        call.duration,
      );
    }
    /**
     Bytes the call moved, when it succeeded.
     */
    const moved = ((call.result ?? 0) > 0) ? (call.result ?? 0) : 0;
    if (READ_CALLS.has(call.name)) {
      bytesRead += moved;
      readByPath.set(
        path ?? '(none)',
        (readByPath.get(path ?? '(none)') ?? 0) + moved,
      );
    }
    if (WRITE_CALLS.has(call.name)) {
      bytesWritten += moved;
      writtenByPath.set(
        path ?? '(none)',
        (writtenByPath.get(path ?? '(none)') ?? 0) + moved,
      );
    }
    if ((call.name === 'openat') && ((call.result ?? -1) >= 0) && /O_CREAT/v.test(call.args)) {
      filesCreated += 1;
    }
    if (RENAME_CALLS.has(call.name) && ((call.result ?? -1) === 0)) {
      renames += 1;
    }
    if (REMOVE_CALLS.has(call.name) && ((call.result ?? -1) === 0)) {
      removals += 1;
    }
    if ((call.name === 'execve') && (call.result === 0)) {
      /**
       Program that started.
       */
      const program = path?.split('/').at(-1) ?? '(unknown)';
      processes.set(
        program,
        (processes.get(program) ?? 0) + 1,
      );
      commands.push(`${((call.time - (calls.at(0)?.time ?? call.time)) * 1000).toFixed(0)} ms ${(/\[(.*?)\]/v.exec(call.args)?.[1] ?? program).replaceAll(/"-c", "[^"]*",? ?/gv, '').slice(0, COMMAND_LIMIT)}`);
    }
  }
  /**
   Time of the first and the last call.
   */
  const first = calls.at(0);
  const last = calls.at(-1);
  return {
    wallSeconds: (first === undefined) || (last === undefined)
      ? 0
      : (last.time + last.duration) - first.time,
    syncs: Object.fromEntries(syncs),
    syncedPaths: Object.fromEntries(syncedPaths),
    bytesRead,
    bytesWritten,
    filesCreated,
    renames,
    removals,
    processes: Object.fromEntries(processes),
    byCall: Object.fromEntries(byCall),
    byPath: Object.fromEntries(byPath),
    writtenByPath: Object.fromEntries(writtenByPath),
    readByPath: Object.fromEntries(readByPath),
    commands,
  };
}

/**
 Total of a group table.

 @param table - groups by key

 @returns the summed count and call time
 */
export function total(table: Readonly<Record<string, Group>>): Group {
  return Object.values(table)
    .reduce(
      function sum(
        accumulated,
        group,
      ) {
        return {
          count: accumulated.count + group.count,
          seconds: accumulated.seconds + group.seconds,
        };
      },
      {
        count: 0,
        seconds: 0,
      },
    );
}
