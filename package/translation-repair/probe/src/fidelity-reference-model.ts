import type {
  DamageAttempt,
  FidelityDamageKind,
} from '@monochromatic-dev/translation-repair-repair/ts';

//region Reviewed calibration references
// Metadata binds reviewed meanings to exact pinned inputs without committing corpus prose.

/**
 Exact UTF-16 range in a pinned file or explicitly identified local reference.

 @example
 ```ts
 const span: FidelityReferenceSpan = { startOffset: 0, endOffset: text.length, hash };
 ```
 */
export type FidelityReferenceSpan = {
  /**
   Inclusive offset under the corpus reader's CRLF normalization.
   */
  readonly startOffset: number;
  /**
   Exclusive offset under that same normalization.
   */
  readonly endOffset: number;
  /**
   Hash of the exact selected text.
   */
  readonly hash: string;
};

/**
 Reviewed local correction, never an edit to the corpus or a production translation filter.

 @example
 ```ts
 const edited = applyReviewedEdits({ reference, edits, referenceId });
 ```
 */
export type FidelityReferenceEdit = {
  /**
   Inclusive offset in the original folded local archive slice.
   */
  readonly startOffset: number;
  /**
   Exclusive offset in that same unedited folded slice.
   */
  readonly endOffset: number;
  /**
   Hash of the removed wording, checked before any edit is applied.
   */
  readonly expectedHash: string;
  /**
   Independently reviewed replacement wording authored outside the peer roster.
   */
  readonly replacement: string;
  /**
   Recorded author identity for calibration provenance.
   */
  readonly author: string;
  /**
   Evidence-based reason for the local correction.
   */
  readonly rationale: string;
};

/**
 One reviewed deterministic damage variant.

 @example
 ```ts
 const expected: FidelityExpectedDamage = { kind: 'deletion', hash, changedChars };
 ```
 */
export type FidelityExpectedDamage = {
  /**
   Existing damage builder selected by the fixture review.
   */
  readonly kind: FidelityDamageKind;
  /**
   Exact damaged-text hash approved during source review.
   */
  readonly hash: string;
  /**
   Builder's changed-content count, not an inferred text-length difference.
   */
  readonly changedChars: number;
};

/**
 Reviewed reference specification, containing provenance rather than full corpus passages.

 @example
 ```ts
 const reference = await readFidelityReference({ pin, spec });
 ```
 */
export type FidelityReferenceSpec = {
  /**
   Stable reference identity, not a production slice index.
   */
  readonly id: string;
  /**
   Corpus entry supplying source, archive and donor.
   */
  readonly entryId: string;
  /**
   Commit under which the review applies.
   */
  readonly corpusSha: string;
  /**
   Reviewed source passage.
   */
  readonly source: FidelityReferenceSpan;
  /**
   Original archive passage before local normalization or correction.
   */
  readonly archive: FidelityReferenceSpan;
  /**
   Reviewed final reference hash after all stated transforms.
   */
  readonly referenceHash: string;
  /**
   Reviewed final length, subject to the unchanged natural-length floor.
   */
  readonly referenceChars: number;
  /**
   Local corrections anchored to the folded original slice.
   */
  readonly edits: readonly FidelityReferenceEdit[];
  /**
   Fixed donor paragraph in the same pinned archive file.
   */
  readonly donor: FidelityReferenceSpan;
  /**
   Every permitted reviewed mutation, in intended trial order.
   */
  readonly damages: readonly FidelityExpectedDamage[];
  /**
   Date of source/reference/damage review, not model benchmark time.
   */
  readonly reviewedOn: string;
};

/**
 Materialized reviewed comparison data; a generic untouched archive is not this type of evidence.

 @example
 ```ts
 const trial = { sourceText: reference.sourceText, cleanText: reference.referenceText, ... };
 ```
 */
export type ReviewedFidelityReference = {
  /**
   Original reviewed provenance remains beside the materialized text.
   */
  readonly spec: FidelityReferenceSpec;
  /**
   Exact source selected from the pin.
   */
  readonly sourceText: string;
  /**
   Source-verified reference, distinguished from the original archive.
   */
  readonly referenceText: string;
  /**
   Generated variants checked against reviewed hashes and single-delta counts.
   */
  readonly damages: readonly Extract<DamageAttempt, { readonly kind: 'damaged'; }>[];
  /**
   The whole pages the passages were cut from, at the same pin. The editor probe prepares the context of its sheet
   from them (the sections either side of the passage, the declared names, the whole original), as the pipeline
   prepares it from the page it repairs.
   */
  readonly pages: {
    /**
     Complete original page.
     */
    readonly sourceFile: string;
    /**
     Complete archive page.
     */
    readonly archiveFile: string;
  };
};

//endregion Reviewed calibration references
