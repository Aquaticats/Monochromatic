import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  mkdtempDisposable,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import type { CostDismissalHit, } from './detect.ts';

import { SIZE_DISMISSAL, } from './patterns.ts';
import {
  appendShadowRecord,
  readRecordedKeys,
  type CostDismissalRecord,
} from './shadow-record.ts';

/**
 Scratch directory for this suite's shadow logs, removed when this module's
 scope unwinds.
 */
await using scratchDirectory = await mkdtempDisposable(join(scratchRoot(), 'shadow-record-',),);

/** Fixture root this suite shares; `scratchDirectory` owns its removal. */
const SCRATCH = scratchDirectory.path;

/**
 Hook events two concurrent observations can share.
 */
const ONE_EVENT = 'pre_tool_use' as const;

/**
 Builds one shadow record carrying the given hits under one event.

 @param hookEvent - Event whose view the record belongs to

 @param hits - Hits the observation found

 @returns the record to append

 @example
 ```ts
 buildRecord({ hookEvent: 'post_tool_use', hits: [hit], });
 ```
 */
function buildRecord(
  {
    hookEvent,
    hits,
  }: {
    readonly hookEvent: string;
    readonly hits: readonly CostDismissalHit[];
  },
): CostDismissalRecord {
  return {
    sessionId: 'fixture-session',
    time: '2026-11-01T00:00:00.000Z',
    transcriptPath: join(
      SCRATCH,
      'fixture-transcript.jsonl',
    ),
    hookEvent,
    textBlocks: 1,
    hits,
  };
}

/**
 Reads every hit a log carries, across records.

 @param logPath - Shadow log to read

 @returns hits in append order

 @example
 ```ts
 await readLogHits('/scratch/shadow.jsonl');
 ```
 */
async function readLogHits(logPath: string,): Promise<CostDismissalHit[]> {
  /**
   Log text written by the appends under test.
   */
  const raw = await readFile(
    logPath,
    'utf8',
  );
  /**
   Hits across every record line, in append order.
   */
  const hits: CostDismissalHit[] = [];
  for (const line of raw.split('\n',)) {
    if (line === '') continue;
    const record = JSON.parse(line,) as CostDismissalRecord;
    hits.push(...record.hits,);
  }
  return hits;
}

await describe({
  name: '',
  children: [
    describe({
      name: appendShadowRecord.name,
      children: [
        //region Concurrent appends

        it({
          name: 'records one hit once across concurrent appends',
          fn: async () => {
            /**
             Log the two concurrent appends share.
             */
            const logPath = join(
              SCRATCH,
              'concurrent.jsonl',
            );
            /**
             One observation carrying a single hit,
             which both appends see identically.
             */
            const record = buildRecord({
              hookEvent: ONE_EVENT,
              hits: [{ pattern: SIZE_DISMISSAL, blockHash: 'concurrent-block', sentenceIndex: 0, },],
            },);
            await Promise.all([
              appendShadowRecord({ record, logPath, }),
              appendShadowRecord({ record, logPath, }),
            ]);
            /**
             Hits the log actually carries after both appends.
             */
            const loggedHits = await readLogHits(logPath,);
            expect(loggedHits.length,)
              .toBe(record.hits.length,);
          },
        },),

        //endregion Concurrent appends

        //region Sequential appends

        it({
          name: 'drops a hit an earlier record already carries',
          fn: async () => {
            /**
             Log the two sequential appends share.
             */
            const logPath = join(
              SCRATCH,
              'sequential.jsonl',
            );
            /**
             One observation carrying a single hit,
             appended twice in turn.
             */
            const record = buildRecord({
              hookEvent: ONE_EVENT,
              hits: [{ pattern: SIZE_DISMISSAL, blockHash: 'sequential-block', sentenceIndex: 1, },],
            },);
            await appendShadowRecord({ record, logPath, });
            await appendShadowRecord({ record, logPath, });
            /**
             Hits the log actually carries after both appends.
             */
            const loggedHits = await readLogHits(logPath,);
            expect(loggedHits.length,)
              .toBe(record.hits.length,);
          },
        },),

        it({
          name: 'keeps a separate view per hook event',
          fn: async () => {
            /**
             Log the two event views share.
             */
            const logPath = join(
              SCRATCH,
              'per-event.jsonl',
            );
            /**
             Hook events whose views stay independent.
             */
            const hookEvents = ['pre_tool_use', 'post_tool_use',] as const;
            /**
             One hit shape both events observe.
             */
            const hits = [{ pattern: SIZE_DISMISSAL, blockHash: 'per-event-block', sentenceIndex: 0, },] as const;
            await Promise.all(hookEvents.map(function appendEventView(hookEvent,) {
              return appendShadowRecord({
                record: buildRecord({
                  hookEvent,
                  hits,
                },),
                logPath,
              },);
            },),);
            /**
             Observations expected: one hit-and-event pair per hit per event.
             */
            const expectedObservations = hits.length * hookEvents.length;
            /**
             Hits the log actually carries across both event views.
             */
            const loggedHits = await readLogHits(logPath,);
            expect(loggedHits.length,)
              .toBe(expectedObservations,);
            /**
             Recorded hit-and-event keys the log exposes to later appends.
             */
            const recordedKeys = await readRecordedKeys(logPath,);
            expect(recordedKeys.size,)
              .toBe(expectedObservations,);
          },
        },),

        //endregion Sequential appends
      ],
    },),
  ],
},);
