/**
 Tests for the translate lane buying no candidate for a passage the archive never translated while the select bench that
 decides it reads under its floor of two ballots.

 XingZ6020 (2026-10-22) tried slices 88 to 105 in one attempt with the select bench at one reachable seat of seven;
 every one ended unfilled for want of a second ballot, and the next attempt asked all eighteen again. The skip reads the
 bench before each purchase and keeps no memory, so every attempt at that reach skips the slice again, records the same
 mark and says why again; the pass once kept a memory of such slices and the owner removed it on 2026-10-29, since the
 payload store already replays the repeat.

 A passage the archive never translated is bought no candidate at all while the select bench reads under the floor of
 two ballots a winner needs: nothing bought could be backed, so the slice keeps the outcome it would have ended with
 and the log says why. A slice a stored record, a twin or the archive itself can answer is not skipped.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { PreparedDocumentPair, } from '@monochromatic-dev/translation-repair-corpus-source';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { makeInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  translateDocument,
  type TranslateModels,
  type TranslateSliceSeating,
} from '../dist/final/node/index.mjs';
import {
  capturingLogger,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import {
  SCRIPTED_STAGE_SETTINGS,
} from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Passage the archive never translated, carried by the insertion slice.
 */
const MISSING_SOURCE = '## 第三节\n\n猫猫也喜欢晒太阳。';

/**
 Roster the lane seats.
 */
const MODELS: TranslateModels = {
  translatorModelIds: [
    SEAT_SYNTHETIC_VISION_WITHHELD,
    SEAT_HYPER_OPENROUTER_VISION_EDITOR,
    SEAT_HYPER_VISION,
  ],
  judgeModelIds: [
    SEAT_SYNTHETIC_VISION_WITHHELD,
    SEAT_HYPER_OPENROUTER_VISION_EDITOR,
    SEAT_HYPER_VISION,
    SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
    SEAT_HYPER_OPENROUTER_UNMEASURED,
    SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  ],
};

/**
 Builds a client whose translators all render the passage and whose judges all
 decline, so a slate is judged twice and backed by nobody, and counts its calls.

 @param calls - sink counting every call

 @returns Client over a scripted panel

 @example
 ```ts
 const client = decliningClient({ calls: { count: 0, }, },);
 ```
 */
function decliningClient({ calls, }: { readonly calls: { count: number; }; },): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by the translate lane',);
    },
    quotas: async () => {
      throw new Error('quotas unused by the translate lane',);
    },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      calls.count += 1;
      /**
       Which sheet this is.
       */
      const schema = request.responseFormat?.json_schema.name ?? '';
      /**
       Reply this sheet gets: a rendering from a translator, a decline from a judge.
       */
      const value: unknown = (schema === 'translation_report')
        ? { translation: '## Section three\n\nThe cat also liked basking in the sun.', }
        : { best: 0, assessment: 'nothing here is supported', };
      if (!request.validate(value,))
        throw new Error(`scripted reply failed the wire guard for ${schema}`,);
      return {
        kind: 'ok',
        value: value as ValueT,
        rawText: JSON.stringify(value,),
      };
    },
  };
}

/**
 Builds a client whose translators all render the passage and whose judges all name the first candidate, so a slate is
 backed by every judge, and counts its calls.

 @param calls - sink counting every call

 @returns Client over a scripted panel

 @example
 ```ts
 const client = backingClient({ calls: { count: 0, }, },);
 ```
 */
function backingClient({ calls, }: { readonly calls: { count: number; }; },): ModelClient {
  return {
    ...decliningClient({ calls, },),
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      calls.count += 1;
      /**
       Which sheet this is.
       */
      const schema = request.responseFormat?.json_schema.name ?? '';
      /**
       Reply this sheet gets: a rendering from a translator, a vote for the first candidate from a judge.
       */
      const value: unknown = (schema === 'translation_report')
        ? { translation: '## Section three\n\nThe cat also liked basking in the sun.', }
        : { best: 1, assessment: 'the first is supported', };
      if (!request.validate(value,))
        throw new Error(`scripted reply failed the wire guard for ${schema}`,);
      return {
        kind: 'ok',
        value: value as ValueT,
        rawText: JSON.stringify(value,),
      };
    },
  };
}

/**
 Prepared pair whose only slice is a passage the archive never translated.

 @returns Pair carrying one insertion

 @example
 ```ts
 const prepared = insertionOnlyPair();
 ```
 */
function insertionOnlyPair(): PreparedDocumentPair {
  /**
   Pair as the pipeline prepares it, whose slicing the insertion replaces.
   */
  const sliced = prepareDocumentPair({
    sourceText: '## 第一节\n\n猫猫在窗台上打盹。\n',
    targetText: '## Section one\n\nThe cat is dozing on the windowsill.\n',
  },);
  return {
    ...sliced,
    slices: [
      {
        source: {
          sliceIndex: 0,
          nodes: [],
          startOffset: 0,
          endOffset: 0,
          text: MISSING_SOURCE,
        },
        target: makeInsertionChunk({
          sliceIndex: 0,
          offset: 0,
        },),
      },
    ],
  };
}

/**
 Prepared pair whose only slice is a passage the archive translates, so the lane has wording to fall back on.

 @returns Pair carrying one content slice

 @example
 ```ts
 const prepared = contentOnlyPair();
 ```
 */
function contentOnlyPair(): PreparedDocumentPair {
  return prepareDocumentPair({
    sourceText: '## 第一节\n\n猫猫在窗台上打盹。\n',
    targetText: '## Section one\n\nThe cat is dozing on the windowsill.\n',
  },);
}

/**
 Runs the lane over the insertion once.

 @param selectReachable - select judges a wet provider would serve, absent when the hook counted none
 @param calls - sink counting every call
 @param lines - sink for every line written
 @param backing - whether the judges back the first candidate rather than decline
 @param sliceCache - cache the lane resumes from and persists to, absent for none
 @param content - whether the slice is one the archive translates

 @returns What the lane made of the passage

 @example
 ```ts
 const result = await attempt({ selectReachable: 1, calls, lines, },);
 ```
 */
async function attempt(
  {
    selectReachable,
    calls,
    lines,
    backing = false,
    sliceCache,
    content = false,
  }: {
    readonly selectReachable?: number;
    readonly calls: { count: number; };
    readonly lines: string[];
    readonly backing?: boolean;
    readonly sliceCache?: Parameters<typeof translateDocument>[0]['sliceCache'];
    readonly content?: boolean;
  },
) {
  return await translateDocument({
    client: backing ? backingClient({ calls, },) : decliningClient({ calls, },),
    prepared: content ? contentOnlyPair() : insertionOnlyPair(),
    models: MODELS,
    insertionAdmission: {
      positions: new Set([0,]),
      findings: [],
    },
    signal: AbortSignal.timeout(HANG_STOP_MS,),
    perCallTimeoutMs: HANG_STOP_MS,
    l: capturingLogger({ messages: lines, },),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
    ...((sliceCache === undefined) ? {} : { sliceCache, }),
    ...((selectReachable === undefined)
      ? {}
      : { beforeSlice: async (): Promise<TranslateSliceSeating> => ({ selectReachable, }), }),
  } as Parameters<typeof translateDocument>[0],);
}

/**
 Lines that say why no call was made for a slice under the floor.

 @param lines - lines the lane wrote

 @returns Those lines

 @example
 ```ts
 const said = whyLines({ lines, },);
 ```
 */
function whyLines({ lines, }: { readonly lines: readonly string[]; },): readonly string[] {
  return lines.filter(function isWhy(line,): boolean {
    return line.includes('below the floor',);
  },);
}

await describe({
  name: 'a passage the archive never translated, on a select bench below the floor',
  concurrency: 1,
  children: [
    it({
      name: 'BUYS NO CANDIDATE FOR A PASSAGE THE ARCHIVE NEVER TRANSLATED while the select bench reads under the '
        + 'floor: no call, the unfilled outcome it would have ended with, the finding that names the bench and one '
        + 'line saying why',
      fn: async () => {
        const sink = { calls: { count: 0, }, lines: [] as string[], };
        const result = await attempt({ selectReachable: 1, ...sink, },);
        expect({
          calls: sink.calls.count,
          reasons: result.unfilled.map(function reasonOf(slice,): string {
            return slice.reason;
          },),
          status: result.status,
          said: whyLines({ lines: sink.lines, },).length,
          saidFloor: sink.lines.some(function names(line,): boolean {
            return line.includes('[skippedBelowFloor] slice 0: 1 of 6 select judges reachable, below the floor of 2',);
          },),
          findingBench: result.findings.includes(
            'stage-bench-below-floor (translate-select reachable 1 of 6, floor 2, no call made)',
          ),
          findingAbsence: result.findings.some(function isAbsence(finding,): boolean {
            return finding.includes('chunk 0',);
          },),
        },).toEqual({
          calls: 0,
          reasons: ['no-candidate-backed',],
          status: 'unfilled',
          said: 1,
          saidFloor: true,
          findingBench: true,
          findingAbsence: true,
        },);
      },
    },),
    it({
      name: 'BUYS AT THE FLOOR, and where no reach was counted: the select bench of two can back a winner, and a hook '
        + 'that counted nothing says nothing against the purchase',
      fn: async () => {
        const atFloor = { calls: { count: 0, }, lines: [] as string[], };
        await attempt({ selectReachable: 2, ...atFloor, },);
        const uncounted = { calls: { count: 0, }, lines: [] as string[], };
        await attempt({ ...uncounted, },);
        expect({
          atFloorAsked: atFloor.calls.count > 0,
          atFloorSaid: whyLines({ lines: atFloor.lines, },).length,
          uncountedAsked: uncounted.calls.count > 0,
        },).toEqual({
          atFloorAsked: true,
          atFloorSaid: 0,
          uncountedAsked: true,
        },);
      },
    },),
    it({
      name: 'BUYS A SLICE THE ARCHIVE TRANSLATES at a reach under the floor as before: the archive stands where the '
        + 'judges decline, so its ending is a settled slice and not the unfilled one this skip stands for',
      fn: async () => {
        const sink = { calls: { count: 0, }, lines: [] as string[], };
        const result = await attempt({ selectReachable: 1, content: true, ...sink, },);
        expect({
          asked: sink.calls.count > 0,
          unfilled: result.unfilled.length,
          said: whyLines({ lines: sink.lines, },).length,
        },).toEqual({
          asked: true,
          unfilled: 0,
          said: 0,
        },);
      },
    },),
    it({
      name: 'DOES NOT SKIP A SLICE A STORED RECORD ANSWERS: the cache is read before the bench is, so a slice an '
        + 'earlier run backed resumes at a reach under the floor, with no call',
      fn: async () => {
        const stored = new Map<string, string>();
        const writer = {
          resumed: new Map<string, never>(),
          persist: async ({ key, serialized, }: { readonly key: string; readonly serialized: string; },) => {
            stored.set(key, serialized,);
            await Promise.resolve();
          },
        };
        const first = { calls: { count: 0, }, lines: [] as string[], };
        const firstResult = await attempt({
          selectReachable: 6,
          backing: true,
          sliceCache: writer,
          ...first,
        },);
        const resumedRecords = new Map(
          [...stored.entries(),].map(function parsed([key, serialized,],) {
            return [key, JSON.parse(serialized,) as never,] as const;
          },),
        );
        const later = { calls: { count: 0, }, lines: [] as string[], };
        const laterResult = await attempt({
          selectReachable: 1,
          backing: true,
          sliceCache: {
            resumed: resumedRecords,
            persist: writer.persist,
          },
          ...later,
        },);
        expect({
          firstSettled: firstResult.status,
          storedRecords: stored.size,
          laterCalls: later.calls.count,
          laterUnfilled: laterResult.unfilled.length,
          laterSaid: whyLines({ lines: later.lines, },).length,
        },).toEqual({
          firstSettled: 'complete',
          storedRecords: 1,
          laterCalls: 0,
          laterUnfilled: 0,
          laterSaid: 0,
        },);
      },
    },),
    it({
      name: 'DOES NOT SKIP A SLICE A TWIN SETTLED: the memo of purchases is read before the bench is, so a slice that '
        + 'asks the question an earlier slice of the run backed reuses that record at a reach under the floor',
      fn: async () => {
        /**
         Pair whose two slices carry the same missing passage.
         */
        const base = insertionOnlyPair();
        const [only,] = base.slices;
        if (only === undefined)
          throw new Error('the fixture pair lost its slice',);
        const pair: PreparedDocumentPair = {
          ...base,
          slices: [
            only,
            {
              source: {
                ...only.source,
                sliceIndex: 1,
              },
              target: makeInsertionChunk({
                sliceIndex: 1,
                offset: 0,
              },),
            },
          ],
        };
        /**
         Reach each slice reads before it starts: the full bench for the first, one seat for the twin.
         */
        const reaches = [
          6,
          1,
        ];
        const counted = { calls: { count: 0, }, asks: 0, };
        const lines: string[] = [];
        const result = await translateDocument({
          client: backingClient({ calls: counted.calls, },),
          prepared: pair,
          models: MODELS,
          insertionAdmission: {
            positions: new Set([
              0,
              1,
            ],),
            findings: [],
          },
          signal: AbortSignal.timeout(HANG_STOP_MS,),
          perCallTimeoutMs: HANG_STOP_MS,
          l: capturingLogger({ messages: lines, },),
          stageSettings: SCRIPTED_STAGE_SETTINGS,
          beforeSlice: async (): Promise<TranslateSliceSeating> => {
            const selectReachable = reaches[Math.min(counted.asks, reaches.length - 1,)] ?? 1;
            counted.asks += 1;
            return await Promise.resolve({ selectReachable, },);
          },
        } as Parameters<typeof translateDocument>[0],);
        expect({
          unfilled: result.unfilled.length,
          said: whyLines({ lines, },).length,
          status: result.status,
        },).toEqual({
          unfilled: 0,
          said: 0,
          status: 'complete',
        },);
      },
    },),
    it({
      name: 'MARKS THE UNFILLED SLICE IT SKIPPED, so the artifact tells a skip for want of a bench from a panel that '
        + 'declined twice',
      fn: async () => {
        const result = await attempt({ selectReachable: 1, calls: { count: 0, }, lines: [], },);
        expect({
          marks: result.unfilled.map(function markOf(slice,) {
            return slice.benchBelowFloor;
          },),
          wording: result.sliceTexts.map(function outcomeOf(text,) {
            return text.outcome;
          },),
        },).toEqual({
          marks: [true,],
          wording: [
            {
              kind: 'unfilled',
              benchBelowFloor: true,
            },
          ],
        },);
      },
    },),
    it({
      name: 'SKIPS AND RECORDS THE SAME SLICE IN EVERY ATTEMPT WITH NO MEMORY BETWEEN THEM: three attempts at a reach '
        + 'of one each make no call, end unfilled as no-candidate-backed with the mark and the finding that names the '
        + 'bench, and say why on one line of their own',
      fn: async () => {
        const sinks = [
          { calls: { count: 0, }, lines: [] as string[], },
          { calls: { count: 0, }, lines: [] as string[], },
          { calls: { count: 0, }, lines: [] as string[], },
        ];
        /**
         What each attempt made of the passage.
         */
        const results = [];
        for (const sink of sinks) {
          // oxlint-disable-next-line no-await-in-loop -- the attempts are consecutive, as a pass reattempts an entry
          results.push(await attempt({ selectReachable: 1, ...sink, },),);
        }
        expect({
          calls: sinks.map(function callsOf(sink,): number {
            return sink.calls.count;
          },),
          said: sinks.map(function saidOf(sink,): number {
            return whyLines({ lines: sink.lines, },).length;
          },),
          statuses: results.map(function statusOf(result,): string {
            return result.status;
          },),
          marks: results.map(function marksOf(result,) {
            return result.unfilled.map(function markOf(slice,) {
              return slice.benchBelowFloor;
            },);
          },),
          findingsNameTheBench: results.map(function namesBench(result,): boolean {
            return result.findings.includes(
              'stage-bench-below-floor (translate-select reachable 1 of 6, floor 2, no call made)',
            );
          },),
          noMemoryFinding: results.some(function hasMemoryFinding(result,): boolean {
            return result.findings.some(function isNotAsked(finding,): boolean {
              return finding.startsWith('translate-not-asked-again',);
            },);
          },),
        },).toEqual({
          calls: [0, 0, 0,],
          said: [1, 1, 1,],
          statuses: ['unfilled', 'unfilled', 'unfilled',],
          marks: [[true,], [true,], [true,],],
          findingsNameTheBench: [true, true, true,],
          noMemoryFinding: false,
        },);
      },
    },),
    it({
      name: 'BUYS THE SLICE ONCE THE SELECT BENCH HAS GROWN TO THE FLOOR, with nothing remembered from the attempts '
        + 'under it: a reach of zero and of one skip it, a reach of two buys it',
      fn: async () => {
        const zero = { calls: { count: 0, }, lines: [] as string[], };
        await attempt({ selectReachable: 0, ...zero, },);
        const one = { calls: { count: 0, }, lines: [] as string[], };
        await attempt({ selectReachable: 1, ...one, },);
        const two = { calls: { count: 0, }, lines: [] as string[], };
        await attempt({ selectReachable: 2, ...two, },);
        expect({
          zeroCalls: zero.calls.count,
          oneCalls: one.calls.count,
          twoAsked: two.calls.count > 0,
        },).toEqual({
          zeroCalls: 0,
          oneCalls: 0,
          twoAsked: true,
        },);
      },
    },),
  ],
},);
