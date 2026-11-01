import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import { textOnPage, } from './source-ships.ts';
import {
  type WouldShipSource,
  wouldShipTextPerSlice,
} from './would-ship-text.ts';

//region Shippable replacements
// WHAT EACH SLICE CONTRIBUTES TO ITS ENTRY'S PAGE, apart from the writer of the page (`publish-fixed.ts`) so that the
// guards, which weigh the page against the archive, and the composition of the original under the grammar
// (`source-ships-compose.ts`) read it without importing the writer.

/**
 Slices whose original is written as literal text, none by default.
 */
const NO_SLICES: ReadonlySet<number> = new Set<number>();

/**
 Turns one settled entry's per-slice readings into the replacements that
 assemble its page.

 A SILENT SLICE CONTRIBUTES EMPTY TEXT AT A CONTENT SPAN AND NOTHING AT ALL AT
 AN ANCHOR, which are two answers rather than one because silence itself means
 two things. Over a span the archive already renders, the empty string is what
 the deciders chose: they agreed the passage carries no wording, and omitting
 the row would republish the archive underneath that decision and undo it. At
 an anchor there is no archive wording to republish and none to remove, so the
 honest contribution is no row at all, leaving `spliceSlices` to pass the gap
 through exactly as it found it.

 THE ANCHOR HALF IS WHAT `XIEPT2` COST FOUR HOURS AND FORTY-EIGHT MINUTES TO
 ESTABLISH. This builder used to hand every silent slice the empty string, and
 `spliceSlices` refuses blank text at an anchor outright, correctly: an anchor
 is where a rendering belongs, so writing nothing there would leave the passage
 missing while the run reported it delivered. That refusal landed in
 `publishFixedPage`, which runs BEFORE the artifact is written, so an entry
 whose translate lane had already recorded slice 12 unfilled three and a half
 hours earlier died at the last step with no artifact and no page kept.
 `translate-absence.ts` promises the opposite and owns the policy: one refused
 anchor costs its own slice rather than the entry. The refusal stays; what
 changes is that a slice deliberately left unfilled no longer presents itself
 to the splice as a blank rendering.

 READ OFF `incumbentKind` RATHER THAN OFF AN EMPTY INCUMBENT, per the same
 file: absence is a mode decided once from the target chunk, and testing the
 text would conflate an anchor with a content span whose archive wording
 genuinely is blank.

 A SLICE THAT SHIPS ITS ORIGINAL CONTRIBUTES THAT TEXT only when the caller asks
 for it, with the note before the first of a run (the owner, 2026-10-23, "Source
 text + note"). The default is the English alone, so a caller that never heard of
 the original cannot put it into the English it rewrites: the guards that weigh the
 page against the archive (the page-assembly passes, the footnote and
 carried-evidence checks) and the consistency pass read it that way, since the
 passes rewrite English and must leave the original alone, and the original's own
 footnote graph is the page check's to report, not a reason to withdraw a slice
 the archive does not hold. Only the writer of the page asks for the original.

 @param artifact - settled entry, read for what each slice would carry

 @param includeSource - whether a slice shipping its original contributes it, true only for the page that publishes

 @param encodedSlices - slices whose original is written as literal text, which `source-ships-compose.ts` fills in
 when the page with the original as written does not parse

 @returns Replacement per slice that contributes one, in comparison order

 @example
 ```ts
 const replacements = shippableReplacements({ artifact, },);
 ```
 */
export function shippableReplacements(
  {
    artifact,
    includeSource = false,
    encodedSlices = NO_SLICES,
  }: {
    readonly artifact: WouldShipSource;
    readonly includeSource?: boolean;
    readonly encodedSlices?: ReadonlySet<number>;
  },
): readonly SliceReplacement[] {
  return wouldShipTextPerSlice({ artifact, },)
    .flatMap(function toReplacement(slice,): readonly SliceReplacement[] {
      /**
       What this slice would carry, or that it carries nothing.
       */
      const { reading, } = slice;

      if (reading.kind === 'wording')
        return [{
          sliceIndex: slice.sliceIndex,
          replacementText: reading.text,
        },];

      if (reading.kind === 'source-ships')
        return includeSource
          ? [{
            sliceIndex: slice.sliceIndex,
            replacementText: textOnPage({
              reading,
              encoding: encodedSlices.has(slice.sliceIndex,) ? 'literal' : 'as-written',
            },),
          },]
          : [];

      // NOTHING SHIPS HERE, and at an anchor nothing is also what the assembler
      // must be told: a row carrying blank text claims a rendering was written
      // where none was, which is the claim the splice exists to refuse.
      if (reading.incumbentKind === 'absent')
        return [];

      return [{
        sliceIndex: slice.sliceIndex,
        replacementText: '',
      },];
    },);
}

//endregion Shippable replacements
