/**
 Tests for the editor prompt sheet and its wire resolution.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { messageText, } from '@monochromatic-dev/translation-repair-provider';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import type {
  AdjudicatedIssue,
  EditableEnvelope,
} from '@monochromatic-dev/translation-repair-shared-contract';
import {
  buildEditorMessages,
  isEditorReportWire,
  resolveEditorEdits,
} from '../dist/final/node/index.mjs';

/**
 Invented translation the fixtures cut regions from.
 */
const TARGET_TEXT = 'The cat naps in the sun. It chases red butterflies. The bowl stays full.';

/**
 Envelope over chosen offsets of the fixture translation.
 */
function envelopeAt(
  {
    startOffset,
    endOffset,
    issueIds,
  }: {
    readonly startOffset: number;
    readonly endOffset: number;
    readonly issueIds: readonly string[];
  },
): EditableEnvelope {
  /**
   Exact text occupying the region.
   */
  const baseText = TARGET_TEXT.slice(startOffset, endOffset,);

  return {
    envelopeId: `envelope/${
      hashContent({
        content: JSON.stringify([
          startOffset,
          endOffset,
          baseText,
        ],),
      },)
    }`,
    startOffset,
    endOffset,
    baseText,
    baseHash: hashContent({ content: baseText, },),
    issueIds,
  };
}

/**
 Accepted single-claim issue for prompt rendering.
 */
function acceptedIssue(
  { suffix, }: { readonly suffix: string; },
): AdjudicatedIssue {
  return {
    issueId: `adjudicated/${suffix}`,
    status: 'accepted',
    severity: 'major',
    claims: [
      {
        claimId: `issue/${suffix}`,
        claim: {
          category: 'accuracy/mistranslation',
          severity: 'major',
          summary: `The ${suffix} clause drifts from the source.`,
          spans: [],
        },
      },
    ],
    tallies: {},
  };
}

/**
 Fixture regions: a replacement then an insertion point.
 */
const ENVELOPES: readonly EditableEnvelope[] = [
  envelopeAt({
    startOffset: 4,
    endOffset: 12,
    issueIds: ['adjudicated/napping',],
  },),
  envelopeAt({
    startOffset: 52,
    endOffset: 52,
    issueIds: ['adjudicated/missing',],
  },),
];

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: buildEditorMessages.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'numbers regions with issues, current text, and context',
          fn: async () => {
            /** Plan for the two-region sheet. */
            const plan = buildEditorMessages({
              sourceText: '猫猫在太阳下打盹。它追红蝴蝶。碗一直是满的。',
              targetText: TARGET_TEXT,
              envelopes: ENVELOPES,
              issues: [
                acceptedIssue({ suffix: 'napping', },),
                acceptedIssue({ suffix: 'missing', },),
              ],
            },);
            /** Sheet text shown to the editor. */
            const sheet = plan.messages[1]?.content ?? '';
            expect(sheet,).toContain('REGION 1',);
            expect(sheet,).toContain('CURRENT TEXT: cat naps',);
            expect(sheet,).toContain('The napping clause drifts from the source.',);
            expect(sheet,).toContain('REGION 2',);
            expect(sheet,).toContain('CURRENT TEXT: (empty; content is missing here)',);
            expect(sheet,).toContain('«REGION 2»',);
            expect(plan.envelopes,).toEqual(ENVELOPES,);
          },
        },),

        it({
          name: 'instructs natural feeling-carrying prose without licensing content loss',
          fn: async () => {
            /** Plan whose system rules are under policy probing. */
            const plan = buildEditorMessages({
              sourceText: '原文',
              targetText: TARGET_TEXT,
              envelopes: ENVELOPES,
              issues: [],
            },);
            /** System rule block shown to the editor. */
            const system = plan.messages[0]?.content ?? '';
            expect(system,).toContain('Emotional completeness and naturalness outrank word-for-word correspondence',);
            // Naturalness and completeness must both hold; the recast
            // permission never becomes permission to drop content.
            expect(system,).toContain('Naturalness never licenses dropping content',);
            expect(system,).toContain('Never introduce content the ORIGINAL does not support',);
            // Foreign phrases keep their wording and gain a meaning beside it.
            expect(system,).toContain('in that original wording, and put its meaning alongside it',);
            expect(system,).toContain('Never replace such a phrase with its meaning alone',);
            // The exception must not swallow the source language itself.
            expect(system,).toContain('Render ordinary prose in the ORIGINAL\'s own language fully into the TRANSLATION\'s language',);
            expect(system,).toContain('does not license leaving an entire source-language quotation untranslated',);
          },
        },),

        it({
          name: 'keeps envelope ids and hashes off the sheet',
          fn: async () => {
            /** Plan for the two-region sheet. */
            const plan = buildEditorMessages({
              sourceText: '原文',
              targetText: TARGET_TEXT,
              envelopes: ENVELOPES,
              issues: [],
            },);
            /** Whole prompt joined for scanning. */
            const wholePrompt = plan
              .messages
              .map(function toContent(message,) {
                return messageText({ message, },);
              },)
              .join('\n',);
            expect(wholePrompt.includes('envelope/',),).toBe(false,);
            expect(wholePrompt.includes('baseHash',),).toBe(false,);
          },
        },),

        it({
          name: 'splices a calibration addendum in as one more enforced rule',
          fn: async () => {
            /** Experimental rule line under test. */
            const addendum = 'Translate every whisker clause separately.';

            /** Plan carrying the addendum. */
            const plan = buildEditorMessages({
              sourceText: '原文',
              targetText: TARGET_TEXT,
              envelopes: ENVELOPES,
              issues: [],
              editorRuleAddendum: addendum,
            },);

            /** System prompt as the editor receives it. */
            const systemPrompt = plan.messages[0]?.content ?? '';
            expect(systemPrompt,).toContain(`- ${addendum}`,);
            // The addendum lands inside the rule list, before the reply shape.
            expect(systemPrompt.indexOf(`- ${addendum}`,),)
              .toBeLessThan(systemPrompt.indexOf('Reply with ONLY',),);

            /** Baseline plan without the addendum. */
            const baseline = buildEditorMessages({
              sourceText: '原文',
              targetText: TARGET_TEXT,
              envelopes: ENVELOPES,
              issues: [],
            },);
            expect((baseline.messages[0]?.content ?? '').includes(addendum,),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: isEditorReportWire.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'accepts well-formed reports and rejects malformed ones',
          fn: async () => {
            expect(isEditorReportWire({
              edits: [
                {
                  region: 1,
                  newText: 'kitten dozes',
                },
              ],
            },),).toBe(true,);
            expect(isEditorReportWire({ edits: [], },),).toBe(true,);
            expect(isEditorReportWire({},),).toBe(false,);
            expect(isEditorReportWire({ edits: [{ region: 'one', newText: 'x', },], },),)
              .toBe(false,);
            expect(isEditorReportWire({ edits: [{ region: 1.5, newText: 'x', },], },),)
              .toBe(false,);
            expect(isEditorReportWire({ edits: [{ region: 1, },], },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: resolveEditorEdits.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'binds region numbers to envelopes with their hashes',
          fn: async () => {
            /** Resolution of edits on both regions. */
            const { operations, findings, } = resolveEditorEdits({
              wire: {
                edits: [
                  {
                    region: 1,
                    newText: 'kitten dozes',
                  },
                  {
                    region: 2,
                    newText: 'It purrs at dusk. ',
                  },
                ],
              },
              envelopes: ENVELOPES,
            },);
            expect(findings,).toHaveLength(0,);
            expect(operations,).toHaveLength(2,);
            expect(operations[0]?.envelopeId,).toBe(ENVELOPES[0]?.envelopeId,);
            expect(operations[0]?.baseHash,).toBe(ENVELOPES[0]?.baseHash,);
            expect(operations[1]?.newText,).toBe('It purrs at dusk. ',);
          },
        },),

        it({
          name: 'records out-of-range and duplicate regions as findings',
          fn: async () => {
            /** Resolution of a ballot with two bad references. */
            const { operations, findings, } = resolveEditorEdits({
              wire: {
                edits: [
                  {
                    region: 0,
                    newText: 'x',
                  },
                  {
                    region: 9,
                    newText: 'x',
                  },
                  {
                    region: 1,
                    newText: 'kitten dozes',
                  },
                  {
                    region: 1,
                    newText: 'tabby rests',
                  },
                ],
              },
              envelopes: ENVELOPES,
            },);
            expect(findings,).toContain('edit-region-out-of-range (0)',);
            expect(findings,).toContain('edit-region-out-of-range (9)',);
            expect(findings,).toContain('duplicate-edit (1)',);
            expect(operations,).toHaveLength(1,);
            expect(operations[0]?.newText,).toBe('kitten dozes',);
          },
        },),

        it({
          name: 'FOLDS an invisible variant out of an edit at intake and names it, so the checkers '
            + 'judge the bytes that ship',
          fn: async () => {
            const { operations, findings, } = resolveEditorEdits({
              wire: {
                edits: [
                  {
                    region: 1,
                    newText: 'a part\u2011time kitten',
                  },
                ],
              },
              envelopes: ENVELOPES,
            },);
            expect(operations[0]?.newText,).toBe('a part-time kitten',);
            expect(findings,).toStrictEqual(['invisible-variant-folded (U+2011 x1)',],);
          },
        },),
      ],
    },),

    describe({
      name: `${isEditorReportWire.name} nulls`,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES a reply that is no object and an edit that is no object, since nothing there names a '
            + 'region or its text',
          fn: async () => {
            // NULL IS THE VALUE THE GUARDS ARE FOR: a number is refused by the
            // field reads behind them anyway, and a field read off null throws.
            expect(isEditorReportWire(null,),).toBe(false,);
            expect(isEditorReportWire({ edits: [null,], },),).toBe(false,);
            expect(isEditorReportWire(5,),).toBe(false,);
            expect(isEditorReportWire({ edits: [5,], },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
