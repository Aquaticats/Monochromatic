import type { AdjudicatedIssue, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Checker sensitivity inputs
// The texts and accepted issues the checker sensitivity runner asks the
// resolution checkers about.
//
// These are cat-themed invention. No corpus text, licensed or otherwise, takes
// part, and nothing here is written anywhere.

/**
 Original the checkers judge against.
 */
export const SOURCE_TEXT: string = '猫猫在窗台上睡觉，太阳移动时她会醒来。';

/**
 Accepted issue every single-issue case asks about: the progressive gloss.
 */
export const TENSE_ISSUE: AdjudicatedIssue = {
  issueId: 'adjudicated/tense',
  status: 'accepted',
  severity: 'major',
  claims: [
    {
      claimId: 'claim/tense',
      claim: {
        category: 'style/awkward-phrasing',
        severity: 'major',
        summary: 'Progressive gloss "is doing the sleeping" reads as machine output.',
        spans: [],
      },
    },
  ],
  tallies: {},
};

/**
 Translation carrying the defect, used as the baseline for every case.
 */
export const DEFECTIVE_TEXT: string = 'The cat is doing the sleeping on the windowsill, and she wakes when the sun moves.';

/**
 Second accepted issue of the mixed sheet: a genuine mistranslation.
 */
export const MEANING_ISSUE: AdjudicatedIssue = {
  issueId: 'adjudicated/meaning',
  status: 'accepted',
  severity: 'critical',
  claims: [
    {
      claimId: 'claim/meaning',
      claim: {
        category: 'accuracy/mistranslation',
        severity: 'critical',
        summary: 'The original says the cat wakes when the sun moves, not that she sleeps through it.',
        spans: [],
      },
    },
  ],
  tallies: {},
};

/**
 Third accepted issue of the mixed sheet: a fabricated defect that is not in
 the text at all.

 Nothing in either version mentions a dog. A checker reading the revision can
 only answer `not-fixed` or refuse; one that answers `fixed` is agreeing with
 the sheet rather than reading.
 */
export const ABSENT_ISSUE: AdjudicatedIssue = {
  issueId: 'adjudicated/absent',
  status: 'accepted',
  severity: 'major',
  claims: [
    {
      claimId: 'claim/absent',
      claim: {
        category: 'accuracy/omission',
        severity: 'major',
        summary: 'The translation omits the dog barking in the garden.',
        spans: [],
      },
    },
  ],
  tallies: {},
};

/**
 Candidate fixing the tense only: the meaning defect survives untouched and
 the fabricated one was never there.
 */
export const MIXED_SHEET_PATCHED_TEXT: string =
  'The cat sleeps on the windowsill, and she sleeps on through the sun moving.';

/**
 Candidate repairing all three stated defects.
 */
export const ALL_FIXED_PATCHED_TEXT: string = 'The cat sleeps on the windowsill, she wakes when the sun moves, '
  + 'and a dog barks in the garden.';

/**
 Original of the all-fixed sheet, which carries the dog clause the issue
 complains of omitting.
 */
export const ALL_FIXED_SOURCE_TEXT: string = `${SOURCE_TEXT}花园里有狗在叫。`;

/**
 One single-issue case: the candidate, and what a discriminating checker
 answers about it.

 @example
 ```ts
 const first: SingleIssueCase = { label: 'untouched', patchedText, expectation: 'not-fixed', };
 ```
 */
type SingleIssueCase = {
  /**
   Case name for the verdict line.
   */
  readonly label: string;

  /**
   Candidate the checkers judge.
   */
  readonly patchedText: string;

  /**
   What a discriminating checker should answer.
   */
  readonly expectation: string;
};

/**
 The three single-issue cases that separate a discriminating checker from a
 rubber-stamping one, in run order.
 */
export const SINGLE_ISSUE_CASES: readonly SingleIssueCase[] = [
  {
    label: 'genuinely-fixed',
    patchedText: 'The cat sleeps on the windowsill, and she wakes when the sun moves.',
    expectation: 'fixed',
  },
  {
    // The candidate IS the defective text. Nothing was repaired at all, and a
    // checker calling this fixed is answering the question it was asked with
    // the answer it always gives.
    label: 'untouched',
    patchedText: DEFECTIVE_TEXT,
    expectation: 'not-fixed',
  },
  {
    // The gloss is gone, so the stated defect is addressed, but the rewrite
    // drops the second clause. A checker reading only for the issue text will
    // call this fixed; one reading the revision will call it worse.
    label: 'fixed-but-damaged',
    patchedText: 'The cat sleeps on the windowsill.',
    expectation: 'fixed-or-worse',
  },
];

//endregion Checker sensitivity inputs
