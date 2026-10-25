import { tagged, } from '@monochromatic-dev/module-logger/ts';
import { contextRoot, } from '@monochromatic-dev/translation-repair-typography/ts';
import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  alterSharedNumber,
  type DamageAttempt,
  deleteOneSentence,
  type FidelityDamageKind,
  insertBorrowedSentence,
} from '@monochromatic-dev/translation-repair-repair/ts';
import { FidelityReferenceError, } from './fidelity-reference-error.ts';
import type {
  FidelityReferenceSpec,
  ReviewedFidelityReference,
} from './fidelity-reference-model.ts';
import {
  applyReviewedEdits,
  reviewedText,
} from './fidelity-reference-text.ts';
import { foldInvisibleVariants, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Materialized reviewed references
// Deterministic builders cannot turn an unverified archive into a clean reference.

/**
 Existing calibration length floor, never satisfied by padding or concatenating a short reference.
 */
export const MIN_REVIEWED_REFERENCE_CHARS = 400;
/**
 Reference verification logger.
 */
const l = contextRoot({ tag: 'fidelity-reference', },);

/**
 Runs the existing damage mechanism selected during source review.

 @param kind - reviewed defect family

 @param sourceText - pinned source passage

 @param referenceText - verified reference

 @param donorText - fixed reviewed donor

 @param referenceId - input named for an unknown requested family

 @returns Existing builder outcome, still requiring hash verification

 @throws {@link FidelityReferenceError} for an unsupported family

 @example
 ```ts
 const damage = reviewedDamage({ kind, sourceText, referenceText, donorText, referenceId });
 ```
 */
function reviewedDamage({
  kind,
  sourceText,
  referenceText,
  donorText,
  referenceId,
}: {
  readonly kind: FidelityDamageKind;
  readonly sourceText: string;
  readonly referenceText: string;
  readonly donorText: string;
  readonly referenceId: string;
},): DamageAttempt {
  if (kind === 'deletion')
    return deleteOneSentence({ cleanText: referenceText, },);
  if (kind === 'insertion')
    return insertBorrowedSentence({
      cleanText: referenceText,
      donorTexts: [donorText],
    },);
  if (kind === 'alteration')
    return alterSharedNumber({
      cleanText: referenceText,
      sourceText,
    },);
  throw new FidelityReferenceError({
    referenceId,
    operation: 'damage',
  },);
}

/**
 Reconstructs only a source-reviewed reference and its exact approved damage variants.
 Review is represented by the caller-owned manifest; byte checks prevent silent drift,
 not semantic inference from the fact that text is an unchanged archive.

 @param sourceFile - complete source read at the specified pin

 @param archiveFile - complete archive read at that same pin

 @param spec - reviewed ranges, edits and variant hashes

 @returns Owned reference data ready for a fixed calibration matrix

 @throws {@link FidelityReferenceError} for any mismatch with the reviewed inputs

 @example
 ```ts
 const reference = buildReviewedFidelityReference({ sourceFile, archiveFile, spec });
 ```
 */
export function buildReviewedFidelityReference({
  sourceFile,
  archiveFile,
  spec,
}: {
  readonly sourceFile: string;
  readonly archiveFile: string;
  readonly spec: FidelityReferenceSpec;
},): ReviewedFidelityReference {
  /**
   Own the provenance so later caller mutation cannot change a verified result.
   */
  const checked = structuredClone(spec,);
  /**
   Function-scoped logging includes identifiers, never reference prose.
   */
  const rl = tagged({
    tag: buildReviewedFidelityReference.name,
    l,
  },);
  if ((checked.id
    .trim()
    === '') || (checked.reviewedOn
      .trim()
      === ''))
    throw new FidelityReferenceError({
      referenceId: checked.id,
      operation: 'request',
    },);
  /**
   Exact current source, not a nearby or generated candidate passage.
   */
  const sourceText = reviewedText({
    text: sourceFile,
    span: checked.source,
    referenceId: checked.id,
    operation: 'source',
  },);
  /**
   Raw archive range before the review's explicit transforms.
   */
  const original = reviewedText({
    text: archiveFile,
    span: checked.archive,
    referenceId: checked.id,
    operation: 'archive',
  },);
  /**
   Same invisible-character normalization used by archive intake.
   */
  const folded = foldInvisibleVariants({ text: original, },)
    .text;
  /**
   Local correction remains calibration data, never a corpus write.
   */
  const referenceText = applyReviewedEdits({
    reference: folded,
    edits: checked.edits,
    referenceId: checked.id,
  },);
  if ((referenceText.length < MIN_REVIEWED_REFERENCE_CHARS)
    || (referenceText.length !== checked.referenceChars)
    || (hashContent({ content: referenceText, },) !== checked.referenceHash)) {
    throw new FidelityReferenceError({
      referenceId: checked.id,
      operation: 'reference',
    },);
  }
  /**
   Donor content must be outside the reference's original archive range.
   */
  const donorText = reviewedText({
    text: archiveFile,
    span: checked.donor,
    referenceId: checked.id,
    operation: 'donor',
  },);
  if ((checked.donor
    .startOffset
    < checked.archive
      .endOffset)
    && (checked.archive
      .startOffset
      < checked.donor
        .endOffset)) {
    throw new FidelityReferenceError({
      referenceId: checked.id,
      operation: 'donor',
    },);
  }
  /**
   Each reviewed family appears once; an empty list is not a calibration.
   */
  const kinds = new Set(checked.damages
    .map(function kindOf(damage,): FidelityDamageKind {
      return damage.kind;
    },),);
  if ((kinds.size === 0) || (kinds.size
    !== checked.damages
      .length))
    throw new FidelityReferenceError({
      referenceId: checked.id,
      operation: 'damage',
    },);
  /**
   Review locks both wording and the builder's stated delta, not merely its family label.
   */
  const damages = checked.damages
    .map(function verify(expected,): Extract<DamageAttempt, { readonly kind: 'damaged'; }> {
    /**
     Mechanically generated twin, not automatically trusted as a valid comparison.
     */
      const damage = reviewedDamage({
        kind: expected.kind,
        sourceText,
        referenceText,
        donorText,
        referenceId: checked.id,
      },);
      if ((damage.kind !== 'damaged') || (damage.damageKind !== expected.kind)
        || (damage.changedChars !== expected.changedChars)
        || (hashContent({ content: damage.damagedText, },) !== expected.hash)) {
        throw new FidelityReferenceError({
          referenceId: checked.id,
          operation: 'damage',
        },);
      }
      return damage;
    },);
  rl.debug(`verified ${checked.id} with ${String(damages.length,)} reviewed variants`,);
  return {
    spec: checked,
    sourceText,
    referenceText,
    damages,
    pages: {
      sourceFile,
      archiveFile,
    },
  };
}

//endregion Materialized reviewed references
