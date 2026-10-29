/**
 Tests that the sheets which MEASURE are told what this corpus is written
 under, and told how the rules land on their own verdicts.

 WHY THIS FILE EXISTS. The rendering auditor, the resolution checker and the
 introduced-defect prober all grade text against the ORIGINAL, and none had
 ever been shown the house rules. Nothing ships from any of the three, which
 is why they were fixed after the deciding sheets rather than with them; what
 they do decide is which defects get worked on next. An auditor that has not
 been told reader protection exists scores a deliberately vague passage as an
 omission, and that grade is then quoted as a measurement of the pipeline.

 THE VERDICT MAPPING IS ASSERTED SEPARATELY from the shared block. The three
 verdict vocabularies are disjoint, so the block deliberately names none of
 them and each sheet supplies its own line. A splice alone would leave a
 checker knowing the rule and not knowing what to answer.

 ONE SENTENCE PER SHEET stands for the block, so rewording the house rules
 does not break three tests at once. The sentence chosen is the one that
 decides the protected case.

 Fixtures are cat-themed invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { buildRenderingAuditMessages, } from '@monochromatic-dev/translation-repair-rendering-audit';
import type { AdjudicatedIssue, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  buildIntroducedDefectMessages,
  buildResolutionMessages,
} from '../dist/final/node/index.mjs';

/**
 Sentence of the house rules that decides the protected case.
 */
const READER_PROTECTION = 'Reader protection outranks completeness';

/**
 Sentence the measuring sheets get and the judging sheets do not.
 */
const SUPPORTED_IS_NOT_ENOUGH = 'SUPPORTED BY THE ORIGINAL IS NOT ENOUGH TO MAKE SOMETHING MISSING A DEFECT';

/**
 Invented passage standing in for corpus text.
 */
const SOURCE_TEXT = '猫猫在窗台上睡觉，太阳移动时她会醒来。';

/**
 Rendering of that passage.
 */
const CANDIDATE_TEXT = 'The cat sleeps on the windowsill and wakes when the sun moves.';

/**
 Accepted issue the checker and the prober are asked about.
 */
const NAPPING_ISSUE: AdjudicatedIssue = {
  issueId: 'adjudicated/napping',
  status: 'accepted',
  severity: 'major',
  claims: [
    {
      claimId: 'claim/napping',
      claim: {
        category: 'style/awkward-phrasing',
        severity: 'major',
        summary: 'Rendering flattens the cat\'s waking into a bare clause.',
        spans: [],
      },
    },
  ],
  tallies: {},
};

/**
 Pulls the standing rules out of one built exchange.

 @param messages - exchange as its builder returned it

 @returns System half, empty when the builder sent none

 @example
 ```ts
 const system = systemOf({ messages, },);
 ```
 */
function systemOf(
  { messages, }: { readonly messages: readonly { readonly role: string; readonly content: string; }[]; },
): string {
  return messages
    .find(function isSystem(message,) {
      return message.role === 'system';
    },)
    ?.content
    ?? '';
}

/**
 System half the rendering auditor is sent.
 */
const auditSystem = systemOf({
  messages: buildRenderingAuditMessages({
    subject: {
      sourceText: SOURCE_TEXT,
      candidateText: CANDIDATE_TEXT,
    },
  },),
},);

/**
 System half every resolution checker is sent.
 */
const checkerSystem = systemOf({
  messages: buildResolutionMessages({
    sourceText: SOURCE_TEXT,
    patchedText: CANDIDATE_TEXT,
    issues: [NAPPING_ISSUE,],
  },).messages,
},);

/**
 System half the introduced-defect prober is sent.
 */
const proberSystem = systemOf({
  messages: buildIntroducedDefectMessages({
    sourceText: SOURCE_TEXT,
    baselineText: CANDIDATE_TEXT,
    regions: [
      {
        envelopeId: 'envelope/napping',
        issueIds: ['adjudicated/napping',],
        before: 'wakes when the sun moves',
        editorAfter: 'stirs as the sun moves on',
      },
    ],
    issues: [NAPPING_ISSUE,],
  },).messages,
},);

await describe({
  name: 'measuring sheets carry the house rules',
  children: [
    it({
      name: 'TELLS all three sheets that reader protection outranks completeness',
      fn: async () => {
        expect(auditSystem.includes(READER_PROTECTION,),).toBe(true,);
        expect(checkerSystem.includes(READER_PROTECTION,),).toBe(true,);
        expect(proberSystem.includes(READER_PROTECTION,),).toBe(true,);
      },
    },),
    it({
      name: 'TELLS all three that the ORIGINAL supporting a detail settles nothing',
      fn: async () => {
        // The judging sheets do not get this sentence: they are choosing
        // between candidates, and this one answers an absolute question.
        expect(auditSystem.includes(SUPPORTED_IS_NOT_ENOUGH,),).toBe(true,);
        expect(checkerSystem.includes(SUPPORTED_IS_NOT_ENOUGH,),).toBe(true,);
        expect(proberSystem.includes(SUPPORTED_IS_NOT_ENOUGH,),).toBe(true,);
      },
    },),
    it({
      name: 'ACCEPTS a protected passage as no-defect-found in the audit',
      fn: async () => {
        expect(auditSystem.includes('the verdict is no-defect-found',),).toBe(true,);
      },
    },),
    it({
      name: 'REFUSES to let the checker answer fixed on an issue that is itself wrong',
      fn: async () => {
        // `fixed` would agree with the sheet rather than read the text, the
        // failure `checker-sensitivity`'s absent-issue fixture exists to catch,
        // and no verdict here means the issue should never have been filed.
        expect(checkerSystem.includes('is answered not-fixed',),).toBe(true,);
        expect(checkerSystem.includes('the verdict is worse',),).toBe(true,);
      },
    },),
    it({
      name: 'NAMES the bullet the prober now has to read against',
      fn: async () => {
        // `PROBE_RULES_HEAD` says content the AFTER text drops is damage only
        // if the ORIGINAL supports it, and on a protected detail it does. The
        // clause naming the interaction is what keeps the two rules from tying.
        expect(proberSystem.includes('even though the ORIGINAL supports the wording that went missing',),).toBe(
          true,
        );
        expect(proberSystem.includes('the verdict is no-introduced-defect-found',),).toBe(true,);
      },
    },),
    it({
      name: 'KEEPS each sheet\'s own task ahead of the policy block',
      fn: async () => {
        // The task decides; the policy qualifies it. A block arriving first
        // reads as the task itself.
        const audit = auditSystem.indexOf('You audit one translated passage',);
        const checker = checkerSystem.indexOf('You are a strict bilingual translation reviewer',);
        const prober = proberSystem.indexOf('auditing an edit for collateral damage',);
        expect(audit,).toBeGreaterThan(-1,);
        expect(checker,).toBeGreaterThan(-1,);
        expect(prober,).toBeGreaterThan(-1,);
        expect(auditSystem.indexOf(READER_PROTECTION,),).toBeGreaterThan(audit,);
        expect(checkerSystem.indexOf(READER_PROTECTION,),).toBeGreaterThan(checker,);
        expect(proberSystem.indexOf(READER_PROTECTION,),).toBeGreaterThan(prober,);
      },
    },),
  ],
},);
