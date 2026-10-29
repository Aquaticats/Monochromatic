import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';
import { describeProducer, } from '@monochromatic-dev/translation-repair-house-rule/ts';

import { archiveReplyVerdict, } from './archive-reply-verdict.ts';
import { recordArchiveBlockNaturalness, } from './archive-block-naturalness.ts';
import { compositeOfDisjointRevisions, } from './archive-composite-revision.ts';
import { voicesAsRead, } from './archive-voices-as-read.ts';
import { replacementCandidates, } from './archive-replacement-candidates.ts';
import {
  archiveBlockSelectionEvidence,
  withArchiveOriginal,
} from './archive-block-selection-evidence.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type ArchiveBlockReviewWire,
  ARCHIVE_BLOCK_DECLINE_CONSEQUENCE,
  ARCHIVE_BLOCK_REVIEW_RESPONSE_FORMAT,
  ARCHIVE_BLOCK_SELECTION_CRITERIA,
  ARCHIVE_BLOCK_SELECTION_TASK,
  buildArchiveBlockReviewMessages,
  isArchiveBlockReviewWire,
} from './archive-block-review-wire.ts';
import {
  decideBestCandidate,
  TranslationRepairInterruptedError,
} from '@monochromatic-dev/translation-repair-translate-stage/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import {
  type FanOutMode,
  gatherStageVoices,
  reachableQuorum,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Archive block review stage

/**
 Evidence marker assigning distinct prior-decline selection responsibility.
 */
const PRIOR_CORRECTION_DECLINE = 'archive correction slate declined';

/**
 Settled review of one unclaimed archive block.
 */
export type ArchiveBlockReviewOutcome = {
  /**
   Whether exact archived text stands or a different replacement was selected.
   */
  readonly kind: 'retained' | 'revised';
  /**
   Original or selected replacement text.
   */
  readonly text: string;
  /**
   Review and selection evidence.
   */
  readonly findings: readonly string[];
};


/**
 Reviews one archive-only block once and selects a correction when any voice rejects it.

 SINGLE ROUND BY DESIGN: a declined correction slate or an empty one retains
 the original block with the decline recorded as a finding, because archive
 wording is the shipping default and reviewer indecision must not withhold
 the entry (doc/planning/translation-repair-no-loop-design.md).

 @param client - provider client

 @param modelIds - reviewers and correction judges

 @param sourceText - whole source searched for support

 @param targetText - page the block stands in, as the revisions already
 applied leave it: the reviewers' context, the quote style a revision is
 restored to, and the page a revision's footnote check reads (ledger B80)

 @param blockText - exact block under review

 @param blockOffset - where the block starts in `targetText`, so a revision
 of a block whose text stands twice is checked where the block stands

 @param priorFindings - latest failed-strategy evidence

 @param identityContext - declared names preparation holds, for the
 reviewers, the correction selectors and the naturalness read (ledger B28)

 @param referenceContext - what the pages the original links say, with the
 attested lines under them; a retention may anchor in one page's text

 @param signal - caller cancellation

 @param exchangeTimeoutMs - per-call bound

 @param l - stage logger

 @param fanOut - seats the review round asks: the window of quorum plus one
 by default, or the whole bench a fixture scripting every seat asks for,
 since the window is rotated by the sheet text and a sheet edit elsewhere
 moves which seats it holds

 @param benchStart - seat the review round's rotation starts at in place of
 the one the sheet text seeds, which a fixture pins so that the order the
 voices are read in, and with it the numbering of the candidates the revisions
 make, moves with no sheet text (`rotatedBench`); absent in production

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Retained original or independently selected replacement

 @throws TranslationRepairInterruptedError when fewer seats answered at all
 than the quorum on the reachable bench needs

 @throws BlockOutsideArchiveError when the page does not carry the block,
 which a revision's footnote check reads it in

 @example
 ```ts
 const result = await runArchiveBlockReviewStage({ ...input, priorFindings: [], });
 ```
 */
export async function runArchiveBlockReviewStage(
  {
    client,
    modelIds,
    sourceText,
    targetText,
    blockText,
    blockOffset,
    priorFindings,
    identityContext,
    referenceContext,
    signal,
    exchangeTimeoutMs,
    l,
    fanOut,
    benchStart,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly modelIds: readonly RosterModelId[];
    readonly sourceText: string;
    readonly targetText: string;
    readonly blockText: string;
    readonly blockOffset: number;
    readonly priorFindings: readonly string[];
    readonly identityContext?: string;
    readonly referenceContext?: string;
    readonly signal: AbortSignal;
    readonly exchangeTimeoutMs: number;
    readonly l: Logger;
    readonly fanOut?: FanOutMode;
    readonly benchStart?: number;
    readonly stageSettings: StageSettings;
  }>,
): Promise<ArchiveBlockReviewOutcome> {
  /**
   Stage boundary for review and independent-selection diagnostics.
   */
  const reviewLog = tagged({
    l,
    tag: runArchiveBlockReviewStage.name,
  },);
  /**
   Declared names and cited pages, passed on only where the page has them,
   to every sheet this stage asks.
   */
  const pageContext = {
    ...((identityContext === undefined) ? {} : { identityContext, }),
    ...((referenceContext === undefined) ? {} : { referenceContext, }),
  };
  /**
   Quorum-bounded review voices.
   */
  const gather = await gatherStageVoices<ArchiveBlockReviewWire>({
    client,
    modelIds,
    messages: buildArchiveBlockReviewMessages({
      sourceText,
      targetText,
      blockText,
      priorFindings,
      ...pageContext,
    },),
    signal,
    exchangeTimeoutMs,
    responseFormat: ARCHIVE_BLOCK_REVIEW_RESPONSE_FORMAT,
    validate: isArchiveBlockReviewWire,
    stage: 'archive-block-review',
    l: reviewLog,
    // Conditional spread keeps the knob absent instead of undefined.
    ...((fanOut === undefined) ? {} : { fanOut, }),
    ...((benchStart === undefined) ? {} : { benchStart, }),
    // THE WRITERS' WINDOW (ledger B713): every `revise` reply is a whole
    // corrected block, so a voice the window cuts is a candidate lost, the
    // owner's reason for the longer window. The selection round after it
    // reads text that exists and keeps the readers' window.
    graceMs: stageSettings
      .grace
      .writerMs,
  },);
  /**
   Replies as the review reads them: a replacement that shows a reader nothing
   is read once, here, as the removal the wire spells as an empty replacement
   (`voicesAsRead`).
   */
  const voices = voicesAsRead({ voices: gather.voices, },);
  /**
   Replies whose claimed source support exists verbatim, each reply dropped
   logged with the test that refused it and the numbers that decided it (ledger
   B715): the run zheermao20 dropped two of four replies and said only that
   two assessments were eligible.
   */
  const anchoredVoices = voices
    .filter(function supportIsAnchored(voice,): boolean {
    /**
     Whether this reply is evidence, and why not when it is not.
     */
      const verdict = archiveReplyVerdict({
        disposition: voice.value
          .disposition,
        sourceQuote: voice.value
          .sourceQuote,
        sourceContext: sourceText,
        referenceContext: referenceContext ?? '',
        blockText,
      },);
      if (verdict.kind === 'stands')
        return true;
      reviewLog.warn(`archive review dropped the ${voice.value
        .disposition} reply of ${voice.modelId}: ${verdict.reason}`,);
      return false;
    },);
  /**
   Revisions earn publication through the independent selector, not through
   other reviewers' retention anchors. The initial review quorum still holds.
   */
  const {
    candidates: revisions,
    withheld,
  } = replacementCandidates({
    voices: anchoredVoices,
    blockText,
    blockOffset,
    targetText,
  },);
  /**
   The lines of the revisions that changed disjoint lines, applied to the
   block together and offered as one more candidate (ledger B712), and the
   findings the gates refused it under.
   */
  const {
    candidate: composite,
    withheld: compositeWithheld,
    contributions,
  } = compositeOfDisjointRevisions({
    revisions,
    blockText,
    blockOffset,
    targetText,
    l: reviewLog,
  },);
  for (const finding of [
    ...withheld,
    ...compositeWithheld,
  ])
    reviewLog.warn(finding,);
  /**
   Evidence carried into selection and later strategies.
   */
  const findings = [...new Set([
    ...priorFindings,
    ...gather.findings,
    ...withheld,
    ...compositeWithheld,
    ...(composite === undefined ? [] : [`archive-composite-offered (${describeProducer(composite.producer,)})`,]),
    ...voices
      .map(function recordFinding(voice,): string {
        return voice.value
          .assessment;
      },),
    ...(anchoredVoices.length
      === voices
        .length ? [] : ['archive review discarded uncorroborated retention claim',]),
  ],),];
  // AN UNHEARD ROSTER IS AN OUTAGE; AN UNANCHORED ONE IS A VERDICT. Only the
  // first interrupts. The second used to be thrown as `provider-unavailable`
  // too, and on 2026-09-02 it ended XIEPT2 in 114 seconds with every seat
  // answering every call: the archive is a placeholder page ("(To-Do)" and
  // translation hints), nine reviewers were heard about its one block, and
  // fewer than the exact half anchored their support in the original. The
  // no-loop design says an unresolved block is retained with its findings and
  // reviewer indecision cannot withhold the entry.
  //
  // AN ANSWER NOBODY COULD READ IS NOT SILENCE EITHER (class thirty-one). On
  // 2026-09-16 Mio13 reviewed the first chat translation with every provider
  // wet: seven of twelve seats spent their whole completion cap reasoning
  // and sent no content, one was cut in the grace window, four were heard,
  // and the stage read "4/12" as an outage. The seats the cap cut were
  // reached and answered; only the shape was lost. So the outage test counts
  // them beside the heard voices, and a bench that answered falls through
  // to the same retention an unanchored one gets.
  //
  // ONE QUORUM SIZES BOTH TESTS, on the bench the router could serve (ledger
  // B27). The participation threshold was once the exact half of the whole
  // bench while the outage test beside it already counted refused seats out
  // of reach, so a short bench whose every reachable seat anchored its quote
  // was left unresolved and never bought its naturalness read.
  /**
   Voices the gather needed, and the anchored voices the review needs, on
   the bench the router could serve.
   */
  const { needed, } = reachableQuorum({
    benchSize: modelIds.length,
    unreachable: gather.unreachable
      .size,
  },);
  if (!gather.quorumMet) {
    /**
     Voices read.
     */
    const heard = gather.voices
      .length;
    /**
     Seats that answered in a shape nothing could read.
     */
    const unread = gather.unreadable
      .size;
    if ((heard + unread) < needed) {
      throw new TranslationRepairInterruptedError({
        reason: 'provider-unavailable',
        findings,
      },);
    }
    reviewLog.warn(
      `archive-block-review: ${String(heard,)} heard and ${String(unread,)} answered unreadably of ${
        String(modelIds.length,)
      } ${
        wordForCount({
          count: modelIds.length,
          one: 'seat',
          many: 'seats',
        },)
      }; the bench answered, so the block is reviewed on what was read`,
    );
  }
  /**
   Replies heard at all, anchored or not.
   */
  const heardCount = gather.voices
    .length;
  // Participation after unsupported anchors are removed is held to the same
  // reachable quorum, since a refused seat can anchor nothing.
  if ((anchoredVoices.length < needed) && (revisions.length === 0)) {
    return {
      kind: 'retained',
      text: blockText,
      findings: [
        ...findings,
        `archive review left the block unresolved: ${String(anchoredVoices.length,)} of ${
          String(heardCount,)
        } replies supplied eligible review evidence, below the ${
          String(needed,)
        } required; the block stands as the archive wrote it`,
      ],
    };
  }
  if (anchoredVoices.every(function retains(voice,): boolean {
    return voice.value
      .disposition
      !== 'revise';
  },)) {
    /**
     Independent wording review after provenance acceptance.
     */
    const naturalnessFindings = await recordArchiveBlockNaturalness({
      client,
      modelIds,
      sourceText,
      blockText,
      ...((identityContext === undefined) ? {} : { identityContext, }),
      signal,
      exchangeTimeoutMs,
      l: reviewLog,
      stageSettings,
    },);
    return {
      kind: 'retained',
      text: blockText,
      findings: [
        ...findings,
        ...naturalnessFindings,
      ],
    };
  }
  /**
   Existing wording is an explicit choice, never an automatically approved one.
   */
  const candidates = withArchiveOriginal({
    revisions: composite === undefined ? revisions : [
      ...revisions,
      composite,
    ],
    blockText,
  },);
  reviewLog.info(
    `archive review: comparing ${String(revisions.length,)} admissible ${
      wordForCount({
        count: revisions.length,
        one: 'revision',
        many: 'revisions',
      },)
    } in ${String(candidates.length,)} ${
      wordForCount({
        count: candidates.length,
        one: 'candidate',
        many: 'candidates',
      },)
    }; ${String(anchoredVoices.length,)} eligible ${
      wordForCount({
        count: anchoredVoices.length,
        one: 'assessment',
        many: 'assessments',
      },)
    } from ${String(heardCount,)} heard ${
      wordForCount({
        count: heardCount,
        one: 'review',
        many: 'reviews',
      },)
    }`,
  );
  /**
   Independently judged correction slate under the unchanged selector quorum.
   */
  const selection = await decideBestCandidate({
    client,
    candidates,
    judgeModelIds: modelIds,
    sourceText,
    task: ARCHIVE_BLOCK_SELECTION_TASK,
    criteria: ARCHIVE_BLOCK_SELECTION_CRITERIA,
    evidence: archiveBlockSelectionEvidence({
      sourceText,
      targetText,
      blockText,
      voices: anchoredVoices,
      candidates,
      ...((composite === undefined)
        ? {}
        : {
          composite: {
            candidate: composite,
            contributions,
          },
        }),
      priorFindings,
      ...pageContext,
    },),
    declineConsequence: ARCHIVE_BLOCK_DECLINE_CONSEQUENCE,
    signal,
    perCallTimeoutMs: exchangeTimeoutMs,
    l: reviewLog,
    stageSettings,
  },);
  /**
   Review plus candidate-selection evidence.
   */
  const settledFindings = [...new Set([
    ...findings,
    ...selection.findings,
  ],),];
  if (selection.kind === 'selected')
    return {
      kind: selection.value === blockText ? 'retained' : 'revised',
      text: selection.value,
      findings: settledFindings,
    };
  // Archive wording is the shipping default; judge indecision is recorded
  // evidence, never authority to withhold the entry.
  return {
    kind: 'retained',
    text: blockText,
    findings: [...new Set([
      ...settledFindings,
      PRIOR_CORRECTION_DECLINE,
    ],),],
  };
}

//endregion Archive block review stage
