/**
 Tests for the worse-voted strip re-applying the gate the patch passed (ledger
 L3 with L4): once an edit's lost markup can be written by a sibling edit, a
 subset of a gated patch is no longer gated by construction.
 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import type { AdjudicatedIssue, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type EditorStageResult,
  NOBODY_WROTE_IT,
  stripWorseVotedEdits,
} from '../dist/final/node/index.mjs';

/**
 Archive text of two clauses whose footnote references the patch swaps.
 */
const TARGET_TEXT = 'The cat[^2] napped. The kitten[^1] played.';

/**
 First clause, the first envelope.
 */
const FIRST_BASE = 'The cat[^2] napped.';

/**
 Second clause, the second envelope.
 */
const SECOND_BASE = 'The kitten[^1] played.';

/**
 The two envelopes in document order.
 */
const ENVELOPES = [
  {
    envelopeId: 'envelope/first',
    startOffset: 0,
    endOffset: FIRST_BASE.length,
    baseText: FIRST_BASE,
    baseHash: hashContent({ content: FIRST_BASE, },),
    issueIds: ['adjudicated/first',],
  },
  {
    envelopeId: 'envelope/second',
    startOffset: FIRST_BASE.length + 1,
    endOffset: TARGET_TEXT.length,
    baseText: SECOND_BASE,
    baseHash: hashContent({ content: SECOND_BASE, },),
    issueIds: ['adjudicated/second',],
  },
] as const;

/**
 Accepted issue served by one envelope.

 @param suffix - which envelope's issue

 @returns Issue

 @example
 ```ts
 const first = acceptedIssue({ suffix: 'first', },);
 ```
 */
function acceptedIssue({ suffix, }: { readonly suffix: string; },): AdjudicatedIssue {
  return {
    issueId: `adjudicated/${suffix}`,
    status: 'accepted',
    severity: 'major',
    claims: [],
    tallies: {},
  };
}

/**
 Editor result whose patch swapped the two references, both edits applied
 under the enforced gate.
 */
const SWAPPED: EditorStageResult = {
  patch: {
    patchedText: 'The cat[^1] napped. The kitten[^2] played.',
    applied: [
      {
        envelopeId: 'envelope/first',
        baseHash: hashContent({ content: FIRST_BASE, },),
        newText: 'The cat[^1] napped.',
      },
      {
        envelopeId: 'envelope/second',
        baseHash: hashContent({ content: SECOND_BASE, },),
        newText: 'The kitten[^2] played.',
      },
    ],
    rejected: [],
  },
  heardEditors: 1,
  rounds: [],
  findings: [],
  shippedProducer: NOBODY_WROTE_IT,
  preservation: {
    mode: 'enforce',
    licensedQuotes: new Map([
      ['envelope/first', [FIRST_BASE,],],
      ['envelope/second', [SECOND_BASE,],],
    ],),
    removableQuotes: new Map(),
    sourceText: 'The cat[^1] and the kitten[^2].',
  },
};

await describe({
  name: stripWorseVotedEdits.name,
  children: [
    it({
      name: 'REFUSES the kept side of a swap once the strip removes the edit that wrote its reference, '
        + 'since re-applying the kept edits with the gate skipped would ship the first clause with its '
        + 'reference gone from the patch',
      fn: async () => {
        const strip = stripWorseVotedEdits({
          editor: SWAPPED,
          envelopes: ENVELOPES,
          creditableIssues: [
            acceptedIssue({ suffix: 'first', },),
            acceptedIssue({ suffix: 'second', },),
          ],
          tallies: {
            'adjudicated/first': { fixed: 3, notFixed: 0, worse: 0, resolved: true, regressed: false, },
            'adjudicated/second': { fixed: 1, notFixed: 0, worse: 2, resolved: false, regressed: true, },
          },
          targetText: TARGET_TEXT,
        },);
        if (!strip.stripped)
          throw new Error('the worse-voted edit was not stripped',);
        expect(strip.editor.patch.applied,).toHaveLength(0,);
        expect(strip.editor.patch.patchedText,).toBe(TARGET_TEXT,);
        expect(strip.editor.patch.rejected.map(function toReason(rejection,): string {
          return rejection.reason;
        },),).toContain('preservation-lost-markup (footnote-reference)',);
      },
    },),

    it({
      name: 'KEEPS the kept edit when it moved nothing, the control showing the refusal is about the move',
      fn: async () => {
        const strip = stripWorseVotedEdits({
          editor: {
            ...SWAPPED,
            patch: {
              patchedText: 'The cat[^2] dozed. The kitten[^2] played.',
              applied: [
                {
                  envelopeId: 'envelope/first',
                  baseHash: hashContent({ content: FIRST_BASE, },),
                  newText: 'The cat[^2] dozed.',
                },
                {
                  envelopeId: 'envelope/second',
                  baseHash: hashContent({ content: SECOND_BASE, },),
                  newText: 'The kitten[^2] played.',
                },
              ],
              rejected: [],
            },
          },
          envelopes: ENVELOPES,
          creditableIssues: [
            acceptedIssue({ suffix: 'first', },),
            acceptedIssue({ suffix: 'second', },),
          ],
          tallies: {
            'adjudicated/first': { fixed: 3, notFixed: 0, worse: 0, resolved: true, regressed: false, },
            'adjudicated/second': { fixed: 1, notFixed: 0, worse: 2, resolved: false, regressed: true, },
          },
          targetText: TARGET_TEXT,
        },);
        if (!strip.stripped)
          throw new Error('the worse-voted edit was not stripped',);
        expect(strip.editor.patch.patchedText,).toBe('The cat[^2] dozed. The kitten[^1] played.',);
      },
    },),
  ],
},);
