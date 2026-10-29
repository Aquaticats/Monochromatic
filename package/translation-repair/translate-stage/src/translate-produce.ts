import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type {
  Candidate,
  DeclaredNamePair,
} from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { SliceSyntax, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  buildTranslateCandidates,
  buildTranslateMessages,
  type DisputedWording,
  foldTranslatorVoices,
  type IncumbentKind,
  isTranslateReportWire,
  producedVolumeBound,
  repairInvalidCandidates,
  TRANSLATE_RESPONSE_FORMAT,
  type TranslateCandidateValue,
  type TranslateFollowupEvidence,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import { floorTranslateVoices, } from './translate-floor.ts';
import {
  type FanOutMode,
  gatherStageVoices,
  type StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';

//region Translate produce
// The half of the translate stage that WRITES: several models render the slice
// independently, invalid renderings go back to their authors, and the distinct
// survivors become a slate with the incumbent among them.
//
// SPLIT FROM JUDGING so one slate can be judged more than once. Everything here
// is stochastic and expensive, and everything after it is a question put to
// judges about a fixed set of texts. While the two were one call, any caller
// wanting to ask that question twice had to buy a second slate, so the two
// answers differed in the candidates as well as in whatever the caller meant to
// vary. The window trial wants the judged evidence varied with the slate held
// still, and the judge-quality bench's position-bias attempt wants the slate
// held still while ballot position
// moves; neither is expressible against a stage that reproduces on every call.
//
// The rosters are NOT checked here. `assertJudgeableProducerRoster` needs both
// sides, so it belongs to whoever holds both: `runTranslateStage` does it, and a
// caller driving the halves directly has to do it too.

/**
 A slate as the judges will receive it, with what producing it cost and found.

 @example
 ```ts
 const slate: ProducedSlate = await produceTranslateSlate({ ... },);
 ```
 */
export type ProducedSlate = {
  /**
   Distinct proposals, incumbent among them when it has text.
   */
  readonly candidates: readonly Candidate<TranslateCandidateValue>[];

  /**
   Translators that answered usably, out of those seated.

   Carried because a decision taken over a thin slate is not the same
   decision, and the judging half has no other way to know.
   */
  readonly heardTranslators: number;

  /**
   Everything gathering, repairing and building recorded, in scorecard-stable
   wording. Every exit of the judging half reports these ahead of its own.
   */
  readonly findings: readonly string[];
};

/**
 Renders one slice several times and returns the slate to judge.

 @param client - injected model client

 @param translatorModelIds - models rendering the slice independently

 @param sourceText - original slice text

 @param incumbentText - translation as it stands, blank where this slice has
 none

 @param incumbentKind - caller-established absence allowing a visible source-break view

 @param incumbentEligible - whether existing translation passes deterministic source floor

 @param identityContext - declared names from both sides' front matter,
 omitted when neither declares anything

 @param archiveDisputeNote - why the existing translation shown is the
 repair lane's stand-in and which of its details are accepted additions, so
 no translator keeps them as page content (class one hundred eight; never
 forwarded until the audit of 2026-09-27)

 @param attestedLines - archive details a cited reference states, shown to
 the translators so their renderings carry them (class thirty-nine)

 @param syntax - syntax role requiring dedicated preservation rules

 @param followupEvidence - latest rejected slate and findings grounding retry

 @param lineStructured - whether the enclosing chunk's original is
 line-structured, decided by the caller

 @param declared - name pairs the front matter declares, which the
 publication rule reads for a linked title naming a declared person (class
 one hundred fourteen)

 @param disputedWordings - wordings a disputed slice refuses, so a translator
 copying the archive there is sent back and then withheld (owner,
 2026-09-27); none elsewhere

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - pipeline logger

 @param fanOut - seats a round asks: the window of quorum plus one by
 default, or the whole bench a fixture scripting every seat asks for, since
 the window is rotated by the sheet text and a sheet edit elsewhere moves
 which seats it holds

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Slate, heard count and findings

 @example
 ```ts
 const slate = await produceTranslateSlate({ client, translatorModelIds, sourceText, incumbentText, lineStructured, signal, perCallTimeoutMs, l, stageSettings, },);
 ```
 */
export async function produceTranslateSlate(
  {
    client,
    translatorModelIds,
    sourceText,
    incumbentText,
    incumbentKind = 'present',
    incumbentEligible = true,
    identityContext,
    archiveDisputeNote,
    pictureContext,
    attestedLines,
    syntax,
    followupEvidence,
    lineStructured,
    declared = [],
    disputedWordings = [],
    signal,
    perCallTimeoutMs,
    l,
    fanOut,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly translatorModelIds: readonly RosterModelId[];
    readonly sourceText: string;
    readonly incumbentText: string;
    readonly incumbentKind?: IncumbentKind;
    readonly incumbentEligible?: boolean;
    readonly identityContext?: string;
    readonly archiveDisputeNote?: string;
    readonly pictureContext?: string;
    readonly attestedLines?: readonly string[];
    readonly syntax?: SliceSyntax;
    readonly followupEvidence?: TranslateFollowupEvidence;
    readonly lineStructured: boolean;
    readonly declared?: readonly DeclaredNamePair[];
    readonly disputedWordings?: readonly DisputedWording[];
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly fanOut?: FanOutMode;
    readonly stageSettings: StageSettings;
  }>,
): Promise<ProducedSlate> {
  /**
   Logger tagged with this half.
   */
  const tl = tagged({
    tag: produceTranslateSlate.name,
    l,
  },);

  /**
   Translator sheet shared by every translator, so their renderings answer the
   same question and stay comparable.
   */
  const plan = buildTranslateMessages({
    sourceText,
    existingText: incumbentText,
    incumbentKind,
    ...((identityContext === undefined) ? {} : { identityContext, }),
    ...((archiveDisputeNote === undefined) ? {} : { archiveDisputeNote, }),
    ...((pictureContext === undefined) ? {} : { pictureContext, }),
    ...((attestedLines === undefined) ? {} : { attestedLines, }),
    ...((syntax === undefined) ? {} : { syntax, }),
    ...((followupEvidence === undefined) ? {} : { followupEvidence, }),
    lineStructured,
  },);

  /**
   Translator replies after retry-to-quorum.
   */
  const gather = await gatherStageVoices({
    client,
    modelIds: translatorModelIds,
    messages: plan.messages,
    signal,
    exchangeTimeoutMs: perCallTimeoutMs,
    // A TRANSLATION IS ABOUT THREE TIMES ITS CHINESE SOURCE at every
    // measured length, so this slice's own size bounds what any answer to
    // it can legitimately be. Without it the only bound is absolute, and
    // one 56-character slice here produced 10381 characters unchallenged.
    maxAnswerChars: producedVolumeBound({ materialChars: sourceText.length, },),
    responseFormat: TRANSLATE_RESPONSE_FORMAT,
    validate: isTranslateReportWire,
    stage: 'translate',
    // A WRITER ROUND: every voice the window cuts is one fewer rendering on
    // the slate, so it waits under the writer window, built in since 2026-09-06
    // (`writer-grace-override.ts`).
    graceMs: stageSettings
      .grace
      .writerMs,
    l: tl,
    // Conditional spread keeps the knob absent instead of undefined.
    ...((fanOut === undefined) ? {} : { fanOut, }),
  },);

  /**
   Translator replies as they would ship, folded before anything reads them,
   so the publication rule, the repair turn and the floor judge the bytes the
   judges see and the page gets (ledger B112).
   */
  const intake = foldTranslatorVoices({ voices: gather.voices, },);

  /**
   Candidates after structural validation, with anything that failed handed
   back to its own author.

   The INCUMBENT is not among these and is never validated into or out of
   the slate. It is the fallback and the text being defended, so a check
   that could drop it would be a check that could delete the archive.
   */
  const repaired = await repairInvalidCandidates({
    client,
    voices: intake.voices,
    sourceText,
    incumbentText,
    // STATED, NOT DEFAULTED (ledger B29): a translator replaces the incumbent
    // the stage judges, which is the page shape the floor after this turn
    // reads too (`translate-floor.ts`).
    pageText: incumbentText,
    ...((syntax === undefined) ? {} : { syntax, }),
    lineStructured,
    declared,
    disputedWordings,
    priorMessages: plan.messages,
    signal,
    perCallTimeoutMs,
    l: tl,
  },);

  /**
   Candidates the deterministic publication rule accepts after the repair
   turn; a text the rule refuses cannot reach the page and is withheld from
   the judges (class sixty-six, XingZ615, 2026-09-19).
   */
  const floored = floorTranslateVoices({
    voices: repaired.voices,
    sourceText,
    incumbentText,
    ...((syntax === undefined) ? {} : { syntax, }),
    lineStructured,
    declared,
    disputedWordings,
    l: tl,
  },);
  /**
   Slate of distinct proposals with the incumbent among them.
   */
  const built = buildTranslateCandidates({
    voices: floored.voices,
    translatorModelIds,
    incumbentText: incumbentEligible ? incumbentText : '',
    lineStructured,
  },);

  return {
    candidates: built.candidates,
    heardTranslators: gather.voices
      .length,
    /**
     Findings shared by every exit after the fan-out.
     */
    findings: [
      ...gather.findings,
      ...intake.findings,
      ...repaired.findings,
      ...floored.findings,
      ...built.findings,
      ...(incumbentEligible ? [] : ['translate incumbent excluded by deterministic source floor',]),
      `translate-candidates (${String(gather.voices
        .length,)}/${String(translatorModelIds.length,)} heard, ${
        String(built.candidates
          .length,)
      } distinct, ${String(built.collapsed,)} collapsed)`,
    ],
  };
}

//endregion Translate produce
