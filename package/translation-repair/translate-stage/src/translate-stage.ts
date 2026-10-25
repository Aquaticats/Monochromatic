import type { Logger, } from '@monochromatic-dev/module-logger/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { SliceSyntax, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { DeclaredNamePair, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import {
  assertJudgeableProducerRoster,
  type ProducerRosterError,
} from '@monochromatic-dev/translation-repair-assembly/ts';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type DisputedWording,
  floorReach,
  type IncumbentKind,
  requireComparedVerdict,
  type TranslateAbsenceError,
  validateTranslatedSlice,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import { runTranslateRepairs, } from './translate-stage-repair.ts';
import type { TranslateStageResult, } from './translate-stage-result.ts';
import { settleUnflooredSlice, } from './translate-unfloored.ts';
import type {
  FanOutMode,
  StageSettings,
} from '@monochromatic-dev/translation-repair-stage/ts';

//region Translate stage
// Every slice is translated from the ORIGINAL by several models independently,
// the translation already in the archive stands as one more candidate, and the
// whole judge roster chooses per slice, with a translator's ballot for its own
// rendering counted at reduced weight.
//
// This is not the editor stage with a different prompt. The editor answers
// "repair these named defects in this region", which cannot reach a passage that
// was never translated and cannot see a slice that is present, fluent and
// mediocre. This asks "render this passage", which reaches both.
//
// A fresh candidate whose Markdown structure, footnote markers, links or
// inline code do not match the ORIGINAL is not dropped: it goes back to the
// model that wrote it with the findings, and that model revises, declines, or
// says the finding is a fact about the passage rather than about its work. See
// `translate-repair.ts`. The apply gate cannot serve here at all, since every
// policy in it is anchored to an edit bounded by an envelope some accepted
// issue named, and a whole-slice replacement has none.
//
// A SLICE THE FLOOR CAN COMPARE NOTHING ON IS SETTLED BEFORE ANY CALL (ledger
// B43; `translate-unfloored.ts`): no candidate could pass there, so nothing
// bought could be used.
//
// DECLARED NAMES ARE CHECKED ONE LEVEL UP, in `translate-slice.ts`
// (`findDroppedDeclaredNames`), against the forms preparation parsed from the
// front matter; this note once said the check was missing, from before it
// existed. What this stage still does not do is check anything that crosses a
// slice boundary.


/**
 Translates one slice from its original and returns the text that ships.

 @param client - injected model client

 @param translatorModelIds - models rendering the slice independently

 @param judgeModelIds - whole roster selection seats, translators included;
 a ballot for the judge's own rendering counts for less

 @param sourceText - original slice text

 @param incumbentText - translation as it stands, blank where this slice has
 none

 @param incumbentKind - whether there is a translation to fall back on,
 decided by the caller from the target chunk rather than from the text being
 blank: a content span holding only whitespace is the archive's own wording,
 and an anchor is a place where a rendering belongs and none exists

 @param identityContext - declared names from both sides' front matter,
 omitted when neither declares anything

 @param attestedLines - archive details a cited reference states, shown to
 the translators so their renderings carry them (class thirty-nine)

 @param referenceContext - what the pages the original cites say, omitted
 when it cites none

 @param neighbouringSourceText - original of the sections either side, shown as
 CONTEXT the candidates are not expected to render. Absent by default, so a
 caller that does not ask for it gets the sheet production has always sent.
 Relocation is why it exists: where the archive carried a passage across a section
 boundary, a judge shown one slice pair sees invention on one side and omission
 on the other, and the judge-quality bench's alteration arm went from 12 of 16 to 15 of 16 when
 the same trial was given exactly this

 @param neighbouringIncumbentText - archive English of the sections either
 side, shown so a passage missing here can be recognised next door rather than
 read as one the archive never had

 @param syntax - syntax role requiring dedicated production and judging rules

 @param lineStructured - whether the enclosing CHUNK's original is
 line-structured, decided by the caller

 @param declared - name pairs the front matter declares, which the
 publication rule reads for a linked title naming a declared person (class
 one hundred fourteen)

 @param disputedWordings - wordings a disputed slice refuses, which excludes
 the archive as incumbent and withholds a translator copying it (owner,
 2026-09-27, "No eligible standing"); none elsewhere

 @param signal - caller abort honored by every exchange

 @param perCallTimeoutMs - deadline per exchange

 @param l - pipeline logger

 @param fanOut - seats a round asks: the window of quorum plus one by
 default, or the whole bench a fixture scripting every seat asks for, since
 the window is rotated by the sheet text and a sheet edit elsewhere moves
 which seats it holds

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns Shipped text with how it was decided

 @throws {@link ProducerRosterError} when the
 roster could not select anything: repeats on either side, no translator, or
 fewer than two judges

 @throws {@link import('./translation-repair-interrupted-error.ts').TranslationRepairInterruptedError}
 when absent-passage correction repeats exact task or providers remain unavailable

 @throws {@link BlankSelectionError} when selection chose text that says
 nothing for a source that says something, in EITHER mode, since that is a
 deletion rather than an outcome

 @throws {@link TranslateAbsenceError}
 carrying `unfloored`, before any call, where the archive holds no
 translation and the floor can compare nothing written for the slice
 (ledger B43); where the archive holds one, it stands with nobody asked

 @example
 ```ts
 const translated = await runTranslateStage({ ... },);
 ```
 */
export async function runTranslateStage(
  {
    client,
    translatorModelIds,
    judgeModelIds,
    sourceText,
    incumbentText,
    incumbentKind,
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
    readonly judgeModelIds: readonly RosterModelId[];
    readonly sourceText: string;
    readonly incumbentText: string;
    readonly incumbentKind: IncumbentKind;
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
    readonly declared?: readonly DeclaredNamePair[];
    readonly disputedWordings?: readonly DisputedWording[];
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly fanOut?: FanOutMode;
    readonly stageSettings: StageSettings;
  }>,
): Promise<TranslateStageResult> {
  // BOTH ROSTERS ARE CHECKED HERE and in neither half, because this is the
  // only place that holds both. A caller driving the halves directly has to
  // make this call itself.
  assertJudgeableProducerRoster({
    producerModelIds: translatorModelIds,
    judgeModelIds,
    role: 'translator',
  },);

  /**
   Whether the floor can pass anything written for this slice, read as the
   floor reads it: every candidate here is floored against this original and
   this page (`translate-produce.ts`, `translate-floor.ts`).
   */
  const reach = floorReach({
    sourceText,
    pageText: incumbentText,
    ...((syntax === undefined) ? {} : { syntax, }),
  },);
  // NOTHING IS BOUGHT WHERE NOTHING BOUGHT COULD PASS (ledger B43): the
  // archive stands where there is one, and the passage stays unfilled where
  // there is none.
  if (reach.kind === 'blind') {
    return settleUnflooredSlice({
      incumbentText,
      incumbentKind,
      detail: reach.detail,
      l,
    },);
  }

  /**
   Deterministic source floor's verdict on the archive wording, absent where
   there is no archive wording; a pass or a refusal, since the ground was
   read as comparable.
   */
  const incumbentVerdict = (incumbentKind === 'present')
    ? requireComparedVerdict({
      verdict: validateTranslatedSlice({
        sourceText,
        candidateText: incumbentText,
        pageText: incumbentText,
        ...((syntax === undefined) ? {} : { syntax, }),
        lineStructured,
        declared,
        disputedWordings,
      },),
    },)
    : undefined;
  /**
   Whether archive wording itself may remain candidate or fallback.
   */
  const incumbentEligible = incumbentVerdict?.kind === 'valid';
  // The finding line alone said a floor refused the archive, not which: the
  // yingying10 read could not tell the class one hundred fourteen refusal from
  // any other without replaying the slice.
  if (incumbentVerdict?.kind === 'invalid') {
    /**
     What the floor said against the archive wording.
     */
    const against = incumbentVerdict.findings
      .join(' | ',);
    l.warn(`translate incumbent excluded by deterministic source floor: ${against}`,);
  }
  /**
   Existing fallback kind after deterministic source floor.
   */
  const effectiveIncumbentKind: IncumbentKind = incumbentEligible ? 'present' : 'absent';

  return await runTranslateRepairs({
    client,
    translatorModelIds,
    judgeModelIds,
    sourceText,
    incumbentText,
    incumbentKind: effectiveIncumbentKind,
    incumbentEligible,
    // Wording that exists and fails the floor is withheld, not absent: its
    // slice ships by preference past a declined follow-up round (owner,
    // 2026-09-27).
    incumbentWithheld: (incumbentKind === 'present') && (!incumbentEligible),
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
    stageSettings,
  },);
}

//endregion Translate stage
