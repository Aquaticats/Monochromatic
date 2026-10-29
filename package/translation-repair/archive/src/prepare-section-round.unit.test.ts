/**
 Tests for when a section-pairing round is bought at all, and for what a
 resumed one republishes.

 ASKED ONLY WHERE THE DETERMINISTIC ALIGNER REFUSED. Measured over the pinned
 corpus, 85 of 92 entries have equal section shape and never reach the aligner,
 and 5 of the remaining 7 align with no refusal. Two entries are ever asked, so
 a gate that leaked would multiply this stage's cost by forty-six.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  parseDocument,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { isSectionPairingWire, } from '@monochromatic-dev/translation-repair-stage';
import { createSyntheticClient, } from '@monochromatic-dev/translation-repair-provider';
import {
  chunkByHeadings,
  pairingQuestionKey,
} from '@monochromatic-dev/translation-repair-slice-reading';
import { buySectionPairing, } from '../dist/final/node/index.mjs';
import { levelCapturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Original with three sections, headed in Chinese.
 */
const SOURCE_TEXT = `## 第一节

猫猫在窗台上打盹。

## 第二节

窗台上有一只鸟。

## 第三节

猫猫也喜欢晒太阳。
`;

/**
 Translation carrying only two of them, so the aligner faces a count mismatch
 it has no affinity to resolve.
 */
const TARGET_TEXT = `## Naps

The cat naps on the windowsill.

## Birds

A bird sits on the windowsill.
`;

/**
 Translation of the same shape, which pairs by index without the aligner
 being consulted at all.
 */
const EQUAL_SHAPE_TARGET = `## Naps

The cat naps on the windowsill.

## Birds

A bird sits on the windowsill.

## Sunbeams

The cat likes the sun too.
`;

/**
 One settled round as the cache stores it, which is what a resume reads back.
 */
type StoredRound = {
  /**
   Correspondences the roster agreed on.
   */
  readonly pairs: readonly {
    readonly source: number;
    readonly target: number;
  }[];

  /**
   What the round reported when it was bought.
   */
  readonly findings: readonly string[];
};

/**
 Reads one stored record back, refusing anything the cache would not write.

 @param serialized - bytes the round persisted

 @returns That round

 @throws Error when the bytes are not a stored round, since a resume built on
 a guess would test the guess rather than the round

 @example
 ```ts
 const stored = storedRoundOf('{"pairs":[],"findings":[]}',);
 ```
 */
function storedRoundOf(serialized: string,): StoredRound {
  /**
   Whatever those bytes hold.
   */
  const parsed: unknown = JSON.parse(serialized,);
  if (!isSectionPairingWire(parsed,))
    throw new Error('stored section round carries no usable pairing',);
  if (!('findings' in parsed))
    throw new Error('stored section round carries no findings list',);
  if (!Array.isArray(parsed.findings,))
    throw new Error('stored section round carries a findings field that is not a list',);
  return parsed as StoredRound;
}

/**
 Roster of two, which is the smallest that can agree or disagree.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
] as const;

/**
 Another roster of two, sharing one seat with `ROSTER`.
 */
const OTHER_ROSTER = [
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
] as const;

/**
 Logger for the round under test.
 */
const l = tagged({ tag: 'prepare-section-round-test', },);

/**
 Pairing both canned voices return, leaving the third original out.
 */
const AGREED_REPLY = '{"pairs":[{"source":0,"target":0},{"source":1,"target":1}]}';

/**
 Builds a client that counts calls and answers each with the same pairing.

 @param reply - body every model returns

 @returns Client over a canned transport, beside the call log

 @example
 ```ts
 const { client, calls, } = countingClient({ reply: AGREED_REPLY, },);
 ```
 */
function countingClient({ reply, }: { readonly reply: string; },) {
  /**
   Calls this client served.
   */
  const calls = { count: 0, };
  return {
    calls,
    client: createSyntheticClient({
      apiKey: 'test-key',
      transport: async function cannedTransport() {
        calls.count += 1;

        // THE CLIENT READS A STREAM, not a completion body: one delta frame and
        // the terminator, which is the smallest well-formed reply.
        return {
          status: 200,
          bodyText: `data: ${
            JSON.stringify({
              choices: [
                {
                  index: 0,
                  delta: { content: reply, },
                },
              ],
            },)
          }\n\ndata: [DONE]\n\n`,
        };
      },
    },),
  };
}

/**
 Reads a document's sections the way the round numbers them.

 @param text - whole document

 @returns Each heading-bounded section's text, in document order

 @example
 ```ts
 const texts = sectionTextsOf({ text: SOURCE_TEXT, },);
 ```
 */
function sectionTextsOf({ text, }: { readonly text: string; },): readonly string[] {
  return chunkByHeadings({ document: parseDocument({ text, },), },)
    .map(function toText(chunk,): string {
      return chunk.text;
    },);
}

/**
 Runs the round over a document pair.

 @param sourceText - whole original

 @param targetText - whole translation

 @param reply - what every canned voice answers

 @param resumed - pairings an earlier run stored

 @param persisted - store this run writes into

 @param modelIds - roster the round asks

 @param logger - logger the round writes to, for a case reading its lines

 @returns What the round settled, beside how many calls it cost

 @example
 ```ts
 const { round, calls, } = await runRound({},);
 ```
 */
async function runRound(
  {
    sourceText = SOURCE_TEXT,
    targetText = TARGET_TEXT,
    reply = AGREED_REPLY,
    resumed = new Map<string, StoredRound>(),
    persisted = new Map<string, string>(),
    modelIds = ROSTER,
    logger = l,
  }: {
    readonly sourceText?: string;
    readonly targetText?: string;
    readonly reply?: string;
    readonly resumed?: ReadonlyMap<string, StoredRound>;
    readonly persisted?: Map<string, string>;
    readonly modelIds?: readonly RosterModelId[];
    readonly logger?: Logger;
  },
) {
  /**
   Client and its call log.
   */
  const {
    client,
    calls,
  } = countingClient({ reply, },);

  /**
   What the round settled.
   */
  const round = await buySectionPairing({
    client,
    modelIds,
    source: parseDocument({ text: sourceText, },),
    target: parseDocument({ text: targetText, },),
    signal: new AbortController().signal,
    exchangeTimeoutMs: HANG_STOP_MS,
    l: logger,
    sectionCache: {
      resumed,
      persist: async function record(
        { key, serialized, },
      ) {
        persisted.set(key, serialized,);
      },
    },
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    round,
    calls,
    persisted,
  };
}

await describe({
  name: buySectionPairing.name,
  children: [
    it({
      name: 'BUYS NOTHING when both sides have the same section shape, because that pairs by index '
        + 'without the aligner being consulted, so there is no refusal to repair and asking would '
        + 'trade a settled answer for an opinion',
      fn: async () => {
        const {
          round,
          calls,
        } = await runRound({ targetText: EQUAL_SHAPE_TARGET, },);
        expect(calls.count,).toBe(0,);
        expect(round.pairing.length,).toBe(0,);
        expect(round.findings.length,).toBe(0,);
      },
    },),

    it({
      name: 'BUYS NOTHING when one side has no sections at all, since there is nothing to pair '
        + 'against',
      fn: async () => {
        const {
          round,
          calls,
        } = await runRound({ targetText: '', },);
        expect(calls.count,).toBe(0,);
        expect(round.pairing.length,).toBe(0,);
      },
    },),

    it({
      name: 'ASKS the roster where the aligner refused, and returns what it agreed on',
      fn: async () => {
        const {
          round,
          calls,
        } = await runRound({},);
        expect(calls.count,).toBe(ROSTER.length,);
        expect(round.pairing,).toEqual([
          {
            source: 0,
            target: 0,
          },
          {
            source: 1,
            target: 1,
          },
        ],);
        expect(round.findings
          .some(function countsVoices(finding,): boolean {
            return finding.startsWith('section-pairing paired',);
          },),).toBe(true,);
      },
    },),

    it({
      name: 'PERSISTS the settled round so a resume buys nothing, and REPUBLISHES its findings off '
        + 'disk rather than reporting a quieter round than the one that was paid for',
      fn: async () => {
        const first = await runRound({},);
        expect(first.persisted
          .size,).toBe(1,);

        /** The stored records, read back as the cache would hand them over. */
        const resumed = new Map([...first.persisted
          .entries(),].map(function toRecord([key, serialized,],): readonly [
            string,
            StoredRound,
          ] {
            return [
              key,
              storedRoundOf(serialized,),
            ];
          },),);
        const second = await runRound({ resumed, },);
        expect(second.calls
          .count,).toBe(0,);
        expect(second.round
          .pairing,).toEqual(first.round
            .pairing,);
        expect(second.round
          .findings,).toEqual(first.round
            .findings,);
      },
    },),

    ...[
      {
        damage: 'NAMES A TRANSLATION SECTION THE DOCUMENT LACKS',
        pairs: [{ source: 0, target: 5, },],
        refusal: 'pairing names translation section 5, and there are 2',
      },
      {
        damage: 'NAMES AN ORIGINAL SECTION THE DOCUMENT LACKS',
        pairs: [{ source: 7, target: 0, },],
        refusal: 'pairing names original section 7, and there are 3',
      },
      {
        damage: 'PAIRS ONE TRANSLATION SECTION WITH TWO ORIGINALS',
        pairs: [{ source: 0, target: 0, }, { source: 1, target: 0, },],
        refusal: 'pairing does not advance on the translation side at position 1',
      },
      {
        damage: 'MOVES BACKWARDS ON THE TRANSLATION SIDE',
        pairs: [{ source: 0, target: 1, }, { source: 1, target: 0, },],
        refusal: 'pairing does not advance on the translation side at position 1',
      },
      {
        damage: 'PAIRS ONE ORIGINAL SECTION WITH TWO TRANSLATIONS',
        pairs: [{ source: 0, target: 0, }, { source: 0, target: 1, },],
        refusal: 'pairing does not advance on the original side at position 1',
      },
    ].map(function damagedCase({ damage, pairs, refusal, },) {
      return it({
        name: `MISSES ON A CACHED SECTION PAIRING THAT ${damage}, warns in the refusal's words, and buys the round again`,
        fn: async () => {
          const fresh = await runRound({},);
          const key = pairingQuestionKey({
            question: 'section',
            sourceTexts: sectionTextsOf({ text: SOURCE_TEXT, },),
            targetTexts: sectionTextsOf({ text: TARGET_TEXT, },),
            pictureContext: '',
            modelIds: ROSTER,
          },);
          const lines: string[] = [];
          const damaged = await runRound({
            resumed: new Map([[key, { pairs, findings: ['an older round',], },],],),
            logger: levelCapturingLogger({ lines, },),
          },);
          expect({
            round: damaged.round,
            calls: damaged.calls.count,
            persisted: damaged.persisted.get(key,),
            warned: lines.filter(function isWarning(line,): boolean {
              return line.startsWith('warn ',);
            },),
          },).toEqual({
            round: fresh.round,
            calls: ROSTER.length,
            persisted: fresh.persisted.get(key,),
            warned: [
              `warn the section pairing cache misses: the record under ${key} does not fit its sections (${refusal}), `
                + 'so the roster is asked again',
              'warn the aligner refused sections on this document: asking 2 voices which of 3 original sections '
                + 'render as which of 2 translation sections',
            ],
          },);
        },
      },);
    },),

    it({
      name: 'ASKS AGAIN UNDER ANOTHER ROSTER (ledger X13): a round one bench settled is not resumed for a '
        + 'bench that never answered it, such as the full bench after a dry reading paired on a subset',
      fn: async () => {
        const first = await runRound({},);
        /** The stored records, read back as the cache would hand them over. */
        const resumed = new Map([...first.persisted
          .entries(),].map(function toRecord([key, serialized,],): readonly [
            string,
            StoredRound,
          ] {
            return [
              key,
              storedRoundOf(serialized,),
            ];
          },),);
        const second = await runRound({
          resumed,
          modelIds: OTHER_ROSTER,
        },);
        expect({
          calls: second.calls.count,
          keysDiffer: [...second.persisted.keys(),].some(function unlikeFirst(key,): boolean {
            return !first.persisted.has(key,);
          },),
        },).toEqual({
          calls: OTHER_ROSTER.length,
          keysDiffer: true,
        },);
      },
    },),

    it({
      name: 'KEYS THE ROUND AS A SECTION QUESTION over the sections\' texts and its roster (ledger X15), '
        + 'so no block round\'s store could ever serve it',
      fn: async () => {
        const { persisted, } = await runRound({},);
        expect([...persisted.keys(),],).toEqual([
          pairingQuestionKey({
            question: 'section',
            sourceTexts: sectionTextsOf({ text: SOURCE_TEXT, },),
            targetTexts: sectionTextsOf({ text: TARGET_TEXT, },),
            pictureContext: '',
            modelIds: ROSTER,
          },),
        ],);
      },
    },),

    it({
      name: 'KEEPS the deterministic aligner when the roster agrees on nothing, returning an empty '
        + 'pairing that the caller must NOT read as "align nothing", and says so in a finding',
      fn: async () => {
        const {
          round,
          persisted,
        } = await runRound({ reply: '{"pairs":[]}', },);
        expect(round.pairing.length,).toBe(0,);
        expect(round.findings,).toContain(
          'section-pairing fell back to the deterministic aligner',
        );

        // CACHED ALL THE SAME, because a round that was answered and agreed
        // nothing is a stable fact about these two documents.
        expect(persisted.size,).toBe(1,);
      },
    },),

    it({
      name: 'REFUSES TO PERSIST a round nobody answered usably, because the roster was unreachable '
        + 'rather than undecided and caching that would freeze one bad minute into every resume',
      fn: async () => {
        const {
          round,
          persisted,
        } = await runRound({ reply: 'not json at all', },);
        expect(round.pairing.length,).toBe(0,);
        expect(round.findings
          .some(function namesTheSilence(finding,): boolean {
            return finding.startsWith('section-pairing no-usable-voice',);
          },),).toBe(true,);
        expect(persisted.size,).toBe(0,);
      },
    },),
  ],
},);
