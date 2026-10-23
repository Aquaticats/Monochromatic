/**
 Tests for the translate lane not buying a slice again in a later attempt of the
 same pass when the bench that decides it could not reach its floor and has not
 grown.

 XingZ6020 (2026-10-22) tried slices 88 to 105 in one attempt with the select
 bench at one reachable seat of seven; every one ended unfilled for want of a
 second ballot, and the next attempt asked all eighteen again.

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
 Runs the lane over the insertion once.

 @param memory - what earlier attempts of the pass left, shared across calls
 @param selectReachable - select judges a wet provider would serve, absent when the hook counted none
 @param calls - sink counting every call
 @param lines - sink for every line written

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
  }: {
    readonly memory: Map<string, unknown>;
    readonly selectReachable?: number;
    readonly calls: { count: number; };
    readonly lines: string[];
  },
) {
  return await translateDocument({
    client: decliningClient({ calls, },),
    prepared: insertionOnlyPair(),
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
    ...((selectReachable === undefined)
      ? {}
      : { beforeSlice: async (): Promise<TranslateSliceSeating> => ({ selectReachable, }), }),
  } as Parameters<typeof translateDocument>[0],);
}

await describe({
  name: 'a slice that ended unfilled on a select bench below the floor',
  concurrency: 1,
  children: [
    it({
      name: 'IS NOT ASKED AGAIN by a later attempt at the same reach: no call, the unfilled outcome and its reason '
        + 'kept, one line saying why',
      fn: async () => {
        const memory = new Map<string, unknown>();
        const first = { calls: { count: 0, }, lines: [] as string[], };
        const second = { calls: { count: 0, }, lines: [] as string[], };
        const firstResult = await attempt({ memory, selectReachable: 1, ...first, },);
        const secondResult = await attempt({ memory, selectReachable: 1, ...second, },);
        expect({
          firstAsked: first.calls.count > 0,
          firstReasons: firstResult.unfilled.map(function reasonOf(slice,): string {
            return slice.reason;
          },),
          secondCalls: second.calls.count,
          secondReasons: secondResult.unfilled.map(function reasonOf(slice,): string {
            return slice.reason;
          },),
          secondStatus: secondResult.status,
          saidWhy: second.lines.filter(function isWhy(line,): boolean {
            return line.includes('not asked again',);
          },).length,
          findingKept: secondResult.findings.some(function isAbsence(finding,): boolean {
            return finding.includes('chunk 0',);
          },),
          findingNamed: secondResult.findings.some(function isNotAsked(finding,): boolean {
            return finding.startsWith('translate-not-asked-again',);
          },),
        },).toEqual({
          firstAsked: true,
          firstReasons: ['no-candidate-backed',],
          secondCalls: 0,
          secondReasons: ['no-candidate-backed',],
          secondStatus: 'unfilled',
          saidWhy: 1,
          findingKept: true,
          findingNamed: true,
        },);
      },
    },),
    it({
      name: 'SAYS WHY ONCE PER SLICE across the attempts that skip it, not once per attempt',
      fn: async () => {
        const memory = new Map<string, unknown>();
        await attempt({ memory, selectReachable: 1, calls: { count: 0, }, lines: [], },);
        const second = { calls: { count: 0, }, lines: [] as string[], };
        const third = { calls: { count: 0, }, lines: [] as string[], };
        await attempt({ memory, selectReachable: 1, ...second, },);
        await attempt({ memory, selectReachable: 1, ...third, },);
        expect({
          secondSaid: second.lines.filter(function isWhy(line,): boolean {
            return line.includes('not asked again',);
          },).length,
          thirdCalls: third.calls.count,
          thirdSaid: third.lines.filter(function isWhy(line,): boolean {
            return line.includes('not asked again',);
          },).length,
        },).toEqual({
          secondSaid: 1,
          thirdCalls: 0,
          thirdSaid: 0,
        },);
      },
    },),
    it({
      name: 'IS ASKED AGAIN once the select bench has grown past what it read when the slice ended',
      fn: async () => {
        const memory = new Map<string, unknown>();
        await attempt({ memory, selectReachable: 0, calls: { count: 0, }, lines: [], },);
        const grown = { calls: { count: 0, }, lines: [] as string[], };
        await attempt({ memory, selectReachable: 1, ...grown, },);
        expect(grown.calls.count > 0,).toBe(true,);
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
      name: 'IS ASKED AS BEFORE where no reach was counted, and a pass that shares no memory forgets it',
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
          freshAsked: fresh.calls.count > 0,
        },).toEqual({
          recorded: 0,
          uncountedAsked: true,
          freshAsked: true,
        },);
      },
    },),
  ],
},);
