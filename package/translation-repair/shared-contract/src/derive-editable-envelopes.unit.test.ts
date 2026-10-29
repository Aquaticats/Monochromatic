/**
 Tests the order in which overlapping accepted issues are carried by the
 editable envelope they merge into. The other behaviors of the derivation (only
 accepted issues cut an envelope, touching spans merge, an insertion keeps its
 place) are pinned in `apply-patch.unit.test.ts`. Fixtures are cat-themed
 invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  type AdjudicatedIssue,
  deriveEditableEnvelopes,
} from '../dist/final/node/index.mjs';
import { envelopeSpansOf, } from './envelope-span.test-fixture.ts';

/**
 Translation the issues' spans cut from.
 */
const TARGET_TEXT = 'The cat naps in the sun.';

/**
 Builds one accepted issue whose one claim marks a span of the translation.

 @param id - what the issue is called after the span it marks

 @param start - first offset of the span

 @param end - offset the span stops before

 @returns Issue the panel accepted

 @example
 ```ts
 const issue = acceptedIssueOver({ id: 'wide', start: 4, end: 12, },);
 ```
 */
function acceptedIssueOver(
  {
    id,
    start,
    end,
  }: {
    readonly id: string;
    readonly start: number;
    readonly end: number;
  },
): AdjudicatedIssue {
  return {
    issueId: `adjudicated/${id}`,
    status: 'accepted',
    severity: 'minor',
    claims: [
      {
        claimId: `issue/${id}`,
        claim: {
          category: 'accuracy/mistranslation',
          severity: 'minor',
          summary: `The ${id} span drifts from the original.`,
          spans: [
            {
              side: 'target',
              nodeId: 'block/1',
              nodeHash: hashContent({ content: TARGET_TEXT, },),
              startOffset: start,
              endOffset: end,
              quotedText: TARGET_TEXT.slice(
                start,
                end,
              ),
            },
          ],
        },
      },
    ],
    tallies: {},
  };
}

await describe({
  name: deriveEditableEnvelopes.name,
  children: [
    it({
      name: 'CUTS ENVELOPES IN DOCUMENT ORDER WHATEVER ORDER THE ISSUES ARRIVED IN, the later span given first',
      fn: async () => {
        /**
         Two issues with separate spans, the later one first.
         */
        const { envelopes, } = deriveEditableEnvelopes({
          issues: [
            acceptedIssueOver({ id: 'sun', start: 20, end: 23, },),
            acceptedIssueOver({ id: 'cat', start: 4, end: 7, },),
          ],
          targetText: TARGET_TEXT,
        },);

        expect(envelopes.map(function baseOf(envelope,): string {
          return envelope.baseText;
        },),).toEqual(['cat', 'sun',],);
      },
    },),
    it({
      name: 'LISTS THE ISSUES OF ONE ENVELOPE SHORTEST SPAN FIRST WHEN THEIR SPANS START AT ONE OFFSET, whichever '
        + 'order the issues arrived in',
      fn: async () => {
        /**
         The two issues, the wider one first, both starting at offset four.
         */
        const { envelopes, } = deriveEditableEnvelopes({
          issues: [
            acceptedIssueOver({ id: 'wide', start: 4, end: 12, },),
            acceptedIssueOver({ id: 'narrow', start: 4, end: 7, },),
          ],
          targetText: TARGET_TEXT,
        },);

        expect(envelopeSpansOf(envelopes,),).toEqual([
          {
            startOffset: 4,
            endOffset: 12,
            baseText: 'cat naps',
            issueIds: ['adjudicated/narrow', 'adjudicated/wide',],
          },
        ],);
      },
    },),
    it({
      name: 'KEEPS A NOTE BETWEEN TRANSLATORS OUT OF EVERY ENVELOPE when the two spans of a quote across it stand either '
        + 'side of the note, so an editor is never handed the note to rewrite or drop',
      fn: async () => {
        /**
         Translation whose first line ends with a note, and the quote the critic copied across it.
         */
        const noted = 'The cat likes the sun.{/* 这句不太对劲 */}\nThen it naps on the cushion.\n';
        /**
         Where the note stands.
         */
        const noteStart = noted.indexOf('{/*',);
        /**
         Where the note ends.
         */
        const noteEnd = noted.indexOf('*/}',) + '*/}'.length;
        /**
         Where the second line's words start.
         */
        const laterStart = noted.indexOf('Then',);
        /**
         Where they end.
         */
        const laterEnd = noted.indexOf('.\n', laterStart,) + 1;
        /**
         One accepted issue whose claim holds the two stretches the quote came back as.
         */
        const crossing: AdjudicatedIssue = {
          issueId: 'adjudicated/crossing',
          status: 'accepted',
          severity: 'minor',
          claims: [
            {
              claimId: 'issue/crossing',
              claim: {
                category: 'accuracy/mistranslation',
                severity: 'minor',
                summary: 'The two lines drift from the original.',
                spans: [
                  {
                    side: 'target',
                    nodeId: 'block/1',
                    nodeHash: hashContent({ content: noted, },),
                    startOffset: 0,
                    endOffset: noteStart,
                    quotedText: noted.slice(
                      0,
                      noteStart,
                    ),
                  },
                  {
                    side: 'target',
                    nodeId: 'block/1',
                    nodeHash: hashContent({ content: noted, },),
                    startOffset: laterStart,
                    endOffset: laterEnd,
                    quotedText: noted.slice(
                      laterStart,
                      laterEnd,
                    ),
                  },
                ],
              },
            },
          ],
          tallies: {},
        };
        const { envelopes, } = deriveEditableEnvelopes({
          issues: [crossing,],
          targetText: noted,
        },);

        expect(envelopeSpansOf(envelopes,).map(function baseOf(span,): string {
          return span.baseText;
        },),).toEqual(['The cat likes the sun.', 'Then it naps on the cushion.',],);
        expect(envelopeSpansOf(envelopes,).some(function holdsNote(span,): boolean {
          return (span.startOffset < noteEnd) && (span.endOffset > noteStart);
        },),).toBe(false,);
      },
    },),
  ],
},);
