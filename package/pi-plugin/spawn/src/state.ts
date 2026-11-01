/**
 Spawn-pi coordination state reads and writes.
 
 @module
 */

import {
  mkdir,
  readdir,
  readFile,
  rename,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  byPidDir,
  reportedStatePath,
  type Environment,
  type PidMapping,
  spawnsDir,
  spawnStatePath,
  type SpawnState,
} from './paths.ts';

//region Module logger

/**
 Module logger tagged for spawn-pi coordination-state IO.
 */
const l = tagged({ tag: 'pi-spawn:state', },);

//endregion Module logger

//region Sentinels

/**
 Sentinel returned when no spawn result is ready to inject.
 
 @example
 ```typescript
 if (result === NOTHING_TO_REPORT) return;
 ```
 */
const NOTHING_TO_REPORT: unique symbol = Symbol('spawn-pi/nothing-to-report',);

/**
 Sentinel returned by a per-candidate read for an entry that does not become a parent result.
 
 @example
 ```typescript
 if (candidate === SKIPPED_CHILD) return;
 ```
 */
const SKIPPED_CHILD: unique symbol = Symbol('spawn-pi/child-skipped-during-scan',);

//endregion Sentinels

//region PID mapping

/**
 Serializes state that may expose JSON hooks.
 
 @param value - State value to serialize.
 
 @returns compact JSON text.
 
 @mutates value - `JSON.stringify` may invoke `toJSON`, getters, or proxy traps.
 
 @example
 ```typescript
 serializeStateValue({ ready: true });
 ```
 */
function serializeStateValue(value: unknown,): string {
  return JSON.stringify(value,) ?? 'null';
}

/**
 Writes parent process identity so spawn-pi CLI can find current Pi session.
 
 @param pid - process id to map.
 
 @param mapping - Pi session {@link PidMapping} identity to write.
 
 @param env - {@link Environment} values controlling destination directory.
 
 @mutates mapping - `JSON.stringify` may invoke hooks on PID mapping values.
 
 @example
 ```typescript
 writePidMapping({ pid: process.pid, mapping });
 ```
 */
async function writePidMapping(
  {
    pid,
    mapping,
    env = process.env,
  }: {
    readonly pid: number;
    mapping: PidMapping;
    readonly env?: Environment;
  },
): Promise<void> {
  /**
   Directory receiving PID mapping files.
   */
  const dir = byPidDir(env,);
  await mkdir(
    dir,
    { recursive: true, },
  );
  await writeFile(
    join(
      dir,
      String(pid,),
    ),
    serializeStateValue(mapping,),
  );
}

//endregion PID mapping

//region Spawn state writes

/**
 Writes initial spawn state before terminal process is launched.
 
 @param state - initial {@link SpawnState} to persist.
 
 @param env - {@link Environment} values controlling destination directory.
 
 @mutates state - `JSON.stringify` may invoke hooks on spawn state values.
 
 @example
 ```typescript
 writeInitialSpawnState({ state });
 ```
 */
async function writeInitialSpawnState(
  {
    state,
    env = process.env,
  }: {
    state: SpawnState;
    readonly env?: Environment;
  },
): Promise<void> {
  await mkdir(
    spawnsDir(env,),
    { recursive: true, },
  );
  await writeFile(
    spawnStatePath({
      spawnId: state.spawnId,
      env,
    },),
    serializeStateValue(state,),
  );
}

/**
 Claims a pre-created spawn state for child Pi session.
 
 @param spawnId - spawn identifier from environment.
 
 @param sessionId - child Pi session identifier.
 
 @param sessionFile - child Pi session file path.
 
 @param env - {@link Environment} values controlling source directory.
 
 @example
 ```typescript
 claimSpawn({ spawnId: 'abc', sessionId: 'child', sessionFile: '/tmp/s.jsonl' });
 ```
 */
async function claimSpawn(
  {
    spawnId,
    sessionId,
    sessionFile,
    env = process.env,
  }: {
    readonly spawnId: string;
    readonly sessionId: string;
    readonly sessionFile: string;
    readonly env?: Environment;
  },
): Promise<void> {
  try {
    /**
     Raw state written by CLI before child launched.
     */
    const raw = await readFile(
      spawnStatePath({
        spawnId,
        env,
      },),
      'utf8',
    );
    /* oxlint-disable typescript/no-unsafe-type-assertion -- trusted JSON file written by spawn-pi CLI. */
    /**
     Parsed spawn state before child ownership claim.
     */
    const state = JSON.parse(raw,) as SpawnState;
    /* oxlint-enable typescript/no-unsafe-type-assertion */

    if (state.sessionId !== '')
      return;

    /**
     State after child session ownership claim.
     */
    const updated: SpawnState = {
      ...state,
      sessionId,
      sessionFile,
    };
    await writeFile(
      spawnStatePath({
        spawnId,
        env,
      },),
      JSON.stringify(updated,),
    );
  }
  catch (error: unknown) {
    // Stale environment or consumed state file: continue without claiming.
    tagged({
      tag: claimSpawn.name,
      l,
    },)
      .debug(`Could not claim spawn state for ${spawnId}: ${String(error,)}`,);
  }
}

/**
 Marks child spawn as completed after first child Pi agent loop.
 
 @param spawnId - spawn identifier from environment.
 
 @param sessionId - child Pi session identifier.
 
 @param lastMessage - assistant text to forward to parent.
 
 @param env - {@link Environment} values controlling source directory.
 
 @example
 ```typescript
 completeSpawn({ spawnId: 'abc', sessionId: 'child', lastMessage: 'done' });
 ```
 */
async function completeSpawn(
  {
    spawnId,
    sessionId,
    lastMessage,
    env = process.env,
  }: {
    readonly spawnId: string;
    readonly sessionId: string;
    readonly lastMessage: string;
    readonly env?: Environment;
  },
): Promise<void> {
  try {
    /**
     Raw state claimed by child session.
     */
    const raw = await readFile(
      spawnStatePath({
        spawnId,
        env,
      },),
      'utf8',
    );
    /* oxlint-disable typescript/no-unsafe-type-assertion -- trusted JSON file written by spawn-pi modules. */
    /**
     Parsed spawn state ready for owner check.
     */
    const state = JSON.parse(raw,) as SpawnState;
    /* oxlint-enable typescript/no-unsafe-type-assertion */

    if (state.sessionId !== sessionId)
      return;

    /**
     State after first child result is ready for parent consumption.
     */
    const updated: SpawnState = {
      ...state,
      lastMessage,
      status: 'stopped',
    };
    await writeFile(
      spawnStatePath({
        spawnId,
        env,
      },),
      JSON.stringify(updated,),
    );
  }
  catch (error: unknown) {
    // Missing or already-consumed spawn state: continue without completing.
    tagged({
      tag: completeSpawn.name,
      l,
    },)
      .debug(`Could not complete spawn state for ${spawnId}: ${String(error,)}`,);
  }
}

//endregion Spawn state writes

//region Completed child consumption

/**
 Formats completed child state into parent-visible context.
 
 @param state - completed {@link SpawnState}.
 
 @returns model-visible result text for parent Pi session.
 
 @example
 ```typescript
 formatSpawnResult(state);
 ```
 */
function formatSpawnResult(state: SpawnState,): string {
  /**
   Last assistant text captured from child Pi.
   */
  const { lastMessage, } = state;
  /**
   Whether child Pi produced assistant text worth forwarding.
   */
  const hasLastMessage = lastMessage.length > 0;

  return [
    `Spawned Pi session completed (spawnId: ${state.spawnId}):`,
    `Session ID: ${state.sessionId}`,
    `Session file: ${state.sessionFile}`,
    hasLastMessage
      ? `Last assistant message:\n${lastMessage}`
      : 'No assistant message was produced.',
  ].join('\n',);
}

/**
 Reads spawn state directory, handling missing directories as empty state.
 
 @param env - {@link Environment} values controlling source directory.
 
 @returns directory entries, or {@link NOTHING_TO_REPORT} when directory is absent.
 
 @example
 ```typescript
 readSpawnsDir();
 ```
 */
async function readSpawnsDir(
  env: Environment = process.env,
): Promise<readonly string[] | typeof NOTHING_TO_REPORT> {
  try {
    return await readdir(spawnsDir(env,),);
  }
  catch (error: unknown) {
    // Absent spawn directory is the normal empty state.
    tagged({
      tag: readSpawnsDir.name,
      l,
    },)
      .debug(`No spawn directory to read: ${String(error,)}`,);
    return NOTHING_TO_REPORT;
  }
}

/**
 Detects JSON spawn state filenames without regex.
 
 @param filename - directory entry name.
 
 @returns whether filename names active spawn JSON.
 
 @example
 ```typescript
 isSpawnJsonFilename('abc.json'); // true
 ```
 */
function isSpawnJsonFilename(filename: string,): boolean {
  return filename.endsWith('.json',);
}

/**
 Drops `.json` suffix from spawn state filename.
 
 @param filename - JSON filename to trim.
 
 @returns spawn identifier derived from filename.
 
 @example
 ```typescript
 spawnIdFromJsonFilename('abc.json'); // 'abc'
 ```
 */
function spawnIdFromJsonFilename(filename: string,): string {
  return filename.slice(
    0,
    filename.length - '.json'.length,
  );
}

/**
 Reads one candidate spawn file, optionally consuming it, when it belongs to the target parent.
 
 @param filename - directory entry under active spawn state directory.
 
 @param parentSessionId - parent Pi session identifier to match.
 
 @param consume - whether matching JSON file is renamed to reported marker.
 
 @param env - {@link Environment} values controlling source directory.
 
 @returns formatted result for matching stopped child, or {@link SKIPPED_CHILD} when skipped.
 
 @example
 ```typescript
 await consumeMatchingChild({ filename: 'abc.json', parentSessionId: 'p', consume: true });
 ```
 */
async function consumeMatchingChild(
  {
    filename,
    parentSessionId,
    consume,
    env = process.env,
  }: {
    readonly filename: string;
    readonly parentSessionId: string;
    readonly consume: boolean;
    readonly env?: Environment;
  },
): Promise<string | typeof SKIPPED_CHILD> {
  if (!isSpawnJsonFilename(filename,))
    return SKIPPED_CHILD;

  /**
   Spawn identifier derived from filename.
   */
  const spawnId = spawnIdFromJsonFilename(filename,);
  try {
    /**
     Raw candidate state text.
     */
    const raw = await readFile(
      spawnStatePath({
        spawnId,
        env,
      },),
      'utf8',
    );
    /* oxlint-disable typescript/no-unsafe-type-assertion -- trusted JSON file written by spawn-pi modules. */
    /**
     Parsed candidate state used for parent and status filtering.
     */
    const state = JSON.parse(raw,) as SpawnState;
    /* oxlint-enable typescript/no-unsafe-type-assertion */

    if ((state.parentSessionId !== parentSessionId) || (state.status !== 'stopped'))
      return SKIPPED_CHILD;

    if (consume) {
      try {
        await rename(
          spawnStatePath({
            spawnId,
            env,
          },),
          reportedStatePath({
            spawnId,
            env,
          },),
        );
      }
      catch (error: unknown) {
        // Lost the race to consume this child: skip without reporting it.
        tagged({
          tag: consumeMatchingChild.name,
          l,
        },)
          .debug(`Could not consume spawn state for ${spawnId}: ${String(error,)}`,);
        return SKIPPED_CHILD;
      }
    }

    return formatSpawnResult(state,);
  }
  catch (error: unknown) {
    // Unreadable or malformed candidate state: skip it.
    tagged({
      tag: consumeMatchingChild.name,
      l,
    },)
      .debug(`Could not read spawn state for ${spawnId}: ${String(error,)}`,);
    return SKIPPED_CHILD;
  }
}

/**
 Scans the spawn directory concurrently for stopped children of a parent session and optionally
 consumes them.
 
 @param parentSessionId - parent Pi session identifier to match.
 
 @param consume - whether matching JSON files are renamed to reported markers.
 
 @param env - {@link Environment} values controlling source directory.
 
 @returns joined formatted results, or {@link NOTHING_TO_REPORT}.
 
 @example
 ```typescript
 await checkCompletedChildren({ parentSessionId: 'parent', consume: true });
 ```
 */
async function checkCompletedChildren(
  {
    parentSessionId,
    consume,
    env = process.env,
  }: {
    readonly parentSessionId: string;
    readonly consume: boolean;
    readonly env?: Environment;
  },
): Promise<string | typeof NOTHING_TO_REPORT> {
  /**
   Directory entries under active spawn state directory.
   */
  const entries = await readSpawnsDir(env,);
  if (entries === NOTHING_TO_REPORT)
    return NOTHING_TO_REPORT;

  /**
   Candidate results read concurrently, one slot per directory entry.
   */
  const candidates = await Promise.all(entries.map(
    function readCandidate(filename,): Promise<string | typeof SKIPPED_CHILD> {
      return consumeMatchingChild({
        filename,
        parentSessionId,
        consume,
        env,
      },);
    },
  ),);

  /**
   Matching formatted results with skipped candidates removed.
   */
  const results = candidates.filter(function isPresent(result,): result is string {
    return result !== SKIPPED_CHILD;
  },);

  return results.length === 0
    ? NOTHING_TO_REPORT
    : results.join('\n\n---\n\n',);
}

//endregion Completed child consumption

export {
  checkCompletedChildren,
  claimSpawn,
  completeSpawn,
  formatSpawnResult,
  isSpawnJsonFilename,
  NOTHING_TO_REPORT,
  readSpawnsDir,
  spawnIdFromJsonFilename,
  writeInitialSpawnState,
  writePidMapping,
};
