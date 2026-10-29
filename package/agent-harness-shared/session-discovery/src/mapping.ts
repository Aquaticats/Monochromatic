/**
 PID mapping file reads for session discovery.
 
 @module
 */

import { join, } from 'node:path';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

import {
  readDirectoryEntries,
  readTextFile,
} from './io.ts';
import { optionalSessionDiscoveryIo, } from './optional-io.ts';
import { SESSION_NOT_FOUND, } from './sentinels.ts';
import type {
  ReadPidMappingOptions,
  SessionDiscoveryMappingOptions,
} from './types.ts';

//region Logger

/**
 Module logger for PID mapping file reads.
 */
const l = tagged({ tag: 'agent-harnesses:session-discovery:mapping', },);

//endregion Logger

//region Direct mapping reads

/**
 Reads PID-to-session mapping for one process id.
 
 @param pid - process identifier to map
 
 @param byPidDir - directory containing PID mapping files
 
 @param io - optional test IO seam
 
 @param parseMapping - host-owned parser for mapping file contents
 
 @returns parsed mapping, or {@link SESSION_NOT_FOUND} when absent or invalid
 
 @example
 ```ts
 await readPidMapping({ pid: 1234, byPidDir, parseMapping: JSON.parse });
 ```
 */
async function readPidMapping<TMapping>(
  {
    pid,
    byPidDir,
    io,
    parseMapping,
  }: ReadPidMappingOptions<TMapping>,
): Promise<TMapping | typeof SESSION_NOT_FOUND> {
  try {
    /**
     Mapping file path for candidate process id.
     */
    const pidFilePath = join(
      byPidDir,
      String(pid,),
    );
    /**
     Mapping file text.
     */
    const raw = await readTextFile({
      path: pidFilePath,
      ...optionalSessionDiscoveryIo(io,),
    },);
    return parseMapping(raw,);
  }
  catch (error: unknown) {
    tagged({
      tag: readPidMapping.name,
      l,
    },)
      .debug(`Could not read pid mapping for ${String(pid,)}: ${String(error,)}`,);
    return SESSION_NOT_FOUND;
  }
}

/**
 Reads PID mapping directory entries.
 
 @param byPidDir - directory containing PID mapping files
 
 @param io - optional test IO seam
 
 @returns filenames, or {@link SESSION_NOT_FOUND} when directory is absent
 
 @example
 ```ts
 await readByPidDir({ byPidDir });
 ```
 */
async function readByPidDir(
  {
    byPidDir,
    io,
  }: Pick<SessionDiscoveryMappingOptions<unknown>, 'byPidDir' | 'io'>,
): Promise<readonly string[] | typeof SESSION_NOT_FOUND> {
  try {
    return await readDirectoryEntries({
      path: byPidDir,
      ...optionalSessionDiscoveryIo(io,),
    },);
  }
  catch (error: unknown) {
    tagged({
      tag: readByPidDir.name,
      l,
    },)
      .debug(`Could not read pid mapping directory ${byPidDir}: ${String(error,)}`,);
    return SESSION_NOT_FOUND;
  }
}

//endregion Direct mapping reads

export {
  readByPidDir,
  readPidMapping,
};
