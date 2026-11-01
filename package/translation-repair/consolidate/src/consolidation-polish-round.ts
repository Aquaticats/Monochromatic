import type { Logger, } from '@monochromatic-dev/logger-core/ts';
import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

import type { ModelClient, } from '@monochromatic-dev/translation-repair-provider/ts';
import type { ConsolidationPolishConfig, } from './consolidation-polish-model.ts';
import {
  wordForCount,
  wrapReplacementText,
} from '@monochromatic-dev/translation-repair-typography/ts';
import {
  type ConsolidationPolishGateOutcome,
  gateConsolidationPolish,
} from './consolidation-polish-gate-stage.ts';
import {
  deriveRefinableEnvelopes,
  parseDocument,
  type RosterModelId,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { RepairJudgedRound, } from '@monochromatic-dev/translation-repair-assembly/ts';
import {
  type RefineStageMode,
  runRefineStage,
} from '@monochromatic-dev/translation-repair-repair/ts';
import {
  type DisputedWording,
  type FloorGroundDisagreementError,
  floorReach,
  requireComparedVerdict,
  sameWording,
  validateTranslatedSlice,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import { unflooredFinding, } from '@monochromatic-dev/translation-repair-translate-stage/ts';
import type { StageSettings, } from '@monochromatic-dev/translation-repair-stage/ts';

//region One bounded consolidation polish round

/**
 Final polish reviews every structurally eligible body paragraph.
 */
const FINAL_POLISH_MINIMUM_CHARS = 0;

/**
 Result of one generation, selection, structural check and fidelity gate.

 @example
 ```ts
 const result: ConsolidationPolishRoundResult = { text: 'The cat slept.', proposedText: 'The cat slept.', changed: false, refinersHeard: [], contributors: [], rounds: [], findings: [] };
 ```
 */
export type ConsolidationPolishRoundResult = {
  /**
   Exact text selected after fidelity gate.
   */
  readonly text: string;

  /**
   Selected refinement before structural and fidelity gates.
   */
  readonly proposedText: string;

  /**
   Whether round replaced its input base.
   */
  readonly changed: boolean;

  /**
   Rewriters returning usable structured reply.
   */
  readonly refinersHeard: readonly RosterModelId[];

  /**
   Models whose work selected text carries.
   */
  readonly contributors: readonly RosterModelId[];

  /**
   Candidate-selection round, when candidates reached judges.
   */
  readonly rounds: readonly RepairJudgedRound[];

  /**
   Fidelity-first comparative gate, when selected proposal passed structure.
   */
  readonly gate?: ConsolidationPolishGateOutcome;

  /**
   Stable generation, selection and gate findings.
   */
  readonly findings: readonly string[];
};

/**
 Reads exact structurally correctable body paragraphs in display order.

 @param text - would-ship Markdown slice

 @returns Paragraph texts reviewer numbers and correction stage envelopes

 @example
 ```ts
 const paragraphs = finalPolishParagraphs({ text: 'The cat slept.' });
 ```
 */
export function finalPolishParagraphs(
  { text, }: { readonly text: string; },
): readonly string[] {
  return deriveRefinableEnvelopes({
    document: parseDocument({ text, },),
    minimumChars: FINAL_POLISH_MINIMUM_CHARS,
  },)
    .envelopes
    .map(function baseTextOf(envelope,): string {
      return envelope.baseText;
    },);
}

/**
 Reads every body block of a would-ship slice in display order, which is
 what the absolute reviewer is shown and may cite.

 EVERY BODY BLOCK, NOT ONLY THE REFINABLE ONES. The reviewer judges the whole
 candidate and locates each finding by paragraph number, and the stage
 refuses a finding that names a paragraph it did not show. Numbering only
 the refinable paragraphs left a blockquote poem with nothing to cite: on the
 Toka_ls rerun of 2026-09-02, slice 10 (a 29-line letter in blockquote) had
 zero refinable paragraphs, so six of nine reviewers who located their
 findings by stanza were refused as out of range and only the three
 "acceptable" ballots survived. A block the polish may not edit can still be
 judged and cited.

 @param text - would-ship Markdown slice

 @returns Block texts reviewer numbers, empty for a slice with no body block

 @example
 ```ts
 const paragraphs = reviewParagraphsOf({ text: '> A poem.\n\nA paragraph.' });
 // => ['> A poem.', 'A paragraph.']
 ```
 */
export function reviewParagraphsOf(
  { text, }: { readonly text: string; },
): readonly string[] {
  return parseDocument({ text, },)
    .nodes
    .filter(function inBody(node,): boolean {
      return node.zone === 'body';
    },)
    .map(function textOf(node,): string {
      return node.text;
    },);
}

/**
 Runs exactly one final-polish generation and its existing deterministic gates.

 @param client - provider client

 @param sourceText - Chinese fidelity anchor

 @param archiveText - archived English evidence

 @param baseText - exact would-ship input to this round, always body prose:
 the polish returns before any round on front matter, the one syntax role

 @param lineStructured - source line-boundary policy

 @param identityContext - declared identities and contributor forms

 @param referenceContext - what the pages the original cites say, with
 their rule, when the original cites any (class forty-one)

 @param archiveDisputeNote - accepted claims against the archive rendering
 on a disputed slice, which the gate reads (ledger S12) and the writers and the
 judges of their rewrites read too (ledger B805)

 @param acceptedRepairClaimsNote - every claim the repair panel accepted for
 the slice, which the writers, the judges of their rewrites and the gate read
 beside the dispute note (ledger B805)

 @param disputedWordings - wordings the slice refuses, which the rule refuses
 as the polished text as it does every other text on the slice (ledger B29)

 @param mode - comparative polish or correction of what judges objected to

 @param sliceIndex - prepared slice position

 @param config - model roles and document-wide definitions

 @param signal - caller cancellation

 @param perCallTimeoutMs - per-exchange deadline

 @param l - stage logger

 @param stageSettings - straggler windows and contest ledger the entry file
 read for this launch (`StageSettings`)

 @returns One bounded proposal round after structure and fidelity selection,
 or the base with nobody asked where the floor can compare nothing

 @throws {@link FloorGroundDisagreementError} when the structural check says
 it compared nothing on ground the reach read as comparable, a fault in this
 code

 @example
 ```ts
 const round = await runConsolidationPolishRound({ client, sourceText, archiveText, baseText, mode: { kind: 'comparative' }, lineStructured: false, sliceIndex: 1, config, signal, perCallTimeoutMs, l, stageSettings, });
 ```
 */
export async function runConsolidationPolishRound(
  {
    client,
    sourceText,
    archiveText,
    baseText,
    lineStructured,
    identityContext,
    referenceContext,
    archiveDisputeNote,
    acceptedRepairClaimsNote,
    disputedWordings,
    mode,
    sliceIndex,
    config,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  }: ForeignBorrowed<{
    readonly client: ModelClient;
    readonly sourceText: string;
    readonly archiveText: string;
    readonly baseText: string;
    readonly lineStructured: boolean;
    readonly identityContext?: string;
    readonly referenceContext?: string;
    readonly archiveDisputeNote?: string;
    readonly acceptedRepairClaimsNote?: string;
    readonly disputedWordings?: readonly DisputedWording[];
    readonly mode: RefineStageMode;
    readonly sliceIndex: number;
    readonly config: ConsolidationPolishConfig;
    readonly signal: AbortSignal;
    readonly perCallTimeoutMs: number;
    readonly l: Logger;
    readonly stageSettings: StageSettings;
  }>,
): Promise<ConsolidationPolishRoundResult> {
  /**
   Whether the floor can pass any polish here, read as the structural check
   reads one: against the original and the base it would replace.
   */
  const reach = floorReach({
    sourceText,
    pageText: baseText,
  },);
  // NOTHING IS BOUGHT WHERE NOTHING BOUGHT COULD PASS (ledger B48, as B43
  // for the translate stage and B45 for the consolidation): on blind ground
  // the structural check can at best leave a polish unvalidated, so the
  // round used to pay its refiners and judges for text it then kept out.
  if (reach.kind === 'blind') {
    l.warn(
      `slice ${String(sliceIndex,)}: the floor can compare nothing here (${reach.detail}), `
        + 'so no refiner is asked and the base stands',
    );
    return {
      text: baseText,
      proposedText: baseText,
      changed: false,
      refinersHeard: [],
      contributors: [],
      rounds: [],
      findings: [
        unflooredFinding({
          stage: 'consolidation-polish',
          detail: reach.detail,
        },),
      ],
    };
  }
  /**
   Paragraphs eligible under final-polish zero-length floor.
   */
  const {
    envelopes,
    definitions: baseDefinitions,
  } = deriveRefinableEnvelopes({
    document: parseDocument({ text: baseText, },),
    minimumChars: FINAL_POLISH_MINIMUM_CHARS,
  },);
  /**
   Archive-wide and current-base definitions visible to structural guards.
   */
  const definitions = [
    config.definitions,
    baseDefinitions,
  ]
    .filter(function present(value,): boolean {
      return value !== '';
    },)
    .join('\n',);
  /**
   One refinement generation and candidate selection.
   */
  const refined = await runRefineStage({
    client,
    refinerModelIds: config.refinerModelIds,
    judgeModelIds: config.judgeModelIds,
    sourceText,
    repairedText: baseText,
    envelopes,
    definitions,
    ...((identityContext === undefined) ? {} : { identityContext, }),
    ...((referenceContext === undefined) ? {} : { referenceContext, }),
    // THE CLAIMS THE REPAIR PANEL ACCEPTED AND THE NOTE ON A DISPUTED ARCHIVE
    // RENDERING reach the writers and the judges of their rewrites as they reach
    // the gate (ledger B805): the gate judged a polish against evidence the
    // writers never saw, and the judges chose among the rewrites without it.
    ...((acceptedRepairClaimsNote === undefined) ? {} : { acceptedRepairClaimsNote, }),
    ...((archiveDisputeNote === undefined) ? {} : { archiveDisputeNote, }),
    declaredNames: config.declaredNames,
    ...((config.declaredSubjectForms === undefined) ? {} : { declaredSubjectForms: config.declaredSubjectForms, }),
    mode,
    sliceIndex,
    signal,
    perCallTimeoutMs,
    l,
    stageSettings,
  },);
  if (!refined.changed) {
    return {
      text: baseText,
      proposedText: baseText,
      changed: false,
      refinersHeard: refined.heard,
      contributors: refined.contributors,
      rounds: refined.rounds,
      findings: refined.findings,
    };
  }
  /**
   Refinement as it would ship: wrapped at its semantic boundaries unless the
   line-structure rule governs the slice, in which case as the refiner wrote
   it, on the evidence `wrapConsolidation` cites.

   BEFORE THE GATE, on the rule `wrapConsolidationProposals` states: the
   deciders judge the bytes that ship. Measured on keyword233, 2026-09-03: the
   consolidation slate shipped wrapped, this round then handed the refiner's
   single-line rewrite to the gate beside that wrapped base, a gate judge
   chose it because it "removes the stilted line breaks", and the page
   shipped single-line where the 2026-09-02 landing had one clause per line.
   Wrapped here, the comparison is between two texts written to the same
   rule, and what the gate approves is what the page carries.
   */
  const polished = lineStructured
    ? refined.refinedText
    : wrapReplacementText({ text: refined.refinedText, },);
  /**
   Whether the wrap altered what the refiner emitted.
   */
  const rewrapped = polished !== refined.refinedText;
  /**
   Whether nothing but layout separates the refinement from the base, which
   may itself stand unwrapped where it is the archive's own wording
   (`sameWording`, ledger B26): a refinement that is the base rewrapped, or
   the base with its soft line breaks where no wrap would put them, publishes
   the page the base already publishes.
   */
  const demoted = sameWording({
    proposal: polished,
    standing: baseText,
    lineStructured,
  },);
  if (demoted) {
    l.info('wording: the polish is the base in all but layout, so the slice keeps what it had',);
    return {
      text: baseText,
      proposedText: polished,
      changed: false,
      refinersHeard: refined.heard,
      contributors: refined.contributors,
      rounds: refined.rounds,
      findings: [
        ...refined.findings,
        'consolidation-polish is the base in all but layout',
      ],
    };
  }
  if (rewrapped) {
    /**
     Lines the refiner wrote.
     */
    const emittedLines = refined.refinedText
      .split('\n',)
      .length;
    /**
     Lines the rule would have it written on.
     */
    const writtenLines = polished
      .split('\n',)
      .length;
    l.info(
      `semantic wrap: rewrapped the polish before its gate, ${String(emittedLines,)} ${
        wordForCount({
          count: emittedLines,
          one: 'line',
          many: 'lines',
        },)
      } as emitted `
        + `against ${String(writtenLines,)} as written`,
    );
  }
  /**
   Structural validity before semantic comparative gate, on ground the
   reach found comparable, so a pass or a refusal.
   */
  const validation = requireComparedVerdict({
    verdict: validateTranslatedSlice({
      sourceText,
      candidateText: polished,
      pageText: baseText,
      lineStructured,
      ...((config.declaredNamePairs === undefined) ? {} : { declared: config.declaredNamePairs, }),
      ...((disputedWordings === undefined) ? {} : { disputedWordings, }),
    },),
  },);
  if (validation.kind === 'invalid') {
    return {
      text: baseText,
      proposedText: polished,
      changed: false,
      refinersHeard: refined.heard,
      contributors: refined.contributors,
      rounds: refined.rounds,
      findings: [
        ...refined.findings,
        ...validation.findings,
        'consolidation-polish structural validation kept approved base',
      ],
    };
  }
  /**
   Existing fidelity-first comparative panel.
   */
  const gate = await gateConsolidationPolish({
    client,
    modelIds: config.gateModelIds,
    subject: {
      sourceText,
      archiveText,
      baseText,
      polishedText: polished,
      mode,
      lineStructured,
      ...((identityContext === undefined) ? {} : { identityContext, }),
      ...((referenceContext === undefined) ? {} : { referenceContext, }),
      ...((archiveDisputeNote === undefined) ? {} : { archiveDisputeNote, }),
      ...((acceptedRepairClaimsNote === undefined) ? {} : { acceptedRepairClaimsNote, }),
    },
    signal,
    exchangeTimeoutMs: perCallTimeoutMs,
    l,
    stageSettings,
  },);
  /**
   Exact text selected by comparative gate.
   */
  const text = (gate.ships === 'polished')
    ? polished
    : baseText;
  return {
    text,
    proposedText: polished,
    changed: text !== baseText,
    refinersHeard: refined.heard,
    contributors: refined.contributors,
    rounds: refined.rounds,
    gate,
    findings: [
      ...refined.findings,
      ...gate.findings,
    ],
  };
}

//endregion One bounded consolidation polish round
