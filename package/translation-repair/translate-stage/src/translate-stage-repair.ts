import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { SliceSyntax, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { DeclaredNamePair, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type DisputedWording,
  type IncumbentKind,
  requireTranslateAbsence,
  TranslateAbsenceError,
  type TranslateFollowupEvidence,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import { produceTranslateSlate, } from './translate-produce.ts';
import { judgeSlateWithRetry, } from './translate-retry.ts';
import type { TranslateStageResult, } from './translate-stage-result.ts';
import { TranslationRepairInterruptedError, } from './translation-repair-interrupted-error.ts';
import type {
  FanOutMode,
  StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';

//region Translate stage repair
// FIXED DEPTH TWO BY DESIGN: one initial round, then at most one statically
// named follow-up round carrying the judges' located rejection evidence, the
// one rejection-driven re-ask the no-loop design retains
// (doc/planning/translation-repair-no-loop-design.md). A second rejection
// rethrows the absence, which the slice attempt settles as an unfilled slice
// rather than a thrown entry.
//
// WORDING THAT EXISTS AND CANNOT SHIP IS NOT AN ABSENCE (owner answer
// 2026-09-27, "Preference + polish"). An archive rendering the floor refuses
// reaches this stage as an absent incumbent, and a second rejection there
// rethrew into a content slice, which the slice attempt rethrows in turn, so
// the entry stopped. The follow-up round now ships past a declined challenge
// round instead, the preferred of the candidates two or more judges named
// and, where none drew two ballots, the wording the archive has (owner
// 2026-10-25, "Two judges must name it"); the first round still defers to
// the follow-up, which is the designed re-ask. Where the follow-up round's
// translators are heard and propose nothing the floor accepts, the wording
// the archive has stands on that round (ledger B39), where the empty slate
// raised and stopped the entry the same way.

/**
 Finding recorded on an absence settled without judging the follow-up, because its slate repeats the one the same
 judges split on in the first round (the lead's adoption of 2026-10-29, open to the owner's veto).
 */
export const REPEATED_SLATE_FINDING: string = 'translate-followup-slate-repeated: the follow-up writers proposed the '
  + 'slate the judges had already declined, so the same judges were not asked it again';

/**
 Everything one produce-and-judge round needs, shared by both rounds.
 */
type TranslateRoundInput = ForeignBorrowed<{
  readonly client: ModelClient;
  readonly translatorModelIds: readonly RosterModelId[];
  readonly judgeModelIds: readonly RosterModelId[];
  readonly sourceText: string;
  readonly incumbentText: string;
  readonly incumbentKind: IncumbentKind;
  readonly incumbentEligible: boolean;
  readonly incumbentWithheld: boolean;
  readonly identityContext?: string;
  readonly lifeEnded?: boolean;
  readonly subjectNames?: readonly string[];
  readonly referenceContext?: string;
  readonly attestedLines?: readonly string[];
  readonly archiveDisputeNote?: string;
  readonly neighbouringIncumbentText?: string;
  readonly neighbouringSourceText?: string;
  readonly pictureContext?: string;
  readonly syntax?: SliceSyntax;
  readonly lineStructured: boolean;
  readonly declared: readonly DeclaredNamePair[];
  readonly disputedWordings: readonly DisputedWording[];
  readonly signal: AbortSignal;
  readonly perCallTimeoutMs: number;
  readonly l: Logger;
  readonly fanOut?: FanOutMode;
}>;

/**
 Whether two slates are the same renderings in the same order, which is the same question for the same judges.

 @param now - candidate texts this round proposed

 @param before - candidate texts the judges rejected in the round before

 @returns Whether every text matches, position by position

 @example
 ```ts
 const repeated = sameSlate({ now: ['A cat naps.',], before: ['A cat naps.',], },);
 ```
 */
function sameSlate(
  {
    now,
    before,
  }: {
    readonly now: readonly string[];
    readonly before: readonly string[];
  },
): boolean {
  return JSON.stringify(now,) === JSON.stringify(before,);
}

/**
 Produces one slate and judges it, optionally under rejection evidence.

 @param input - round configuration shared by both fixed rounds

 @param followupEvidence - located rejection evidence, absent on the initial round

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Settled text and evidence

 @throws {@link TranslateAbsenceError} when judging leaves an absent passage unwritten

 @example
 ```ts
 const result = await produceAndJudgeOnce({ input, stageSettings, },);
 ```
 */
async function produceAndJudgeOnce(
  {
    input,
    followupEvidence,
    stageSettings,
  }: {
    readonly input: TranslateRoundInput;
    readonly followupEvidence?: TranslateFollowupEvidence;
    readonly stageSettings: StageSettings;
  },
): Promise<{
  readonly result: TranslateStageResult;
  readonly candidateTexts: readonly string[];
} | {
  readonly rejection: TranslateAbsenceError;
  readonly candidateTexts: readonly string[];
}> {
  /**
   Slate produced initially or from the located rejection evidence.
   */
  const produced = await produceTranslateSlate({
    client: input.client,
    translatorModelIds: input.translatorModelIds,
    sourceText: input.sourceText,
    incumbentText: input.incumbentText,
    incumbentKind: input.incumbentKind,
    incumbentEligible: input.incumbentEligible,
    ...((input.identityContext === undefined) ? {} : { identityContext: input.identityContext, }),
    ...((input.archiveDisputeNote === undefined) ? {} : { archiveDisputeNote: input.archiveDisputeNote, }),
    ...((input.pictureContext === undefined) ? {} : { pictureContext: input.pictureContext, }),
    ...((input.attestedLines === undefined) ? {} : { attestedLines: input.attestedLines, }),
    ...((input.syntax === undefined) ? {} : { syntax: input.syntax, }),
    ...((followupEvidence === undefined) ? {} : { followupEvidence, }),
    lineStructured: input.lineStructured,
    declared: input.declared,
    disputedWordings: input.disputedWordings,
    signal: input.signal,
    perCallTimeoutMs: input.perCallTimeoutMs,
    l: input.l,
    // Conditional spread keeps the knob absent instead of undefined.
    ...((input.fanOut === undefined) ? {} : { fanOut: input.fanOut, }),
    stageSettings,
  },);
  /**
   Rejected candidate texts in deterministic slate order, for evidence.
   */
  const candidateTexts = produced
    .candidates
    .map(function candidateText(candidate,): string {
      return candidate
        .value
        .text;
    },);
  // A FOLLOW-UP SLATE THE FIRST ROUND ALREADY PUT TO THESE JUDGES IS NOT PUT TO THEM AGAIN (the lead's adoption of
  // 2026-10-29, open to the owner's veto). XingZ6022 slices 107 and 114 asked two judges, who split one ballot each, the
  // same two renderings four times: the first ask, its challenge, and the same two in the follow-up, then shipped the
  // source text under a note. The writers are still asked, since a fresh slate is the follow-up's one new question, and
  // a slate that differs is judged as before. Where the archive has wording that cannot ship the follow-up ships past a
  // decline, which is a question of its own, so it asks.
  if ((followupEvidence !== undefined)
    && (!input.incumbentWithheld)
    && sameSlate({
      now: candidateTexts,
      before: followupEvidence.candidateTexts,
    },)) {
    input.l
      .info('translate stage: the follow-up slate repeats the first; the same judges are not asked it again',);
    return {
      rejection: new TranslateAbsenceError({
        reason: followupEvidence.reason,
        findings: [REPEATED_SLATE_FINDING,],
      },),
      candidateTexts,
    };
  }
  try {
    return {
      result: await judgeSlateWithRetry({
        judging: {
          client: input.client,
          produced,
          judgeModelIds: input.judgeModelIds,
          sourceText: input.sourceText,
          incumbentText: input.incumbentText,
          incumbentKind: input.incumbentKind,
          ...((input.identityContext === undefined) ? {} : { identityContext: input.identityContext, }),
          ...((input.lifeEnded === undefined) ? {} : { lifeEnded: input.lifeEnded, }),
          ...((input.subjectNames === undefined) ? {} : { subjectNames: input.subjectNames, }),
          ...((input.referenceContext === undefined) ? {} : { referenceContext: input.referenceContext, }),
          ...((input.archiveDisputeNote === undefined) ? {} : { archiveDisputeNote: input.archiveDisputeNote, }),
          ...((input.neighbouringSourceText === undefined)
            ? {}
            : { neighbouringSourceText: input.neighbouringSourceText, }),
          ...((input.neighbouringIncumbentText === undefined)
            ? {}
            : { neighbouringIncumbentText: input.neighbouringIncumbentText, }),
          ...((input.pictureContext === undefined) ? {} : { pictureContext: input.pictureContext, }),
          ...((input.syntax === undefined) ? {} : { syntax: input.syntax, }),
          lineStructured: input.lineStructured,
          withheldStanding: input.incumbentWithheld,
          // Only the follow-up round ships past a decline; the first defers
          // to it.
          shipPastDecline: input.incumbentWithheld && (followupEvidence !== undefined),
          signal: input.signal,
          perCallTimeoutMs: input.perCallTimeoutMs,
          l: input.l,
          ...((input.fanOut === undefined) ? {} : { fanOut: input.fanOut, }),
          stageSettings,
        },
      },),
      candidateTexts,
    };
  }
  catch (error) {
    if (input.signal
      .aborted)
      throw input.signal
        .reason;
    /**
     The absence judging raised; anything else leaves as it came.
     */
    const rejection = requireTranslateAbsence({ error, },);
    if (rejection.reason === 'no-voice-heard') {
      throw new TranslationRepairInterruptedError({
        reason: 'provider-unavailable',
        findings: rejection.findings,
      },);
    }
    return {
      rejection,
      candidateTexts,
    };
  }
}

/**
 Produces and judges an absent passage at fixed depth two.

 The follow-up round carries the judges' located rejection evidence,
 the form the redesign measured as the one safe re-ask shape;
 a second rejection rethrows so the slice settles unfilled,
 never as a thrown entry.

 @param client - injected model client

 @param translatorModelIds - models rendering each task independently

 @param judgeModelIds - models judging each produced slate

 @param sourceText - original passage to render

 @param incumbentText - existing translation, blank for absent passage

 @param incumbentKind - whether fallback text exists and passes deterministic source floor

 @param incumbentEligible - whether existing text may appear on candidate slate

 @param incumbentWithheld - whether the absent incumbent is wording that
 exists and cannot ship, so the follow-up round ships a candidate two or more
 judges named, or keeps that wording, past a decline rather than rethrowing
 (owner, 2026-09-27 and 2026-10-25)

 @param disputedWordings - wordings a disputed slice refuses as candidates
 (owner, 2026-09-27, "No eligible standing")

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - pipeline logger

 @param fanOut - seats a round asks: the window of quorum plus one by
 default, or the whole bench a fixture scripting every seat asks for, since
 the window is rotated by the sheet text and a sheet edit elsewhere moves
 which seats it holds

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Settled text and evidence, the first round's findings and the
 follow-up's named before the follow-up's own where there was one (ledger B41)

 @throws {@link TranslateAbsenceError} when both fixed rounds leave the passage
 unwritten, carrying both rounds' findings

 @throws {@link TranslationRepairInterruptedError} when no judging voice was heard

 @example
 ```ts
 const result = await runTranslateRepairs({ ...inputs, });
 ```
 */
export async function runTranslateRepairs(
  {
    client,
    translatorModelIds,
    judgeModelIds,
    sourceText,
    incumbentText,
    incumbentKind,
    incumbentEligible,
    incumbentWithheld,
    identityContext,
    lifeEnded,
    subjectNames,
    referenceContext,
    attestedLines,
    archiveDisputeNote,
    neighbouringIncumbentText,
    neighbouringSourceText,
    pictureContext,
    syntax,
    lineStructured,
    declared,
    disputedWordings,
    signal,
    perCallTimeoutMs,
    l,
    fanOut,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly translatorModelIds: readonly RosterModelId[];
    readonly judgeModelIds: readonly RosterModelId[];
    readonly sourceText: string;
    readonly incumbentText: string;
    readonly incumbentKind: IncumbentKind;
    readonly incumbentEligible: boolean;
    readonly incumbentWithheld: boolean;
    readonly identityContext?: string;
    readonly lifeEnded?: boolean;
    readonly subjectNames?: readonly string[];
    readonly referenceContext?: string;
    readonly attestedLines?: readonly string[];
    readonly archiveDisputeNote?: string;
    readonly neighbouringIncumbentText?: string;
    readonly neighbouringSourceText?: string;
    readonly pictureContext?: string;
    readonly syntax?: SliceSyntax;
    readonly lineStructured: boolean;
    readonly declared: readonly DeclaredNamePair[];
    readonly disputedWordings: readonly DisputedWording[];
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly fanOut?: FanOutMode;
    readonly stageSettings: StageSettings;
  }>,
): Promise<TranslateStageResult> {
  /**
   Round configuration shared by the initial and follow-up rounds.
   */
  const input: TranslateRoundInput = {
    client,
    translatorModelIds,
    judgeModelIds,
    sourceText,
    incumbentText,
    incumbentKind,
    incumbentEligible,
    incumbentWithheld,
    ...((identityContext === undefined) ? {} : { identityContext, }),
    ...((lifeEnded === undefined) ? {} : { lifeEnded, }),
    ...((subjectNames === undefined) ? {} : { subjectNames, }),
    ...((referenceContext === undefined) ? {} : { referenceContext, }),
    ...((attestedLines === undefined) ? {} : { attestedLines, }),
    ...((archiveDisputeNote === undefined) ? {} : { archiveDisputeNote, }),
    ...((neighbouringIncumbentText === undefined) ? {} : { neighbouringIncumbentText, }),
    ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
    ...((pictureContext === undefined) ? {} : { pictureContext, }),
    ...((syntax === undefined) ? {} : { syntax, }),
    lineStructured,
    declared,
    disputedWordings,
    signal,
    perCallTimeoutMs,
    l,
    // Conditional spread keeps the knob absent instead of undefined.
    ...((fanOut === undefined) ? {} : { fanOut, }),
  };
  /**
   Initial round with no rejection evidence.
   */
  const first = await produceAndJudgeOnce({
    input,
    stageSettings,
  },);
  if ('result' in first)
    return first.result;
  /**
   Why the judges rejected the first round.
   */
  const firstReason = first.rejection
    .reason;
  /**
   How many candidates the first round's judges rejected.
   */
  const rejectedCount = first.candidateTexts
    .length;
  /**
   What names the follow-up round and why it was asked, for the log and the
   record alike.
   */
  const followupNamed = `after ${firstReason}, ${String(rejectedCount,)} rejected ${
    wordForCount({
      count: rejectedCount,
      one: 'candidate',
      many: 'candidates',
    },)
  }`;
  l.info(`translate stage: one follow-up round ${followupNamed}`,);
  /**
   Single follow-up round carrying the located rejection evidence.
   */
  const second = await produceAndJudgeOnce({
    input,
    followupEvidence: {
      reason: firstReason,
      candidateTexts: first.candidateTexts,
      findings: first.rejection
        .findings,
    },
    stageSettings,
  },);
  // THE FIRST ROUND'S EVIDENCE IS KEPT, then the follow-up is named, then its
  // own record follows (ledger B41): the record showed the follow-up round
  // alone, so a slice filled or left unfilled at depth two could not say its
  // first slate was declined, nor why, where the judges' retry keeps every
  // ask it made.
  /**
   The first round's findings with the follow-up named after them.
   */
  const firstRound = [
    ...first.rejection
      .findings,
    `translate-followup-round (${followupNamed})`,
  ];
  if ('result' in second) {
    return {
      ...second.result,
      findings: [
        ...firstRound,
        ...second.result
          .findings,
      ],
    };
  }
  // Depth two is spent: the absence is settled evidence now, and the slice
  // attempt records the passage as unfilled instead of throwing the entry.
  // It carries no finalists: the judges' retry settles every tie before an
  // absence leaves it (`translate-retry.ts`).
  throw new TranslateAbsenceError({
    reason: second.rejection
      .reason,
    findings: [
      ...firstRound,
      ...second.rejection
        .findings,
    ],
  },);
}

//endregion Translate stage repair
