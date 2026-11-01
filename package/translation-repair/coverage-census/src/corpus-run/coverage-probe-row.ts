import type {
  CoverageAnswer,
  CoverageCandidate,
} from '@monochromatic-dev/translation-repair-archive/ts';
import { exchangeFailureText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';

//region Coverage probe row
// One candidate's question and what came back, as the coverage probe keeps it.

/**
 One candidate's question and what came back.

 @example
 ```ts
 const row: ProbeRow = failedRow({ entryId: 'Mittens', candidate, asked: 8, error, },);
 ```
 */
export type ProbeRow = {
  /**
   Corpus entry the passage belongs to.
   */
  readonly entryId: string;

  /**
   Whether a whole section or one block was refused.
   */
  readonly scale: string;

  /**
   Where the passage sits, for reading beside the censuses.
   */
  readonly where: string;

  /**
   Passage length, since a long one is a different question from a line.
   */
  readonly sourceChars: number;

  /**
   What the roster concluded.
   */
  readonly kind: string;

  /**
   Voices that anchored full coverage in the document.
   */
  readonly anchoredFull: number;

  /**
   Voices that anchored partial coverage.
   */
  readonly anchoredPartial: number;

  /**
   Voices reporting nothing renders it.
   */
  readonly absent: number;

  /**
   Voices whose quote was not in the document.
   */
  readonly unanchored: number;

  /**
   Voices heard at all.
   */
  readonly heard: number;

  /**
   Models asked, which the verdict threshold is taken over.
   */
  readonly asked: number;

  /**
   Quotes claimed and not found, kept because a near miss and an invention are
   different failures that the counts alone cannot tell apart.
   */
  readonly unanchoredQuotes: readonly string[];

  /**
   Anchored quotes, so a reader can check the verdict against the archive.
   */
  readonly evidence: readonly string[];

  /**
   Roster degradation findings, empty when quorum was met.
   */
  readonly findings: readonly string[];
};

/**
 Where a candidate passage sits, in the terms the censuses print.

 @param candidate - passage no pairing covers

 @returns `section N`, or `pair N block M`

 @example
 ```ts
 const where = whereOf({ candidate, },);
 ```
 */
export function whereOf({ candidate, }: { readonly candidate: CoverageCandidate; },): string {
  return (candidate.scale === 'section')
    ? `section ${String(candidate.sourceIndex,)}`
    : `pair ${String(candidate.pairIndex,)} block ${String(candidate.sourceIndex,)}`;
}

/**
 The row of a candidate the roster answered about.

 @param entryId - entry the passage belongs to

 @param candidate - passage asked about

 @param answer - what the roster concluded

 @returns The row

 @example
 ```ts
 const row = answeredRow({ entryId, candidate, answer, },);
 ```
 */
export function answeredRow(
  {
    entryId,
    candidate,
    answer,
  }: {
    readonly entryId: string;
    readonly candidate: CoverageCandidate;
    readonly answer: CoverageAnswer;
  },
): ProbeRow {
  return {
    entryId,
    scale: candidate.scale,
    where: whereOf({ candidate, },),
    sourceChars: candidate.sourceText
      .length,
    kind: answer.verdict
      .kind,
    anchoredFull: answer.verdict
      .anchoredFull,
    anchoredPartial: answer.verdict
      .anchoredPartial,
    absent: answer.verdict
      .absent,
    unanchored: answer.verdict
      .unanchored,
    heard: answer.verdict
      .heard,
    asked: answer.verdict
      .asked,
    unanchoredQuotes: answer.verdict
      .unanchoredQuotes,
    evidence: answer.verdict
      .evidence,
    findings: answer.findings,
  };
}

/**
 The row of a candidate whose call failed, kept because a measurement that
 drops its failures reports a success rate of one.

 @param entryId - entry the passage belongs to

 @param candidate - passage asked about

 @param asked - models the roster held

 @param error - what the call threw, of unknown type by construction

 @returns A row of kind `failed` with every count zero and the failure as its
 one finding

 @example
 ```ts
 const row = failedRow({ entryId, candidate, asked: 8, error, },);
 ```
 */
export function failedRow(
  {
    entryId,
    candidate,
    asked,
    error,
  }: {
    readonly entryId: string;
    readonly candidate: CoverageCandidate;
    readonly asked: number;
    readonly error: unknown;
  },
): ProbeRow {
  return {
    entryId,
    scale: candidate.scale,
    where: whereOf({ candidate, },),
    sourceChars: candidate.sourceText
      .length,
    kind: 'failed',
    anchoredFull: 0,
    anchoredPartial: 0,
    absent: 0,
    unanchored: 0,
    heard: 0,
    asked,
    evidence: [],
    unanchoredQuotes: [],
    findings: [exchangeFailureText({ error, },),],
  };
}

//endregion Coverage probe row
