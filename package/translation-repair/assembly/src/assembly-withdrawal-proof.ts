import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  type FootnoteGraphFinding,
  footnoteIdentifiers,
  footnoteMentions,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  type SliceReplacement,
  spliceSlices,
} from '@monochromatic-dev/translation-repair-page-pass/ts';
import { isInsertionChunk, } from '@monochromatic-dev/translation-repair-shared-contract/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import { introducedFootnoteFindings, } from './assembly-regressions.ts';

//region Withdrawal proof
// A SLICE IS WITHDRAWN FOR A FOOTNOTE FINDING ONLY WHEN THE WITHDRAWAL PROVABLY CLEARS IT on the page as it will be
// written (the lead's adoption of 2026-10-29, open to the owner's veto). XingZ6022 slice 44 kept an archive text that
// had dropped the reference `[^3]`; the translate lane's text for slice 115, an anchor the archive never translated
// whose original holds definition 3, carried the definition; and the guard withdrew slice 115 for the orphan. The
// withdrawal could not clear it, because a page ships an untranslated anchor's original under a note
// (`source-ships.ts` in the page-check package), and that original carries definition 3. A good translation was
// lost and the orphan stayed.
//
// SO A WITHDRAWN ANCHOR IS COUNTED AS CARRYING ITS ORIGINAL, and a withdrawn content slice as carrying its archive
// text, exactly as `singleStructuralWithdrawal` splices a real page rather than an isolated slice. The finding is
// cleared when the page so made no longer introduces that same finding; a different finding the withdrawal exposes
// is the next round's.
//
// A MISSING REFERENCE IS BLAMED ON THE SLICE THAT LOST IT: of the slices whose count of the reference moved down, the
// ones whose withdrawal proves out are withdrawn alone, leaving the definition beside them. Failing both, nothing is
// withdrawn and the guard reports the orphan, which the publish check then names.

/**
 Replacements whose slice changed how often it mentions an identifier.

 @param identifierKey - `convention identifier` at fault

 @param replacements - replacements still standing

 @param incumbentBySlice - incumbent text of every slice, by chunk index

 @returns Chunk indices to withdraw

 @example
 ```ts
 const culprits = suspectsFor({ identifierKey, replacements, incumbentBySlice, },);
 ```
 */
function suspectsFor(
  {
    identifierKey,
    replacements,
    incumbentBySlice,
  }: {
    readonly identifierKey: string;
    readonly replacements: readonly SliceReplacement[];
    readonly incumbentBySlice: ReadonlyMap<string, string>;
  },
): readonly number[] {
  return replacements
    .filter(function changedIt(replacement,): boolean {
      // EVERY STANDING REPLACEMENT NAMES A PREPARED SLICE: the guard refuses one
      // that does not (`assertReplacementsChange`) before building this map
      // from the same slices, so a miss is a broken invariant, never a slice
      // with no text.
      /**
       Archive's own text for this slice.
       */
      const incumbentText = nonNullishOrThrow(
        incumbentBySlice.get(String(replacement.sliceIndex,),),
      );

      /**
       Mentions counted in it.
       */
      const beforeCounts = footnoteIdentifiers({ text: incumbentText, },);

      /**
       Mentions counted in the accepted text.
       */
      const afterCounts = footnoteIdentifiers({
        text: replacement.replacementText,
      },);
      // EITHER role: a defect names an identifier, and both the reference that
      // points at it and the definition that answers it can be what moved.
      return [
        `reference ${identifierKey}`,
        `definition ${identifierKey}`,
      ].some(function moved(key,): boolean {
        return (beforeCounts.get(key,) ?? 0) !== (afterCounts.get(key,) ?? 0);
      },);
    },)
    .map(function toIndex(replacement,): number {
      return replacement.sliceIndex;
    },);
}

/**
 Whether a text defines a footnote identifier under either convention.

 @param text - original text of one slice

 @param identifier - identifier as the footnote graph keys it

 @returns Whether the text carries a definition of it

 @example
 ```ts
 const owns = definesIdentifier({ text: '[^1]: 那是它最喜欢的位置。', identifier: '1', },);
 ```
 */
function definesIdentifier(
  {
    text,
    identifier,
  }: {
    readonly text: string;
    readonly identifier: string;
  },
): boolean {
  return footnoteMentions({ text, },)
    .some(function defines(mention,): boolean {
      return (mention.role === 'definition') && (mention.identifier === identifier);
    },);
}

/**
 Narrows the suspects of a doubled definition to the carriers whose original
 does not define the note.

 THE SLICE WHOSE ORIGINAL DEFINES THE NOTE OWNS IT (class forty-nine,
 shi_Yumiaoya3, 2026-09-17): on a skeleton archive both the opening section,
 which wrote the note in beside its reference, and the definitions slice
 changed their count from nothing to one, so both were withdrawn and the
 page lost the section and the note alike. Where every suspect's original
 defines it, or none does, the suspects stand as found.

 @param finding - defect under attribution

 @param suspects - chunk indices whose mention count moved

 @param sourceBySlice - original text of every slice, by chunk index

 @returns Chunk indices to withdraw

 @example
 ```ts
 const culprits = withoutOwners({ finding, suspects, sourceBySlice, },);
 ```
 */
function withoutOwners(
  {
    finding,
    suspects,
    sourceBySlice,
  }: {
    readonly finding: FootnoteGraphFinding;
    readonly suspects: readonly number[];
    readonly sourceBySlice: ReadonlyMap<string, string>;
  },
): readonly number[] {
  if (finding.kind !== 'duplicate-definition')
    return suspects;
  /**
   Suspects whose original never defines the note.
   */
  const strangers = suspects.filter(function ownsNothing(sliceIndex,): boolean {
    // A suspect is a standing replacement's slice, which the guard checked
    // against the prepared slices this map was built from.
    /**
     Original text of the suspect's slice.
     */
    const sourceText = nonNullishOrThrow(
      sourceBySlice.get(String(sliceIndex,),),
    );
    return !definesIdentifier({
      text: sourceText,
      identifier: finding.identifier,
    },);
  },);
  return (strangers.length === 0) ? suspects : strangers;
}

/**
 Whether withdrawing one replacement clears one finding on the page as it will be written.

 @param finding - footnote defect the assembly introduced

 @param sliceIndex - slice whose replacement would be withdrawn

 @param standing - replacements standing at the start of the round

 @param targetText - inherited document the replacements are spliced over

 @param slices - prepared locations for every replacement

 @returns Whether the page with that replacement withdrawn no longer introduces that finding

 @example
 ```ts
 const clears = withdrawalClears({ finding, sliceIndex: 115, standing, targetText, slices, },);
 ```
 */
export function withdrawalClears(
  {
    finding,
    sliceIndex,
    standing,
    targetText,
    slices,
  }: {
    readonly finding: FootnoteGraphFinding;
    readonly sliceIndex: number;
    readonly standing: readonly SliceReplacement[];
    readonly targetText: string;
    readonly slices: readonly ChunkPair[];
  },
): boolean {
  /**
   The slice, whose original an untranslated anchor ships.
   */
  const withdrawn = slices.find(function isIt(slice,): boolean {
    return slice.target
      .sliceIndex
      === sliceIndex;
  },);
  /**
   What the page carries at the slice once its replacement is gone: the original at an anchor, which nothing else
   fills, and the archive's own text elsewhere.
   */
  const carried: readonly SliceReplacement[] = ((withdrawn !== undefined) && isInsertionChunk(withdrawn.target,))
    ? [{
      sliceIndex,
      replacementText: withdrawn.source
        .text,
    },]
    : [];
  /**
   The page as it will be written with that one replacement withdrawn.
   */
  const assembledText = spliceSlices({
    targetText,
    slices,
    replacements: [
      ...standing.filter(function other(replacement,): boolean {
        return replacement.sliceIndex !== sliceIndex;
      },),
      ...carried,
    ],
  },);
  /**
   Defects that page introduces over the inherited document.
   */
  const introduced = introducedFootnoteFindings({
    incumbentText: targetText,
    assembledText,
  },);
  return !introduced.some(function sameFinding(other,): boolean {
    return (other.kind === finding.kind)
      && (other.convention === finding.convention)
      && (other.identifier === finding.identifier);
  },);
}

/**
 Replacements whose slice lost the reference to a footnote its archive text held.

 @param finding - footnote defect the assembly introduced

 @param replacements - replacements standing at the start of the round

 @param incumbentBySlice - archive text of every slice, by chunk index

 @returns Chunk indices whose reference count to the identifier fell

 @example
 ```ts
 const lost = lostReferenceSlices({ finding, replacements, incumbentBySlice, },);
 ```
 */
function lostReferenceSlices(
  {
    finding,
    replacements,
    incumbentBySlice,
  }: {
    readonly finding: FootnoteGraphFinding;
    readonly replacements: readonly SliceReplacement[];
    readonly incumbentBySlice: ReadonlyMap<string, string>;
  },
): readonly number[] {
  /**
   Key under which the identifier's references are counted.
   */
  const referenceKey = `reference ${finding.convention} ${finding.identifier}`;
  return replacements
    .filter(function lostIt(replacement,): boolean {
      /**
       Archive's own text for this slice, which every standing replacement has.
       */
      const incumbentText = nonNullishOrThrow(
        incumbentBySlice.get(String(replacement.sliceIndex,),),
      );
      /**
       Mentions counted in the archive's text.
       */
      const beforeCounts = footnoteIdentifiers({ text: incumbentText, },);
      /**
       Mentions counted in the replacement.
       */
      const afterCounts = footnoteIdentifiers({ text: replacement.replacementText, },);
      return (beforeCounts.get(referenceKey,) ?? 0) > (afterCounts.get(referenceKey,) ?? 0);
    },)
    .map(function toIndex(replacement,): number {
      return replacement.sliceIndex;
    },);
}

/**
 One footnote finding beside what the guard may withdraw for it.

 @example
 ```ts
 const attribution: Attribution = { finding, suspected: 1, proven: [], };
 ```
 */
export type Attribution = {
  /**
   The defect the assembly introduced.
   */
  readonly finding: FootnoteGraphFinding;

  /**
   How many slices were suspected, by the count of mentions that moved. Zero means nobody can be blamed.
   */
  readonly suspected: number;

  /**
   The slices whose withdrawal provably clears the finding: the ones that lost the reference where any proves out,
   else the suspects that do.
   */
  readonly proven: readonly number[];
};

/**
 Reads which of a finding's suspects may be withdrawn.

 @param finding - footnote defect the assembly introduced

 @param standing - replacements standing at the start of the round

 @param targetText - inherited document the replacements are spliced over

 @param slices - prepared locations for every replacement

 @param incumbentBySlice - archive text of every slice, by chunk index

 @param sourceBySlice - original text of every slice, by chunk index

 @returns Finding beside how many were suspected and which of them provably clear it

 @example
 ```ts
 const attribution = attributeFinding({ finding, standing, targetText, slices, incumbentBySlice, sourceBySlice, },);
 ```
 */
export function attributeFinding(
  {
    finding,
    standing,
    targetText,
    slices,
    incumbentBySlice,
    sourceBySlice,
  }: {
    readonly finding: FootnoteGraphFinding;
    readonly standing: readonly SliceReplacement[];
    readonly targetText: string;
    readonly slices: readonly ChunkPair[];
    readonly incumbentBySlice: ReadonlyMap<string, string>;
    readonly sourceBySlice: ReadonlyMap<string, string>;
  },
): Attribution {
  /**
   Slices whose mention count of the identifier moved, the original's owner of a doubled definition left out.
   */
  const suspects = withoutOwners({
    finding,
    suspects: suspectsFor({
      identifierKey: `${finding.convention} ${finding.identifier}`,
      replacements: standing,
      incumbentBySlice,
    },),
    sourceBySlice,
  },);
  /**
   Whether withdrawing one slice clears this finding.

   @param sliceIndex - slice to withdraw

   @returns Whether the page then no longer introduces the finding
   */
  function clears(sliceIndex: number,): boolean {
    return withdrawalClears({
      finding,
      sliceIndex,
      standing,
      targetText,
      slices,
    },);
  }
  /**
   Slices that lost the reference, which only an orphan definition has a lost reference to blame.
   */
  const lost = (finding.kind === 'orphan-definition')
    ? lostReferenceSlices({
      finding,
      replacements: standing,
      incumbentBySlice,
    },)
    : [];
  /**
   The ones that lost it whose withdrawal proves out.
   */
  const provenLost = lost.filter(clears,);
  return {
    finding,
    suspected: suspects.length,
    proven: (provenLost.length > 0) ? provenLost : suspects.filter(clears,),
  };
}

/**
 Whether every finding of a round has suspects, so that where none is proven the orphan is reported and nothing is
 withdrawn; a finding nobody can be blamed for keeps the guard's blanket withdrawal.

 @param attributions - each finding beside what may be withdrawn for it

 @returns Whether there is a finding and each of them has suspects

 @example
 ```ts
 const reportOnly = everyFindingSuspected({ attributions, },);
 ```
 */
export function everyFindingSuspected(
  { attributions, }: { readonly attributions: readonly Attribution[]; },
): boolean {
  return (attributions.length > 0)
    && attributions.every(function hasSuspects(attribution,): boolean {
      return attribution.suspected > 0;
    },);
}

/**
 The guard's lines for a round's footnote findings: reverted where a withdrawal clears it, reported where none does.

 @param attributions - each finding beside what may be withdrawn for it

 @param round - zero-based round, printed from one

 @returns One line per finding, in scorecard-stable wording

 @example
 ```ts
 findings.push(...footnoteRoundFindings({ attributions, round, },),);
 ```
 */
export function footnoteRoundFindings(
  {
    attributions,
    round,
  }: {
    readonly attributions: readonly Attribution[];
    readonly round: number;
  },
): readonly string[] {
  return attributions.map(function lineOf(attribution,): string {
    /**
     The defect, and the slices whose withdrawal clears it.
     */
    const {
      finding,
      proven,
    } = attribution;
    /**
     The finding as every line names it, with the round it was seen in.
     */
    const named = `${finding.kind} ${finding.convention} ${finding.identifier} (round ${String(round + 1,)})`;
    return (proven.length > 0)
      ? `assembly-footnote-reverted ${named}`
      : `assembly-footnote-reported ${named}: no withdrawal clears it on the page as it will be written`;
  },);
}

//endregion Withdrawal proof
