import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ChatMessage, } from '@monochromatic-dev/llm-llm-type/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import { allInInputOrder, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { SliceSyntax, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  type DisputedWording,
  disputedWordingFindings,
} from './disputed-wording.ts';
import {
  CONTRIBUTOR_AUTHORITY_FINDING,
  type DeclaredNamePair,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import { exchangeFailureLogText, } from '@monochromatic-dev/translation-repair-provider-stream/ts';
import {
  attemptStageCall,
  type HeardVoice,
  markFailureWritten,
  type StageVoice,
} from '@monochromatic-dev/translation-repair-stage/ts';
import { producedVolumeBound, } from './produced-volume-bound.ts';
import {
  buildTranslateRepairMessages,
  isTranslateRepairWire,
  TRANSLATE_REPAIR_RESPONSE_FORMAT,
  type TranslateRepairWire,
} from './translate-repair-wire.ts';
import { requireComparedVerdict, } from './translate-floor-ground.ts';
import { validateTranslatedSlice, } from './translate-validate.ts';
import type { TranslateReportWire, } from './translate-wire.ts';
import { foldTranslation, } from './translator-answer-fold.ts';
import { sameWording, } from './wording-key.ts';

//region Translate repair
// Structural validation, and the conversation a failing candidate gets instead
// of being dropped.
//
// User decision, 2026-08-15. The alternatives were dropping an invalid
// candidate, showing judges everything, and dropping only on reference damage;
// all three were rejected in favour of asking the model that wrote it. What
// comes back is one of three answers, and the interesting one is the third: a
// model can say the finding is a fact about the passage rather than about its
// work, which no filter could ever have collected.
//
// The INCUMBENT never passes through here. It is the fallback and the thing
// being defended, so a validator that could drop it would be a validator that
// could delete the archive.

/**
 Stage label of the follow-up turn, which the exchange's own lines and the
 line a re-ask's failure under the caller's stop leaves both open with.
 */
const REPAIR_STAGE = 'translate-repair';

/**
 One translator's final text after validation, with what happened to it.

 @example
 ```ts
 const outcome: RepairOutcome = { voice, findings: [], };
 ```
 */
export type RepairOutcome = {
  /**
   Voice to build candidate from,
   absent when non-defensible contributor violation was not repaired.
   */
  readonly voice?: HeardVoice<TranslateReportWire>;

  /**
   What validation and the follow-up turn recorded, in scorecard-stable
   wording.
   */
  readonly findings: readonly string[];
};

/**
 What becomes of a refused candidate whose author revised nothing usable, in
 the words the run log uses.

 @param contributorViolation - whether it respelled a contributor name, which
 no defence keeps

 @returns Clause finishing a log line about that candidate

 @example
 ```ts
 l.warn(`${modelId} gave no usable answer, so ${unrevisedFate({ contributorViolation, },)}`,);
 ```
 */
function unrevisedFate({ contributorViolation, }: { readonly contributorViolation: boolean; },): string {
  return contributorViolation
    ? 'its candidate is dropped, since a contributor respelling is not defensible'
    : 'its candidate goes on as written, for the floor to judge';
}

/**
 Validates one candidate and, when it fails, asks its author about it.

 @param client - injected model client

 @param voice - this translator's reply

 @param sourceText - original slice the candidate renders

 @param incumbentText - translation as it stands, which a matching candidate
 collapses into

 @param pageText - text this candidate would replace, whose block shape it has
 to carry. The same as `incumbentText` for a translator, and the ARCHIVE
 rather than the winning lane for a consolidator

 @param priorMessages - exact messages that produced the candidate

 @param signal - caller abort honored by the follow-up exchange

 @param perCallTimeoutMs - deadline for it

 @param lineStructured - whether the line-structure rule governs this slice,
 which makes merging its lines a fault the author is sent back to fix

 @param declared - name pairs the front matter declares, which the
 publication rule reads for a linked title naming a declared person (class
 one hundred fourteen)

 @param disputedWordings - wordings a disputed slice refuses, which a
 candidate copying one is sent back over rather than collapsed into (owner,
 2026-09-27, "No eligible standing")

 @param l - stage logger

 @returns Final voice for this model plus what was recorded

 @throws {@link import('./translate-floor-ground.ts').FloorGroundDisagreementError}
 when the floor says it compared nothing, which both callers rule out before
 any writer is asked (ledger B43, B45)

 @throws Whatever the follow-up exchange raised, once the caller aborted,
 having written it on this seat's line and marked it written
 (`markFailureWritten`) unless it is the caller's reason itself

 @example
 ```ts
 const outcome = await repairOneCandidate({ client, voice, sourceText, ... },);
 ```
 */
async function repairOneCandidate(
  {
    client,
    voice,
    sourceText,
    incumbentText,
    pageText,
    syntax,
    lineStructured,
    declared = [],
    disputedWordings = [],
    priorMessages,
    signal,
    perCallTimeoutMs,
    l,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly voice: HeardVoice<TranslateReportWire>;
    readonly sourceText: string;
    readonly incumbentText: string;
    readonly pageText: string;
    readonly syntax?: SliceSyntax;
    readonly lineStructured: boolean;
    readonly declared?: readonly DeclaredNamePair[];
    readonly disputedWordings?: readonly DisputedWording[];
    readonly priorMessages: readonly ChatMessage[];
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
  }>,
): Promise<RepairOutcome> {
  /**
   Logger tagged with this candidate's turn.
   */
  const rl = tagged({
    tag: repairOneCandidate.name,
    l,
  },);
  // A candidate that reproduced the incumbent is about to COLLAPSE into it, and
  // the incumbent is never validated, so validating this copy would spend a
  // follow-up call to repair text that is not going to be on the ballot. Worse,
  // a revision would break the match and destroy the `translate-matched-incumbent`
  // signal, on exactly the slices where the incumbent diverges from its source
  // and the match is most worth knowing about.
  //
  // NOT ON A DISPUTED SLICE, whose incumbent is refused rather than kept: a
  // copy of it is sent back like any other refused text (owner, 2026-09-27).
  /**
   What the rule says of this candidate as a disputed wording, none where it
   is not one.
   */
  const disputedCopy = disputedWordingFindings({
    candidateText: voice.value
      .translation,
    disputedWordings,
  },);
  if (
    sameWording({
      proposal: voice.value
        .translation,
      standing: incumbentText,
      lineStructured,
    },)
      && (disputedCopy.length === 0)
  )
    return {
      voice,
      findings: [],
    };

  /**
   Structural verdict over what this model returned: a pass or a refusal.

   NEVER THAT THE FLOOR COMPARED NOTHING. Both stages that ask for this turn
   settle a slice the floor can compare nothing on before any writer is
   asked (ledger B43 for the translate stage, B45 for the consolidation), so
   no candidate reaches here on such a slice; this turn let one stand
   unvalidated there, and an admitted insertion wrote it into the page.
   */
  const validation = requireComparedVerdict({
    verdict: validateTranslatedSlice({
      sourceText,
      candidateText: voice.value
        .translation,
      pageText,
      ...((syntax === undefined) ? {} : { syntax, }),
      lineStructured,
      declared,
      disputedWordings,
    },),
  },);
  // A PASS ON A DOWNGRADED PAGE IS RECORDED, never silent. The strict grammar
  // refuses a span cut through an element, and the page side falls back to
  // plain markdown so the floor still has blocks to compare. That reading is
  // looser than the one a candidate is held to, so a pass under it is weaker
  // evidence and says so here.
  if ((validation.kind === 'valid') && (validation.pageGrammar === 'relaxed')) {
    rl.info(`${voice.modelId}: candidate passed against a page read as plain markdown, since strict MDX refused the page`,);
    return {
      voice,
      findings: ['translate-page-downgraded (page read as plain markdown; strict MDX refused it)',],
    };
  }
  if (validation.kind === 'valid')
    return {
      voice,
      findings: [],
    };

  /**
   What validation found, recorded whatever the author answers.
   */
  const found = validation.findings
    .map(function toFinding(finding,): string {
      return `translate-invalid (${voice.modelId}): ${finding}`;
    },);
  /**
   Whether original candidate violates non-defensible target authority floor.
   */
  const contributorViolation = validation.findings
    .includes(CONTRIBUTOR_AUTHORITY_FINDING,);
  /**
   What becomes of the original candidate when no revision is taken, as the
   log says it.
   */
  const fate = unrevisedFate({ contributorViolation, },);
  rl.warn(
    `${voice.modelId}: candidate fails the publication rule, so its author is asked about it: ${
      validation.findings
        .join(' ',)
    }`,
  );
  /**
   Characters of finding text this answer has to address.

   COUNTED BECAUSE THE REPAIR WIRE EXPLAINS ITSELF. Its `translatorNote` field
   answers these findings, and the producing wire has no such field, so the
   slice alone does not bound what a correct answer here can run to.
   */
  const findingsChars = validation.findings
    .join('',)
    .length;

  /**
   The author's answer to its own findings.
   */
  const answer = await (async function asked(): Promise<StageVoice<TranslateRepairWire>> {
    try {
      return await attemptStageCall({
        client,
        modelId: voice.modelId,
        messages: buildTranslateRepairMessages({
          priorMessages,
          priorTranslation: voice.value
            .translation,
          findings: validation.findings,
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
        l: rl,
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
        rl.warn(
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
  })();
  if (!answer.heard) {
    rl.warn(`${voice.modelId} gave no usable answer to its findings, so ${fate}`,);
    return {
      ...((contributorViolation) ? {} : { voice, }),
      findings: [
        ...found,
        `translate-repair-unheard (${voice.modelId})`,
      ],
    };
  }

  /**
   What the author decided, and why.
   */
  const {
    resolution,
    translation,
    translatorNote: explanation,
  } = answer.value;
  if (resolution !== 'revised') {
    rl.warn(`${voice.modelId} answered ${resolution} to its findings (${explanation}), so ${fate}`,);
    return {
      ...((contributorViolation) ? {} : { voice, }),
      findings: [
        ...found,
        `translate-repair-${resolution} (${voice.modelId}): ${explanation}`,
      ],
    };
  }

  /**
   The revision as it would ship, folded before the recheck reads it, as the
   lane folded the answer it revises (ledger B112).
   */
  const revision = foldTranslation({
    modelId: voice.modelId,
    translation,
  },);

  /**
   Whether the revision actually resolved what was found, a pass or a
   refusal on the ground the first verdict was read on.
   */
  const rechecked = requireComparedVerdict({
    verdict: validateTranslatedSlice({
      sourceText,
      candidateText: revision.translation,
      pageText,
      ...((syntax === undefined) ? {} : { syntax, }),
      lineStructured,
      declared,
      disputedWordings,
    },),
  },);

  // A revision that still fails is NOT taken. The model was asked to fix these
  // findings and did not, so nothing says the new text is better, while the
  // original is at least what it produced with the whole sheet in front of it.
  if (rechecked.kind === 'invalid') {
    rl.warn(`${voice.modelId}: revision still fails the publication rule and is not taken, so ${fate}`,);
    return {
      ...((contributorViolation) ? {} : { voice, }),
      findings: [
        ...found,
        ...revision.findings,
        `translate-repair-unresolved (${voice.modelId}): ${explanation}`,
      ],
    };
  }
  rl.info(`translate-repair: ${voice.modelId} revised its candidate`,);
  return {
    voice: {
      modelId: voice.modelId,
      value: { translation: revision.translation, },
    },
    findings: [
      ...found,
      ...revision.findings,
      `translate-repair-revised (${voice.modelId})`,
    ],
  };
}

/**
 Validates every fresh candidate and gives each failing one back to its
 author.

 Candidates are handled CONCURRENTLY, one follow-up call per failing model at
 most, so a slice where every translator diverged costs one extra round rather
 than one extra round each.

 @param client - injected model client

 @param voices - heard translator replies

 @param sourceText - original every candidate renders

 @param incumbentText - translation as it stands, so a candidate reproducing
 it is left alone

 @param pageText - text these candidates would replace, whose blocks they have
 to carry, defaulting to the incumbent because that is what a translator
 replaces. A consolidator replaces the ARCHIVE while its incumbent is the lane
 that won, and the two are different texts

 @param priorMessages - exact messages every translator was given

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param lineStructured - whether the line-structure rule governs this slice,
 which makes merging its lines a fault the author is sent back to fix

 @param declared - name pairs the front matter declares, which the
 publication rule reads for a linked title naming a declared person (class
 one hundred fourteen)

 @param disputedWordings - wordings a disputed slice refuses, which a
 candidate copying one is sent back over rather than collapsed into (owner,
 2026-09-27, "No eligible standing")

 @param l - stage logger

 @returns Final voices in the order given, plus every finding

 @throws {@link import('./translate-floor-ground.ts').FloorGroundDisagreementError}
 when the floor says it compared nothing, which both callers rule out before
 any writer is asked (ledger B43, B45)

 @throws Whatever the first failing candidate's turn raised, in input order,
 once the caller aborted. Each follow-up exchange that fails of its own as
 the caller stops writes that failure on its seat's line as it ends, which
 for a later-listed seat still in flight can be after this rejects

 @example
 ```ts
 const { voices, findings, } = await repairInvalidCandidates({ ... },);
 ```
 */
export async function repairInvalidCandidates(
  {
    client,
    voices,
    sourceText,
    incumbentText,
    pageText = incumbentText,
    syntax,
    lineStructured = false,
    declared = [],
    disputedWordings = [],
    priorMessages,
    signal,
    perCallTimeoutMs,
    l,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly voices: readonly HeardVoice<TranslateReportWire>[];
    readonly sourceText: string;
    readonly incumbentText: string;
    readonly pageText?: string;
    readonly syntax?: SliceSyntax;
    readonly lineStructured?: boolean;
    readonly declared?: readonly DeclaredNamePair[];
    readonly disputedWordings?: readonly DisputedWording[];
    readonly priorMessages: readonly ChatMessage[];
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
  }>,
): Promise<{
  readonly voices: readonly HeardVoice<TranslateReportWire>[];
  readonly findings: readonly string[];
}> {
  /**
   One outcome per heard voice.
   */
  const outcomes = await allInInputOrder({
    members: voices.map(async function repairEach(voice,): Promise<RepairOutcome> {
      return await repairOneCandidate({
        client,
        voice,
        sourceText,
        incumbentText,
        pageText,
        ...((syntax === undefined) ? {} : { syntax, }),
        lineStructured,
        declared,
        disputedWordings,
        priorMessages,
        signal,
        perCallTimeoutMs,
        l,
      },);
    },),
  },);

  return {
    voices: outcomes.flatMap(function toVoice(
      outcome,
    ): readonly HeardVoice<TranslateReportWire>[] {
      return (outcome.voice === undefined) ? [] : [outcome.voice,];
    },),
    findings: outcomes.flatMap(function toFindings(outcome,): readonly string[] {
      return outcome.findings;
    },),
  };
}

//endregion Translate repair
