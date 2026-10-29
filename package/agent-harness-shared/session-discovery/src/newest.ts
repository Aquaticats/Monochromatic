/**
 Newest PID mapping fallback for session discovery.
 
 @module
 */

import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  readFileStat,
  readTextFile,
} from './io.ts';
import { readByPidDir, } from './mapping.ts';
import { optionalSessionDiscoveryIo, } from './optional-io.ts';
import { SESSION_NOT_FOUND, } from './sentinels.ts';
import type {
  FindByMostRecentOptions,
  MappingCandidate,
  SessionDiscoveryMappingOptions,
} from './types.ts';

//region Logger

/**
 Module logger for newest mapping fallback reads.
 */
const l = tagged({ tag: 'agent-harnesses:session-discovery:newest', },);

//endregion Logger

//region Candidate reads

/**
 Reads one mapping file with its modification time for newest fallback.
 
 @param filename - mapping filename under `byPidDir`
 
 @param byPidDir - directory containing PID mapping files
 
 @param io - optional test IO seam
 
 @param parseMapping - host-owned parser for mapping file contents
 
 @returns mapping candidate, or {@link SESSION_NOT_FOUND} when unreadable
 
 @example
 ```ts
 await readMappingCandidate({ filename: '1234', byPidDir, parseMapping });
 ```
 */
async function readMappingCandidate<TMapping>(
  {
    filename,
    byPidDir,
    io,
    parseMapping,
  }: SessionDiscoveryMappingOptions<TMapping> & { readonly filename: string; },
): Promise<MappingCandidate<TMapping> | typeof SESSION_NOT_FOUND> {
  try {
    /**
     Candidate mapping file path.
     */
    const filePath = join(
      byPidDir,
      filename,
    );
    /**
     File metadata and contents read concurrently.
     */
    const [stats, raw,] = await Promise.all([
      readFileStat({
        path: filePath,
        ...optionalSessionDiscoveryIo(io,),
      },),
      readTextFile({
        path: filePath,
        ...optionalSessionDiscoveryIo(io,),
      },),
    ],);
    return {
      mapping: parseMapping(raw,),
      mtime: stats.mtimeMs,
    };
  }
  catch (error: unknown) {
    tagged({
      tag: readMappingCandidate.name,
      l,
    },)
      .debug(`Could not read mapping candidate ${filename}: ${String(error,)}`,);
    return SESSION_NOT_FOUND;
  }
}

//endregion Candidate reads

//region Fallback scan

/**
 Finds newest mapping as fallback for sandboxed process trees.
 
 @param byPidDir - directory containing PID mapping files
 
 @param io - optional test IO seam
 
 @param parseMapping - host-owned parser for mapping file contents
 
 @returns newest parsed mapping, or {@link SESSION_NOT_FOUND}
 
 @example
 ```ts
 await findByMostRecent({ byPidDir, parseMapping });
 ```
 */
async function findByMostRecent<TMapping>(
  {
    byPidDir,
    io,
    parseMapping,
  }: FindByMostRecentOptions<TMapping>,
): Promise<TMapping | typeof SESSION_NOT_FOUND> {
  /**
   Mapping directory entries to inspect.
   */
  const entries = await readByPidDir({
    byPidDir,
    ...optionalSessionDiscoveryIo(io,),
  },);
  if (entries === SESSION_NOT_FOUND)
    return SESSION_NOT_FOUND;

  /**
   Candidate mappings read concurrently, one slot per directory entry.
   */
  const candidates = await Promise.all(entries.map(
    function readCandidate(filename,): Promise<MappingCandidate<TMapping> | typeof SESSION_NOT_FOUND> {
      return readMappingCandidate({
        filename,
        byPidDir,
        ...optionalSessionDiscoveryIo(io,),
        parseMapping,
      },);
    },
  ),);

  /**
   Most recent readable mapping across PID files.
   */
  const newest = candidates.reduce<MappingCandidate<TMapping> | typeof SESSION_NOT_FOUND>(
    function pickNewer(
      current,
      candidate,
    ): MappingCandidate<TMapping> | typeof SESSION_NOT_FOUND {
      if (candidate === SESSION_NOT_FOUND)
        return current;
      if ((current === SESSION_NOT_FOUND) || (candidate.mtime > current.mtime))
        return candidate;
      return current;
    },
    SESSION_NOT_FOUND,
  );

  return newest === SESSION_NOT_FOUND
    ? SESSION_NOT_FOUND
    : newest.mapping;
}

//endregion Fallback scan

export {
  findByMostRecent,
  readMappingCandidate,
};
