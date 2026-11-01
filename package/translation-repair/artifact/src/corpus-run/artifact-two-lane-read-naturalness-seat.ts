import { requireExactKeys, } from '../artifact-exact-guard.ts';
import {
  ArtifactParseError,
  requireArray,
  requireCount,
  requireRecord,
  requireString,
} from '@monochromatic-dev/translation-repair-artifact-pool/ts';
import { reachableQuorum, } from '@monochromatic-dev/translation-repair-stage/ts';
import type {
  ArtifactNaturalnessFinding,
  ArtifactNaturalnessReviewRound,
  ArtifactNaturalnessReviewSeat,
} from './artifact-two-lane-consolidate.ts';

//region Artifact absolute naturalness seat read
// DELIBERATELY DUPLICATED, and checked rather than trusted, on the grounds
// `artifact-two-lane-comparison.ts` gives for the lane verdict.
// `uniqueNaturalnessFindings` is this artifact version's rule for a round's
// aggregate findings, and `absolute-naturalness-review-stage.ts` keeps the
// live rule (`uniqueFindings`) with the same body. The round reader recomputes
// the aggregate with this copy and refuses a file whose stored findings
// disagree, so a change to the live rule stops a corpus pass instead of
// quietly reinterpreting artifacts already on disk. Merging the two would
// remove that check (the duplicate-body census flagged the pair on
// 2026-09-28, audit area six).

/**
 Statuses a recorded seat may carry, keyed by the frozen
 `ArtifactNaturalnessReviewSeat` rather than listed, so the compiler refuses
 a status the seat record gains that this reader lacks. The order is the order
 a refusal names them in.
 */
const SEAT_STATUSES: Readonly<Record<ArtifactNaturalnessReviewSeat['status'], true>> = {
  acceptable: true,
  unacceptable: true,
  unusable: true,
};

/**
 Whether a recorded value names a status a seat may carry.

 @param value - status as the artifact carries it

 @returns Whether it is one of the statuses `SEAT_STATUSES` holds

 @example
 ```ts
 const known = isSeatStatus('acceptable',);
 ```
 */
function isSeatStatus(value: unknown,): value is ArtifactNaturalnessReviewSeat['status'] {
  return ((typeof value) === 'string')
    && Object.hasOwn(
      SEAT_STATUSES,
      value,
    );
}

/**
 Reads paragraph-located finding.

 @param value - unknown finding

 @param path - artifact path

 @returns Validated finding

 @example
 ```ts
 const finding = parseFinding({ value, path, });
 ```
 */
function parseFinding(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): ArtifactNaturalnessFinding {
  /**
   Finding under exact schema-eight shape.
   */
  const record = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record,
    allowed: [
      'paragraph',
      'problem',
    ],
    path,
  },);
  /**
   One-based paragraph position.
   */
  const paragraph = requireCount({
    value: record.paragraph,
    path: `${path}.paragraph`,
  },);
  if (paragraph < 1) {
    throw new ArtifactParseError({
      path: `${path}.paragraph`,
      reason: 'one-based paragraph number',
    },);
  }
  /**
   Concise actionable problem.
   */
  const problem = requireString({
    value: record.problem,
    path: `${path}.problem`,
  },);
  if (problem === '') {
    throw new ArtifactParseError({
      path: `${path}.problem`,
      reason: 'non-empty actionable defect',
    },);
  }
  return {
    paragraph,
    problem,
  };
}

/**
 Reads finding list in stored order.

 @param value - unknown list

 @param path - artifact path

 @returns Validated findings

 @example
 ```ts
 const findings = parseNaturalnessFindings({ value: [], path: 'review.findings', });
 ```
 */
export function parseNaturalnessFindings(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): readonly ArtifactNaturalnessFinding[] {
  return requireArray({
    value,
    path,
  },)
    .map(function readOne(
      entry,
      at,
    ): ArtifactNaturalnessFinding {
      return parseFinding({
        value: entry,
        path: `${path}[${String(at,)}]`,
      },);
    },);
}

/**
 Reads one accounted reviewer seat and cross-validates status fields.

 @param value - unknown seat

 @param path - artifact path

 @returns Validated seat

 @example
 ```ts
 const seat = parseNaturalnessReviewSeat({ value, path, });
 ```
 */
export function parseNaturalnessReviewSeat(
  {
    value,
    path,
  }: {
    readonly value: unknown;
    readonly path: string;
  },
): ArtifactNaturalnessReviewSeat {
  /**
   Seat under exact schema-eight shape.
   */
  const record = requireRecord({
    value,
    path,
  },);
  requireExactKeys({
    record,
    allowed: [
      'modelId',
      'status',
      'findings',
      'reason',
    ],
    path,
  },);
  /**
   Status the seat names, before it is known to be one a seat may carry.
   */
  const { status, } = record;
  if (!isSeatStatus(status,)) {
    throw new ArtifactParseError({
      path: `${path}.status`,
      reason: `one of ${
        Object.keys(SEAT_STATUSES,)
          .join(', ',)
      }`,
    },);
  }
  /**
   Findings under status consistency check.
   */
  const findings = parseNaturalnessFindings({
    value: record.findings,
    path: `${path}.findings`,
  },);
  /**
   Explanation under status consistency check.
   */
  const reason = requireString({
    value: record.reason,
    path: `${path}.reason`,
  },);
  if ((status === 'acceptable') && (findings.length > 0)) {
    throw new ArtifactParseError({
      path: `${path}.findings`,
      reason: 'empty findings for acceptable seat',
    },);
  }
  if ((status === 'unacceptable') && (findings.length === 0)) {
    throw new ArtifactParseError({
      path: `${path}.findings`,
      reason: 'at least one finding for unacceptable seat',
    },);
  }
  if ((status === 'unusable') && ((findings.length > 0) || (reason !== ''))) {
    throw new ArtifactParseError({
      path,
      reason: 'unusable seat with empty findings and reason',
    },);
  }
  return {
    modelId: requireString({
      value: record.modelId,
      path: `${path}.modelId`,
    },),
    status,
    findings,
    reason,
  };
}

/**
 Tests exact ordered equality between located finding lists.

 @param left - first list

 @param right - second list

 @returns Whether same findings occupy same positions

 @example
 ```ts
 sameNaturalnessFindings({ left, right, });
 ```
 */
export function sameNaturalnessFindings(
  {
    left,
    right,
  }: {
    readonly left: readonly ArtifactNaturalnessFinding[];
    readonly right: readonly ArtifactNaturalnessFinding[];
  },
): boolean {
  return (left.length === right.length) && left.every(function same(
    value,
    index,
  ): boolean {
    /**
     Finding at same stored position.
     */
    const candidate = right[index];
    return (candidate !== undefined)
      && (candidate.paragraph === value.paragraph)
      && (candidate.problem === value.problem);
  },);
}

/**
 Deduplicates exact located findings in first occurrence order.

 @param findings - roster-ordered findings

 @returns First copy of each exact finding

 @example
 ```ts
 const unique = uniqueNaturalnessFindings({ findings, });
 ```
 */
export function uniqueNaturalnessFindings(
  { findings, }: { readonly findings: readonly ArtifactNaturalnessFinding[]; },
): readonly ArtifactNaturalnessFinding[] {
  return findings.filter(function firstOccurrence(
    finding,
    index,
  ): boolean {
    return findings.findIndex(function same(candidate,): boolean {
      return (candidate.paragraph === finding.paragraph)
        && (candidate.problem === finding.problem);
    },) === index;
  },);
}

/**
 Derives fail-closed verdict from accounted seats.

 @param seats - every requested reviewer seat

 @param quorumOver - explicit wider basis, or legacy interpretation from recorded seats

 @param unreachable - bench seats the review counted out of reach (ledger
 E3), none in a record written before it counted them

 @param lastSeatGrace - whether the round was decided with the last-seat
 grace of 2026-10-22, as a round of generation twenty or later was
 (`artifactGenerationLetsGraceOpenOnLastSeat`)

 @returns Verdict implied by quorum and rejection

 @example
 ```ts
 const verdict = naturalnessVerdictOf({ seats, lastSeatGrace: true, });
 ```
 */
export function naturalnessVerdictOf(
  {
    seats,
    quorumOver = seats.length,
    unreachable = 0,
    lastSeatGrace,
  }: {
    readonly seats: readonly ArtifactNaturalnessReviewSeat[];
    readonly quorumOver?: number;
    readonly unreachable?: number;
    readonly lastSeatGrace: boolean;
  },
): ArtifactNaturalnessReviewRound['verdict'] {
  /**
   Seats carrying usable verdict.
   */
  const usable = seats.filter(function usableSeat(seat,): boolean {
    return seat.status !== 'unusable';
  },);
  /**
   The same quorum the stage closed on: half the bench, or the reachable
   share of it where seats were out of reach, which with none out of reach
   is exactly the exact-half quorum every earlier record was decided by.
   */
  const { needed, } = reachableQuorum({
    benchSize: quorumOver,
    unreachable,
    lastSeatGrace,
  },);
  if (usable.length < needed)
    return 'quorum-not-met';
  if (usable.some(function rejects(seat,): boolean {
    return seat.status === 'unacceptable';
  },))
    return 'unacceptable';
  return 'acceptable';
}

//endregion Artifact absolute naturalness seat read
