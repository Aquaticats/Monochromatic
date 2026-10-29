import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { SliceReplacement, } from '../splice-slices.ts';
import {
  colonAt,
  isHeadingLine,
  pageTextBySlice,
  pageTextOf,
  slicesInOrder,
  splitHeading,
  withRewrittenText,
} from './assembly-page-text.ts';
import {
  NO_NUMBER,
  type OrdinalStyle,
  readHanNumeral,
  readOrdinalStyle,
  renderOrdinal,
  styleKey,
} from './ordinal-style.ts';

//region Heading series unify
// CLASS SIXTY-EIGHT (XingZ616, 2026-09-19). The original's ten section
// headings `其一：伊良子` to `其十：锦心` reached the settled page in seven
// ordinal styles: "One:", none, "Three:", "IV:", "The Fifth:", "Part Six:",
// none, none, "IX:", "Ten:". Every slice floor passed its own heading and
// no judge sees two of them at once, so a parallel series has no one style.
// THE PAGE DECIDES ONCE, HERE, where every heading is in view, and renders
// the series in that one style.
//
// CLASS EIGHTY-NINE (XingZ624, 2026-09-23) took the style from the lanes'
// plurality, which moved run to run: XingZ622 shipped "Part One:" to "Part
// Ten:" on a plurality of "part cardinal" renderings and XingZ624 dropped
// every ordinal on a plurality of three bare headings, both over the same
// archive, which heads the sections by name alone. Its remedy was the
// archive's style wherever the archive headed two or more of the series,
// bare or numbered. That was the pipeline's own decision, not the owner's.
//
// CLASS EIGHTY-NINE'S RULE IS REPLACED (XingZ6015, 2026-10-20) because the
// bare archive style is not a numbering style. Both lanes rendered the
// numbers ("Part One:" to "Part Ten:"), the lane contest kept them, the
// repair panel logged the archive's dropped number as an accuracy omission,
// and this pass then wrote nine bare headings over them: the shipped page
// lost every number the original gives. The original numbers the series, so
// THE PAGE KEEPS A NUMBER ON EVERY HEADING OF IT, in one style that depends
// on the original and the archive alone, never on what the lanes wrote:
// the archive's style where the archive itself numbers the series in one
// style, else the house style (`HOUSE_SERIES_STYLE`, "Part One:"). The same
// original and archive give the same style every run, which was the point
// of class eighty-nine, and no heading loses a number the original gives or
// gains one it lacks.
//
// A SLICE IS PAIRED WITH THE ORIGINAL'S NUMBERS ONLY WHERE IT CARRIES AS MANY
// HEADINGS (ledger B84). Pairing is by position inside a slice, so a page
// slice that dropped a heading, or an archive slice that heads a section of
// its own, would hand every later heading the wrong number or the wrong
// style; such a slice is left as it stands and read for nothing.

/**
 Mark the original's numbered headings open with.
 */
const SERIES_MARK = '其';

/**
 Style a numbered series takes where the archive does not number it: "Part
 One:". Chosen from what `renderOrdinal` writes and the stored runs, since no
 human archive in the corpus numbers a heading series: XingZ6015's lanes wrote
 "Part N:" on 14 of the 18 numbered headings they rendered (the other four
 were "Three:", "Six:", "The Ninth:" and "Nine:"), and XingZ622 shipped
 "Part One:" to "Part Ten:". "Part" names the sections without calling them
 chapters.
 */
export const HOUSE_SERIES_STYLE: OrdinalStyle = {
  leader: 'part',
  form: 'cardinal',
};

/**
 One heading of the series as the page carries it.
 */
type SeriesHeading = {
  /**
   Slice whose text carries the heading.
   */
  readonly sliceIndex: number;

  /**
   Line of that slice's page text the heading sits on.
   */
  readonly line: number;

  /**
   Number the original gives the heading.
   */
  readonly value: number;

  /**
   Heading marks.
   */
  readonly marks: string;

  /**
   Title after the number, as rendered.
   */
  readonly rest: string;
};

/**
 Numbers of the original's numbered headings in one slice, in heading
 order, zero for a heading that is not numbered.

 @param text - original text of the slice

 @returns One value per heading line

 @example
 ```ts
 seriesValues({ text: '### 其三：猫\n\n猫。', },); // [3]
 ```
 */
function seriesValues({ text, }: { readonly text: string; },): readonly number[] {
  return text.split('\n',)
    .filter(function heading(line,): boolean {
      return isHeadingLine({ line, },);
    },)
    .map(function valueOf(line,): number {
      /**
       Title of the heading.
       */
      const { title, } = splitHeading({ line, },);
      if (!title.startsWith(SERIES_MARK,))
        return 0;
      /**
       Where the number ends.
       */
      const colon = colonAt({ title, },);
      if (colon === (-1))
        return 0;
      return readHanNumeral({ text: title.slice(
        SERIES_MARK.length,
        colon,
      ), },);
    },);
}

/**
 Reads one rendered heading of the series.

 @param line - rendered heading line

 @returns Marks, the title after any number, and the style the number took

 @example
 ```ts
 readRendered({ line: '### Part Six: Mikä', },);
 ```
 */
function readRendered(
  { line, }: { readonly line: string; },
): Pick<SeriesHeading, 'marks' | 'rest'> & { readonly style: OrdinalStyle; } {
  /**
   Marks and title.
   */
  const {
    marks,
    title,
  } = splitHeading({ line, },);
  /**
   Where a number would end.
   */
  const colon = colonAt({ title, },);
  if (colon === (-1))
    return {
      marks,
      rest: title,
      style: NO_NUMBER,
    };
  /**
   Text before the colon.
   */
  const prefix = title.slice(
    0,
    colon,
  )
    .trim();
  /**
   Style of the text before the colon.
   */
  const style = readOrdinalStyle({ prefix, },);
  if (style.form === 'none')
    return {
      marks,
      rest: title,
      style,
    };
  /**
   Title after the colon.
   */
  const rest = title.slice(colon + 1,)
    .trim();
  return {
    marks,
    rest,
    style,
  };
}

/**
 What the archive says about the series' style: one numbered style, or
 nothing to follow.
 */
type ArchiveSeriesStyle =
  | {
    readonly rendered: true;

    /**
     Numbered style every archive heading of the series shares.
     */
    readonly style: OrdinalStyle;
  }
    | { readonly rendered: false; };

/**
 The archive heads fewer than two of the series, heads them in more than one
 style, or heads them by name alone.
 */
const ARCHIVE_SILENT: ArchiveSeriesStyle = { rendered: false, };

/**
 Style the archive numbers the series in: the one numbered style shared by
 every archive heading standing where the original numbers one; silent where
 the archive heads fewer than two of them, heads them in more than one style,
 or heads them by name alone, which numbers nothing.

 @param slices - prepared pairs in slice order

 @returns Archive's style, or that there is none to read

 @example
 ```ts
 const archived = archiveStyle({ slices, },);
 ```
 */
function archiveStyle({ slices, }: { readonly slices: readonly ChunkPair[]; },): ArchiveSeriesStyle {
  /**
   Style of every archive heading at a numbered position.
   */
  const styles: OrdinalStyle[] = [];
  for (const slice of slices) {
    /**
     Numbers of the original's headings here, in heading order.
     */
    const values = seriesValues({ text: slice.source
      .text, },);
    if (values.every(function unnumbered(value,): boolean {
      return value === 0;
    },))
      continue;
    /**
     Archive heading lines of this slice, in order.
     */
    const lines = slice.target
      .text
      .split('\n',)
      .filter(function heading(line,): boolean {
        return isHeadingLine({ line, },);
      },);
    // AN ARCHIVE SLICE IS READ ONLY WHERE ITS HEADINGS PAIR WITH THE
    // ORIGINAL'S (ledger B84). One that heads a section of its own, or merges
    // two, puts every later heading of the slice at another position, and its
    // own heading once read as a numbered one hid the archive's style.
    if (lines.length !== values.length)
      continue;
    values.forEach(function readArchive(
      value,
      at,
    ): void {
      if (value === 0)
        return;
      /**
       Archive heading at this position, which exists because the slice
       carries as many headings as the original's.
       */
      const line = nonNullishOrThrow(lines[at],);
      /**
       Archive heading as rendered.
       */
      const { style, } = readRendered({ line, },);
      styles.push(style,);
    },);
  }
  /**
   Earliest archive style, if the archive heads the series at all.
   */
  const [first,] = styles;
  if ((styles.length < 2) || (first === undefined))
    return ARCHIVE_SILENT;
  /**
   Distinct style keys the archive used.
   */
  const keys = new Set(styles.map(function keyOf(style,): string {
    return styleKey({ style, },);
  },),);
  return ((keys.size === 1) && (first.form !== 'none'))
    ? {
      rendered: true,
      style: first,
    }
    : ARCHIVE_SILENT;
}

/**
 Style the original's numbered series is rendered in: the archive's where the
 archive numbers the series, else the house style.

 @param archived - what the archive says about the series' style

 @returns Style, whose form is never `none`

 @example
 ```ts
 const style = seriesStyle({ archived: archiveStyle({ slices, },), },);
 ```
 */
function seriesStyle({ archived, }: { readonly archived: ArchiveSeriesStyle; },): OrdinalStyle {
  return archived.rendered ? archived.style : HOUSE_SERIES_STYLE;
}

/**
 Renders every numbered heading of the original's series in one style.

 @param slices - prepared pairs, whose original names the series

 @param replacements - what the page would write per slice

 @returns Replacements with the series unified, the rewritten rows alone,
 and one finding per rewritten heading

 @example
 ```ts
 const unified = unifyHeadingSeries({ slices, replacements, },);
 ```
 */
export function unifyHeadingSeries(
  {
    slices,
    replacements,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly replacements: readonly SliceReplacement[];
  },
): {
  readonly replacements: readonly SliceReplacement[];
  readonly restored: readonly SliceReplacement[];
  readonly findings: readonly string[];
} {
  /**
   Page text per slice.
   */
  const pageText = pageTextBySlice({
    slices,
    replacements,
  },);
  /**
   Every numbered heading as the page renders it.
   */
  const headings: SeriesHeading[] = [];
  for (const slice of slicesInOrder({ slices, },)) {
    /**
     Index of this slice.
     */
    const { sliceIndex, } = slice.target;
    /**
     Numbers of the original's headings here, in heading order.
     */
    const values = seriesValues({ text: slice.source
      .text, },);
    if (values.every(function unnumbered(value,): boolean {
      return value === 0;
    },))
      continue;
    /**
     Page lines of this slice, which the page text holds for every slice.
     */
    const lines = pageTextOf({
      pageText,
      sliceIndex,
    },)
      .split('\n',);
    // A PAGE SLICE IS READ ONLY WHERE ITS HEADINGS PAIR WITH THE ORIGINAL'S
    // (ledger B84). A slice that dropped or gained a heading puts every later
    // heading at another position, and pairing them once gave the third
    // section the second's number. Such a slice keeps its headings as the page
    // has them.
    /**
     Heading lines the page carries in this slice.
     */
    const pageHeadings = lines.filter(function heading(line,): boolean {
      return isHeadingLine({ line, },);
    },);
    if (pageHeadings.length !== values.length)
      continue;
    /**
     Headings met so far in the page text.
     */
    let met = 0;
    lines.forEach(function readLine(
      line,
      at,
    ): void {
      if (!isHeadingLine({ line, },))
        return;
      /**
       Number of the original's heading at this position, zero if none,
       which exists because the slice carries as many headings as the
       original's.
       */
      const value = nonNullishOrThrow(values[met],);
      met += 1;
      if (value === 0)
        return;
      /**
       Marks and title of the heading, whose rendered number the series
       style replaces.
       */
      const {
        marks,
        rest,
      } = readRendered({ line, },);
      headings.push({
        sliceIndex,
        line: at,
        value,
        marks,
        rest,
      },);
    },);
  }
  if (headings.length < 2)
    return {
      replacements,
      restored: [],
      findings: [],
    };
  /**
   Style the archive renders the series in, if it renders it.
   */
  const archived = archiveStyle({ slices: slicesInOrder({ slices, },), },);
  /**
   Style the series takes: the archive's where it numbers the series, else
   the house style.
   */
  const style = seriesStyle({ archived, },);
  /**
   Whose style it is, for the finding.
   */
  const whose = archived.rendered ? 'the archive\'s own style' : 'the house style';
  /**
   Page text per slice after the rewrites.
   */
  const rewritten = new Map<number, string>();
  /**
   One finding per rewritten heading.
   */
  const findings: string[] = [];
  for (const heading of headings) {
    /**
     Heading as the series style spells it.
     */
    const unified = `${heading.marks} ${renderOrdinal({
      value: heading.value,
      style,
    },)}: ${heading.rest}`;
    /**
     Slice lines as they stand after earlier rewrites.
     */
    const current = rewritten.get(heading.sliceIndex,)
      ?? pageTextOf({
        pageText,
        sliceIndex: heading.sliceIndex,
      },);
    /**
     Those lines.
     */
    const lines = current.split('\n',);
    /**
     Heading line as the page had it, which exists because a rewrite replaces
     a line and never adds or removes one.
     */
    const before = nonNullishOrThrow(lines[heading.line],);
    if (before === unified)
      continue;
    lines[heading.line] = unified;
    rewritten.set(
      heading.sliceIndex,
      lines.join('\n',),
    );
    findings.push(
      `heading-series-unified (slice ${String(heading.sliceIndex,)}: "${before}" to "${unified}"; `
        + `${String(headings.length,)} numbered headings in the style of "${styleKey({ style, },)}", ${whose})`,
    );
  }
  /**
   Replacements with the rewritten slices folded in.
   */
  const folded = withRewrittenText({
    replacements,
    rewritten,
  },);
  return {
    ...folded,
    findings,
  };
}

//endregion Heading series unify
