/**
 Guards class fifty-one (shi_Yumiaoya4, 2026-09-17): a claim of coverage
 whose quote sits inside a target region the pairing already assigned to
 another source slice is no evidence that THIS passage is rendered, since
 that English is the rendering of a different original. One voice quoted the
 archive's farewell line as partial coverage of the death paragraphs, the
 claim anchored, and the split it made left the passage a recorded gap.
 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  parseDocument,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract';
import type { HeardVoice, } from '@monochromatic-dev/translation-repair-stage';
import {
  type CoverageReportWire,
  judgeCoverage,
} from '../dist/final/node/index.mjs';

/**
 Translation every quote is checked against.
 */
const TARGET_TEXT = `The cat sleeps on the windowsill each morning and naps on its cushion at noon.

She watches the birds outside.
`;

/**
 That translation as an anchor target.
 */
const TARGET = {
  text: TARGET_TEXT,
  nodes: parseDocument({ text: TARGET_TEXT, },).nodes,
};

/**
 Sentence the pairing assigned to another source slice.
 */
const FOREIGN_SENTENCE = 'She watches the birds outside.';

/**
 Region of that sentence in the target.
 */
const FOREIGN_REGION = {
  startOffset: TARGET_TEXT.indexOf(FOREIGN_SENTENCE,),
  endOffset: TARGET_TEXT.indexOf(FOREIGN_SENTENCE,) + FOREIGN_SENTENCE.length,
};

/**
 Builds one heard voice carrying a coverage reply.

 @param modelId - roster member the reply came from

 @param coverage - degree it claims

 @param quote - English it points at

 @returns Voice shaped as the roster returns them

 @example
 ```ts
 const voice = voiceOf({ modelId: 'hf:cat/Cat-A' as RosterModelId, coverage: 'none', quote: '', },);
 ```
 */
function voiceOf(
  {
    modelId,
    coverage,
    quote,
  }: {
    readonly modelId: RosterModelId;
    readonly coverage: CoverageReportWire['coverage'];
    readonly quote: string;
  },
): HeardVoice<CoverageReportWire> {
  return {
    modelId,
    value: {
      coverage,
      copiedSpan: quote,
      assessment: 'fixture',
    },
  };
}

/**
 Three voices: one partial claim quoting the foreign sentence, two absent.
 */
const VOICES = [
  voiceOf({
    modelId: 'hf:cat/Cat-A' as RosterModelId,
    coverage: 'partial',
    quote: FOREIGN_SENTENCE,
  },),
  voiceOf({
    modelId: 'hf:cat/Cat-B' as RosterModelId,
    coverage: 'none',
    quote: '',
  },),
  voiceOf({
    modelId: 'hf:cat/Cat-C' as RosterModelId,
    coverage: 'none',
    quote: '',
  },),
];

await describe({
  name: 'a coverage claim quoting another slice\'s paired rendering (class fifty-one)',
  children: [
    it({
      name: 'COUNTS the claim as partial coverage when no region is foreign, the control that shows '
        + 'the quote anchors',
      fn: async () => {
        const verdict = judgeCoverage({
          voices: VOICES,
          document: TARGET,
          asked: 3,
          quorumMet: true,
        },);
        expect(verdict.anchoredPartial,).toBe(1,);
        expect(verdict.kind,).toBe('absent',);
      },
    },),
    it({
      name: 'DROPS the claim when its quote sits inside a region paired to another source slice, '
        + 'neither coverage nor a vote for absence, so two absent voices of three decide',
      fn: async () => {
        const verdict = judgeCoverage({
          voices: VOICES,
          document: TARGET,
          foreignRegions: [FOREIGN_REGION,],
          asked: 3,
          quorumMet: true,
        },);
        expect(verdict.anchoredPartial,).toBe(0,);
        expect(verdict.misattributed,).toBe(1,);
        expect(verdict.misattributedQuotes,).toEqual([FOREIGN_SENTENCE,],);
        expect(verdict.absent,).toBe(2,);
        expect(verdict.kind,).toBe('absent',);
      },
    },),
    it({
      name: 'KEEPS a FULL claim inside a foreign region, which is what an original merged into its '
        + 'neighbour\'s paragraph looks like from here',
      fn: async () => {
        const verdict = judgeCoverage({
          voices: [
            voiceOf({
              modelId: 'hf:cat/Cat-A' as RosterModelId,
              coverage: 'full',
              quote: FOREIGN_SENTENCE,
            },),
          ],
          document: TARGET,
          foreignRegions: [FOREIGN_REGION,],
          asked: 1,
          quorumMet: true,
        },);
        expect(verdict.anchoredFull,).toBe(1,);
        expect(verdict.misattributed,).toBe(0,);
        expect(verdict.kind,).toBe('carried',);
      },
    },),
    it({
      name: 'KEEPS a claim whose quote lies outside every foreign region',
      fn: async () => {
        const verdict = judgeCoverage({
          voices: [
            voiceOf({
              modelId: 'hf:cat/Cat-A' as RosterModelId,
              coverage: 'partial',
              quote: 'naps on its cushion at noon',
            },),
          ],
          document: TARGET,
          foreignRegions: [FOREIGN_REGION,],
          asked: 1,
          quorumMet: true,
        },);
        expect(verdict.anchoredPartial,).toBe(1,);
        expect(verdict.misattributed,).toBe(0,);
      },
    },),
  ],
},);
