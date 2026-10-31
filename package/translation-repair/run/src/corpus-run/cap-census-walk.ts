import {
  readdir,
  stat,
} from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import { ARTIFACTS_DIR, } from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { RUN_LOG_SUFFIX, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  PROMPT_PAYLOADS_DIR,
  SLICE_CACHE_DIR,
} from './runs-layout.ts';

//region Cap census walk
// WHERE THE CAP CENSUS LOOKS FOR LOGS (ledger P10). The agent root holds
// thousands of logs, some large, so the walk names paths only: it opens
// nothing, and a path it cannot read is counted rather than fatal.
//
// A LOG IS A RECORD FILE, a name ending `.log.jsonl`, which a pass writes under `<run folder>/log/<day>/` through the
// shared logger (`typography/src/run-log.ts`). A `.log` capture of a console is no log here: nothing in this package
// writes one, and one named like a log is a file the walk drops by its name like any other file.
//
// WHAT IS NAMED ON STDERR IS WHAT COULD HAVE HELD A LOG (ledger B991). A
// directory it cannot list may hold one, and a link named like a log may stand
// for one, so both are named and counted as unreadable. A link to nothing under
// any other name holds nothing the walk would read, so it is counted apart and
// not named: on 2026-10-29, 312 of the 315 paths the walk reported were such
// links (browser profile locks, scratch `bin/git` links) and buried the summary.

/**
 How deep below a named directory the walk goes: the run directories under the
 agent root sit one or two levels down.
 */
const MAX_DEPTH = 3;

/**
 Directories that hold no run log and are large: caches, artifacts, payloads.
 */
const SKIPPED_DIRS: ReadonlySet<string> = new Set([
  SLICE_CACHE_DIR,
  ARTIFACTS_DIR,
  'node_modules',
  PROMPT_PAYLOADS_DIR,
],);

/**
 One path the walk has yet to visit.
 */
type PendingPath = {
  /**
   Path to visit.
   */
  readonly path: string;

  /**
   How far below its named root it sits.
   */
  readonly depth: number;

  /**
   Whether its directory listing named it a symbolic link, which a stat follows to a target that may be missing.
   */
  readonly link: boolean;
};

/**
 What visiting one path found.
 */
type Visit =
  | { readonly kind: 'log'; }
    | {
      readonly kind: 'children';
      readonly children: readonly PendingPath[]
    }
    | { readonly kind: 'nothing'; }
    | { readonly kind: 'dangling-link'; }
    | { readonly kind: 'unreadable'; };

/**
 Whether a read failed because the path, or the target of a link, is not there.

 @param error - what the read raised

 @returns Whether it carries the system code for a missing path

 @example
 ```ts
 const missing = isMissingPath({ error, },);
 ```
 */
function isMissingPath({ error, }: { readonly error: unknown; },): boolean {
  if (!Error.isError(error,))
    return false;
  return ('code' in error) && (error.code === 'ENOENT');
}

/**
 Whether a failed visit was a link to nothing that no log can be named by.

 A LINK TO NOTHING HOLDS NO LOG: a directory it stood for would be there to list, and a file it stood for to read. Its
 name decides whether the census would have taken it: only a name ending `.log.jsonl` is a log, so a dangling link under any
 other name (a browser profile's `SingletonLock`, a scratch `bin/git`) cannot change the result. A path named on the
 command line is always reported, whatever it is.

 @param next - path that failed

 @param error - what the read raised

 @returns Whether the failure may be passed over without a word

 @example
 ```ts
 const passedOver = isLogLessDanglingLink({ next, error, },);
 ```
 */
function isLogLessDanglingLink(
  {
    next,
    error,
  }: {
    readonly next: PendingPath;
    readonly error: unknown;
  },
): boolean {
  if ((!next.link) || (next.depth === 0))
    return false;

  /**
   The path the failed visit was for.
   */
  const { path, } = next;
  if (path.endsWith(RUN_LOG_SUFFIX,))
    return false;
  return isMissingPath({ error, },);
}

/**
 Says on stderr that a path could not be read, with what the system answered.

 @param path - path the census could not read

 @param error - what the read raised

 @example
 ```ts
 reportCapCensusUnreadable({ path: '/runs/other-session', error, },);
 ```
 */
export function reportCapCensusUnreadable(
  {
    path,
    error,
  }: {
    readonly path: string;
    readonly error: unknown;
  },
): void {
  console.error(`cap-census: cannot read ${path}: ${String(error,)}`,);
}

/**
 Visits one path of the walk.

 @param next - path and depth

 @returns Whether it is a log, a directory with children to visit, nothing to
 read, or unreadable

 @example
 ```ts
 const found = await visit({ next: { path: '/runs', depth: 0, }, },);
 ```
 */
async function visit({ next, }: { readonly next: PendingPath; },): Promise<Visit> {
  try {
    /**
     What the path is.
     */
    const info = await stat(next.path,);
    if (info.isFile())
      return next.path
        .endsWith(RUN_LOG_SUFFIX,) ? { kind: 'log', } : { kind: 'nothing', };
    if ((!info.isDirectory()) || (next.depth > MAX_DEPTH))
      return { kind: 'nothing', };

    /**
     The directory's entries.
     */
    const entries = await readdir(
      next.path,
      { withFileTypes: true, },
    );
    return {
      kind: 'children',
      children: entries
        .filter(function visited(entry,): boolean {
          // A REGULAR FILE THAT IS NO LOG IS DROPPED BY ITS NAME, before any stat: it would be read as nothing.
          /**
           The entry's name.
           */
          const { name, } = entry;
          return (!SKIPPED_DIRS.has(name,)) && ((!entry.isFile()) || name.endsWith(RUN_LOG_SUFFIX,));
        },)
        .map(function child(entry,): PendingPath {
          return {
            path: join(
              next.path,
              entry.name,
            ),
            depth: next.depth + 1,
            link: entry.isSymbolicLink(),
          };
        },),
    };
  }
  catch (error) {
    // A LINK TO NOTHING UNDER A NAME NO LOG HAS IS PASSED OVER, counted apart and not named: it holds nothing the
    // census could read, and naming it buries the summary (ledger B991).
    if (isLogLessDanglingLink({
      next,
      error,
    },))
      return { kind: 'dangling-link', };

    // A PATH THE WALK CANNOT READ IS COUNTED, NOT FATAL: the agent root holds
    // other sessions' directories this user cannot open.
    reportCapCensusUnreadable({
      path: next.path,
      error,
    },);
    return { kind: 'unreadable', };
  }
}

/**
 Every `.log.jsonl` file under the named paths, files named directly included.

 @param roots - files or directories named on the command line

 @returns Log paths, each once however many named paths reach it, how many
 paths the walk could not read (each named on stderr), and how many dangling
 links under a name no log has it passed over without a word

 @example
 ```ts
 const { logs, unreadable, danglingLinks, } = await capCensusLogsUnder({ roots: ['/runs',], },);
 ```
 */
export async function capCensusLogsUnder(
  { roots, }: { readonly roots: readonly string[]; },
): Promise<{
  readonly logs: readonly string[];
  readonly unreadable: number;
  readonly danglingLinks: number
}> {
  /**
   Paths still to visit, a work stack so the walk needs no recursion.
   */
  const pending: PendingPath[] = roots.map(function rootOf(path,): PendingPath {
    return {
      path,
      depth: 0,
      link: false,
    };
  },);

  /**
   Log files found.
   */
  const logs: string[] = [];

  /**
   Paths the walk could not read.
   */
  const unreadable: string[] = [];

  /**
   Dangling links passed over for a name no log has.
   */
  const danglingLinks: string[] = [];
  for (let next = pending.pop(); next !== undefined; next = pending.pop()) {
    /* oxlint-disable no-await-in-loop -- a work-stack walk: each directory's entries decide what is visited next */
    /**
     What this path turned out to be.
     */
    const found = await visit({ next, },);
    /* oxlint-enable no-await-in-loop */
    if (found.kind === 'log')
      logs.push(next.path,);
    else if (found.kind === 'children')
      pending.push(...found.children,);
    else if (found.kind === 'unreadable')
      unreadable.push(next.path,);
    else if (found.kind === 'dangling-link')
      danglingLinks.push(next.path,);
    // A PATH THAT IS NEITHER, a file other than a log or a directory past the
    // depth, has nothing to read.
  }
  // A LOG REACHED TWICE IS READ ONCE: a directory and a log inside it, or one
  // log named twice, would otherwise count each of its calls twice and move the
  // 99th percentile the cap rule reads.
  return {
    logs: [...new Map(logs.map(function byResolved(path,): readonly [
      string,
      string
    ] {
      return [
        resolve(path,),
        path,
      ];
    },),)
      .values(),],
    unreadable: unreadable.length,
    danglingLinks: danglingLinks.length,
  };
}

//endregion Cap census walk
