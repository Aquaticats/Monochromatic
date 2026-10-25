import { nonNullishOrThrow, } from '@monochromatic-dev/module-or-throw/ts';
import { fragmentBody, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';
import { encodeAsLiteralMdx, } from './source-ships-encoding.ts';
import type {
  WouldShipReading,
  WouldShipSlice,
  WouldShipSource,
} from './would-ship-text.ts';

//region Source ships
// WHAT A PAGE CARRIES WHERE THE ARCHIVE HAS NO TEXT AND NO WORDING WAS BACKED.
// The owner, 2026-10-23, asked what a shipped page carries for such a slice,
// chose "Source text + note": the page carries the original text of the
// passage, preceded by a note saying it has not been translated yet and the
// original follows, footnote definitions included; a later run replaces it
// once a wording is backed. Options offered and not chosen: the note alone;
// the silent gap the pages shipped before (XingZ6020 shipped 88 of 118 slices
// and the other 30, 31.5 percent of the original, left no trace on the page);
// and unbacked wording marked as unreviewed, which would break the standing
// rule that one judge alone never decides.
//
// WHAT THE PAGE CARRIES OF A SLICE IS THE SLICE'S TEXT AS THE ARTIFACT RECORDS IT, with the blank lines and spaces at
// its ends trimmed, and the slices of one run joined by one blank line (the splice composes the separator, whatever
// separated them in the original). Text that sits between slices in the original, an HTML comment among it, belongs to
// no slice and is not carried. The text is written as recorded whenever the page with it parses; otherwise the braces
// and less-than signs that are syntax are escaped (`source-ships-encoding.ts`), run by run and then everywhere, and
// the last resort ships the page without the original (`source-ships-compose.ts`).
//
// A READING AND NOT A RECORD OF ITS OWN: the original of every slice is already
// in the artifact (`lanes.translate.delivery[].sourceText`), and the slice's
// silence is already decided by the stages, so this file only reads the two
// together. The page, the page check, the tally and `verify-published` all
// read the same reading and so agree.

/**
 The note a run of untranslated slices carries before the original text,
 as a Markdown block quote on its own line.

 CANADIAN ENGLISH AND PLAIN TEXT, so it needs no encoding at any boundary it
 crosses: it holds no angle bracket, brace, backtick or link.
 */
export const UNTRANSLATED_PASSAGE_NOTE = '> This passage has not been translated yet. The original text follows.';

/**
 Turns the silence at an anchor into the slice shipping its original text.

 ONLY AN ANCHOR NOTHING FILLED: a content span the deciders emptied keeps its
 silence, since the archive had wording there and they removed it. A source
 that renders as nothing has no passage to carry. A source with no lanes to
 record it (the consistency pass composes the English alone) is read as it
 was before this reading existed.

 @param artifact - source being read

 @param sliceIndex - slice the reading is for

 @param reading - what the stages left at the slice

 @returns Reading unchanged, or the slice shipping its original text

 @example
 ```ts
 const carried = carryingSource({ artifact, sliceIndex: 3, reading, },);
 ```
 */
export function carryingSource(
  {
    artifact,
    sliceIndex,
    reading,
  }: {
    readonly artifact: WouldShipSource;
    readonly sliceIndex: number;
    readonly reading: WouldShipReading;
  },
): WouldShipReading {
  if ((reading.kind !== 'nothing-ships') || (reading.incumbentKind !== 'absent'))
    return reading;
  /**
   The row recording the original of this slice, absent when the source carries no lanes or no row for it.
   */
  const recorded = artifact.lanes
    ?.translate
    .delivery
    .find(function namesIt(row,): boolean {
      return row.sliceIndex === sliceIndex;
    },);
  if ((recorded === undefined) || rendersAsNothing({ text: recorded.sourceText, },))
    return reading;
  return {
    kind: 'source-ships',
    reason: reading.reason,
    incumbentKind: 'absent',
    text: recorded.sourceText,
    // A SLICE READ ALONE OPENS ITS OWN RUN; `markRuns` closes it where a neighbour carries the same note.
    opensRun: true,
  };
}

/**
 The same artifact without its record of the originals of some slices, which reads those slices as silent and the others
 as they were.

 @param artifact - source being read

 @param slices - slice indices whose original the page is shipped without

 @returns Artifact whose named slices nothing fills are silent again; the artifact itself where none is named

 @example
 ```ts
 const weighed = withoutSourceRecordAt({ artifact, slices: [88, 89,], },);
 ```
 */
export function withoutSourceRecordAt<Artifact extends WouldShipSource>(
  {
    artifact,
    slices,
  }: {
    readonly artifact: Artifact;
    readonly slices: readonly number[];
  },
): Artifact {
  /**
   The record of the originals, absent for a source composing the English alone.
   */
  const { lanes, } = artifact;
  if ((lanes === undefined) || (slices.length === 0))
    return artifact;
  /**
   The rows recording the originals.
   */
  const { delivery: rows, } = lanes.translate;
  /**
   The rows of the slices whose original stays on the page.
   */
  const delivery = rows.filter(function stays(row,): boolean {
    return !slices.includes(row.sliceIndex,);
  },);
  return {
    ...artifact,
    lanes: {
      ...lanes,
      translate: {
        ...lanes.translate,
        delivery,
      },
    },
  };
}

/**
 Marks which slices continue a run of slices shipping their original, so the
 note stands once before a contiguous run and not once per slice.

 A RUN IS SLICES NEXT TO EACH OTHER IN THE ORDER THE ROWS GIVE, and any slice
 that ships English or nothing between two of them ends it.

 @param slices - every slice's reading, in comparison-row order

 @returns Same readings, the ones that continue a run no longer opening one

 @example
 ```ts
 const marked = markRuns({ slices, },);
 ```
 */
export function markRuns(
  { slices, }: { readonly slices: readonly WouldShipSlice[]; },
): readonly WouldShipSlice[] {
  return slices.map(function markIt(
    slice,
    at,
  ): WouldShipSlice {
    /**
     What this slice ships.
     */
    const { reading, } = slice;
    if ((at === 0) || (reading.kind !== 'source-ships'))
      return slice;
    /**
     What the slice before it ships.
     */
    const { reading: previous, } = nonNullishOrThrow(slices[at - 1],);
    if (previous.kind !== 'source-ships')
      return slice;
    return {
      ...slice,
      reading: {
        ...reading,
        opensRun: false,
      },
    };
  },);
}

/**
 A reading that puts text on the page, which is every reading but the silent one.
 */
export type TextCarryingReading = Exclude<WouldShipReading, { readonly kind: 'nothing-ships'; }>;

/**
 Whether a reading puts any text on the page at its slice.

 @param reading - what the slice would carry

 @returns Whether the reading carries wording or the original, false for a silence

 @example
 ```ts
 if (carriesText(reading,)) page.push(textOnPage({ reading, },),);
 ```
 */
export function carriesText(reading: WouldShipReading,): reading is TextCarryingReading {
  return reading.kind !== 'nothing-ships';
}

/**
 How a slice's original is written onto the page.

 `as-written` is the text the artifact records, and `literal` the same text with the braces and less-than signs that
 are syntax escaped (`source-ships-encoding.ts`).
 */
export type SourceEncoding = 'as-written' | 'literal';

/**
 Text a reading puts on the page at its slice, the note included where the
 slice opens a run.

 @param reading - what the slice would carry, other than a silence

 @param encoding - how an original is written; wording is never encoded

 @returns Exact text the splice receives

 @example
 ```ts
 const text = textOnPage({ reading, },);
 ```
 */
export function textOnPage(
  {
    reading,
    encoding = 'as-written',
  }: {
    readonly reading: TextCarryingReading;
    readonly encoding?: SourceEncoding;
  },
): string {
  if (reading.kind === 'wording')
    return reading.text;
  /**
   The original without the blank lines around it, which the note's own blank line replaces.
   */
  const body = fragmentBody({
    fragment: (encoding === 'literal')
      ? encodeAsLiteralMdx({ text: reading.text, },)
      : reading.text,
  },);
  return reading.opensRun ? `${UNTRANSLATED_PASSAGE_NOTE}\n\n${body}` : body;
}

//endregion Source ships
