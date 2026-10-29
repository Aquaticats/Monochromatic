/**
 Tests for production proof before translating source-only passages.

 A live page had one wholly omitted linked factual paragraph, but verbose
 English elsewhere made whole-page length look complete. These cat fixtures
 pin that local destination evidence rescues that class only when whole-page
 coverage independently says the passage is absent.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type {
  InsertionAdmission,
  PreparedDocumentPair,
} from '@monochromatic-dev/translation-repair-corpus-source';
import type { BenchSeating, } from '@monochromatic-dev/translation-repair-translate-floor';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading';
import { TranslationRepairInterruptedError, } from '@monochromatic-dev/translation-repair-translate-stage';
import {
  type ChatJsonOutcome,
  type ChatJsonRequest,
  messageText,
  type ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import {
  makeInsertionChunk,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import { decidePassInsertionAdmission, } from '../../dist/final/node/index.mjs';
import { capturingLogger, } from '@monochromatic-dev/translation-repair-test-fixture/ts/capturing-logger.test-fixture.ts';
import {
  SEAT_HYPER_ONLY,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_OPENROUTER_ONLY,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { rejectionOf, } from '@monochromatic-dev/translation-repair-test-fixture/ts/rejecting-call.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';
import {
  BENCH_UNREACHABLE,
  stopLandingHook,
  throwBenchUnreachable,
  UNASKED_CLIENT,
} from '../stop-landing-hook.test-fixture.ts';

/**
 Production-shaped test roster.
 */
const ROSTER: readonly RosterModelId[] = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
];

/**
 Coverage reply one roster seat returns.
 */
type ScriptedCoverage = {
  readonly coverage: 'full' | 'partial' | 'none';
  readonly copiedSpan: string;
};

/**
 Scripted provider voice that fails before returning coverage.
 */
const COVERAGE_VOICE_LOST: unique symbol = Symbol('scripted coverage voice lost',);

/**
 One scripted seat outcome.
 */
type ScriptedCoverageOutcome = ScriptedCoverage | typeof COVERAGE_VOICE_LOST;

/**
 No-op logger accepted by production module.
 */
const l = tagged({ tag: 'pass-insertion-admission-test', },);

/**
 Whole target carrying enough unrelated prose to defeat page shortfall.
 */
const LONG_TARGET = `## Cats\n\n${'The cat sleeps in warm sunlight. '.repeat(20,)}`;

/**
 Builds one prepared source-only passage.

 @param sourcePassage - original with no target wording beside it

 @param targetText - whole translation searched by coverage

 @returns Preparation holding one insertion slice

 @example
 ```ts
 const prepared = preparedGap({ sourcePassage: '猫。', targetText: '' });
 ```
 */
function preparedGap(
  {
    sourcePassage,
    targetText,
  }: {
    readonly sourcePassage: string;
    readonly targetText: string;
  },
): PreparedDocumentPair {
  return {
    sourceText: `${sourcePassage}\n${'猫在窗台晒太阳。'.repeat(20,)}`,
    targetText,
    slices: [{
      source: {
        kind: 'content',
        sliceIndex: 0,
        nodes: [],
        startOffset: 0,
        endOffset: sourcePassage.length,
        text: sourcePassage,
      },
      target: makeInsertionChunk({
        sliceIndex: 0,
        offset: 0,
      },),
    },],
    lineStructuredSliceIndices: new Set(),
    declaredNames: [],
    alignmentFindings: [],
    unclaimedTargetBlocks: [],
    alignmentPairCount: 1,
  };
}

/**
 Client returning one scripted coverage reply per seat, or failing that seat.

 @param replies - initial roster-order replies

 @param followupReplies - replies to prior-verdict challenge

 @returns Client serving only coverage stage

 @example
 ```ts
 const client = coverageClient({ replies: [{ coverage: 'none', copiedSpan: '' }] });
 ```
 */
function coverageClient(
  {
    replies,
    followupReplies = [],
  }: {
    readonly replies: readonly ScriptedCoverageOutcome[];
    readonly followupReplies?: readonly ScriptedCoverageOutcome[];
  },
): ModelClient {
  /**
   Iterator advancing one response per roster seat.
   */
  const responses = replies.values();
  /**
   Follow-up responses advanced independently from initial recovery retries.
   */
  const followups = followupReplies.values();
  return {
    chatText: async () => {
      throw new Error('chatText unused by coverage',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      /**
       Whether latest unresolved verdict is present in this prompt.
       */
      const isFollowup = request.messages.some(function hasPriorVerdict(message,): boolean {
        return messageText({ message, },).includes('PRIOR UNRESOLVED VERDICT',);
      },);
      /**
       Next scripted seat outcome, which must exist for every requested seat.
       */
      const next = isFollowup ? followups.next() : responses.next();
      if (next.done === true)
        throw new Error('coverage stage asked beyond scripted roster',);
      const { value: reply, } = next;
      if ((typeof reply) === 'symbol') {
        if (reply === COVERAGE_VOICE_LOST)
          throw new Error('scripted lost coverage voice',);
        throw new Error('unknown scripted coverage outcome',);
      }
      const value: unknown = {
        ...reply,
        assessment: 'scripted',
      };
      if (!request.validate(value,))
        throw new Error('scripted coverage reply failed wire guard',);
      return {
        kind: 'ok',
        value: value as ValueT,
        rawText: JSON.stringify(value,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by coverage',);
    },
  };
}

/**
 Runs one admission case.

 @param sourcePassage - insertion source

 @param targetText - whole target page

 @param replies - initial roster replies

 @param followupReplies - replies to prior-verdict challenge

 @returns Admission from production module

 @example
 ```ts
 const admission = await runAdmission({ sourcePassage: '猫。', targetText: '', replies: [] });
 ```
 */
async function runAdmission(
  {
    sourcePassage,
    targetText,
    replies,
    followupReplies,
  }: {
    readonly sourcePassage: string;
    readonly targetText: string;
    readonly replies: readonly ScriptedCoverageOutcome[];
    readonly followupReplies?: readonly ScriptedCoverageOutcome[];
  },
): Promise<InsertionAdmission> {
  return await decidePassInsertionAdmission({
    client: coverageClient({
      replies,
      ...((followupReplies === undefined) ? {} : { followupReplies, }),
    },),
    prepared: preparedGap({ sourcePassage, targetText, },),
    modelIds: ROSTER,
    overlap: 1,
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
}

/**
 Opening half of a disclosure block the archive never carried: the tag and
 its summary, which the slicer gives to the first block inside the container.
 */
const OPEN_HALF = '<details style="margin-top: 0.5rem;">\n<summary>猫的故事</summary>';

/**
 Closing half of the same block: its last paragraph and the closing tag.
 */
const CLOSE_HALF = '猫在夜里回家了。\n\n</details>';

/**
 Builds a preparation whose two source-only slices own one container's halves.

 @param targetText - whole translation searched by coverage

 @returns Preparation holding the opening half at slice 0 and the closing half at slice 1

 @example
 ```ts
 const prepared = preparedHalves({ targetText: 'Cat.', },);
 ```
 */
function preparedHalves({ targetText, }: { readonly targetText: string; },): PreparedDocumentPair {
  /**
   Offset of the closing half in the source.
   */
  const closeStart = OPEN_HALF.length + 2;
  return {
    sourceText: `${OPEN_HALF}\n\n${CLOSE_HALF}\n${'猫在窗台晒太阳。'.repeat(20,)}`,
    targetText,
    slices: [
      {
        source: {
          kind: 'content',
          sliceIndex: 0,
          nodes: [],
          startOffset: 0,
          endOffset: OPEN_HALF.length,
          text: OPEN_HALF,
        },
        target: makeInsertionChunk({ sliceIndex: 0, offset: 0, },),
      },
      {
        source: {
          kind: 'content',
          sliceIndex: 1,
          nodes: [],
          startOffset: closeStart,
          endOffset: closeStart + CLOSE_HALF.length,
          text: CLOSE_HALF,
        },
        target: makeInsertionChunk({ sliceIndex: 1, offset: 0, },),
      },
    ],
    lineStructuredSliceIndices: new Set(),
    declaredNames: [],
    alignmentFindings: [],
    unclaimedTargetBlocks: [],
    alignmentPairCount: 2,
  };
}

/**
 Repeats one reply across whole roster.

 @param reply - answer every seat gives

 @returns One answer per roster seat

 @example
 ```ts
 const replies = unanimous({ coverage: 'none', copiedSpan: '' });
 ```
 */
function unanimous(reply: ScriptedCoverage,): readonly ScriptedCoverage[] {
  return ROSTER.map(function sameReply(): ScriptedCoverage {
    return reply;
  },);
}

/**
 Paragraph the archive never carried, holding a source link and a marker:
 the link admits it on destination evidence whatever the budget has left.
 */
const MARKER_PASSAGE = '猫的记录见[备忘](https://example.test/cat-record)[^1]。';

/**
 Builds a preparation whose two source-only slices are a marker's paragraph and a definition.

 @param definition - definition block standing last, as definitions do

 @returns Preparation holding the paragraph at slice 0 and the definition at slice 1

 @example
 ```ts
 const prepared = preparedDefinition({ definition: '[^1]: 猫。', },);
 ```
 */
function preparedDefinition({ definition, }: { readonly definition: string; },): PreparedDocumentPair {
  /**
   Offset of the definition in the source.
   */
  const definitionStart = MARKER_PASSAGE.length + 2;
  return {
    sourceText: `${MARKER_PASSAGE}\n\n${definition}\n`,
    targetText: LONG_TARGET,
    slices: [
      {
        source: {
          kind: 'content',
          sliceIndex: 0,
          nodes: [],
          startOffset: 0,
          endOffset: MARKER_PASSAGE.length,
          text: MARKER_PASSAGE,
        },
        target: makeInsertionChunk({ sliceIndex: 0, offset: 0, },),
      },
      {
        source: {
          kind: 'content',
          sliceIndex: 1,
          nodes: [],
          startOffset: definitionStart,
          endOffset: definitionStart + definition.length,
          text: definition,
        },
        target: makeInsertionChunk({ sliceIndex: 1, offset: 0, },),
      },
    ],
    lineStructuredSliceIndices: new Set(),
    declaredNames: [],
    alignmentFindings: [],
    unclaimedTargetBlocks: [],
    alignmentPairCount: 2,
  };
}

/**
 Source paragraph the archive translated, standing before the tail.
 */
const TRANSLATED_SOURCE = '橘猫在窗台上睡了整个下午，阳光把它的毛烤得暖烘烘的。';

/**
 Its archive rendering, running long enough that the whole-page budget reads the page as complete.
 */
const TRANSLATED_TARGET = 'The orange cat slept on the windowsill all afternoon while the sun warmed its fur through. '
  .repeat(3,);

/**
 Source paragraph after the archive's last agreed pair, never translated,
 larger than the last pair's rendering could have absorbed.
 */
const TAIL_SOURCE = '猫在夜里回家了，蜷在暖炉旁边睡着了。它梦见了窗台上的阳光，梦见了院子里的麻雀，梦见了那只总在墙头等它的白猫。'
  + '天亮以后，它会再一次出门，沿着老路走到河边去。';

/**
 Builds a preparation whose archive stops after one agreed pair.

 @param tailLast - whether the source-only slice stands after the pair (the tail) or before it (interior)

 @returns Preparation holding one paired slice and one source-only slice

 @example
 ```ts
 const prepared = preparedStoppedArchive({ tailLast: true, },);
 ```
 */
function preparedStoppedArchive({ tailLast, }: { readonly tailLast: boolean; },): PreparedDocumentPair {
  /**
   Slice the archive translated.
   */
  const paired: ChunkPair = {
    source: {
      kind: 'content',
      sliceIndex: tailLast ? 0 : 1,
      nodes: [],
      startOffset: 0,
      endOffset: TRANSLATED_SOURCE.length,
      text: TRANSLATED_SOURCE,
    },
    target: {
      kind: 'content',
      sliceIndex: tailLast ? 0 : 1,
      nodes: [],
      startOffset: 0,
      endOffset: TRANSLATED_TARGET.length,
      text: TRANSLATED_TARGET,
    },
  };
  /**
   Slice the archive never reached.
   */
  const missing: ChunkPair = {
    source: {
      kind: 'content',
      sliceIndex: tailLast ? 1 : 0,
      nodes: [],
      startOffset: 0,
      endOffset: TAIL_SOURCE.length,
      text: TAIL_SOURCE,
    },
    target: makeInsertionChunk({ sliceIndex: tailLast ? 1 : 0, offset: 0, },),
  };
  return {
    sourceText: tailLast
      ? `${TRANSLATED_SOURCE}\n\n${TAIL_SOURCE}\n`
      : `${TAIL_SOURCE}\n\n${TRANSLATED_SOURCE}\n`,
    targetText: TRANSLATED_TARGET,
    slices: tailLast
      ? [
        paired,
        missing,
      ]
      : [
        missing,
        paired,
      ],
    lineStructuredSliceIndices: new Set(),
    declaredNames: [],
    alignmentFindings: [],
    unclaimedTargetBlocks: [],
    alignmentPairCount: 1,
  };
}

/**
 Opening half of a disclosure block: the tag and its summary.
 */
const DEFICIT_OPEN_SOURCE = '<details>\n\n<summary>猫的故事</summary>';

/**
 Archive's rendering of the opening half, the tag written its own way.
 */
const DEFICIT_OPEN_TARGET = '<details style="margin-top: 0.5rem;">\n<summary>The cat\'s story</summary>';

/**
 First paragraph inside the block, translated.
 */
const DEFICIT_FIRST_SOURCE = '橘猫在窗台上睡了整个下午。';

/**
 Archive's rendering of the first paragraph, running long enough that the
 whole-page budget reads the page as complete.
 */
const DEFICIT_FIRST_TARGET = 'The orange cat slept on the windowsill all afternoon while the sun warmed its fur through. '
  .repeat(3,)
  .trimEnd();

/**
 Paragraph inside the block the archive never rendered.
 */
const DEFICIT_MISSING_SOURCE = '猫在夜里回家了，谁也没有听见门响。';

/**
 The archive's rendering of that paragraph, for the control whose block is
 not short.
 */
const DEFICIT_MISSING_TARGET = 'The cat came home at night, and nobody heard the door.';

/**
 Closing half: the last paragraph and the closing tag.
 */
const DEFICIT_CLOSE_SOURCE = '猫不喜欢洗澡。\n\n</details>';

/**
 Archive's rendering of the closing half.
 */
const DEFICIT_CLOSE_TARGET = 'The cat does not like baths.\n\n</details>';

/**
 Builds a preparation whose source-only paragraph stands inside a disclosure
 block, the archive's block short of it or, for the control, carrying a
 rendering of it inside the closing half.

 @param archiveShort - whether the archive's block has one paragraph fewer
 than the original's

 @returns Preparation holding the opening half, the first paragraph, the
 source-only paragraph and the closing half

 @example
 ```ts
 const prepared = preparedContainerDeficit({ archiveShort: true, },);
 ```
 */
function preparedContainerDeficit(
  { archiveShort, }: { readonly archiveShort: boolean; },
): PreparedDocumentPair {
  /**
   The original, block by block.
   */
  const sourceText = `${
    [
      DEFICIT_OPEN_SOURCE,
      DEFICIT_FIRST_SOURCE,
      DEFICIT_MISSING_SOURCE,
      DEFICIT_CLOSE_SOURCE,
    ].join('\n\n',)
  }\n`;
  /**
   The archive's closing half, carrying the paragraph for the control.
   */
  const closeTarget = archiveShort
    ? DEFICIT_CLOSE_TARGET
    : `${DEFICIT_MISSING_TARGET}\n\n${DEFICIT_CLOSE_TARGET}`;
  /**
   The archive, block by block.
   */
  const targetText = `${
    [
      DEFICIT_OPEN_TARGET,
      DEFICIT_FIRST_TARGET,
      closeTarget,
    ].join('\n\n',)
  }\n`;
  /**
   Where the archive's closing half starts, which is where the source-only
   paragraph would land.
   */
  const closeTargetStart = targetText.indexOf(closeTarget,);
  /**
   Builds one paired slice from both sides' texts.

   @param sliceIndex - where the slice stands

   @param source - original text

   @param target - archive text

   @returns Slice paired with its rendering

   @example
   ```ts
   const slice = pairedAt({ sliceIndex: 0, source: DEFICIT_OPEN_SOURCE, target: DEFICIT_OPEN_TARGET, },);
   ```
   */
  function pairedAt(
    {
      sliceIndex,
      source,
      target,
    }: {
      readonly sliceIndex: number;
      readonly source: string;
      readonly target: string;
    },
  ): ChunkPair {
    /**
     Where the original text starts.
     */
    const sourceStart = sourceText.indexOf(source,);
    /**
     Where the archive text starts.
     */
    const targetStart = targetText.indexOf(target,);
    return {
      source: {
        kind: 'content',
        sliceIndex,
        nodes: [],
        startOffset: sourceStart,
        endOffset: sourceStart + source.length,
        text: source,
      },
      target: {
        kind: 'content',
        sliceIndex,
        nodes: [],
        startOffset: targetStart,
        endOffset: targetStart + target.length,
        text: target,
      },
    };
  }
  /**
   Where the source-only paragraph starts in the original.
   */
  const missingStart = sourceText.indexOf(DEFICIT_MISSING_SOURCE,);
  return {
    sourceText,
    targetText,
    slices: [
      pairedAt({
        sliceIndex: 0,
        source: DEFICIT_OPEN_SOURCE,
        target: DEFICIT_OPEN_TARGET,
      },),
      pairedAt({
        sliceIndex: 1,
        source: DEFICIT_FIRST_SOURCE,
        target: DEFICIT_FIRST_TARGET,
      },),
      {
        source: {
          kind: 'content',
          sliceIndex: 2,
          nodes: [],
          startOffset: missingStart,
          endOffset: missingStart + DEFICIT_MISSING_SOURCE.length,
          text: DEFICIT_MISSING_SOURCE,
        },
        target: makeInsertionChunk({
          sliceIndex: 2,
          offset: closeTargetStart,
        },),
      },
      pairedAt({
        sliceIndex: 3,
        source: DEFICIT_CLOSE_SOURCE,
        target: closeTarget,
      },),
    ],
    lineStructuredSliceIndices: new Set(),
    declaredNames: [],
    alignmentFindings: [],
    unclaimedTargetBlocks: [],
    alignmentPairCount: 3,
  };
}

/**
 Admits the one source-only paragraph of a container preparation, every
 voice finding it absent, recording what the admission logged.

 @param archiveShort - whether the archive's block is one paragraph short

 @returns Admission beside the lines it logged

 @example
 ```ts
 const { admission, messages, } = await deficitAdmission({ archiveShort: true, },);
 ```
 */
async function deficitAdmission(
  { archiveShort, }: { readonly archiveShort: boolean; },
): Promise<{
  readonly admission: InsertionAdmission;
  readonly messages: readonly string[];
}> {
  /**
   Lines the admission logged.
   */
  const messages: string[] = [];
  /**
   The admission.
   */
  const admission = await decidePassInsertionAdmission({
    client: coverageClient({ replies: unanimous({ coverage: 'none', copiedSpan: '', },), },),
    prepared: preparedContainerDeficit({ archiveShort, },),
    modelIds: ROSTER,
    overlap: 1,
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l: capturingLogger({ messages, },),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    admission,
    messages,
  };
}

/**
 Opening of the deficit finding for the paragraph at slice 2.
 */
const DEFICIT_FINDING_OPENING = 'insertion-container-deficit-admitted (slice 2 inside details of slices 0 to 3';

/**
 Roster a hook hands back after a dry-out, none of it the fixture's own.
 */
const RESEATED_ROSTER: readonly RosterModelId[] = [
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_ONLY,
  SEAT_OPENROUTER_ONLY,
];

/**
 Builds a client recording which seat every coverage call asked, each
 answering that the archive carries none of the passage.

 @param asked - sink for the seat of every call

 @returns Client serving only the coverage stage

 @example
 ```ts
 const client = seatRecordingCoverageClient({ asked: [], },);
 ```
 */
function seatRecordingCoverageClient(
  { asked, }: { readonly asked: RosterModelId[]; },
): ModelClient {
  return {
    chatText: async () => {
      throw new Error('chatText unused by coverage',);
    },
    chatJson: async <ValueT,>(
      request: ChatJsonRequest<ValueT>,
    ): Promise<ChatJsonOutcome<ValueT>> => {
      asked.push(request.modelId,);
      /**
       Coverage reply every seat gives.
       */
      const value: unknown = {
        coverage: 'none',
        copiedSpan: '',
        assessment: 'scripted',
      };
      if (!request.validate(value,))
        throw new Error('scripted coverage reply failed wire guard',);
      return {
        kind: 'ok',
        value: value as ValueT,
        rawText: JSON.stringify(value,),
      };
    },
    quotas: async () => {
      throw new Error('quotas unused by coverage',);
    },
  };
}

/**
 Admits the one scripted gap, recording the seats asked.

 @param beforeCandidate - hook handing the candidate its roster, none for a
 caller with no hook

 @returns Seats the coverage calls asked

 @example
 ```ts
 const asked = await coverageSeatsAsked({},);
 ```
 */
async function coverageSeatsAsked(
  { beforeCandidate, }: { readonly beforeCandidate?: () => Promise<BenchSeating>; },
): Promise<readonly RosterModelId[]> {
  /**
   Seat of every call.
   */
  const asked: RosterModelId[] = [];
  await decidePassInsertionAdmission({
    client: seatRecordingCoverageClient({ asked, },),
    prepared: preparedGap({
      sourcePassage: '猫在院子里追蝴蝶。',
      targetText: LONG_TARGET,
    },),
    modelIds: ROSTER,
    overlap: 1,
    signal: new AbortController().signal,
    perCallTimeoutMs: HANG_STOP_MS,
    l,
    ...((beforeCandidate === undefined) ? {} : { beforeCandidate, }),
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return asked;
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: decidePassInsertionAdmission.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'WRITES A LATER CANDIDATE\'S OWN FAILURE UNDER THE CALLER\'S STOP, its hook refused by an unreachable '
            + 'bench as the stop landed while the first candidate ended in the caller\'s reason, and refuses with '
            + 'that reason',
          fn: async () => {
            /**
             Every line the admission wrote.
             */
            const messages: string[] = [];
            /**
             The caller's steering, which the later candidate's hook stops.
             */
            const steering = new AbortController();
            /**
             Why the caller stopped the admission.
             */
            const reason = new Error('the owner called the cats in',);
            /**
             Preparation holding one source-only passage.
             */
            const gap = preparedGap({
              sourcePassage: '猫在院子里追蝴蝶。',
              targetText: LONG_TARGET,
            },);
            /**
             That passage's slice, which a second slice repeats so both are prose candidates.
             */
            const first = nonNullishOrThrow(gap.slices[0],);
            /**
             What the admission refused with.
             */
            const refusal = await rejectionOf(async function admitsUnderTheStop(): Promise<unknown> {
              return await decidePassInsertionAdmission({
                client: UNASKED_CLIENT,
                prepared: {
                  ...gap,
                  slices: [
                    first,
                    {
                      source: {
                        ...first.source,
                        sliceIndex: 1,
                      },
                      target: makeInsertionChunk({
                        sliceIndex: 1,
                        offset: 0,
                      },),
                    },
                  ],
                },
                modelIds: ROSTER,
                overlap: 2,
                signal: steering.signal,
                perCallTimeoutMs: HANG_STOP_MS,
                l: capturingLogger({ messages, },),
                beforeCandidate: stopLandingHook<BenchSeating>({
                  steering,
                  reason,
                  afterStop: throwBenchUnreachable,
                },),
                stageSettings: SCRIPTED_STAGE_SETTINGS,
              },);
            },);
            expect(refusal,).toBe(reason,);
            expect(messages,).toEqual([
              `[decidePassInsertionAdmission] [mapOverlapped] slice 1 failed (${BENCH_UNREACHABLE.message}), and the `
                + 'caller stopped the work',
            ],);
          },
        },),
        it({
          name: 'ADMITS a linked factual passage on destination evidence when roster says whole passage is absent, '
            + 'even though unrelated verbosity makes whole page look long enough',
          fn: async () => {
            const admission = await runAdmission({
              sourcePassage: 'The cat record is linked at [memo](https://example.test/cat-record).',
              targetText: LONG_TARGET,
              replies: unanimous({ coverage: 'none', copiedSpan: '', },),
            },);
            expect([...admission.positions,],).toEqual([0,],);
          },
        },),
        it({
          name: 'ADMITS a link-free passage within whole-page shortfall when roster says it is absent',
          fn: async () => {
            const admission = await runAdmission({
              sourcePassage: '猫的记录没有译文。',
              targetText: 'Cat.',
              replies: unanimous({ coverage: 'none', copiedSpan: '', },),
            },);
            expect([...admission.positions,],).toEqual([0,],);
          },
        },),
        it({
          name: 'REFUSES an absent verdict with neither deterministic corroborator, preserving duplicate protection',
          fn: async () => {
            // Not admitted and not thrown: the round settles once and the refusal
            // lives on the findings instead of pausing the entry.
            const admission = await runAdmission({
              sourcePassage: '猫的记录没有译文。',
              targetText: LONG_TARGET,
              replies: unanimous({ coverage: 'none', copiedSpan: '', },),
            },);
            expect([...admission.positions,],).toEqual([],);
            expect(
              admission.findings
                .some(function namesUnresolved(finding,): boolean {
                  return finding.includes('insertion-unresolved-after-single-round',);
                },),
            ).toBe(true,);
          },
        },),
        it({
          name: 'RECORDS full coverage as carried elsewhere instead of insertion or interruption',
          fn: async () => {
            const admission = await runAdmission({
              sourcePassage: '[Cat](https://example.test/cat-record) sleeps.',
              targetText: LONG_TARGET,
              replies: unanimous({ coverage: 'full', copiedSpan: '## Cats\n\nThe cat sleeps in warm sunlight.', },),
            },);
            expect([...admission.positions,],).toEqual([],);
            expect(admission.carried,).toEqual([{
              position: 0,
              sliceIndex: 0,
              sourceText: '[Cat](https://example.test/cat-record) sleeps.',
              evidence: [
                '## Cats\n\nThe cat sleeps in warm sunlight.',
                '## Cats\n\nThe cat sleeps in warm sunlight.',
                '## Cats\n\nThe cat sleeps in warm sunlight.',
              ],
            },],);
          },
        },),
        it({
          name: 'REFUSES partial coverage because inserting whole passage would duplicate carried content',
          fn: async () => {
            // An empty follow-up script doubles as the no-re-ask proof: any second
            // coverage round would throw inside the scripted client.
            const admission = await runAdmission({
              sourcePassage: '[Cat](https://example.test/cat-record) sleeps and dreams.',
              targetText: LONG_TARGET,
              replies: unanimous({ coverage: 'partial', copiedSpan: 'The cat sleeps in warm sunlight.', },),
            },);
            expect([...admission.positions,],).toEqual([],);
            expect(admission.carried,).toEqual([],);
            expect(
              admission.findings
                .some(function namesUnresolved(finding,): boolean {
                  return finding.includes('insertion-unresolved-after-single-round (slice 0',);
                },),
            ).toBe(true,);
          },
        },),
        it({
          name: 'REFUSES a split roster in one round rather than treating one absence voice as proof',
          fn: async () => {
            // The scripted follow-up would prove absence, but no follow-up may be
            // asked: the single round records the split and moves on.
            // THE FULL QUOTE MUST ANCHOR: the bare sentence occurs twenty times in
            // the target and locates nowhere (ambiguous-quote), which left this
            // fixture split between an absence and an unanchorable claim, the shape
            // class forty-eight now admits by the shortfall; the heading makes it
            // unique, so the split here is between two anchored, opposite votes.
            const admission = await runAdmission({
              sourcePassage: '[Cat](https://example.test/cat-record) sleeps.',
              targetText: LONG_TARGET,
              replies: [
                { coverage: 'full', copiedSpan: '## Cats\n\nThe cat sleeps in warm sunlight.', },
                { coverage: 'none', copiedSpan: '', },
                COVERAGE_VOICE_LOST,
              ],
              followupReplies: unanimous({ coverage: 'none', copiedSpan: '', },),
            },);
            expect([...admission.positions,],).toEqual([],);
            expect(
              admission.findings
                .some(function namesUnresolved(finding,): boolean {
                  return finding.includes('insertion-unresolved-after-single-round',);
                },),
            ).toBe(true,);
          },
        },),
        it({
          name: 'ADMITS THE OPENING HALF OF A CONTAINER BESIDE ITS ADMITTED CLOSING HALF when the roster split on '
            + 'the summary alone (class fifty-seven, XingZ607: two disclosure blocks lost their opening halves, '
            + 'the closing tags shipped alone, and the translate lane withdrew every slice)',
          fn: async () => {
            // Slice 0, the summary, splits one full against one partial against one
            // absent, all anchored, so on its own it stays unresolved; slice 1, the
            // block's end, is absent by every voice and the page is short of it.
            const admission = await decidePassInsertionAdmission({
              client: coverageClient({
                replies: [
                  { coverage: 'full', copiedSpan: 'Cat.', },
                  { coverage: 'partial', copiedSpan: 'Cat.', },
                  { coverage: 'none', copiedSpan: '', },
                  ...unanimous({ coverage: 'none', copiedSpan: '', },),
                ],
              },),
              prepared: preparedHalves({ targetText: 'Cat.', },),
              modelIds: ROSTER,
              overlap: 1,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect([...admission.positions,].toSorted(function ascending(
              left,
              right,
            ): number {
              return left - right;
            },),).toEqual([0, 1,],);
            expect(
              admission.findings
                .some(function namesTheHalf(finding,): boolean {
                  return finding.startsWith('insertion-container-half-admitted (slice 0 beside slice 1',);
                },),
            ).toBe(true,);
          },
        },),
        it({
          name: 'LEAVES BOTH HALVES UNFILLED when neither is admitted on its own evidence, so the rule widens '
            + 'nothing without a corroborated half',
          fn: async () => {
            const admission = await decidePassInsertionAdmission({
              client: coverageClient({
                replies: [
                  { coverage: 'full', copiedSpan: 'Cat.', },
                  { coverage: 'partial', copiedSpan: 'Cat.', },
                  { coverage: 'none', copiedSpan: '', },
                  { coverage: 'full', copiedSpan: 'Cat.', },
                  { coverage: 'partial', copiedSpan: 'Cat.', },
                  { coverage: 'none', copiedSpan: '', },
                ],
              },),
              prepared: preparedHalves({ targetText: 'Cat.', },),
              modelIds: ROSTER,
              overlap: 1,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect([...admission.positions,],).toEqual([],);
            expect(
              admission.findings
                .some(function namesTheHalf(finding,): boolean {
                  return finding.startsWith('insertion-container-half-admitted',);
                },),
            ).toBe(false,);
          },
        },),
        it({
          name: 'THROWS provider-unavailable when every coverage voice is lost, never a quality refusal',
          fn: async () => {
            let thrown: unknown;
            try {
              await runAdmission({
                sourcePassage: '[Cat](https://example.test/cat-record) sleeps.',
                targetText: LONG_TARGET,
                replies: [ COVERAGE_VOICE_LOST, COVERAGE_VOICE_LOST, COVERAGE_VOICE_LOST, ],
              },);
            }
            catch (error) {
              thrown = error;
            }
            expect(thrown,).toBeInstanceOf(TranslationRepairInterruptedError,);
            expect((thrown as TranslationRepairInterruptedError).reason,).toBe('provider-unavailable');
          },
        },),
        it({
          name: 'TREATS trailing-slash destination spellings as same address rather than false local corroboration',
          fn: async () => {
            const admission = await runAdmission({
              sourcePassage: '[Cat](https://example.test/cat-record/) sleeps.',
              targetText: `${LONG_TARGET}\nhttps://example.test/cat-record`,
              replies: unanimous({ coverage: 'none', copiedSpan: '', },),
            },);
            expect([...admission.positions,],).toEqual([],);
          },
        },),
        it({
          name: 'READS a reference-style source destination as local corroboration',
          fn: async () => {
            const admission = await runAdmission({
              sourcePassage: '[Cat memo][memo]\n\n[memo]: https://example.test/cat-record',
              targetText: LONG_TARGET,
              replies: unanimous({ coverage: 'none', copiedSpan: '', },),
            },);
            expect([...admission.positions,],).toEqual([0,],);
          },
        },),
      ],
    },),

    describe({
      name: 'a definition the budget refused beside the marker that references it (class fifty-nine, XingZ608)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ADMITS THE DEFINITION BESIDE THE ADMITTED MARKER when the whole-page budget has no room left '
            + '(XingZ608: the budget ran out 80 code points before the definitions and the assembly withdrew '
            + 'every marker carrier)',
          fn: async () => {
            // Both slices are absent by every voice; the page is long, so only the
            // link admits slice 0, and slice 1 is admitted by slice 0's marker.
            const admission = await decidePassInsertionAdmission({
              client: coverageClient({
                replies: [
                  ...unanimous({ coverage: 'none', copiedSpan: '', },),
                  ...unanimous({ coverage: 'none', copiedSpan: '', },),
                ],
              },),
              prepared: preparedDefinition({ definition: '[^1]: 猫在夜里回家了。', },),
              modelIds: ROSTER,
              overlap: 1,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect([...admission.positions,].toSorted(function ascending(
              left,
              right,
            ): number {
              return left - right;
            },),).toEqual([
              0,
              1,
            ],);
            expect(
              admission.findings
                .some(function namesDefinition(finding,): boolean {
                  return finding.startsWith('insertion-definition-admitted (slice 1 defines 1, referenced by slice 0',);
                },),
            ).toBe(true,);
          },
        },),
        it({
          name: 'LEAVES A DEFINITION NOTHING REFERENCES unresolved under the same budget',
          fn: async () => {
            const admission = await decidePassInsertionAdmission({
              client: coverageClient({
                replies: [
                  ...unanimous({ coverage: 'none', copiedSpan: '', },),
                  ...unanimous({ coverage: 'none', copiedSpan: '', },),
                ],
              },),
              prepared: preparedDefinition({ definition: '[^2]: 猫不喜欢洗澡。', },),
              modelIds: ROSTER,
              overlap: 1,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect([...admission.positions,],).toEqual([0,],);
            expect(
              admission.findings
                .some(function namesUnresolved(finding,): boolean {
                  return finding.startsWith('insertion-unresolved-after-single-round (slice 1',);
                },),
            ).toBe(true,);
          },
        },),
      ],
    },),

    describe({
      name: 'an archive that stops before the source does (owner, 2026-09-19; XingZ608)',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ADMITS THE UNTRANSLATED TAIL on the pairing\'s evidence when the translated part runs long enough '
            + 'to spend the whole-page budget',
          fn: async () => {
            const admission = await decidePassInsertionAdmission({
              client: coverageClient({ replies: unanimous({ coverage: 'none', copiedSpan: '', },), },),
              prepared: preparedStoppedArchive({ tailLast: true, },),
              modelIds: ROSTER,
              overlap: 1,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect([...admission.positions,],).toEqual([1,],);
            expect(
              admission.findings
                .some(function namesTail(finding,): boolean {
                  return finding.includes('insertion-corroboration (slice 1, tail admitted',);
                },),
            ).toBe(true,);
          },
        },),
        it({
          name: 'STILL REFUSES an interior passage on the same long page, since the tail is read off the pairing '
            + 'and the interior keeps the whole-page budget',
          fn: async () => {
            const admission = await decidePassInsertionAdmission({
              client: coverageClient({ replies: unanimous({ coverage: 'none', copiedSpan: '', },), },),
              prepared: preparedStoppedArchive({ tailLast: false, },),
              modelIds: ROSTER,
              overlap: 1,
              signal: new AbortController().signal,
              perCallTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);
            expect([...admission.positions,],).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: `${decidePassInsertionAdmission.name} over a container the archive carries short (class sixty-one)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ADMITS the absent paragraph on the deficit, records the finding, and logs it under the admission\'s '
            + 'tag; the same paragraph inside a block the archive carries whole is neither admitted on a deficit nor '
            + 'logged as one, the control',
          fn: async () => {
            /**
             Admission over the short block.
             */
            const short = await deficitAdmission({ archiveShort: true, },);
            /**
             Admission over the whole block.
             */
            const whole = await deficitAdmission({ archiveShort: false, },);
            /**
             What one admission made of the paragraph.

             @param run - admission and its lines

             @returns Whether the paragraph was admitted, and whether the finding
             was recorded and logged

             @example
             ```ts
             const read = deficitReading({ run: short, },);
             ```
             */
            function deficitReading(
              { run, }: { readonly run: Awaited<ReturnType<typeof deficitAdmission>>; },
            ): {
              readonly admitted: boolean;
              readonly recorded: boolean;
              readonly logged: boolean;
            } {
              return {
                admitted: run.admission
                  .positions
                  .has(2,),
                recorded: run.admission
                  .findings
                  .some(function opensTheFinding(finding,): boolean {
                    return finding.startsWith(DEFICIT_FINDING_OPENING,);
                  },),
                logged: run.messages.some(function logsTheFinding(message,): boolean {
                  return message.startsWith(`[${decidePassInsertionAdmission.name}] ${DEFICIT_FINDING_OPENING}`,);
                },),
              };
            }
            expect({
              short: deficitReading({ run: short, },),
              whole: deficitReading({ run: whole, },),
            },).toEqual({
              short: {
                admitted: true,
                recorded: true,
                logged: true,
              },
              whole: {
                admitted: false,
                recorded: false,
                logged: false,
              },
            },);
          },
        },),
      ],
    },),

    describe({
      name: `${decidePassInsertionAdmission.name} re-seated under a hold (ledger X12)`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'ASKS A CANDIDATE OF THE ROSTER ITS HOOK RETURNS, so a roster re-read after a provider dry-out is '
            + 'the one asked whether the archive carries the passage, rather than the roster read before it',
          fn: async () => {
            /**
             Seats a caller with no hook asked.
             */
            const control = await coverageSeatsAsked({},);
            /**
             Seats asked when the hook re-seats the candidate elsewhere.
             */
            const moved = await coverageSeatsAsked({
              beforeCandidate: async (): Promise<BenchSeating> => ({ modelIds: RESEATED_ROSTER, }),
            },);
            expect({
              controlOnRoster: (control.length > 0) && control.every(function onRoster(seat,): boolean {
                return ROSTER.includes(seat,);
              },),
              movedAskedAny: moved.length > 0,
              outsideReseated: moved.filter(function outside(seat,): boolean {
                return !RESEATED_ROSTER.includes(seat,);
              },),
            },).toEqual({
              controlOnRoster: true,
              movedAskedAny: true,
              outsideReseated: [],
            },);
          },
        },),
      ],
    },),
  ],
},);
