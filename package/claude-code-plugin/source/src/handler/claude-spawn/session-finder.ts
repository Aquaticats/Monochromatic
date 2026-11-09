/**
 Session identity resolution for the spawn-claude CLI.
 
 Finds the calling Claude session by walking the process tree or falling
 back to the most recently modified PID coordination file.
 
 @module
 */

import {
  findByMostRecent as findSharedByMostRecent,
  findCallingSession as findSharedCallingSession,
  SESSION_NOT_FOUND as SHARED_SESSION_NOT_FOUND,
  walkProcessTreeFrom as walkSharedProcessTreeFrom,
} from '@monochromatic-dev/agent-harness-shared-session-discovery/ts';

import {
  makeOwnedFileParser,
  type OwnedFileParser,
} from '../../runtime/handler-runtime.ts';
import {
  byPidDir,
  type Environment,
  type PidMapping,
} from './paths.ts';

//region Sentinels

/**
 Sentinel returned when a session lookup finds nothing: no coordination file,
 no readable parent PID, or no `.by-pid/` directory.
 
 @example
 ```ts
 if (identity === SESSION_NOT_FOUND) throw new Error('No Claude session found');
 ```
 */
const SESSION_NOT_FOUND: typeof SHARED_SESSION_NOT_FOUND = SHARED_SESSION_NOT_FOUND;

//endregion Sentinels

//region Host mapping parser

/**
 Shared discovery options specialized for Claude spawn PID mapping files.
 */
type ClaudeDiscoveryOptions = {
  /**
   Directory containing Claude spawn PID mapping files.
   */
  readonly byPidDir: string;
  /**
   Parser for Claude spawn PID mapping JSON.
   */
  readonly parseMapping: typeof parsePidMapping;
};

/**
 Builds shared discovery options for a Claude spawn environment.
 
 @param env - environment values controlling mapping directory
 
 @returns shared discovery options for Claude spawn
 
 @example
 ```ts
 claudeDiscoveryOptions({ HOME: '/home/me' });
 ```
 */
function claudeDiscoveryOptions(env: Environment = process.env,): ClaudeDiscoveryOptions {
  return {
    byPidDir: byPidDir(env,),
    parseMapping: parsePidMapping,
  };
}

/**
 Parses Claude spawn PID mapping JSON.
 
 @example
 ```ts
 parsePidMapping('{"sessionId":"s","transcriptPath":"/tmp/s.jsonl"}');
 ```
 */
const parsePidMapping: OwnedFileParser<PidMapping> = makeOwnedFileParser<PidMapping>();

//endregion Host mapping parser

//region Session discovery adapter

/**
 Walks up the process tree starting from a given PID.
 
 @param pid - PID to start the walk from
 
 @param env - environment values controlling mapping directory
 
 @returns first matching mapping, or {@link SESSION_NOT_FOUND}
 
 @example
 ```ts
 const mapping = walkProcessTreeFrom({ pid: process.ppid });
 ```
 */
function walkProcessTreeFrom(
  {
    pid,
    env = process.env,
  }: {
    readonly pid: number;
    readonly env?: Environment;
  },
): Promise<PidMapping | typeof SESSION_NOT_FOUND> {
  return walkSharedProcessTreeFrom({
    pid,
    ...claudeDiscoveryOptions(env,),
  },);
}

/**
 Walks up the process tree from the current process to find the Claude
 session identity by checking each ancestor PID against the `.by-pid/`
 coordination directory.
 
 When invoked via Bash tool, the process tree is:
   Claude -\> [sandbox?] -\> shell -\> spawn-claude
 The SessionStart hook writes `.by-pid/[claudePid]`, so we walk up until we
 find a matching PID file.
 
 @param env - environment values controlling mapping directory
 
 @returns session identity of calling Claude instance, or {@link SESSION_NOT_FOUND} if not found
 
 @example
 ```ts
 const identity = findByProcessTree();
 if (identity !== SESSION_NOT_FOUND) console.log(identity.sessionId);
 ```
 */
function findByProcessTree(env: Environment = process.env,): Promise<PidMapping | typeof SESSION_NOT_FOUND> {
  return walkProcessTreeFrom({
    pid: process.ppid,
    env,
  },);
}

/**
 Scans all `.by-pid/` files and returns the most recently written one.
 
 Fallback for when the process tree walk fails, which happens inside the
 Bash tool sandbox (separate PID namespace, so host PIDs from `.by-pid/`
 don't appear in `/proc`).
 
 @param env - environment values controlling mapping directory
 
 @returns session identity from most recent PID file, or {@link SESSION_NOT_FOUND} if none exist
 
 @example
 ```ts
 const identity = await findByMostRecent();
 if (identity !== SESSION_NOT_FOUND) console.log(identity.sessionId);
 ```
 */
function findByMostRecent(env: Environment = process.env,): Promise<PidMapping | typeof SESSION_NOT_FOUND> {
  return findSharedByMostRecent(claudeDiscoveryOptions(env,),);
}

/**
 Finds the calling Claude session identity.
 
 Tries the process tree walk first (precise, works outside sandbox), then
 falls back to the most recently modified `.by-pid/` file (works inside
 sandbox where PIDs don't match the host namespace).
 
 @param env - environment values controlling mapping directory
 
 @returns session identity, or {@link SESSION_NOT_FOUND} if no coordination files exist
 
 @example
 ```ts
 const identity = findCallingSession();
 if (identity === SESSION_NOT_FOUND) throw new Error('No Claude session found');
 ```
 */
function findCallingSession(env: Environment = process.env,): Promise<PidMapping | typeof SESSION_NOT_FOUND> {
  return findSharedCallingSession({
    startPid: process.ppid,
    ...claudeDiscoveryOptions(env,),
  },);
}

//endregion Session discovery adapter

export {
  findByMostRecent,
  findByProcessTree,
  findCallingSession,
  SESSION_NOT_FOUND,
};
