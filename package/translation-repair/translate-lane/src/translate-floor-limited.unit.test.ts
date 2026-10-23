/**
 Tests for the translate lane not buying a slice again in a later attempt of the
 same pass when the bench that decides it could not reach its floor and has not
 grown.

 XingZ6020 (2026-10-22) tried slices 88 to 105 in one attempt with the select
 bench at one reachable seat of seven; every one ended unfilled for want of a
 second ballot, and the next attempt asked all eighteen again.

 A passage the archive never translated is bought no candidate at all while
 the select bench reads under the floor of two ballots a winner needs: nothing
 bought could be backed, so the slice keeps the outcome it would have ended
 with and the log says why, once. A slice a stored record, a twin or the archive
 itself can answer is not skipped.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

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
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

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

 @param memory - what earlier attempts of the pass left, shared across calls
 @param selectReachable - select judges a wet provider would serve, absent when the hook counted none
 @param calls - sink counting every call
 @param lines - sink for every line written
 @param backing - whether the judges back the first candidate rather than decline
 @param sliceCache - cache the lane resumes from and persists to, absent for none
 @param content - whether the slice is one the archive translates

 @returns What the lane made of the passage

 @example
 ```ts
 const result = await attempt({ memory: new Map(), selectReachable: 1, calls, lines, },);
 ```
 */
async function attempt(
  {
    memory,
    selectReachable,
    calls,
    lines,
    backing = false,
    sliceCache,
    content = false,
  }: {
    readonly memory: Map<string, unknown>;
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
    floorLimited: memory,
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
    return line.includes('below the floor',) || line.includes('not asked again',);
  },);
}

await describe({
  name: 'a slice that ended unfilled on a select bench below the floor',
  concurrency: 1,
  children: [
    it({
      name: 'BUYS NO CANDIDATE FOR A PASSAGE THE ARCHIVE NEVER TRANSLATED while the select bench reads under the floor: '
        + 'no call, the unfilled outcome it would have ended with, the finding that names the bench and one line saying why',
      fn: async () => {
        const memory = new Map<string, unknown>();
        const sink = { calls: { count: 0, }, lines: [] as string[], };
        const result = await attempt({ memory, selectReachable: 1, ...sink, },);
        expect({
          calls: sink.calls.count,
          reasons: result.unfilled.map(function reasonOf(slice,): string {
            return slice.reason;
          },),
          status: result.status,
          said: whyLines({ lines: sink.lines, },).length,
          saidFloor: sink.lines.some(function names(line,): boolean {
            return line.includes('1 of 6 select judges reachable, below the floor of 2',);
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
        await attempt({ memory: new Map(), selectReachable: 2, ...atFloor, },);
        const uncounted = { calls: { count: 0, }, lines: [] as string[], };
        await attempt({ memory: new Map(), ...uncounted, },);
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
        const result = await attempt({ memory: new Map(), selectReachable: 1, content: true, ...sink, },);
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
          memory: new Map(),
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
          memory: new Map(),
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
          floorLimited: new Map(),
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
      name: 'IS NOT ASKED AGAIN by a later attempt at the same reach, and says nothing more: the unfilled outcome and '
        + 'its reason kept without a call, the finding that names it, and the one line the first attempt wrote',
      fn: async () => {
        const memory = new Map<string, unknown>();
        const first = { calls: { count: 0, }, lines: [] as string[], };
        const second = { calls: { count: 0, }, lines: [] as string[], };
        await attempt({ memory, selectReachable: 1, ...first, },);
        const secondResult = await attempt({ memory, selectReachable: 1, ...second, },);
        expect({
          firstSaid: whyLines({ lines: first.lines, },).length,
          secondCalls: second.calls.count,
          secondReasons: secondResult.unfilled.map(function reasonOf(slice,): string {
            return slice.reason;
          },),
          secondStatus: secondResult.status,
          secondSaid: whyLines({ lines: second.lines, },).length,
          findingKept: secondResult.findings.some(function isAbsence(finding,): boolean {
            return finding.includes('chunk 0',);
          },),
          findingNamed: secondResult.findings.some(function isNotAsked(finding,): boolean {
            return finding.startsWith('translate-not-asked-again',);
          },),
        },).toEqual({
          firstSaid: 1,
          secondCalls: 0,
          secondReasons: ['no-candidate-backed',],
          secondStatus: 'unfilled',
          secondSaid: 0,
          findingKept: true,
          findingNamed: true,
        },);
      },
    },),
    it({
      name: 'SAYS WHY ONCE PER SLICE across the attempts that skip it, not once per attempt',
      fn: async () => {
        const memory = new Map<string, unknown>();
        const first = { calls: { count: 0, }, lines: [] as string[], };
        const second = { calls: { count: 0, }, lines: [] as string[], };
        const third = { calls: { count: 0, }, lines: [] as string[], };
        await attempt({ memory, selectReachable: 1, ...first, },);
        await attempt({ memory, selectReachable: 1, ...second, },);
        await attempt({ memory, selectReachable: 1, ...third, },);
        expect({
          firstSaid: whyLines({ lines: first.lines, },).length,
          secondSaid: whyLines({ lines: second.lines, },).length,
          thirdSaid: whyLines({ lines: third.lines, },).length,
          thirdCalls: third.calls.count,
        },).toEqual({
          firstSaid: 1,
          secondSaid: 0,
          thirdSaid: 0,
          thirdCalls: 0,
        },);
      },
    },),
    it({
      name: 'IS ASKED AGAIN once the select bench has grown to the floor, and not while it has grown but stays under it',
      fn: async () => {
        const memory = new Map<string, unknown>();
        await attempt({ memory, selectReachable: 0, calls: { count: 0, }, lines: [], },);
        const stillUnder = { calls: { count: 0, }, lines: [] as string[], };
        await attempt({ memory, selectReachable: 1, ...stillUnder, },);
        const grown = { calls: { count: 0, }, lines: [] as string[], };
        await attempt({ memory, selectReachable: 2, ...grown, },);
        expect({
          stillUnderCalls: stillUnder.calls.count,
          grownAsked: grown.calls.count > 0,
        },).toEqual({
          stillUnderCalls: 0,
          grownAsked: true,
        },);
      },
    },),
    it({
      name: 'IS NOT RECORDED when the select bench met the floor, since a full panel that backed nothing twice has '
        + 'said something about the slice, and a later attempt may ask it again as it does now',
      fn: async () => {
        const memory = new Map<string, unknown>();
        await attempt({ memory, selectReachable: 2, calls: { count: 0, }, lines: [], },);
        const later = { calls: { count: 0, }, lines: [] as string[], };
        await attempt({ memory, selectReachable: 2, ...later, },);
        expect({
          recorded: memory.size,
          askedAgain: later.calls.count > 0,
        },).toEqual({
          recorded: 0,
          askedAgain: true,
        },);
      },
    },),
    it({
      name: 'IS ASKED AS BEFORE where no reach was counted, and a pass that shares no memory skips the slice all the '
        + 'same, since the floor needs no memory',
      fn: async () => {
        const memory = new Map<string, unknown>();
        await attempt({ memory, calls: { count: 0, }, lines: [], },);
        const uncounted = { calls: { count: 0, }, lines: [] as string[], };
        await attempt({ memory, ...uncounted, },);
        const fresh = { calls: { count: 0, }, lines: [] as string[], };
        await attempt({ memory: new Map(), selectReachable: 1, ...fresh, },);
        expect({
          recorded: memory.size,
          uncountedAsked: uncounted.calls.count > 0,
          freshCalls: fresh.calls.count,
        },).toEqual({
          recorded: 0,
          uncountedAsked: true,
          freshCalls: 0,
        },);
      },
    },),
  ],
},);
