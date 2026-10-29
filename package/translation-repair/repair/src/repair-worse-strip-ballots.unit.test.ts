/**
 Tests for the ballots that strip an edit from a winning patch (ledger L3).

 TWO BALLOTS FOR UNDO TOO (owner, 2026-10-25, "Two ballots for undo too"): an
 edit is stripped only where two or more ballots name worse and the worse
 weight exceeds the fixed and not-fixed weight together, the condition that
 sets `regressed` on the issue's tally. The tallies here come from the real
 `tallyResolutionChecks` over scripted ballots, so a case moves only when the
 definition does and not when a literal does.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure';
import {
  type ResolutionVerdict,
  tallyResolutionChecks,
} from '@monochromatic-dev/translation-repair-adjudication';
import { UNATTRIBUTED_TEXT, } from '@monochromatic-dev/translation-repair-house-rule';
import type { AdjudicatedIssue, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type EditorStageResult,
  NOBODY_WROTE_IT,
  stripWorseVotedEdits,
} from '../dist/final/node/index.mjs';

/**
 Archive text of two clauses.
 */
const TARGET_TEXT = 'The cat napped. The kitten played.';

/**
 First clause, the first envelope.
 */
const FIRST_BASE = 'The cat napped.';

/**
 Second clause, the second envelope.
 */
const SECOND_BASE = 'The kitten played.';

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
 Issue ids in sheet order.
 */
const ISSUE_IDS = [
  'adjudicated/first',
  'adjudicated/second',
] as const;

/**
 Editor result whose patch rewrote both clauses.
 */
const REWRITTEN: EditorStageResult = {
  patch: {
    patchedText: 'The cat dozed. The kitten pounced.',
    applied: [
      {
        envelopeId: 'envelope/first',
        baseHash: hashContent({ content: FIRST_BASE, },),
        newText: 'The cat dozed.',
      },
      {
        envelopeId: 'envelope/second',
        baseHash: hashContent({ content: SECOND_BASE, },),
        newText: 'The kitten pounced.',
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
    sourceText: 'The cat napped. The kitten played.',
  },
};

/**
 Accepted issue served by one envelope.

 @param envelopeName - which envelope's issue, `first` or `second`

 @returns Issue

 @example
 ```ts
 const first = issueServedBy({ envelopeName: 'first', },);
 ```
 */
function issueServedBy({ envelopeName, }: { readonly envelopeName: string; },): AdjudicatedIssue {
  return {
    claims: [],
    tallies: {},
    severity: 'major',
    status: 'accepted',
    issueId: `adjudicated/${envelopeName}`,
  };
}

/**
 Strips the second clause's edit under the verdicts its checkers cast, the
 first clause being confirmed by three checkers.

 @param secondVerdicts - verdict each checker casts on the second clause's issue, in order

 @returns What the strip made of the patch

 @example
 ```ts
 const strip = stripUnder({ secondVerdicts: ['worse', 'fixed',], },);
 ```
 */
function stripUnder(
  { secondVerdicts, }: { readonly secondVerdicts: readonly ResolutionVerdict[]; },
): ReturnType<typeof stripWorseVotedEdits> {
  /**
   Ballots keyed by checker: three confirm the first clause, and each given
   verdict is one more checker's ballot on the second.
   */
  const ballots = Object.fromEntries(['a', 'b', 'c', 'd',].map(function toBallot(checker, at,) {
    return [
      checker,
      {
        verdicts: {
          ...((at < 3) ? { 'adjudicated/first': 'fixed', } : {}),
          ...((secondVerdicts[at] === undefined) ? {} : { 'adjudicated/second': secondVerdicts[at], }),
        } as Readonly<Record<string, ResolutionVerdict>>,
        findings: [],
      },
    ];
  },),);
  return stripWorseVotedEdits({
    editor: REWRITTEN,
    envelopes: ENVELOPES,
    creditableIssues: [
      issueServedBy({ envelopeName: 'first', },),
      issueServedBy({ envelopeName: 'second', },),
    ],
    tallies: tallyResolutionChecks({
      issueIds: ISSUE_IDS,
      ballots,
      authorship: UNATTRIBUTED_TEXT,
    },),
    targetText: TARGET_TEXT,
  },);
}

await describe({
  name: stripWorseVotedEdits.name,
  children: [
    it({
      name: 'KEEPS AN UNCONFIRMED EDIT ONE CHECKER VOTED WORSE beside one fixed: one worse ballot is one voice '
        + '(owner, 2026-10-25, "Two ballots for undo too")',
      fn: async () => {
        expect(stripUnder({ secondVerdicts: ['worse', 'fixed',], },),).toEqual({ stripped: false, },);
      },
    },),

    it({
      name: 'KEEPS AN UNCONFIRMED EDIT ONE CHECKER VOTED WORSE beside two not-fixed',
      fn: async () => {
        expect(stripUnder({ secondVerdicts: ['worse', 'not-fixed', 'not-fixed',], },),).toEqual({ stripped: false, },);
      },
    },),

    it({
      name: 'STRIPS AN UNCONFIRMED EDIT TWO CHECKERS VOTED WORSE against one fixed, the positive control for the '
        + 'lone ballot: the cases are reading the count of worse ballots and not their absence',
      fn: async () => {
        /**
         What the strip made of the patch.
         */
        const strip = stripUnder({ secondVerdicts: ['worse', 'worse', 'fixed',], },);
        if (!strip.stripped)
          throw new Error('the edit two checkers voted worse was not stripped',);
        expect(strip.issueIds,).toEqual(['adjudicated/second',],);
        expect(strip.editor.patch.patchedText,).toBe('The cat dozed. The kitten played.',);
      },
    },),

    it({
      name: 'KEEPS AN UNCONFIRMED EDIT whose two worse ballots are level with two fixed ballots: worse has to '
        + 'strictly outweigh the rest as well as come from two judges',
      fn: async () => {
        expect(
          stripUnder({ secondVerdicts: ['worse', 'worse', 'fixed', 'fixed',], },),
        ).toEqual({ stripped: false, },);
      },
    },),
  ],
},);
