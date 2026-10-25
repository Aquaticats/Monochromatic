import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  type FidelityDamageKind,
  sentenceRegionAround,
  type SentenceRegion,
} from '@monochromatic-dev/translation-repair-repair/ts';
import {
  type AdjudicatedIssue,
  computeIssueClaimId,
  deriveEditableEnvelopes,
  type EditableEnvelope,
  type IssueClaim,
} from '@monochromatic-dev/translation-repair-shared-contract/ts';
import {
  type EditorSheetContext,
  editorSheetContextOf,
} from './editor-fidelity-context.ts';
import {
  differingMiddle,
  digitRunAround,
} from './editor-fidelity-difference.ts';
import { insertionRegionWithNeighbours, } from './editor-fidelity-region.ts';
import { FidelityReferenceError, } from './fidelity-reference-error.ts';
import type {
  FidelityReferenceSpec,
  ReviewedFidelityReference,
} from './fidelity-reference-model.ts';

//region Editor fidelity trials
// What the editor fidelity probe seats a card on: a reviewed reference whose damaged twin has a fix that is a pure
// function of the clean text, the accepted issue a critic would have filed on it, and the envelope the pipeline cuts
// from that issue.
//
// TWO OF THE THREE DEFECTS THE JUDGE PROBE MEASURES. The INSERTION is a sentence borrowed from elsewhere in the
// document, spliced in; the fix is to remove it. The ALTERATION changes a number the original also states; the fix is
// to put the number back. The DELETION is left out, and the request that names it is refused: its fix is a sentence
// the editor must write, and no deterministic test scores an editor's rendering against the archive's own wording
// (the brief ruled no model judges).
//
// ONE TRIAL PER TWIN, NOT FOUR. The judge probe multiplies each twin by two directions and two ballot positions
// because a judge sees a ballot; an editor is handed the damaged text and the issue, so those arrangements would ask
// the same question four times.
//
// THE REGION IS THE SENTENCE A CRITIC WOULD QUOTE, WIDENED FOR THE INSERTION. For the alteration it is the sentence
// holding the number, so an editor may rewrite the sentence and the score can see what else it touched. For the
// insertion it is the inserted sentence with the sentence either side of it on its line (`editor-fidelity-region.ts`),
// so that deleting the quoted text, which needs no reading of the original, is not a clean fix. An insertion with no
// neighbour on its line is refused: its region would be the sentence alone.

/**
 Defect families the probe builds, in the manifest's order of difficulty: the removal of a sentence, then the
 correction of a number.

 @example
 ```ts
 const kinds: readonly EditorDamageKind[] = EDITOR_DAMAGE_KINDS;
 ```
 */
export const EDITOR_DAMAGE_KINDS = [
  'insertion',
  'alteration',
] as const;

/**
 A defect family the probe builds.

 @example
 ```ts
 const kind: EditorDamageKind = 'insertion';
 ```
 */
export type EditorDamageKind = typeof EDITOR_DAMAGE_KINDS[number];

/**
 One seat of a card on one reviewed twin.

 @example
 ```ts
 const [trial,] = editorFidelityTrials({ references, damageKinds: EDITOR_DAMAGE_KINDS, },);
 ```
 */
export type EditorFidelityTrial = {
  /**
   Handle of the trial, `<reference id>/<damage kind>`, as the judge probe names its own.
   */
  readonly trialId: string;

  /**
   Reviewed specification the twin came from.
   */
  readonly spec: FidelityReferenceSpec;

  /**
   Defect the damaged text carries.
   */
  readonly damageKind: EditorDamageKind;

  /**
   Builder's reviewed changed-content count.
   */
  readonly changedChars: number;

  /**
   Safe structural description of the intentional delta.
   */
  readonly damageDetail: string;

  /**
   Original-language passage the editor reads beside the translation.
   */
  readonly sourceText: string;

  /**
   The reviewed reference, which is the text a perfect fix produces.
   */
  readonly cleanText: string;

  /**
   The translation the editor is handed, carrying the defect.
   */
  readonly damagedText: string;

  /**
   Where the defect begins, which is the same offset in the clean and the damaged text.
   */
  readonly defectStart: number;

  /**
   What the damaged text holds at the defect: the inserted sentence with the separator the splice added, or the
   altered number whole.
   */
  readonly damagedDefect: string;

  /**
   What the clean text holds there: nothing for an insertion, the original number for an alteration.
   */
  readonly cleanDefect: string;

  /**
   The region the accepted issue quotes, in the damaged text: the sentence the defect sits in, and for an insertion
   the sentence either side of it on its line as well.
   */
  readonly region: SentenceRegion;

  /**
   What the sheet carries besides the passage and its issue: the sections either side, the declared names and the whole
   original, prepared from the pages the passage was cut from.
   */
  readonly context: EditorSheetContext;

  /**
   The one accepted issue, as the panel would have left it.
   */
  readonly issues: readonly AdjudicatedIssue[];

  /**
   The envelope the pipeline cuts from that issue.
   */
  readonly envelopes: readonly EditableEnvelope[];
};

/**
 Category and summary the accepted issue carries for each defect, stated as a critic states a defect's kind and
 nothing the text said, so the summary never carries corpus prose and never hands the editor the fix.
 */
const CLAIMS: Readonly<Record<EditorDamageKind, Pick<IssueClaim, 'category' | 'summary'>>> = {
  insertion: {
    category: 'accuracy/addition',
    summary: 'The translation adds a sentence the original does not state.',
  },
  alteration: {
    category: 'accuracy/mistranslation',
    summary: 'A number in the translation differs from the number the original states.',
  },
};

/**
 Whether a family is one the probe builds.

 @param kind - family read off a reference

 @returns Whether the probe builds it

 @example
 ```ts
 // yields false
 const builds = isEditorDamageKind('deletion',);
 ```
 */
function isEditorDamageKind(kind: FidelityDamageKind,): kind is EditorDamageKind {
  return EDITOR_DAMAGE_KINDS.some(function is(candidate,): boolean {
    return candidate === kind;
  },);
}

/**
 Builds the accepted issue and the envelope for one region.

 @param kind - defect family

 @param damagedText - the translation the issue is filed on

 @param region - sentences the issue quotes

 @returns The issue and the envelope the pipeline cuts from it

 @example
 ```ts
 const { issues, envelopes, } = issueFor({ kind: 'insertion', damagedText, region, },);
 ```
 */
function issueFor(
  {
    kind,
    damagedText,
    region,
  }: {
    readonly kind: EditorDamageKind;
    readonly damagedText: string;
    readonly region: SentenceRegion;
  },
): {
  readonly issues: readonly AdjudicatedIssue[];
  readonly envelopes: readonly EditableEnvelope[];
} {
  /**
   Claim a critic would have filed, quoting the region byte for byte as the preservation gate requires.
   */
  const claim: IssueClaim = {
    ...CLAIMS[kind],
    severity: 'major',
    spans: [
      {
        side: 'target',
        nodeId: 'probe/region',
        nodeHash: hashContent({ content: damagedText, },),
        startOffset: region.startOffset,
        endOffset: region.endOffset,
        quotedText: damagedText.slice(
          region.startOffset,
          region.endOffset,
        ),
      },
    ],
  };
  /**
   Deterministic identity of the claim.
   */
  const claimId = computeIssueClaimId({ claim, },);
  /**
   The issue after a panel accepted it.
   */
  const issue: AdjudicatedIssue = {
    issueId: `adjudicated/${hashContent({ content: claimId, },)}`,
    status: 'accepted',
    severity: 'major',
    claims: [
      {
        claimId,
        claim,
      },
    ],
    tallies: {},
  };
  return {
    issues: [issue,],
    envelopes: deriveEditableEnvelopes({
      issues: [issue,],
      targetText: damagedText,
    },)
      .envelopes,
  };
}

/**
 Expands reviewed twins into the trials the editor probe seats a card on.

 @param references - materialized and hash-verified reviewed inputs

 @param damageKinds - requested families, each one the probe builds

 @returns One trial per reference and requested family, in reference and manifest order

 @throws {@link FidelityReferenceError} when no family is requested, one is repeated, one is not built by this probe
 (a deletion has no deterministic fix), one is carried by no reference or an inserted sentence has no neighbour on its
 line to quote it with

 @example
 ```ts
 const trials = editorFidelityTrials({ references, damageKinds: EDITOR_DAMAGE_KINDS, },);
 ```
 */
export function editorFidelityTrials(
  {
    references,
    damageKinds,
  }: {
    readonly references: readonly ReviewedFidelityReference[];
    readonly damageKinds: readonly FidelityDamageKind[];
  },
): readonly EditorFidelityTrial[] {
  if ((damageKinds.length === 0) || (new Set(damageKinds,).size !== damageKinds.length)) {
    throw new FidelityReferenceError({
      referenceId: 'damage selection',
      operation: 'request',
    },);
  }
  /**
   Families this probe has no deterministic fix for.
   */
  const unbuilt = damageKinds.filter(function isUnbuilt(kind,): boolean {
    return !isEditorDamageKind(kind,);
  },);
  if (unbuilt.length > 0) {
    throw new FidelityReferenceError({
      referenceId: `damage selection (${unbuilt.join(', ',)})`,
      operation: 'request',
    },);
  }

  /**
   Every trial exists before a model is asked.
   */
  const trials = references.flatMap(function referenceTrials(reference,): readonly EditorFidelityTrial[] {
    /**
     The context of the reference's page, prepared once for both families.
     */
    const context = editorSheetContextOf({ reference, },);
    return reference.damages
      .filter(function requested(damage,): boolean {
        return damageKinds.includes(damage.damageKind,);
      },)
      .flatMap(function trialOf(damage,): readonly EditorFidelityTrial[] {
        if (!isEditorDamageKind(damage.damageKind,))
          return [];
        /**
         Where the twin and the reference part ways.
         */
        const middle = differingMiddle({
          cleanText: reference.referenceText,
          damagedText: damage.damagedText,
        },);
        /**
         The middle read whole: a number by its digit run, a sentence as the splice wrote it.
         */
        const whole = (damage.damageKind === 'alteration')
          ? digitRunAround({
            text: damage.damagedText,
            start: middle.start,
            end: middle.damagedEnd,
          },)
          : {
            start: middle.start,
            grown: 0,
          };
        /**
         Where the defect ends in the damaged text.
         */
        const damagedEnd = middle.damagedEnd + whole.grown;
        /**
         Where the defect ends in the clean text.
         */
        const cleanEnd = middle.cleanEnd + whole.grown;
        /**
         The sentences the defect sits in.
         */
        const sentences = sentenceRegionAround({
          text: damage.damagedText,
          startOffset: whole.start,
          endOffset: damagedEnd,
        },);
        /**
         The region quoted: the sentences, widened by their neighbours when the defect is an inserted sentence.
         */
        const {
          region,
          neighbours,
        } = (damage.damageKind === 'insertion')
          ? insertionRegionWithNeighbours({
            text: damage.damagedText,
            region: sentences,
          },)
          : {
            region: sentences,
            neighbours: 1,
          };
        if (neighbours === 0) {
          throw new FidelityReferenceError({
            referenceId: reference.spec
              .id,
            operation: 'damage',
          },);
        }
        return [
          {
            trialId: `${reference.spec
              .id}/${damage.damageKind}`,
            spec: reference.spec,
            damageKind: damage.damageKind,
            changedChars: damage.changedChars,
            damageDetail: damage.damageDetail,
            sourceText: reference.sourceText,
            cleanText: reference.referenceText,
            damagedText: damage.damagedText,
            defectStart: whole.start,
            damagedDefect: damage.damagedText
              .slice(
              whole.start,
              damagedEnd,
              ),
            cleanDefect: reference.referenceText
              .slice(
              whole.start,
              cleanEnd,
              ),
            region,
            context,
            ...issueFor({
              kind: damage.damageKind,
              damagedText: damage.damagedText,
              region,
            },),
          },
        ];
      },);
  },);

  /**
   A nonempty prefix is not evidence that every requested family was materialized.
   */
  const unavailable = damageKinds.filter(function missing(kind,): boolean {
    return !trials.some(function represents(trial,): boolean {
      return trial.damageKind === kind;
    },);
  },);
  if (unavailable.length > 0) {
    throw new FidelityReferenceError({
      referenceId: `damage selection (${unavailable.join(', ',)})`,
      operation: 'request',
    },);
  }
  return trials;
}

//endregion Editor fidelity trials
