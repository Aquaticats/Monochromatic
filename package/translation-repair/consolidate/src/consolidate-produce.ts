import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { ProposalValidity, } from './consolidate-validity-floor.ts';
import {
  buildConsolidateMessages,
  type ConsolidateSubject,
} from './consolidate-wire.ts';
import {
  gatherStageVoices,
  type HeardVoice,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  foldTranslatorVoices,
  isTranslateReportWire,
  repairInvalidCandidates,
  TRANSLATE_RESPONSE_FORMAT,
  type TranslateReportWire,
  validateTranslatedSlice,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';

//region Consolidate produce
// THE PRODUCING HALF of the consolidation, kept apart from the deciding half
// for the reason the translate lane was split: a slate that exists can be
// judged more than once, and every measurement of this stage rests on comparing
// arms over ONE slate. A stage that rebought its proposals per arm would be
// comparing different candidates and reporting the difference as a decision.
//
// THE PAGE IS THE STRUCTURAL STANDARD, NOT THE ORIGINAL. `validateTranslatedSlice`
// is given `pageText`, the archive's own rendering, so a candidate has to carry
// the block sequence the page carries. Measured on the first calibration slice,
// where the Chinese is one paragraph and the archive renders it as a block quote
// with an attribution line: anchored to the source alone, all six consolidations
// were called invalid and the winner had flattened both into one paragraph.
//
// THE PAGE IS THE ARCHIVE, NOT THE LANE THAT WON. The winning lane is the
// INCUMBENT, which is a different job: a candidate that reproduces it is left
// alone rather than re-asked.

/**
 A slate as it leaves the producing half.

 @example
 ```ts
 const produced: ProducedConsolidations = { voices: [], validity: [], findings: [], };
 ```
 */
export type ProducedConsolidations = {
  /**
   Proposals that survived gathering and any repair round, in the order the
   roster answered.
   */
  readonly voices: readonly HeardVoice<TranslateReportWire>[];

  /**
   Each surviving proposal's structural verdict AFTER the repair round, which
   is what the validity floor reads.
   */
  readonly validity: readonly ProposalValidity[];

  /**
   Verdicts as they stood BEFORE the repair round, kept because a slate that
   needed repairing and one that did not are different facts about the roster
   and the deciding half cannot tell them apart afterwards.
   */
  readonly validityBefore: readonly ProposalValidity[];

  /**
   What gathering and repairing recorded, in scorecard-stable wording.
   */
  readonly findings: readonly string[];
};

/**
 Buys one slate of consolidations, sending invalid ones back to their authors.

 @param client - provider client this borrows

 @param roster - voices seated to produce

 @param subject - slice with both lane renderings and what the contest said

 @param standingText - wording in place, which a repair round shows an author
 as the incumbent it must not simply reproduce

 @param signal - cancellation for the whole producing half

 @param perCallTimeoutMs - bound on any single exchange

 @param l - stage logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Slate, each proposal's verdict before and after repair, and findings

 @example
 ```ts
 const produced = await produceConsolidations({ client, roster, subject, standingText, signal, perCallTimeoutMs, l, stageSettings, },);
 ```
 */
export async function produceConsolidations(
  {
    client,
    roster,
    subject,
    standingText,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly roster: readonly RosterModelId[];
    readonly subject: ConsolidateSubject;
    readonly standingText: string;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<ProducedConsolidations> {
  /**
   Logger tagged with this half.
   */
  const pl = tagged({
    tag: produceConsolidations.name,
    l,
  },);

  /**
   Sheet every producer is given, kept because the repair round shows an
   author what it was originally asked.
   */
  const messages = buildConsolidateMessages({ subject, },);

  /**
   Every proposal the roster answered with, retried to quorum so a lost voice
   here means what it means in production.
   */
  const gather = await gatherStageVoices({
    client,
    modelIds: roster,
    messages,
    signal,
    exchangeTimeoutMs: perCallTimeoutMs,
    responseFormat: TRANSLATE_RESPONSE_FORMAT,
    validate: isTranslateReportWire,
    stage: produceConsolidations.name,
    // A WRITER ROUND: every voice the window cuts is one fewer proposal, so it
    // waits under the writer window, built in since 2026-09-06
    // (`writer-grace-override.ts`).
    graceMs: stageSettings
      .grace
      .writerMs,
    l: pl,
  },);

  /**
   Proposals as they would ship, folded before anything reads them, so the
   verdicts, the repair round and the validity floor judge the bytes the
   judges see and the page gets (ledger B112).
   */
  const intake = foldTranslatorVoices({ voices: gather.voices, },);

  /**
   Checks one proposal against the page it would be written into.

   @param voice - proposal to check

   @returns Identity beside the guard's verdict

   @example
   ```ts
   const checked = checkVoice(voice,);
   ```
   */
  function checkVoice(voice: HeardVoice<TranslateReportWire>,): ProposalValidity {
    return {
      modelId: voice.modelId,
      validation: validateTranslatedSlice({
        sourceText: subject.sourceText,
        candidateText: voice.value
          .translation,
        pageText: subject.incumbentText,
        ...((subject.syntax === undefined) ? {} : { syntax: subject.syntax, }),
        lineStructured: subject.lineStructured,
        ...((subject.declared === undefined) ? {} : { declared: subject.declared, }),
        ...((subject.disputedWordings === undefined) ? {} : { disputedWordings: subject.disputedWordings, }),
      },),
    };
  }

  /**
   Verdicts as the roster first answered.
   */
  const validityBefore = intake.voices
    .map(checkVoice,);

  // SENT BACK TO ITS OWN AUTHOR rather than dropped, which is the treatment an
  // invalid translated slice already gets. A proposal refused for shape may be
  // the most faithful rendering on the slate.
  /**
   The slate after every refused author has had one more turn at it.
   */
  const repaired = await repairInvalidCandidates({
    client,
    voices: intake.voices,
    sourceText: subject.sourceText,
    incumbentText: standingText,
    pageText: subject.incumbentText,
    ...((subject.syntax === undefined) ? {} : { syntax: subject.syntax, }),
    lineStructured: subject.lineStructured,
    // THE SAME FLOOR THE FIRST CHECK READS: declared names were left out of
    // this re-check until the audit, and the disputed wordings join them.
    ...((subject.declared === undefined) ? {} : { declared: subject.declared, }),
    ...((subject.disputedWordings === undefined) ? {} : { disputedWordings: subject.disputedWordings, }),
    priorMessages: messages,
    signal,
    perCallTimeoutMs,
    l: pl,
  },);

  return {
    voices: repaired.voices,
    validity: repaired.voices
      .map(checkVoice,),
    validityBefore,
    findings: [
      ...gather.findings,
      ...intake.findings,
      ...repaired.findings,
    ],
  };
}

//endregion Consolidate produce
