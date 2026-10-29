/**
 Counters and mismatch records for cross-project sharing,
 written per process while verification is on.
 
 @module
 */

import {
  appendFileSync,
  mkdirSync,
  writeFileSync,
} from 'node:fs';
import { join, } from 'node:path';
import { threadId, } from 'node:worker_threads';

import { tagged, } from '@monochromatic-dev/logger-core/ts';

/**
 Shared-analysis telemetry logger.
 */
const l = tagged({ tag: 'shared-analysis-telemetry', },);

/**
 Environment variable that switches verification on when set to `1`.
 */
export const VERIFY_SHARED_ENVIRONMENT_VARIABLE = 'PRPT_VERIFY_SHARED';

/**
 Environment variable naming the directory that receives one counter file per process.
 */
export const VERIFY_SHARED_DIRECTORY_ENVIRONMENT_VARIABLE = 'PRPT_VERIFY_SHARED_DIR';

/**
 Environment variable that adds the closure entries behind each key to the trace when set to `1`.
 */
export const TRACE_CLOSURES_ENVIRONMENT_VARIABLE = 'PRPT_VERIFY_SHARED_CLOSURES';

/**
 Mismatch records kept per process so the counter file stays small.
 */
const MAX_MISMATCH_RECORDS = 200;

/**
 One shared hit whose fresh analysis disagreed with the cached payload.
 */
export type SharedMismatch = {
  readonly fileName: string;
  readonly project: string;
  readonly writerProject: string;
  readonly parts: readonly string[];
};

/**
 Mutable counters of one process.
 */
const state = {
  eligible: 0,
  writes: 0,
  invalid: 0,
  misses: 0,
  hits: 0,
  crossProjectHits: 0,
  verified: 0,
  mismatches: 0,
};

/**
 Sources that kept their per-project entry,
 counted by reason.
 */
const fallbackCounts = new Map<string, number>();

/**
 Mismatch records kept for the counter file.
 */
const mismatchRecords: SharedMismatch[] = [];

/**
 Fallback examples kept per process so the counter file stays small.
 */
const MAX_FALLBACK_EXAMPLES = 60;

/**
 One line per early fallback naming the source and what caused it.
 */
const fallbackExamples: string[] = [];

/**
 Tests whether verification is switched on for this process.
 
 @returns whether the verification environment variable is `1`.
 
 @example
 ```ts
 if (sharedVerificationEnabled()) verify();
 ```
 */
export function sharedVerificationEnabled(): boolean {
  return process.env[VERIFY_SHARED_ENVIRONMENT_VARIABLE] === '1';
}

/**
 Rewrites this process's counter file,
 so the numbers survive however the host ends the process.
 */
function flush(): void {
  /**
   Directory receiving counter files, absent when none was named.
   */
  const directory = process.env[VERIFY_SHARED_DIRECTORY_ENVIRONMENT_VARIABLE];
  if (directory === undefined)
    return;
  try {
    // oxlint-disable-next-line no-restricted-syntax/no-sync -- Verification counters must reach disk before the host process can end.
    mkdirSync(
      directory,
      { recursive: true, },
    );
    // oxlint-disable-next-line no-restricted-syntax/no-sync -- Verification counters must reach disk before the host process can end.
    writeFileSync(
      join(
        directory,
        `${String(process.pid,)}-${String(threadId,)}.json`,
      ),
      JSON.stringify({
        pid: process.pid,
        threadId,
        eligible: state.eligible,
        writes: state.writes,
        invalid: state.invalid,
        misses: state.misses,
        hits: state.hits,
        crossProjectHits: state.crossProjectHits,
        verified: state.verified,
        mismatches: state.mismatches,
        fallbacks: Object.fromEntries(fallbackCounts,),
        fallbackExamples,
        mismatchRecords,
      },),
    );
  }
  catch (error) {
    l.debug(`counter file skipped: ${String(error,)}`,);
  }
}

/**
 Appends one line to a per-process trace file next to the counter file.
 
 @param suffix - Trace file name suffix.
 
 @param record - JSON-safe record written as one line.
 */
function trace({
  suffix,
  record,
}: {
  readonly suffix: string;
  readonly record: unknown;
},): void {
  /**
   Directory receiving counter files, absent when none was named.
   */
  const directory = process.env[VERIFY_SHARED_DIRECTORY_ENVIRONMENT_VARIABLE];
  if (directory === undefined)
    return;
  try {
    // oxlint-disable-next-line no-restricted-syntax/no-sync -- Verification traces must reach disk before the host process can end.
    mkdirSync(
      directory,
      { recursive: true, },
    );
    // oxlint-disable-next-line no-restricted-syntax/no-sync -- Verification traces must reach disk before the host process can end.
    appendFileSync(
      join(
        directory,
        `${String(process.pid,)}-${String(threadId,)}-${suffix}`,
      ),
      `${JSON.stringify(record,)}\n`,
    );
  }
  catch (error) {
    l.debug(`trace skipped: ${String(error,)}`,);
  }
}

/**
 Counts one source whose cross-project key was computed and traces its components.
 
 @param fileName - Source path.
 
 @param project - Configuration path of the project that computed the key.
 
 @param digest - Key digest.
 
 @param parts - Digest of each key component.
 
 @example
 ```ts
 recordSharedEligible({ fileName, project, digest, parts });
 ```
 */
export function recordSharedEligible({
  fileName,
  project,
  digest,
  parts,
}: {
  readonly fileName: string;
  readonly project: string;
  readonly digest: string;
  readonly parts: Readonly<Record<string, string>>;
},): void {
  if (!sharedVerificationEnabled())
    return;
  state.eligible++;
  trace({
    suffix: 'keys.jsonl',
    record: {
      fileName,
      project,
      digest,
      parts,
    },
  },);
  flush();
}

/**
 Traces the closure entries of one source when closure tracing is switched on.
 
 @param fileName - Source path.
 
 @param entries - Closure entries behind the key.
 
 @example
 ```ts
 recordSharedClosure({ fileName, entries });
 ```
 */
export function recordSharedClosure({
  fileName,
  entries,
}: {
  readonly fileName: string;
  readonly entries: readonly string[];
},): void {
  if ((!sharedVerificationEnabled()) || (process.env[TRACE_CLOSURES_ENVIRONMENT_VARIABLE] !== '1'))
    return;
  trace({
    suffix: 'closures.jsonl',
    record: {
      fileName,
      entries,
    },
  },);
}

/**
 Counts one source that keeps its per-project entry.
 
 @param fileName - Source path.
 
 @param reason - Why sharing was refused.
 
 @param detail - The reference or option that caused the refusal.
 
 @example
 ```ts
 recordSharedFallback({ fileName, reason: 'script-source' });
 ```
 */
export function recordSharedFallback({
  fileName,
  reason,
  detail,
}: {
  readonly fileName: string;
  readonly reason: string;
  readonly detail?: string;
},): void {
  if (!sharedVerificationEnabled())
    return;
  fallbackCounts.set(
    reason,
    (fallbackCounts.get(reason,) ?? 0) + 1,
  );
  if (fallbackExamples.length < MAX_FALLBACK_EXAMPLES) {
    fallbackExamples.push(`${reason}: ${fileName}${detail === undefined ? '' : ` (${detail})`}`,);
  }
  l.debug(`${fileName} keeps its per-project entry: ${reason}`,);
  flush();
}

/**
 Counts one shared entry written, one present but invalid, or one absent.
 
 @param kind - Which of the three happened.
 
 @example
 ```ts
 recordSharedStore('write');
 ```
 */
export function recordSharedStore(kind: 'write' | 'invalid' | 'miss',): void {
  if (!sharedVerificationEnabled())
    return;
  if (kind === 'write')
    state.writes++;
  else if (kind === 'invalid')
    state.invalid++;
  else
    state.misses++;
  flush();
}

/**
 Counts one shared hit and, when fresh analysis agreed or disagreed, its outcome.
 
 @param crossProject - Whether another project wrote the entry.
 
 @param mismatch - Disagreement found by fresh analysis,
 absent when the analysis agreed.
 
 @example
 ```ts
 recordSharedHit({ crossProject: true });
 ```
 */
export function recordSharedHit({
  crossProject,
  mismatch,
}: {
  readonly crossProject: boolean;
  readonly mismatch?: SharedMismatch;
},): void {
  if (!sharedVerificationEnabled())
    return;
  state.hits++;
  if (crossProject)
    state.crossProjectHits++;
  if (mismatch === undefined)
    state.verified++;
  else {
    state.mismatches++;
    if (mismatchRecords.length < MAX_MISMATCH_RECORDS)
      mismatchRecords.push(mismatch,);
    /**
     Differing parts rendered on one line.
     */
    const renderedParts = mismatch.parts
      .join('; ',);
    l.error(
      `shared entry for ${mismatch.fileName} disagrees with fresh analysis in ${mismatch.project} (written by ${mismatch.writerProject}): ${renderedParts}`,
    );
  }
  flush();
}
