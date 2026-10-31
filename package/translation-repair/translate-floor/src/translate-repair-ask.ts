import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';

import type {
  ModelClient,
  ReaskSeat,
} from '@monochromatic-dev/translation-repair-provider/ts';
import { exchangeFailureLogText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  attemptStageCall,
  type HeardVoice,
  markFailureWritten,
  type StageVoice,
} from '@monochromatic-dev/translation-repair-stage/ts';
import { producedVolumeBound, } from './produced-volume-bound.ts';
import {
  buildAnotherTranslatorRepairMessages,
  buildTranslateRepairMessages,
  isTranslateRepairWire,
  TRANSLATE_REPAIR_RESPONSE_FORMAT,
  type TranslateRepairWire,
} from './translate-repair-wire.ts';
import type { TranslateReportWire, } from './translate-wire.ts';

//region Translate repair ask
// THE FOLLOW-UP TURN ITSELF, apart from what is made of its answer (`translate-repair.ts`).

/**
 Stage label of the follow-up turn, which the exchange's own lines and the
 line a re-ask's failure under the caller's stop leaves both open with.
 */
export const REPAIR_STAGE = 'translate-repair';

/**
 Asks a candidate's author about the findings on it, and hands a model other than the author, where the author's
 reasoning runaway is re-asked of one, a conversation that names the text as another translator's (ledger B1100).

 @param client - injected model client

 @param voice - candidate whose author is asked

 @param findings - what validation found in it

 @param priorMessages - exact messages that produced the candidate

 @param sourceText - original the candidate renders, whose size bounds the answer

 @param reaskSeat - role, bench and shared record of the authors of the slate

 @param signal - caller abort honored by the exchange

 @param perCallTimeoutMs - deadline for it

 @param l - logger of the candidate's turn

 @returns Answer as data; `answeredBy` names the model that gave it where the re-ask went to another model

 @throws Whatever the exchange raised, once the caller aborted, having written it on this seat's line and marked it
 written (`markFailureWritten`) unless it is the caller's reason itself

 @example
 ```ts
 const answer = await askAboutFindings({ client, voice, findings, priorMessages, sourceText, reaskSeat, signal, perCallTimeoutMs, l, },);
 ```
 */
export async function askAboutFindings(
  {
    client,
    voice,
    findings,
    priorMessages,
    sourceText,
    reaskSeat,
    signal,
    perCallTimeoutMs,
    l,
  }: {
    readonly client: ModelClient;
    readonly voice: HeardVoice<TranslateReportWire>;
    readonly findings: readonly string[];
    readonly priorMessages: readonly ChatMessage[];
    readonly sourceText: string;
    readonly reaskSeat: ReaskSeat;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
  },
): Promise<StageVoice<TranslateRepairWire>> {
  /**
   Characters of finding text this answer has to address.

   COUNTED BECAUSE THE REPAIR WIRE EXPLAINS ITSELF. Its `translatorNote` field
   answers these findings, and the producing wire has no such field, so the
   slice alone does not bound what a correct answer here can run to.
   */
  const findingsChars = findings
    .join('',)
    .length;
  try {
    return await attemptStageCall({
      client,
      modelId: voice.modelId,
      messages: buildTranslateRepairMessages({
        priorMessages,
        priorTranslation: voice.value
          .translation,
        findings,
      },),
      signal,
      exchangeTimeoutMs: perCallTimeoutMs,
      // THE SAME BOUND AS PRODUCING, over more material. This call re-renders
      // the same slice, so the slice still bounds it; it also argues with the
      // findings, which producing never had to do.
      maxAnswerChars: producedVolumeBound({
        materialChars: sourceText.length + findingsChars,
      },),
      responseFormat: TRANSLATE_REPAIR_RESPONSE_FORMAT,
      validate: isTranslateRepairWire,
      stage: REPAIR_STAGE,
      reaskSeat: {
        ...reaskSeat,
        // ANOTHER MODEL IS TOLD WHOSE TEXT IT REVISES, not handed the author's turn as its own (ledger B1100).
        otherModelMessages: buildAnotherTranslatorRepairMessages({
          priorMessages,
          priorTranslation: voice.value
            .translation,
          findings,
        },),
      },
      l,
    },);
  }
  catch (error) {
    // A FAILURE OF THE RE-ASK'S OWN IS WRITTEN HERE, inside the member, as
    // it ends. `attemptStageCall` rethrows unwritten once the caller has
    // aborted, and `repairInvalidCandidates` throws only the first failure
    // in input order, so a deadline or a provider's refusal another seat's
    // re-ask ended in as the stop arrived would reach no line (ledger B365
    // left this open). Rendered as the round's seats are (`askOnce` in
    // `stage-round.ts`), so a refusal keeps its status and the provider's
    // words; the caller's own reason is no failure of this seat and is not
    // written. Lines land in the order the re-asks end.
    if (signal.aborted && (error !== signal.reason)) {
      l.warn(
        `${REPAIR_STAGE} ${voice.modelId}: ${exchangeFailureLogText({ error, },)}, as the caller stopped the repair`,
      );
      // MARKED AS WRITTEN, so the layers this failure passes through on its
      // way out, the twin memo around a consolidation buy, the translate
      // attempt's abort line and the overlapped map, leave its words to
      // this line (ledger B412).
      markFailureWritten({ error, },);
    }
    throw error;
  }
}

//endregion Translate repair ask
