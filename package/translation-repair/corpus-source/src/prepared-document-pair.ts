import type { ArchiveOriginalSpan, } from './archive-original-note.ts';
import type {
  ChunkPair,
  SectionBlockPairing,
} from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { DeclaredNamePair, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import type { SectionPair, } from '@monochromatic-dev/translation-repair-stage/ts';
import type { AttestedDetail, } from '@monochromatic-dev/translation-repair-reference/ts';

//region Prepared document pair
// The shapes preparation produces, moved out of `document-preparation.ts` at
// its line budget: what a document pair is reduced to, and a target block the
// pairing left without a source claim.

/**
 Target block pairing roster left without source claim.

 @example
 ```ts
 const block: UnclaimedTargetBlock = {
   location: { kind: 'aligned-pair', pairIndex: 0, },
   blockId: 'block/2',
   startOffset: 41,
   endOffset: 73,
 };
 ```
 */
export type UnclaimedTargetBlock = {
  /**
   Alignment location whose source claim is absent.
   */
  readonly location: {
    /**
     Block sits inside aligned section pair.
     */
    readonly kind: 'aligned-pair';

    /**
     Aligned pair index.
     */
    readonly pairIndex: number;
  } | {
    /**
     Whole target section sits outside alignment.
     */
    readonly kind: 'target-section';

    /**
     Target section index.
     */
    readonly sectionIndex: number;
  };

  /**
   Stable parser id within target document.
   */
  readonly blockId: string;

  /**
   First target-text offset owned by block.
   */
  readonly startOffset: number;

  /**
   Target-text boundary immediately after block.
   */
  readonly endOffset: number;
};

/**
 Who the page's front matter comes from.

 `rendered` is the lanes' rendering of the source's metadata, judged like
 every other slice; `archive` is the archive's own bytes, left alone by the
 owner's rule of 2026-09-08 (`corpus-run/archive-front-matter.ts`).

 @example
 ```ts
 const authority: FrontMatterAuthority = 'archive';
 ```
 */
export type FrontMatterAuthority = 'archive' | 'rendered';

/**
 A document pair reduced to the slices both lanes run over.

 @example
 ```ts
 const prepared = prepareDocumentPair({ sourceText, targetText, },);
 ```
 */
export type PreparedDocumentPair = {
  /**
   Marks body-only slicing rebuilt for artifact generations before five.

   Current preparations omit it and use metadata-aware identity scheme.
   */
  readonly legacyIdentity?: true;

  /**
   Marks a preparation whose front matter is the archive's, left alone,
   rather than a slice zero the lanes render; omitted when they render it.
   */
  readonly frontMatterAuthority?: 'archive';

  /**
   Spans of the archive its translators' notes call the English original,
   which no slice covers and which ship as they stand (the owner's rule of
   2026-09-08, `archive-original-note.ts`); omitted when the archive carries
   no such note or the preparation was asked not to seal.
   */
  readonly archiveOriginalSpans?: readonly ArchiveOriginalSpan[];

  /**
   Original document this preparation was made from.
   */
  readonly sourceText: string;

  /**
   Whether the original's corpus entry records a death (`info.died` in its
   `info.yml`). True exactly when the source side declares one; false for a
   preparation made without an entry's `info.yml`, which records no death.
   `prepareDocumentPair` always states it. The sheets that judge tense read
   this flag, never the identity block.
   */
  readonly lifeEnded?: boolean;

  /**
   Declared names of the page's subject on the translation side (`name` in its
   front matter, then each alias in `info.alias`), which the tense candidates
   read as subjects. Present only when {@link PreparedDocumentPair.lifeEnded}
   is true and the translation declares a name or an alias.
   */
  readonly subjectNames?: readonly string[];

  /**
   Translation the target spans and offsets index into.

   Carried so a lane assembles against the document it was prepared from. A
   driver handed a preparation and an unrelated translation would splice at
   offsets that mean nothing there, and produce plausible-looking text.
   */
  readonly targetText: string;

  /**
   Paragraph-bound slice pairs across every aligned section, indexed globally
   in document order.
   */
  readonly slices: readonly ChunkPair[];

  /**
   Slice indexes the line-structure rule governs, inherited from the enclosing
   chunk rather than decided per slice.
   */
  readonly lineStructuredSliceIndices: ReadonlySet<number>;

  /**
   Declared names and handles from front matter plus target-authoritative
   contributor identities, the original's pronoun for its subject, the notes
   both documents carry and any web-lookup evidence, joined into prompt block.
   Each line says which kind it is; the sheets read declarations as
   authoritative and notes and lookups as vocabulary evidence only.

   Absent rather than empty when nothing is declared, noted or looked up, so
   a caller spreading it into a prompt never emits a heading with nothing
   under it.
   */
  readonly identityContext?: string;

  /**
   What the pages the original links say, one line per page, for the critic
   and panel sheets (class thirty-five, 2026-09-16): a detail the archive
   carries that the original does not state but a cited reference does is
   accurate detail the translator took from the original's own references.

   Absent rather than empty when the original links nowhere or no key is set.
   */
  readonly referenceContext?: string;

  /**
   Archive details a cited reference states, attested by the bench with
   quotes checked word for word (class thirty-seven, 2026-09-16), which the
   repair lane screens addition claims against before the panel. The sheets
   read the same details as ATTESTED lines inside `referenceContext`.

   Absent when nothing was attested.
   */
  readonly attestedDetails?: readonly AttestedDetail[];

  /**
   Declared name and contributor forms as TRANSLATION side spells them.

   SEPARATE FROM `identityContext`, which is prose for a prompt. These are the
   strings a guard compares, and the guard exists because asking a model to
   keep a name does not work: probed on the repair lane's own judge sheet,
   six of six judges preferred the candidate that dropped a declared alias.

   TRANSLATION SIDE ONLY, because the text being guarded is English.
   */
  readonly declaredNames: readonly string[];

  /**
   The forms among `declaredNames` that the TRANSLATION's front matter declares
   for the entry's own subject: the display name and every alias, one person.

   WHAT IT BUYS. A guard that finds a declared form lost can read another of
   these standing in its place, one for one, as a swap between two names of the
   same person rather than as a person unnamed. A contributor handle is not in
   it, because a contributor is another person.

   OPTIONAL so a pair built before the identity was named, or by a caller with
   no front matter, reads as naming nobody and every guard keeps the plain rule
   of dropping nothing it carried. Absence can only refuse more, never less.
   */
  readonly declaredSubjectForms?: readonly string[];

  /**
   Name pairs the front matter declares on both sides, source form beside
   its declared rendering, read by the publication rule's declared-name floor
   (class one hundred fourteen) wherever a candidate is validated.

   OPTIONAL so a pair built before the floor existed, or by a caller with no
   front matter, reads as declaring nothing and leaves the floor silent.
   */
  readonly declaredNamePairs?: readonly DeclaredNamePair[];

  /**
   Alignment findings in scorecard-stable wording.
   */
  readonly alignmentFindings: readonly string[];

  /**
   Archive blocks pairing roster deliberately left outside every source claim.

   STRUCTURED APART FROM `alignmentFindings` so publication safety never parses
   diagnostic prose to decide whether unreviewed archive wording exists.
   */
  readonly unclaimedTargetBlocks: readonly UnclaimedTargetBlock[];

  /**
   Entries in the aligned unit list, which is the count worth logging beside
   the slice count: a document with far more slices than units subdivided
   heavily.

   INSERTIONS ARE INCLUDED, and they are not section PAIRS: an insertion names
   a place in the translation where an untranslated original belongs, so it
   has an original on one side and a boundary on the other. The name predates
   insertions existing and is kept because settled version 2 artifacts record
   it, and because every consumer wants exactly this number: it is the bound
   `parseBlockPairing` refuses a section index against, and a real-pair
   count there would falsely refuse a block pairing filed after an insertion.

   WHICH entries are insertions is reported by
   {@link PreparedDocumentPair.alignmentFindings}, where each one names its
   source section and either the boundary it was placed at or the refusal that
   stopped it. That is the separate report, rather than a second count here.
   */
  readonly alignmentPairCount: number;

  /**
   Pairing this slicing was built on, echoed back so what gets recorded is the
   object slicing consumed rather than a second copy assembled beside it.

   ABSENT WHEN NOBODY WAS ASKED, present and possibly empty when somebody was.
   A section missing from a present list had no pairing consumed for it, and
   which of the several reasons applies is legible from
   {@link PreparedDocumentPair.alignmentFindings}, not from here.
   */
  readonly blockPairing?: readonly SectionBlockPairing[];

  /**
   Section pairing this alignment was built on, echoed back the same way and
   for the same reason as {@link PreparedDocumentPair.blockPairing}: the
   artifact records the value slicing consumed, not a copy assembled beside it.

   ABSENT WHEN THE DETERMINISTIC ALIGNER DECIDED THE SECTIONS, present when a
   supplied pairing did. The roster shell supplies one only when its section
   round agreed on at least one pair, so a present list is non-empty in
   production; a direct caller's empty list is echoed as consumed rather than
   normalised away, because an empty supplied pairing aligns nothing and the
   deterministic path aligns by shape, and those are different slicings.
   */
  readonly sectionPairing?: readonly SectionPair[];
};

//endregion Prepared document pair
