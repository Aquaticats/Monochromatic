/**
 Tests for the shell that buys a pairing and hands it to preparation.

 WHAT THESE PIN is the two things a settled entry now keeps about its pairing:
 the correspondences themselves, echoed back out of the map preparation
 consumed, and how many voices stood behind them, which was logged and never
 recorded. A section two voices paired and one six voices paired are different
 evidence about the same slicing.

 THE VOICE COUNT CARRIES ITS SECTION. The stage is asked one section at a time
 and cannot say which, so counts filed from there would arrive as a run of
 identical-shaped lines naming no section at all.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import type { PairedSectionRecord, } from '@monochromatic-dev/translation-repair-slice-reading';
import type {
  BenchSeating,
  SliceCache,
} from '@monochromatic-dev/translation-repair-translate-floor';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  createSyntheticClient,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { prepareDocumentPairWithRoster, } from '../dist/final/node/index.mjs';
import {
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_OPENROUTER_ONLY,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { cannedClient, } from '@monochromatic-dev/translation-repair-provider/ts/streaming-reply-client.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Original side, two blocks so the section is worth a question.
 */
const SOURCE_TEXT = '猫睡在盒子里。\n\n它整个下午都没有动。';

/**
 Translation side, two blocks against the two originals.
 */
const TARGET_TEXT = 'The cat slept in the box.\n\nShe did not move all afternoon.';

/**
 Roster of two, which is the smallest that can agree.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
] as const;

/**
 Logger for the shell under test.
 */
const l = tagged({ tag: 'prepare-with-pairing-test', },);

/**
 Builds client that fails if cache path buys any exchange.

 @returns Client refusing every transport call

 @example
 ```ts
 const client = refusingClient();
 ```
 */
function refusingClient(): ReturnType<typeof createSyntheticClient> {
  return createSyntheticClient({
    apiKey: 'test-key',
    transport: async function refuseTransport(): Promise<never> {
      throw new Error('cached preparation bought an exchange',);
    },
  },);
}

/**
 Builds a pairing cache backed by a map that outlives one run.

 ROUND-TRIPS THROUGH THE SERIALIZATION rather than storing the record by
 reference, because the defect under test is a record whose findings never
 reached disk. A stub that kept the object would pass while the bytes carried
 only pairs.

 @param stored - map surviving between the two runs of a case

 @returns Cache resuming from `stored` and writing back into it

 @example
 ```ts
 const cache = memoryPairingCache({ stored, },);
 ```
 */
function memoryPairingCache(
  { stored, }: { readonly stored: Map<string, PairedSectionRecord>; },
): SliceCache<PairedSectionRecord> {
  return {
    resumed: stored,
    persist: async ({ key, serialized, },) => {
      stored.set(
        key,
        JSON.parse(serialized,) as PairedSectionRecord,
      );
    },
  };
}

/**
 Rosters an alternating hook hands over in turn: two seats each, the smallest
 that can agree, disjoint from each other and from the fixture's own.
 */
const ALTERNATES: readonly (readonly RosterModelId[])[] = [
  [
    SEAT_SYNTHETIC_TEXT_EVERYWHERE,
    SEAT_SYNTHETIC_VISION_WITHHELD,
  ],
  [
    SEAT_HYPER_ONLY,
    SEAT_OPENROUTER_ONLY,
  ],
];

/**
 Original with three Chinese-headed sections of two paragraphs each, whose
 headings share no token with the translation's, so the aligner refuses and
 the section round is bought before each paired section's block round.
 */
const SECTIONED_SOURCE = `## 第一节

猫猫在窗台上打盹。

它整个下午都没有动。

## 第二节

窗台上有一只鸟。

猫猫看着它。

## 第三节

猫猫也喜欢晒太阳。

它晒了很久。
`;

/**
 Translation carrying only two of those sections.
 */
const SECTIONED_TARGET = `## Naps

The cat naps on the windowsill.

She did not move all afternoon.

## Birds

A bird sits on the windowsill.

The cat watches it.
`;

/**
 One structured call: the seat asked, and how many rosters the hook had
 handed over when it was asked.
 */
type RoundAsk = {
  /**
   Seat the call asked.
   */
  readonly seat: RosterModelId;
  /**
   Handovers so far, zero before the first.
   */
  readonly handover: number;
};

/**
 Roster the alternating hook handed over at a handover.

 @param handover - handovers so far

 @returns That handover's roster, none before the first

 @example
 ```ts
 const roster = alternateAt({ handover: 1, },);
 ```
 */
function alternateAt({ handover, }: { readonly handover: number; },): readonly RosterModelId[] {
  return (handover === 0) ? [] : (ALTERNATES[(handover - 1) % ALTERNATES.length] ?? []);
}

/**
 Prepares the sectioned fixture, recording every call's seat and handover.

 @param alternating - whether a hook hands over the alternates in turn, or
 none is given

 @returns Every structured call the preparation made

 @example
 ```ts
 const asks = await roundsAsked({ alternating: true, },);
 ```
 */
async function roundsAsked(
  { alternating, }: { readonly alternating: boolean; },
): Promise<readonly RoundAsk[]> {
  /**
   Handovers the hook made so far.
   */
  const hook = { handovers: 0, };
  /**
   Every structured call.
   */
  const asks: RoundAsk[] = [];
  /**
   Client pairing a question's first two blocks or sections in order, a
   reply a sheet may refuse, which leaves only the seat to record.
   */
  const client: ModelClient = {
    chatText: async () => {
      throw new Error('chatText not used',);
    },
    chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
      asks.push({ seat: request.modelId, handover: hook.handovers, },);
      /**
       Pairing every seat gives.
       */
      const value: unknown = { pairs: [{ source: 0, target: 0, }, { source: 1, target: 1, },], };
      return request.validate(value,)
        ? { kind: 'ok', value, rawText: JSON.stringify(value,), }
        : { kind: 'schema-mismatch', rawText: JSON.stringify(value,), detail: 'fixture answers another sheet', };
    },
    quotas: async () => {
      throw new Error('quotas not used',);
    },
  };
  await prepareDocumentPairWithRoster({
    client,
    modelIds: ROSTER,
    sourceText: SECTIONED_SOURCE,
    targetText: SECTIONED_TARGET,
    signal: new AbortController().signal,
    exchangeTimeoutMs: HANG_STOP_MS,
    l,
    ...(alternating
      ? {
        beforeSection: async (): Promise<BenchSeating> => {
          hook.handovers += 1;
          return { modelIds: alternateAt({ handover: hook.handovers, },), };
        },
      }
      : {}),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return asks;
}

/**
 Texts of the target slices prepared over the two-block pair under a budget.

 @param sliceCharBudget - budget handed to the preparation, none for the built-in one

 @returns Target text of each slice, in slice order

 @example
 ```ts
 const slices = await targetSlicesUnder({ sliceCharBudget: 1, },);
 ```
 */
async function targetSlicesUnder(
  { sliceCharBudget, }: { readonly sliceCharBudget?: number; },
): Promise<readonly string[]> {
  /**
   Preparation of the pair, both voices agreeing on the one-to-one pairing.
   */
  const { prepared, } = await prepareDocumentPairWithRoster({
    client: cannedClient({
      replyByModel: [
        '{"pairs":[{"source":0,"target":0},{"source":1,"target":1}]}',
        '{"pairs":[{"source":0,"target":0},{"source":1,"target":1}]}',
      ],
    },),
    modelIds: ROSTER,
    sourceText: SOURCE_TEXT,
    targetText: TARGET_TEXT,
    signal: new AbortController().signal,
    exchangeTimeoutMs: HANG_STOP_MS,
    l,
    ...((sliceCharBudget === undefined) ? {} : { sliceCharBudget, }),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return prepared.slices
    .map(function targetOf(slice,): string {
      return slice.target
        .text;
    },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: prepareDocumentPairWithRoster.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name:
            'ECHOES THE AGREED PAIRING back on the preparation, so the record a settled entry keeps is the '
              + 'object slicing consumed rather than a second copy assembled beside it',
          fn: async () => {
            const { prepared, } = await prepareDocumentPairWithRoster({
              client: cannedClient({
                replyByModel: [
                  '{"pairs":[{"source":0,"target":0},{"source":1,"target":1}]}',
                  '{"pairs":[{"source":0,"target":0},{"source":1,"target":1}]}',
                ],
              },),
              modelIds: ROSTER,
              sourceText: SOURCE_TEXT,
              targetText: TARGET_TEXT,
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(prepared.blockPairing,).toEqual([{
              sectionIndex: 0,
              pairs: [
                {
                  source: 0,
                  target: 0,
                },
                {
                  source: 1,
                  target: 1,
                },
              ],
            },],);
          },
        },),
        it({
          name: 'HANDS THE SLICE BUDGET ON TO PREPARATION, so a budget of one character cuts each paired block into '
            + 'a slice of its own where the built-in budget holds both blocks in one',
          fn: async () => {
            expect({
              builtIn: await targetSlicesUnder({},),
              oneCharacter: await targetSlicesUnder({ sliceCharBudget: 1, },),
            },).toEqual({
              builtIn: ['The cat slept in the box.\n\nShe did not move all afternoon.',],
              oneCharacter: [
                'The cat slept in the box.',
                'She did not move all afternoon.',
              ],
            },);
          },
        },),
        it({
          name: 'ATTACHES DETAILS TRANSCRIPT TO MATCHED MEDIA ON COLD AND WARM PREPARATION so picture evidence reaches quality stages and cached pairing cannot bypass normalization',
          fn: async () => {
            /**
             Literal site path placeholder.
             */
            const pathToken = [
              '$',
              '{path}',
            ].join('',);
            /**
             Shared source and target media marker.
             */
            const media = `<PhotoScroll photos={[ '${pathToken}/photos/letter.webp']} />`;
            /**
             Source fixture with image carrying letter.
             */
            const sourceText = `About the cat.\n\n${media}\n\nRemember the cat.`;
            /**
             Archive fixture with details transcript before same image.
             */
            const targetText = `About the cat.\n\n<details>\n<summary>Letter</summary>\n> Translated letter.\n</details>\n\n${media}\n\nRemember the cat.`;
            /**
             Cache shared across cold and warm preparation.
             */
            const stored = new Map<string, PairedSectionRecord>();
            const pairingCache = memoryPairingCache({ stored, },);

            const first = await prepareDocumentPairWithRoster({
              client: cannedClient({
                replyByModel: [
                  '{"pairs":[{"source":0,"target":0},{"source":1,"target":3},{"source":2,"target":4}]}',
                  '{"pairs":[{"source":0,"target":0},{"source":1,"target":3},{"source":2,"target":4}]}',
                ],
              },),
              modelIds: ROSTER,
              sourceText,
              targetText,
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              pairingCache,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            const resumed = await prepareDocumentPairWithRoster({
              client: refusingClient(),
              modelIds: ROSTER,
              sourceText,
              targetText,
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              pairingCache,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(first.prepared.unclaimedTargetBlocks,).toEqual([]);
            expect(first.prepared.slices.some(function carriesTranscript(slice,): boolean {
              return slice.target.text.includes('Translated letter.',);
            },),).toBe(true,);
            expect(resumed.prepared.blockPairing,).toEqual(first.prepared.blockPairing,);
          },
        },),
        it({
          name: 'RECONTESTS CONTESTED PAIRING instead of replaying settlement failure, then caches recovered split',
          fn: async () => {
            /**
             Cache shared by contested and recovered attempts.
             */
            const stored = new Map<string, PairedSectionRecord>();
            /**
             Original blocks shared by both attempts.
             */
            const sourceText = '猫睡在盒子里。\n\n它整个下午都没有动。';
            /**
             Translation blocks shared by both attempts.
             */
            const targetText = 'The cat slept in the box.\n\nShe stayed still.\n\nAll afternoon.';
            /**
             Cache boundary proving first attempt leaves nothing terminal.
             */
            const pairingCache = memoryPairingCache({ stored, },);
            await prepareDocumentPairWithRoster({
              client: cannedClient({
                replyByModel: [
                  '{"pairs":[{"source":0,"target":0},{"source":1,"target":1}]}',
                  '{"pairs":[{"source":0,"target":0},{"source":1,"target":1}]}',
                  '{"pairs":[{"source":0,"target":0},{"source":1,"target":2}]}',
                  '{"pairs":[{"source":0,"target":0},{"source":1,"target":2}]}',
                ],
              },),
              modelIds: [
                ...ROSTER,
                SEAT_SYNTHETIC_TEXT_EVERYWHERE,
                SEAT_SYNTHETIC_VISION_WITHHELD,
              ],
              sourceText,
              targetText,
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              pairingCache,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(stored.size,).toBe(0,);

            /**
             Recovered attempt whose roster corroborates one-to-many split.
             */
            const recovered = await prepareDocumentPairWithRoster({
              client: cannedClient({
                replyByModel: [
                  '{"pairs":[{"source":0,"target":0},{"source":1,"target":1},{"source":1,"target":2}]}',
                  '{"pairs":[{"source":0,"target":0},{"source":1,"target":1},{"source":1,"target":2}]}',
                ],
              },),
              modelIds: ROSTER,
              sourceText,
              targetText,
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              pairingCache,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(recovered.prepared.blockPairing?.at(0,)?.pairs,).toEqual([
              { source: 0, target: 0, },
              { source: 1, target: 1, },
              { source: 1, target: 2, },
            ],);
            expect(recovered.prepared.unclaimedTargetBlocks,).toEqual([]);
            expect(stored.size,).toBe(1,);
          },
        },),
        it({
          name: 'DOES NOT CACHE PAIRING THAT LEAVES ARCHIVE BLOCK UNCLAIMED, allowing next bounded attempt to seek safer correspondence',
          fn: async () => {
            const stored = new Map<string, PairedSectionRecord>();
            await prepareDocumentPairWithRoster({
              client: cannedClient({
                replyByModel: [
                  '{"pairs":[{"source":0,"target":0}]}',
                  '{"pairs":[{"source":0,"target":0}]}',
                ],
              },),
              modelIds: ROSTER,
              sourceText: '猫睡着了。',
              targetText: 'The cat slept.\n\nAn archive-only aside.',
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              pairingCache: memoryPairingCache({ stored, },),
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(stored.size,).toBe(0,);
          },
        },),
        it({
          name:
            'RECORDS HOW MANY VOICES AGREED, naming the section, and files it on the channel that reaches the '
              + 'artifact rather than only on the log nobody keeps',
          fn: async () => {
            const {
              prepared,
              findings,
            } = await prepareDocumentPairWithRoster({
              client: cannedClient({
                replyByModel: [
                  '{"pairs":[{"source":0,"target":0},{"source":1,"target":1}]}',
                  '{"pairs":[{"source":0,"target":0},{"source":1,"target":1}]}',
                ],
              },),
              modelIds: ROSTER,
              sourceText: SOURCE_TEXT,
              targetText: TARGET_TEXT,
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            /**
             What the counts have to say, section and all.
             */
            const expected = 'block-pairing section 0 paired 2 of 2 original and 2 of 2 translation blocks '
              + 'across 2 relations, from 2 usable voices of 2 heard';
            expect(findings,).toContain(expected,);

            // THE ARTIFACT READS THE PREPARATION, not this return value, and
            // `assertFindingsDescribePreparation` refuses a build where the two
            // disagree. So the finding is worth nothing unless it is on both.
            expect(prepared.alignmentFindings,).toContain(expected,);
          },
        },),
        it({
          name:
            'RECORDS AN EMPTY PAIRING when the roster was asked and agreed nothing, which is a different fact '
              + 'from no roster having been asked and must not read as the same absence',
          fn: async () => {
            const {
              prepared,
              findings,
            } = await prepareDocumentPairWithRoster({
              client: cannedClient({ replyByModel: ['{"pairs":[]}',], },),
              modelIds: ROSTER,
              sourceText: SOURCE_TEXT,
              targetText: TARGET_TEXT,
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect(prepared.blockPairing,).toEqual([],);
            expect(findings,).toContain('block-pairing section 0 fell back to scoring',);

            // NO COUNT LINE WHERE NOTHING WAS AGREED would be the wrong reading:
            // the voices were heard and usable, they simply named nothing, and that
            // is exactly the case the counts are worth recording for.
            expect(findings,).toContain(
              'block-pairing section 0 paired 0 of 2 original and 0 of 2 translation blocks across 0 relations, from 2 usable voices of 2 heard',
            );
          },
        },),

        it({
          name:
            'REPUBLISHES EVERY FINDING OFF A CACHED SECTION, having asked nobody. The cache stored a bare '
              + 'list of pairs until 2026-08-22, so a resumed entry reported a silent round: no per-section '
              + 'counts, no fallback notice, no voice-level finding. The two runs are compared whole rather '
              + 'than by sampled string, because a replay that keeps some findings and drops others is the '
              + 'shape this defect actually had',
          fn: async () => {
            /**
             Records surviving between the two runs, as a resumed pass finds them.
             */
            const stored = new Map<string, PairedSectionRecord>();

            const cold = await prepareDocumentPairWithRoster({
              client: cannedClient({
                replyByModel: [
                  '{"pairs":[{"source":0,"target":0},{"source":1,"target":1}]}',
                  '{"pairs":[{"source":0,"target":0},{"source":1,"target":1}]}',
                ],
              },),
              modelIds: ROSTER,
              sourceText: SOURCE_TEXT,
              targetText: TARGET_TEXT,
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              pairingCache: memoryPairingCache({ stored, },),
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            // POSITIVE CONTROL. Two empty lists compare equal, so a replay that lost
            // everything would satisfy the `warm.findings` comparison against a run that said
            // nothing. The cold run has to have reported something first.
            expect(cold.findings.length > 0,).toBe(true,);
            expect(stored.size,).toBe(1,);

            /**
             Calls the resumed run made, which must stay at none.
             */
            let calls = 0;

            const warm = await prepareDocumentPairWithRoster({
              client: createSyntheticClient({
                apiKey: 'test-key',
                transport: async function countingTransport() {
                  calls += 1;
                  throw new Error('the resumed run bought a pairing it already had',);
                },
              },),
              modelIds: ROSTER,
              sourceText: SOURCE_TEXT,
              targetText: TARGET_TEXT,
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              pairingCache: memoryPairingCache({ stored, },),
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(calls,).toBe(0,);
            expect(warm.findings,).toEqual(cold.findings,);
            expect(warm.prepared.blockPairing,).toEqual(cold.prepared.blockPairing,);
          },
        },),
      ],
    },),

    describe({
      name: `${prepareDocumentPairWithRoster.name} re-seated under a hold (ledger X12)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS EVERY ROUND OF THE ROSTER ITS HOOK LAST HANDED OVER, the section round and each section\'s '
            + 'block round alike, so a roster re-read after a provider dry-out asks the rounds after it rather than '
            + 'any roster read before it',
          fn: async () => {
            /**
             Calls a caller with no hook made.
             */
            const control = await roundsAsked({ alternating: false, },);
            /**
             Calls made while the hook alternates the rosters.
             */
            const moved = await roundsAsked({ alternating: true, },);
            expect({
              controlAskedAny: control.length > 0,
              controlOffRoster: control.filter(function offRoster(ask,): boolean {
                return !(ROSTER as readonly RosterModelId[]).includes(ask.seat,);
              },),
              sectionRoundAsked: moved.some(function atFirstHandover(ask,): boolean {
                return ask.handover === 1;
              },),
              blockRoundAsked: moved.some(function atSecondHandover(ask,): boolean {
                return ask.handover === 2;
              },),
              movedStrayed: moved.filter(function strayed(ask,): boolean {
                return !alternateAt({ handover: ask.handover, },).includes(ask.seat,);
              },),
            },).toEqual({
              controlAskedAny: true,
              controlOffRoster: [],
              sectionRoundAsked: true,
              blockRoundAsked: true,
              movedStrayed: [],
            },);
          },
        },),
      ],
    },),
  ],
},);
