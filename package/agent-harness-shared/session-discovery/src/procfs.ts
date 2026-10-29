/**
 Procfs parent-PID reader for session discovery.
 
 @module
 */

import { splitWhitespace, } from '@monochromatic-dev/agent-harness-shared-text-scan/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

import { readTextFile, } from './io.ts';
import { optionalSessionDiscoveryIo, } from './optional-io.ts';
import { SESSION_NOT_FOUND, } from './sentinels.ts';
import type { SessionDiscoveryIo, } from './types.ts';

//region Logger

/**
 Module logger for procfs parent-PID reads.
 */
const l = tagged({ tag: 'agent-harnesses:session-discovery:procfs', },);

//endregion Logger

//region Procfs scanning

/**
 Reads parent PID from Linux `/proc/{pid}/status`.
 
 @param pid - process identifier to inspect
 
 @param io - optional test IO seam
 
 @returns parent PID, or {@link SESSION_NOT_FOUND} when unavailable
 
 @example
 ```ts
 await readParentPid({ pid: 1234 });
 ```
 */
async function readParentPid(
  {
    pid,
    io,
  }: {
    readonly pid: number;
    readonly io?: SessionDiscoveryIo;
  },
): Promise<number | typeof SESSION_NOT_FOUND> {
  if (io?.readParentPid !== undefined)
    return await io.readParentPid(pid,);

  try {
    /**
     Status file contents for process.
     */
    const statusContent = await readTextFile({
      path: `/proc/${String(pid,)}/status`,
      ...optionalSessionDiscoveryIo(io,),
    },);
    /**
     Status line carrying parent process id.
     */
    const ppidLine = statusContent.split('\n',)
      .find(function isPpidLine(line,): boolean {
        return line.startsWith('PPid:',);
      },);

    if (ppidLine === undefined)
      return SESSION_NOT_FOUND;

    /**
     Parsed parent process id from status line.
     */
    const parentPid = Math.trunc(Number(
      splitWhitespace(ppidLine,)[1]
        ?? '0',
    ),);

    return Number.isFinite(parentPid,)
      ? parentPid
      : SESSION_NOT_FOUND;
  }
  catch (error: unknown) {
    tagged({
      tag: readParentPid.name,
      l,
    },)
      .debug(`Could not read parent pid for ${String(pid,)}: ${String(error,)}`,);
    return SESSION_NOT_FOUND;
  }
}

//endregion Procfs scanning

export { readParentPid, };
