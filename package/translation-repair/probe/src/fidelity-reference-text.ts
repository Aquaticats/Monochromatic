import { hashContent, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  FidelityReferenceError,
  type FidelityReferenceOperation,
} from './fidelity-reference-error.ts';
import type {
  FidelityReferenceEdit,
  FidelityReferenceSpan,
} from './fidelity-reference-model.ts';

//region Exact reviewed text
// Hashes bind review to literal ranges, not fuzzy spelling searches or generated candidates.

/**
 Reads a nonempty reviewed range and refuses any identity or coordinate drift.

 @param text - complete pinned file or explicitly identified folded reference

 @param span - exact reviewed coordinates and hash

 @param referenceId - input named by refusal

 @param operation - source of this range

 @returns Exact reviewed substring

 @throws {@link FidelityReferenceError} for invalid coordinates or hash mismatch

 @example
 ```ts
 const sourceText = reviewedText({ text: sourceFile, span: spec.source, referenceId: spec.id, operation: 'source' });
 ```
 */
export function reviewedText({
  text,
  span,
  referenceId,
  operation,
}: {
  readonly text: string;
  readonly span: FidelityReferenceSpan;
  readonly referenceId: string;
  readonly operation: FidelityReferenceOperation;
},): string {
  if ((!Number.isSafeInteger(span.startOffset,)) || (!Number.isSafeInteger(span.endOffset,))
    || (span.startOffset < 0)
    || (span.endOffset <= span.startOffset)
    || (span.endOffset > text.length)) {
    throw new FidelityReferenceError({
      referenceId,
      operation,
    },);
  }
  /**
   One literal substring, never normalized to make an incorrect hash pass.
   */
  const selected = text.slice(
    span.startOffset,
    span.endOffset,
  );
  if (hashContent({ content: selected, },) !== span.hash)
    throw new FidelityReferenceError({
      referenceId,
      operation,
    },);
  return selected;
}

/**
 Applies only reviewed, disjoint corrections anchored to the original folded slice.
 Every edit is checked against that same original, not against preceding replacements.

 @param reference - folded original archive slice

 @param edits - reviewed local changes

 @param referenceId - input named by refusal

 @returns Corrected reference without changing the corpus

 @throws {@link FidelityReferenceError} for overlap, stale text or missing provenance

 @example
 ```ts
 const corrected = applyReviewedEdits({ reference, edits: spec.edits, referenceId: spec.id });
 ```
 */
export function applyReviewedEdits({
  reference,
  edits,
  referenceId,
}: {
  readonly reference: string;
  readonly edits: readonly FidelityReferenceEdit[];
  readonly referenceId: string;
},): string {
  if (edits.length === 0)
    return reference;
  /**
   Original-coordinate order allows one reconstruction without offset rebasing.
   */
  const ordered = edits.toSorted(function byStart(
    left,
    right,
  ): number {
    return left.startOffset - right.startOffset;
  },);
  /**
   Each untouched segment is copied once.
   */
  const pieces: string[] = [];
  /**
   Next original character not yet copied.
   */
  const cursor = { at: 0, };
  for (const edit of ordered) {
    if ((edit.startOffset < cursor.at) || (edit.author
      .trim()
      === '')
      || (edit.rationale
        .trim()
        === ''))
      throw new FidelityReferenceError({
        referenceId,
        operation: 'edit',
      },);
    reviewedText({
      text: reference,
      span: {
        startOffset: edit.startOffset,
        endOffset: edit.endOffset,
        hash: edit.expectedHash,
      },
      referenceId,
      operation: 'edit',
    },);
    pieces.push(
      reference.slice(
        cursor.at,
        edit.startOffset,
      ),
      edit.replacement,
    );
    cursor.at = edit.endOffset;
  }
  pieces.push(reference.slice(cursor.at,),);
  return pieces.join('',);
}

//endregion Exact reviewed text
