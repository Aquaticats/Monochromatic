/**
 Shared inject check that scans for completed child sessions
 and returns context text for the parent.
 
 Called from hook handlers with two modes:
 - **Consuming** (`consume: true`): renames `.json` to `.reported`,
   preventing future reads. Used by all delivery hooks; first one to
   fire wins.
 - **Non-consuming** (`consume: false`): reads but does not rename.
   Reserved for diagnostic or observability use cases where the result
   should remain available for a consuming hook to pick up later.
 
 @module
 */

import {
  readFile,
  readdir,
  rename,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  parseSpawnState,
  SPAWNS_DIR,
  type SpawnState,
} from './paths.ts';

/**
 Formats a completed spawn state into a human-readable context string.
 
 @param state - spawn state to format
 
 @returns multi-line context string describing completed child session
 
 @example
 ```ts
 const text = formatSpawnResult(state);
 // "Spawned Claude session completed (spawnId: abc-123):\n..."
 ```
 */
function formatSpawnResult(state: SpawnState,): string {
  return [
    `Spawned Claude session completed (spawnId: ${state.spawnId}):`,
    `Session ID: ${state.sessionId}`,
    `Transcript: ${state.transcriptPath}`,
    state.lastMessage
      .length
      > 0
      ? `Last assistant message:\n${state.lastMessage}`
      : 'No assistant message was produced.',
  ]
    .join('\n',);
}

/**
 Sentinel returned by {@link checkCompletedChildren} when no completed child
 is pending delivery.
 
 A unique symbol rather than `null`: the caller narrows on identity
 (`=== NOTHING_TO_REPORT`), keeping the context string free of a nullish union.
 */
const NOTHING_TO_REPORT: unique symbol = Symbol('claude-spawn/nothing-to-report',);

/**
 Module logger for the completed-child scan.
 */
const loggerForSpawns = tagged({ tag: 'claude-spawn-inject', },);

/**
 Lists the filenames in the spawns coordination directory.
 
 @returns directory entries, or {@link NOTHING_TO_REPORT} when the directory is
   missing or unreadable (there is then nothing to deliver)
 
 @example
 ```ts
 const entries = readSpawnsDir();
 ```
 */
async function readSpawnsDir(): Promise<readonly string[] | typeof NOTHING_TO_REPORT> {
  try {
    return await readdir(SPAWNS_DIR,);
  }
  catch (error: unknown) {
    // No spawns directory means no child was ever spawned for this HOME, which
    // is the ordinary case; the caught value still goes to the log so a genuine
    // permission or I/O failure is distinguishable from an absent directory.
    tagged({
      tag: readSpawnsDir.name,
      l: loggerForSpawns,
    },)
      .debug(`no spawns directory to scan: ${String(error,)}`,);
    return NOTHING_TO_REPORT;
  }
}

/**
 Whether a per-file scan produced deliverable context.
 
 @param result - scan result for one file
 
 @returns true when the result contains context text
 */
function hasReport(result: string | typeof NOTHING_TO_REPORT,): result is string {
  return result !== NOTHING_TO_REPORT;
}

/**
 Reads one spawn-state file and consumes it when it belongs to a stopped child.
 
 @param filename - candidate filename under {@link SPAWNS_DIR}
 
 @param parentSessionId - session identifier of calling session
 
 @param consume - whether to rename matched files to `.reported`
 
 @returns formatted child result, or {@link NOTHING_TO_REPORT} when not deliverable
 */
async function readCompletedChild(
  {
    filename,
    parentSessionId,
    consume,
  }: {
    readonly filename: string;
    readonly parentSessionId: string;
    readonly consume: boolean;
  },
): Promise<string | typeof NOTHING_TO_REPORT> {
  if (!filename.endsWith('.json',))
    return NOTHING_TO_REPORT;

  /**
   Absolute path to the candidate spawn-state file.
   */
  const filePath = join(
    SPAWNS_DIR,
    filename,
  );
  /**
   Sibling `.reported` path used to consume the entry atomically via rename.
   The `.json` suffix guard above means the slice always drops the suffix.
   */
  const reportedPath = join(
    SPAWNS_DIR,
    `${
      filename.slice(
        0,
        -'.json'.length,
      )
    }.reported`,
  );

  try {
    /**
     Raw JSON for the candidate; parsed below to recover the spawn state.
     */
    const raw = await readFile(
      filePath,
      'utf8',
    );
    /**
     Parsed spawn state used to filter by parent session and stopped status.
     */
    const state = parseSpawnState(raw,);

    if (state.parentSessionId
      !== parentSessionId)
      return NOTHING_TO_REPORT;

    if (state.status
      !== 'stopped')
      return NOTHING_TO_REPORT;

    if (consume) {
      try {
        await rename(
          filePath,
          reportedPath,
        );
      }
      catch (error: unknown) {
        /**
         Another hook invocation already renamed this file, which is the ordinary
         race this rename exists to lose.
         */
        tagged({
          tag: readCompletedChild.name,
          l: loggerForSpawns,
        },)
          .debug(`spawn state already consumed by another hook: ${String(error,)}`,);
        return NOTHING_TO_REPORT;
      }
    }

    return formatSpawnResult(state,);
  }
  catch (error: unknown) {
    // A corrupt or unreadable state file holds a completed child's result that
    // will now never be delivered, so this is reported rather than skipped
    // silently: without the name and the error it is indistinguishable from an
    // empty directory.
    tagged({
      tag: readCompletedChild.name,
      l: loggerForSpawns,
    },)
      .warn(`spawn state unreadable, so its result cannot be delivered: ${filename}: ${String(error,)}`,);
    return NOTHING_TO_REPORT;
  }
}

/**
 Scans the spawns directory for children of the given session that have
 stopped and not yet been reported.
 
 When `consume` is true, atomically renames `{spawnId}.json` to
 `{spawnId}.reported` to prevent duplicate injection across concurrent hook
 invocations. When `consume` is false, reads the state without renaming:
 callers should treat the result as best-effort since the file may be
 consumed by a later reliable hook invocation.
 
 @param parentSessionId - session identifier of calling session
 
 @param consume - whether to rename matched files to `.reported`. Use `true`
   from reliable delivery hooks (UserPromptSubmit, Stop), `false` from
   best-effort hooks (PreToolUse, PostToolUse, etc.).
 
 @returns combined context string, or {@link NOTHING_TO_REPORT} if nothing to report
 
 @example
 ```ts
 const context = await checkCompletedChildren({ parentSessionId: 'abc', consume: true });
 ```
 */
async function checkCompletedChildren(
  {
    parentSessionId,
    consume,
  }: {
    readonly parentSessionId: string;
    readonly consume: boolean;
  },
): Promise<string | typeof NOTHING_TO_REPORT> {
  /**
   Filenames in `SPAWNS_DIR`, or {@link NOTHING_TO_REPORT} when the directory is missing or unreadable.
   */
  const entries = await readSpawnsDir();

  if (entries === NOTHING_TO_REPORT)
    return NOTHING_TO_REPORT;

  /**
   Per-file completed-child results in directory order.
   */
  const results = (await Promise.all(
    entries.map(function readEntry(filename,): Promise<string | typeof NOTHING_TO_REPORT> {
      return readCompletedChild({
        filename,
        parentSessionId,
        consume,
      },);
    },),
  )).filter(function keepReport(result,): result is string {
    return hasReport(result,);
  },);

  if (results.length
    === 0)
    return NOTHING_TO_REPORT;

  return results.join('\n\n---\n\n',);
}

export {
  checkCompletedChildren,
  NOTHING_TO_REPORT,
};
