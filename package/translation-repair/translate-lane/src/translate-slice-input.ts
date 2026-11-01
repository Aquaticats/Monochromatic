import type {
  ChunkPair,
  SliceSyntax,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { isInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type DisputedWording,
  disputedWordingFindings,
  type IncumbentKind,
} from '@monochromatic-dev/translation-repair-translate-floor/ts';
import type { DeclaredNamePair, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import { attestedDetailLines, } from '@monochromatic-dev/translation-repair-reference/ts';
import { splitTargetOnlyRun, } from './target-only-run.ts';
import {
  type PreparedDocumentPair,
  preparedSheetFactsOf,
} from '@monochromatic-dev/translation-repair-corpus-source/ts';

//region Deterministic pre-stage surface
// Production settlement and calibration share inputs without sharing publication-disposition guards.

/**
 Text and governance supplied before the translation stage computes its messages and seat windows.
 Provider clients, electorates, deadlines and publication disposition remain caller-owned.

 @example
 ```ts
 const result = await runTranslateStage({ ...surface.stageInput, client, translatorModelIds, judgeModelIds, signal, perCallTimeoutMs, l });
 ```
 */
export type TranslateSliceStageInput = {
  /**
   Canonical source passage, not its presentation-only writer display.
   */
  readonly sourceText: string;
  /**
   Incumbent wording remaining after the existing structural protection.
   */
  readonly incumbentText: string;
  /**
   Target chunk variant, not a guess based on string length.
   */
  readonly incumbentKind: IncumbentKind;
  /**
   Existing document-level declarations and vocabulary context.
   */
  readonly identityContext?: string;
  /**
   Whether the original's entry records a death, carried from the preparation
   so the judges see the tense candidates of the texts on offer.
   */
  readonly lifeEnded?: boolean;
  /**
   Names the translation declares for its subject, its display name and its
   aliases, read by the tense candidates as named subjects; absent when the
   translation declares none.
   */
  readonly subjectNames?: readonly string[];
  /**
   What the pages the original cites say, when it cites any.
   */
  readonly referenceContext?: string;
  /**
   Archive details a cited reference states, one sheet line each, present
   only when the entry attested any (class thirty-nine).
   */
  readonly attestedLines?: readonly string[];
  /**
   Why the incumbent is the repair lane's stand-in and which details are
   accepted additions, present only on a disputed slice (class one hundred
   eight).
   */
  readonly archiveDisputeNote?: string;
  /**
   Wordings a disputed slice refuses as incumbent and as candidate, present
   only where the stand-in may not stand (owner, 2026-09-27, "No eligible
   standing").
   */
  readonly disputedWordings?: readonly DisputedWording[];
  /**
   Supplied source neighbors retain their context-only role.
   */
  readonly neighbouringSourceText?: string;
  /**
   Supplied archive neighbors retain their existing comparison role.
   */
  readonly neighbouringIncumbentText?: string;
  /**
   Previously acquired picture evidence, never bought by this projection.
   */
  readonly pictureContext?: string;
  /**
   Existing syntax role for metadata-aware stages.
   */
  readonly syntax?: SliceSyntax;
  /**
   Production child-level governance, including inherited parent treatment.
   */
  readonly lineStructured: boolean;
  /**
   Name pairs the front matter declares, for the publication rule's
   declared-name floor (class one hundred fourteen).
   */
  readonly declared?: readonly DeclaredNamePair[];
};

/**
 Rewritable stage inputs beside archive content the stage must not grade or regenerate.

 @example
 ```ts
 const { stageInput, protectedText, } = translateSliceInput({ slice, prepared });
 ```
 */
export type TranslateSliceInput = {
  /**
   Complete original incumbent span retained for publication restoration.
   */
  readonly archiveText: string;
  /**
   Existing target-only run held outside writing and judging.
   */
  readonly protectedText: string;
  /**
   Exact data supplied to the unchanged translation-stage operation.
   */
  readonly stageInput: TranslateSliceStageInput;
};

/**
 Projects a prepared slice through the existing target-only protection and input-governance rules.
 Performs no I/O, model call, publication decision or new context lookup.
 Protection excludes a known target-only run from both writing and judging;
 reattaching it only after selection would still put protected material into the comparison.
 Callers retain logging, restore protected text and decide separately whether wording may publish.

 @param slice - prepared source/target pair whose target index selects governance

 @param prepared - owning preparation supplying identity context and child line flags

 @param neighbouringSourceText - already-built source context passed through unchanged

 @param neighbouringIncumbentText - already-built incumbent context passed through unchanged

 @param pictureContext - already-acquired corroborated picture context passed through unchanged

 @param archiveStandIn - repair lane's text standing in for a disputed archive
 rendering (class one hundred seven), judged and restored as though it were
 the archive so the disputed wording is neither a candidate nor the fallback

 @param archiveDisputeNote - sheet note naming the accepted additions the
 stand-in answers for, so no writer or judge reads them as page content
 (class one hundred eight)

 @param disputedWordings - wordings a disputed slice refuses, where the
 stand-in may not stand; the judged part of the archive is refused with
 them when a target-only run was held out (owner, 2026-09-27)

 @returns Stage surface and protected archive material without changing the operation being measured

 @example
 ```ts
 const surface = translateSliceInput({ slice, prepared, pictureContext });
 ```
 */
export function translateSliceInput(
  {
    slice,
    prepared,
    neighbouringSourceText,
    neighbouringIncumbentText,
    pictureContext,
    archiveStandIn,
    archiveDisputeNote,
    disputedWordings = [],
  }: {
    readonly slice: ChunkPair;
    readonly prepared: PreparedDocumentPair;
    readonly archiveStandIn?: string;
    readonly archiveDisputeNote?: string;
    readonly disputedWordings?: readonly DisputedWording[];
    readonly neighbouringSourceText?: string;
    readonly neighbouringIncumbentText?: string;
    readonly pictureContext?: string;
  },
): TranslateSliceInput {
  /**
   Complete archive wording before target-only protection.
   */
  const pageWording = slice.target
    .text;
  /**
   Wording the stage judges as the archive: the stand-in on a disputed slice
   (class one hundred seven), the archive's own elsewhere.
   */
  const archiveText = archiveStandIn ?? pageWording;
  /**
   Canonical original this slice renders.
   */
  const sourceText = slice.source
    .text;
  /**
   Existing transcript protection applied before either writing or judging.
   */
  const {
    judgedText,
    protectedText,
  } = splitTargetOnlyRun({
    sourceText,
    incumbentText: archiveText,
  },);
  /**
   Attested lines the translators are shown, none when nothing was attested
   (class thirty-nine).
   */
  const attestedLines = attestedDetailLines({ details: prepared.attestedDetails ?? [], },);
  /**
   First refused wording, whose reason the judged part shares.
   */
  const [firstDisputed,] = disputedWordings;
  /**
   What the rule says of the archive text the stage reads as a disputed
   wording, none where it is not one: an eligible stand-in is not, and its
   judged part is then not refused either.
   */
  const archiveFindings = disputedWordingFindings({
    candidateText: archiveText,
    disputedWordings,
  },);
  /**
   The refused wordings with the archive's judged part among them where the
   archive text is itself refused, since the judged part is then the
   incumbent the stage reads.
   */
  const refused = ((firstDisputed === undefined)
    || (archiveFindings.length === 0)
    || (judgedText === archiveText))
    ? disputedWordings
    : [
      ...disputedWordings,
      {
        text: judgedText,
        reason: firstDisputed.reason,
      },
    ];
  return {
    archiveText,
    protectedText,
    stageInput: {
      sourceText,
      incumbentText: judgedText,
      incumbentKind: isInsertionChunk(slice.target,) ? 'absent' : 'present',
      lineStructured: prepared.lineStructuredSliceIndices
        .has(slice.target
          .sliceIndex,),
      ...preparedSheetFactsOf({ prepared, },),
      ...((attestedLines.length === 0) ? {} : { attestedLines, }),
      ...((archiveDisputeNote === undefined) ? {} : { archiveDisputeNote, }),
      ...((refused.length === 0) ? {} : { disputedWordings: refused, }),
      ...((neighbouringSourceText === undefined) ? {} : { neighbouringSourceText, }),
      ...((neighbouringIncumbentText === undefined) ? {} : { neighbouringIncumbentText, }),
      ...((pictureContext === undefined) ? {} : { pictureContext, }),
      ...((slice.syntax === undefined) ? {} : { syntax: slice.syntax, }),
      ...((prepared.declaredNamePairs === undefined) ? {} : { declared: prepared.declaredNamePairs, }),
    },
  };
}

//endregion Deterministic pre-stage surface
