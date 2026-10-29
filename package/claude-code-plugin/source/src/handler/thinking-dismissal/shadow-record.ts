/**
 Append-only shadow record of thinking-dismissal hits.

 The log lives on this machine and carries hashes and indexes only,
 never reasoning text.

 @module
 */

import type { ReadonlyDeep, } from 'type-fest';

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  appendFile,
  mkdir,
  readFile,
} from 'node:fs/promises';
import {
  homedir,
} from 'node:os';
import {
  dirname,
  join,
} from 'node:path';

import type { ThinkingDismissalHit, } from './detect.ts';

import { NOT_PARSED, } from './jsonl.ts';

/**
 Module logger for shadow-record reads and writes.
 */
const loggerForRecord = tagged({ tag: 'thinking-dismissal-record', },);

/**
 Environment variable overriding the shadow log path,
 for tests and disposable runs.
 */
const SHADOW_LOG_ENV = 'MONOCHROMATIC_THINKING_DISMISSAL_LOG' as const;

/**
 Log lines read back for de-duplication,
 so overlapping tail reads across stops do not re-record one hit.
 */
const SEEN_LOG_LINES = 512;

/**
 One shadow record, appended per stop.

 Carries no reasoning text: the log lives on this machine,
 and everything recorded here is safe to read back as metadata.
 */
type ThinkingDismissalRecord = {
  /**
   Session the stop belongs to.
   */
  readonly sessionId: string;

  /**
   Time the record was written.
   */
  readonly time: string;

  /**
   Transcript the scan read.
   */
  readonly transcriptPath: string;

  /**
   Thinking blocks the tail scan examined.
   */
  readonly thinkingBlocks: number;

  /**
   Hits not recorded by an earlier stop of this session.
   */
  readonly hits: readonly ThinkingDismissalHit[];
};

/**
 Parses one shadow-log JSONL line,
 or returns {@link NOT_PARSED} when the line is not a record.

 @param line - One JSONL line

 @returns The parsed record, or {@link NOT_PARSED}

 @example
 ```ts
 parseRecord('{"hits":[]}');
 ```
 */
function parseRecord(line: string,): ThinkingDismissalRecord | typeof NOT_PARSED {
  try {
    /* oxlint-disable-next-line typescript/no-unsafe-type-assertion -- narrowing a machine-written log line after its parse succeeds */
    return JSON.parse(line,) as ThinkingDismissalRecord;
  } catch (error) {
    loggerForRecord.warn(`skipping an unparseable shadow line: ${String(error,)}`,);
    return NOT_PARSED;
  }
}

/**
 Resolves the shadow log path from ambient configuration.

 @param env - Environment to read the override from

 @returns Path of the append-only shadow log

 @example
 ```ts
 shadowLogPath({}); // => '<home>/.local/state/monochromatic/thinking-dismissal/shadow.jsonl'
 ```
 */
function shadowLogPath(env: ReadonlyDeep<NodeJS.ProcessEnv>,): string {
  /**
   Configured override, empty when the environment carries none.
   */
  const override = env[SHADOW_LOG_ENV] ?? '';
  if (override === '') {
    return join(
      homedir(),
      '.local',
      'state',
      'monochromatic',
      'thinking-dismissal',
      'shadow.jsonl',
    );
  }
  return override;
}

/**
 Reads the hit keys an earlier record already carries.

 Bounded to the log's last {@link SEEN_LOG_LINES} lines,
 which covers every hit a tail re-scan can repeat.

 @param logPath - Shadow log to read

 @returns Keys of the form `blockHash:sentenceIndex:pattern`

 @example
 ```ts
 await readRecordedKeys('/state/shadow.jsonl');
 ```
 */
async function readRecordedKeys(logPath: string,): Promise<ReadonlySet<string>> {
  /**
   Keys already recorded by earlier stops.
   */
  const keys = new Set<string>();
  /**
   Log text, empty when no log exists yet.
   */
  let raw = '';
  try {
    raw = await readFile(
      logPath,
      'utf8',
    );
  } catch (error) {
    loggerForRecord.debug(`no shadow log to read back yet: ${String(error,)}`,);
    return keys;
  }
  /**
   Log lines, oldest first; the tail of them is what overlapping rescans repeat.
   */
  const lines = raw.split('\n',)
    .filter(function isNonEmpty(line,) {
      return line !== '';
    },);
  for (const line of lines.slice(-SEEN_LOG_LINES,)) {
    /**
     Parsed record, or the skip sentinel for a malformed line.
     */
    const record = parseRecord(line,);
    if (record === NOT_PARSED) continue;
    for (const hit of record.hits) {
      keys.add(`${hit.blockHash}:${hit.sentenceIndex}:${hit.pattern}`,);
    }
  }
  return keys;
}

/**
 Appends one shadow record to the log,
 dropping hits an earlier record already carries.

 @param record - Record to append

 @param logPath - Shadow log to append to

 @example
 ```ts
 await appendShadowRecord({ record, logPath: '/state/shadow.jsonl' });
 ```
 */
async function appendShadowRecord(
  {
    record,
    logPath,
  }: {
    readonly record: ThinkingDismissalRecord;
    readonly logPath: string;
  },
): Promise<void> {
  /**
   Recorded keys, copied mutable so this run can extend them.
   */
  const recorded = new Set(await readRecordedKeys(logPath,),);
  /**
   Hits this record contributes, in block order.
   */
  const fresh: ThinkingDismissalHit[] = [];
  for (const hit of record.hits) {
    /**
     Deduplication key of the hit.
     */
    const key = `${hit.blockHash}:${hit.sentenceIndex}:${hit.pattern}`;
    if (recorded.has(key)) continue;
    recorded.add(key,);
    fresh.push(hit,);
  }
  /**
   Record as written: the same metadata with only new hits.
   */
  const written: ThinkingDismissalRecord = {
    ...record,
    hits: fresh,
  };
  await mkdir(
    dirname(logPath,),
    { recursive: true, },
  );
  await appendFile(
    logPath,
    `${JSON.stringify(written,)}\n`,
    'utf8',
  );
}

export type { ThinkingDismissalRecord, };

export {
  appendShadowRecord,
  readRecordedKeys,
  SEEN_LOG_LINES,
  SHADOW_LOG_ENV,
  shadowLogPath,
};
