/**
 Measurement of what the roster-paired preparation cuts when a non-empty archive does not align with the original's
 sections (ledger B1047, the side observation of round w373).

 THE DETERMINISTIC ALIGNER pairs nothing for an original of two sections against an archive of one heading or one
 paragraph, and `prepareDocumentPair` alone then cuts no slice. The real pass pairs the sections through the roster
 first (`prepareDocumentPairWithRoster`), so the figure that matters is what comes out of that path, with a scripted
 reply standing in for the roster. These cases pin the measured counts and say what each shows; they are a
 measurement, and they change when the finding is fixed, which this round does not do.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import { tagged, } from '@monochromatic-dev/logger-core/ts';
import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import type {
  ChatJsonOutcome,
  ChatJsonRequest,
  ModelClient,
} from '@monochromatic-dev/translation-repair-provider';
import { isInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract';
import { prepareDocumentPairWithRoster, } from '../dist/final/node/index.mjs';
import { cannedClient, } from '@monochromatic-dev/translation-repair-provider/ts/streaming-reply-client.test-fixture.ts';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';
import { HANG_STOP_MS, } from '@monochromatic-dev/translation-repair-test-fixture/ts/hang-stop.test-fixture.ts';
import { SCRIPTED_STAGE_SETTINGS, } from '@monochromatic-dev/translation-repair-stage/ts/stage-settings.test-fixture.ts';

/**
 Heading of the original's first section.
 */
const FIRST_HEADING = '## 简介';

/**
 Heading of the original's second section.
 */
const SECOND_HEADING = '## 后记';

/**
 Original of two heading sections, each with one paragraph.
 */
const TWO_SECTIONS = `${FIRST_HEADING}\n\n一只在书店陪伴大家的虎斑猫。\n\n${SECOND_HEADING}\n\n她睡在柜台上。\n`;

/**
 Archive of one heading section, which no heading of the original shares a word with.
 */
const ONE_HEADING = '## Notes\n\nA tabby who kept the bookshop company and slept on the counter.\n';

/**
 Archive of one paragraph and no heading.
 */
const ONE_PARAGRAPH = 'A tabby who kept the bookshop company and slept on the counter.\n';

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
const l = tagged({ tag: 'prepare-unaligned-archive-test', },);

/**
 What a preparation cut, and what it said about it.
 */
type Measured = {
  /**
   Slices the preparation cut.
   */
  readonly slices: number;

  /**
   How many of those slices point at an insertion offset rather than at archive text.
   */
  readonly anchors: number;

  /**
   Headings of the original that no slice carries, so the page ships without that section's original.
   */
  readonly uncovered: readonly string[];

  /**
   Pairing the roster agreed, as the preparation echoes it.
   */
  readonly sectionPairing: readonly { readonly source: number; readonly target: number; }[];

  /**
   Alignment findings the preparation recorded about sections.
   */
  readonly refusals: readonly string[];
};

/**
 Measures the preparation over the real path, with a scripted reply for every seat.

 @param targetText - archive the original is prepared against

 @param reply - body every scripted voice answers with

 @returns Slices cut, how many are anchors, the headings left uncovered, the pairing agreed and the refusals

 @example
 ```ts
 const measured = await measureWithRoster({ targetText: ONE_HEADING, reply: '{"pairs":[]}', },);
 ```
 */
async function measureWithRoster(
  {
    targetText,
    reply,
  }: {
    readonly targetText: string;
    readonly reply: string;
  },
): Promise<Measured> {
  /**
   Preparation of the pair.
   */
  const { prepared, } = await prepareDocumentPairWithRoster({
    client: cannedClient({ replyByModel: [reply, reply,], },),
    modelIds: ROSTER,
    sourceText: TWO_SECTIONS,
    targetText,
    signal: new AbortController().signal,
    exchangeTimeoutMs: HANG_STOP_MS,
    l,
    stageSettings: SCRIPTED_STAGE_SETTINGS,
  },);
  return {
    slices: prepared.slices.length,
    anchors: prepared.slices
      .filter(function isAnchor(slice,): boolean {
        return isInsertionChunk(slice.target,);
      },)
      .length,
    uncovered: [FIRST_HEADING, SECOND_HEADING,].filter(function isUncovered(heading,): boolean {
      return !prepared.slices.some(function carries(slice,): boolean {
        return slice.source.text.includes(heading,);
      },);
    },),
    sectionPairing: prepared.sectionPairing ?? [],
    refusals: prepared.alignmentFindings.filter(function isRefusal(finding,): boolean {
      return finding.startsWith('alignment ',);
    },),
  };
}

/**
 What the roster-paired preparation measures for one archive, by what the scripted voices answer.

 @param targetText - archive the original is prepared against

 @returns Measurement under a reply pairing the first original section, the second, or none

 @example
 ```ts
 const { paired, } = await measureUnderEachReply({ targetText: ONE_HEADING, },);
 ```
 */
async function measureUnderEachReply(
  { targetText, }: { readonly targetText: string; },
): Promise<Record<'first' | 'second' | 'none', Measured>> {
  return {
    first: await measureWithRoster({
      targetText,
      reply: '{"pairs":[{"source":0,"target":0}]}',
    },),
    second: await measureWithRoster({
      targetText,
      reply: '{"pairs":[{"source":1,"target":0}]}',
    },),
    none: await measureWithRoster({
      targetText,
      reply: '{"pairs":[]}',
    },),
  };
}

/**
 What the preparation measures when the roster pairs the original's first section with the archive's only one.
 */
const FIRST_PAIRED: Measured = {
  slices: 2,
  anchors: 1,
  uncovered: [SECOND_HEADING,],
  sectionPairing: [{ source: 0, target: 0, },],
  refusals: [
    'alignment structure-mismatch (source section 1: source-only (roster-unpaired); not anchored (beyond-shortfall))',
  ],
};

/**
 What it measures when the roster pairs the second section instead.
 */
const SECOND_PAIRED: Measured = {
  slices: 2,
  anchors: 1,
  uncovered: [FIRST_HEADING,],
  sectionPairing: [{ source: 1, target: 0, },],
  refusals: [
    'alignment structure-mismatch (source section 0: source-only (roster-unpaired); not anchored (beyond-shortfall))',
  ],
};

/**
 What it measures when the roster pairs nothing, so the deterministic aligner decides and refuses every section.
 */
const NONE_PAIRED: Measured = {
  slices: 0,
  anchors: 0,
  uncovered: [FIRST_HEADING, SECOND_HEADING,],
  sectionPairing: [],
  refusals: [
    'alignment structure-mismatch (source section 0: source-only (ambiguous); not anchored (may-pair))',
    'alignment structure-mismatch (source section 1: source-only (ambiguous); not anchored (may-pair))',
    'alignment structure-mismatch (target section 0: target-only (ambiguous); passes through unrepaired)',
  ],
};

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'an archive that does not align with the two sections of the original',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'MEASURES THE DETERMINISTIC PREPARATION AT ZERO SLICES for one heading and for one paragraph, '
            + 'which is the figure of round w373 (B1047)',
          fn: async () => {
            expect([ONE_HEADING, ONE_PARAGRAPH,].map(function sliceCount(targetText,): number {
              return prepareDocumentPair({
                sourceText: TWO_SECTIONS,
                targetText,
              },).slices.length;
            },),).toEqual([0, 0,],);
          },
        },),

        it({
          name: 'MEASURES THE ROSTER-PAIRED PREPARATION: ZERO SLICES ONLY WHEN THE ROSTER PAIRS NOTHING, and two '
            + 'slices with the section the roster left out carried by none, for one heading and for one paragraph '
            + '(B1047, a finding this round does not fix)',
          fn: async () => {
            expect({
              heading: await measureUnderEachReply({ targetText: ONE_HEADING, },),
              paragraph: await measureUnderEachReply({ targetText: ONE_PARAGRAPH, },),
            },).toEqual({
              heading: {
                first: FIRST_PAIRED,
                second: SECOND_PAIRED,
                none: NONE_PAIRED,
              },
              paragraph: {
                first: FIRST_PAIRED,
                second: SECOND_PAIRED,
                none: NONE_PAIRED,
              },
            },);
          },
        },),
      ],
    },),

    describe({
      name: 'an archive with no body',
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CUTS AN ANCHOR FOR EVERY ORIGINAL SECTION WITHOUT ASKING THE ROSTER, since the section round has no '
            + 'archive section to pair and a block round has no archive block, so no pairing is bought or stored '
            + 'for it and no cache record of one depends on the alignment (B1045)',
          fn: async () => {
            /**
             Every structured call a voice was asked for.
             */
            const asked: string[] = [];
            /**
             Client that records any exchange it is made.
             */
            const client: ModelClient = {
              chatText: async () => {
                throw new Error('chatText not used',);
              },
              chatJson: async <ValueT,>(request: ChatJsonRequest<ValueT>,): Promise<ChatJsonOutcome<ValueT>> => {
                asked.push(request.modelId,);
                throw new Error('an archive with no body bought an exchange',);
              },
              quotas: async () => {
                throw new Error('quotas not used',);
              },
            };
            /**
             Preparation of the original against an empty archive.
             */
            const { prepared, } = await prepareDocumentPairWithRoster({
              client,
              modelIds: ROSTER,
              sourceText: TWO_SECTIONS,
              targetText: '',
              signal: new AbortController().signal,
              exchangeTimeoutMs: HANG_STOP_MS,
              l,
              stageSettings: SCRIPTED_STAGE_SETTINGS,
            },);

            expect(asked,).toEqual([],);
            expect(prepared.slices.map(function isAnchorAtZero(slice,): readonly [boolean, number,] {
              return [
                isInsertionChunk(slice.target,),
                slice.target.startOffset,
              ];
            },),).toEqual([[true, 0,], [true, 0,],],);
            expect(prepared.slices.map(function toHeading(slice,): string {
              return slice.source.text
                .split('\n',)[0] ?? '';
            },),).toEqual([FIRST_HEADING, SECOND_HEADING,],);
          },
        },),
      ],
    },),
  ],
},);
