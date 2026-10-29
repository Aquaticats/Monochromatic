import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import {
  ABSOLUTE_NATURALNESS_REVIEW_RESPONSE_FORMAT,
  type AbsoluteNaturalnessFinding,
  type AbsoluteNaturalnessReviewPerspective,
  type AbsoluteNaturalnessReviewSubject,
  type AbsoluteNaturalnessReviewWire,
  buildAbsoluteNaturalnessReviewMessages,
  isAbsoluteNaturalnessReviewWire,
} from './absolute-naturalness-review-wire.ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  NaturalnessQuorumError,
  validNaturalnessQuorum,
} from '@monochromatic-dev/translation-repair-adjudication/ts';
import {
  type FanOutMode,
  runWindowedRounds,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Absolute naturalness review stage

/**
 One roster seat as absolute review accounted for it.

 @example
 ```ts
 const seat: AbsoluteNaturalnessReviewSeat = { modelId: 'hf:zai-org/GLM-5.3-Flash', status: 'acceptable', findings: [], reason: 'ready' };
 ```
 */
export type AbsoluteNaturalnessReviewSeat = {
  /**
   Roster model asked.
   */
  readonly modelId: RosterModelId;

  /**
   Usable verdict or named lack of usable reply.
   */
  readonly status: 'acceptable' | 'unacceptable' | 'unusable';

  /**
   Actionable defects from rejecting usable voice.
   */
  readonly findings: readonly AbsoluteNaturalnessFinding[];

  /**
   Usable voice explanation, empty for unusable seat.
   */
  readonly reason: string;
};

/**
 Absolute review verdict derived from every accounted seat.
 */
export type AbsoluteNaturalnessReviewVerdict =
  | 'acceptable'
    | 'unacceptable'
    | 'quorum-not-met';

/**
 Auditable absolute review of exact would-ship text.

 @example
 ```ts
 const review: AbsoluteNaturalnessReviewOutcome = { quorumOver: 0, unreachable: 0, candidateDigest: 'sha256:abc', candidateText: '', paragraphCount: 0, paragraphDigests: [], seats: [], usable: 0, verdict: 'quorum-not-met', findings: [] };
 ```
 */
export type AbsoluteNaturalnessReviewOutcome = {
  /**
   Effective wider bench size, retained independently of seats the window asked.
   */
  readonly quorumOver: number;

  /**
   Bench seats the quorum counted out of reach: seats the router refused for
   want of a wet provider, and bench seats this review could not ask
   (ledger E3). The quorum is the reachable share of the bench where these
   leave it short of half.
   */
  readonly unreachable: number;

  /**
   Digest binding review to exact candidate bytes.
   */
  readonly candidateDigest: string;

  /**
   Exact candidate text reviewer was shown.
   */
  readonly candidateText: string;

  /**
   Structurally correctable paragraphs reviewer was shown.
   */
  readonly paragraphCount: number;

  /**
   Digest of each structurally correctable paragraph in reviewer order.
   */
  readonly paragraphDigests: readonly string[];

  /**
   Every requested roster seat in request order.
   */
  readonly seats: readonly AbsoluteNaturalnessReviewSeat[];

  /**
   Number of seats returning usable structured verdict.
   */
  readonly usable: number;

  /**
   Fail-closed aggregate verdict.
   */
  readonly verdict: AbsoluteNaturalnessReviewVerdict;

  /**
   Rejection findings in deterministic roster order without exact duplicates.
   */
  readonly findings: readonly AbsoluteNaturalnessFinding[];
};

/**
 Removes exact duplicate located findings while preserving roster order.

 THE ARTIFACT READER KEEPS ITS OWN COPY ON PURPOSE
 (`uniqueNaturalnessFindings`, `artifact-two-lane-read-naturalness-seat.ts`):
 it is the stored artifact version's rule, and the reader refuses a file this
 rule's output disagrees with, so a change here shows up rather than
 rewriting what old artifacts mean.

 @param findings - model findings in roster order

 @returns First occurrence of each paragraph and problem pair

 @example
 ```ts
 uniqueFindings({ findings: [{ paragraph: 1, problem: 'stiff phrasing', }], });
 ```
 */
function uniqueFindings(
  { findings, }: { readonly findings: readonly AbsoluteNaturalnessFinding[]; },
): readonly AbsoluteNaturalnessFinding[] {
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
 Reviews exact would-ship body text against absolute publication naturalness.

 Exact-half quorum starts bounded grace for remaining seats.
 Every usable rejection that arrives before settlement remains decisive,
 while one unreliable provider cannot make whole-roster participation mandatory.

 @param client - provider client

 @param modelIds - every independent reviewer seat

 @param quorumOver - bench size the quorum is taken over, when the seats
 asked are a window of a wider bench; defaults to the seats asked

 @param subject - source context and exact candidate

 @param perspective - distinct defect-discovery or acceptance-challenge task

 @param signal - caller cancellation

 @param exchangeTimeoutMs - deadline accounting unavailable seat

 @param fanOut - seats a round asks: the window of quorum plus one by
 default, or the whole bench a fixture scripting every seat asks for

 @param l - parent logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Candidate-bound absolute verdict and every seat status

 @throws {@link NaturalnessQuorumError} when quorum basis cannot cover requested seats

 @example
 ```ts
 const review = await reviewAbsoluteNaturalness({ client, modelIds, subject, signal, exchangeTimeoutMs, l, stageSettings, });
 ```
 */
export async function reviewAbsoluteNaturalness(
  {
    client,
    modelIds,
    subject,
    perspective = 'defect-discovery',
    quorumOver = modelIds.length,
    signal,
    exchangeTimeoutMs,
    l,
    fanOut,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly modelIds: readonly RosterModelId[];
    readonly subject: AbsoluteNaturalnessReviewSubject;
    readonly perspective?: AbsoluteNaturalnessReviewPerspective;
    readonly quorumOver?: number;
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly fanOut?: FanOutMode;
    readonly stageSettings: StageSettings;
  }>,
): Promise<AbsoluteNaturalnessReviewOutcome> {
  /**
   Stage-specific logger.
   */
  const rl = tagged({
    l,
    tag: reviewAbsoluteNaturalness.name,
  },);
  if (!validNaturalnessQuorum({
    quorumOver,
    seatCount: modelIds.length,
  },)) {
    throw new NaturalnessQuorumError({
      quorumOver,
      seatCount: modelIds.length,
    },);
  }
  /**
   Structurally correctable paragraphs shown to every reviewer.
   */
  const paragraphCount = subject.paragraphs
    .length;
  /**
   Digest of each exact paragraph shown to reviewers.
   */
  const paragraphDigests = subject.paragraphs
    .map(function digestParagraph(paragraph,): string {
      return hashContent({ content: paragraph, },);
    },);
  /**
   Every requested outcome after every seat has settled or reached deadline,
   the quorum the rounds closed on, and the seats they counted out of reach.

   SIZED ON THE SEATS THAT COULD ANSWER (ledger E3; the short-bench rule of
   2026-09-09): usable voices must reach half the bench, or the reachable
   share of it where the router refused seats or, for a confirmation, where
   the seats asked are fewer than the bench. XingZ624 closed a confirmation
   3 of 5 usable against a quorum of 4 over 8 with two seats refused out of
   budget; the reachable share is 2.
   */
  const {
    outcomes,
    quorum,
    unreachable,
  } = await runWindowedRounds({
    client,
    modelIds,
    messages: buildAbsoluteNaturalnessReviewMessages({
      subject,
      perspective,
    },),
    signal,
    exchangeTimeoutMs,
    responseFormat: ABSOLUTE_NATURALNESS_REVIEW_RESPONSE_FORMAT,
    validate: function fitsSubject(value,): value is AbsoluteNaturalnessReviewWire {
      if (!isAbsoluteNaturalnessReviewWire(value,))
        return false;
      /**
       Located findings after wire validation.
       */
      const { findings, } = value;
      return findings.every(function existingParagraph(finding,): boolean {
        return finding.paragraph <= paragraphCount;
      },);
    },
    stage: 'absolute-naturalness-review',
    l: rl,
    quorumOver,
    graceMs: stageSettings
      .grace
      .roundMs,
    // Conditional spread keeps the knob absent instead of undefined.
    ...((fanOut === undefined) ? {} : { fanOut, }),
  },);
  /**
   Stable seat records, including calls with no usable ballot.
   */
  const seats = outcomes.map(function toSeat(
    outcome,
  ): AbsoluteNaturalnessReviewSeat {
    /**
     Usable or unavailable voice under requested model id.
     */
    const { voice, } = outcome;
    if (!voice.heard) {
      return {
        modelId: outcome.modelId,
        status: 'unusable',
        findings: [],
        reason: '',
      };
    }
    /**
     Validated structured reviewer reply.
     */
    const reply = voice.value;
    return {
      modelId: outcome.modelId,
      status: reply.isAcceptable ? 'acceptable' : 'unacceptable',
      findings: reply.findings,
      reason: reply.assessment,
    };
  },);
  /**
   Seats carrying usable structured reply.
   */
  const usableSeats = seats.filter(function isUsable(seat,): boolean {
    return seat.status !== 'unusable';
  },);
  /**
   Material rejection findings in roster order.
   */
  const findings = uniqueFindings({
    findings: usableSeats.flatMap(function rejected(
      seat,
    ): readonly AbsoluteNaturalnessFinding[] {
      return (seat.status === 'unacceptable') ? seat.findings : [];
    },),
  },);
  /**
   Fail-closed verdict: thin review cannot approve, and any rejection blocks.
   */
  const verdict: AbsoluteNaturalnessReviewVerdict = (usableSeats.length
    < quorum.needed)
    ? 'quorum-not-met'
    : usableSeats.some(function rejected(seat,): boolean {
      return seat.status === 'unacceptable';
    },)
      ? 'unacceptable'
      : 'acceptable';
  /**
   Per-seat status and finding identities without candidate or reviewer wording.
   */
  const seatSummary = seats
    .map(function summarizeSeat(seat,): string {
      /**
       Finding count rendered without finding wording.
       */
      const findingCount = String(seat.findings
        .length,);
      /**
       Located paragraphs making repeated findings comparable across rounds.
       */
      const findingParagraphParts = seat.findings
        .map(function paragraphOf(finding,): string {
          return String(finding.paragraph,);
        },);
      /**
       Located paragraph sequence before empty fallback.
       */
      const joinedFindingParagraphs = findingParagraphParts.join('+',);
      /**
       Located paragraph sequence or explicit absence.
       */
      const findingParagraphs = joinedFindingParagraphs === ''
        ? 'none'
        : joinedFindingParagraphs;
      /**
       Wording digests making recurrence comparable without exposing wording.
       */
      const findingDigestParts = seat.findings
        .map(function digestFinding(finding,): string {
          return hashContent({ content: finding.problem, },);
        },);
      /**
       Wording digest sequence before empty fallback.
       */
      const joinedFindingDigests = findingDigestParts.join('+',);
      /**
       Wording digest sequence or explicit absence.
       */
      const findingDigests = joinedFindingDigests === ''
        ? 'none'
        : joinedFindingDigests;
      return `${seat.modelId}:${seat.status}:findings=${findingCount}`
        + `:paragraphs=${findingParagraphs}:digests=${findingDigests}`;
    },)
    .join(';',);
  rl.info(
    `absolute naturalness review: ${String(usableSeats.length,)}/${String(modelIds.length,)} usable, ${verdict}, `
      + `quorumNeeded=${String(quorum.needed,)} quorumOver=${String(quorumOver,)} unreachable=${
        String(unreachable,)
      }, seats=${seatSummary}, uniqueFindings=${String(findings.length,)}`,
  );
  return {
    quorumOver,
    unreachable,
    candidateDigest: hashContent({ content: subject.candidateText, },),
    candidateText: subject.candidateText,
    paragraphCount,
    paragraphDigests,
    seats,
    usable: usableSeats.length,
    verdict,
    findings,
  };
}

//endregion Absolute naturalness review stage
