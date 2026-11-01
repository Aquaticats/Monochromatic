import type { ArchiveOriginalSpan, } from '@monochromatic-dev/translation-repair-corpus-source/ts';
import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  spliceDisjointEdits,
  type SpliceEdit,
} from '@monochromatic-dev/translation-repair-markup/ts';
import type { SliceReplacement, } from '@monochromatic-dev/translation-repair-page-pass/ts';
import { archiveItalicSpans, } from './archive-italic-spans.ts';
import {
  emphasisSpans,
  oneLine,
} from './emphasis-spans.ts';
import {
  rewriteEverySlice,
  type TextRewrite,
} from './page-slice-rewrite.ts';
import {
  inProse,
  protectedRanges,
} from '@monochromatic-dev/translation-repair-glossary/ts';

//region Archive italic title restore
// CLASS ONE HUNDRED SEVENTY-THREE (TianqiChen6668, 2026-09-26): the archive
// names an anime as *Anohana: The Flower We Saw That Day*, yet the page wrote
// the same words as “Anohana:\nThe Flower We Saw That Day”, in quotation
// marks and broken after the colon, so the page named the work in a form
// the archive never used. This page-assembly pass reads the words the
// archive sets in italics and, where a page slice quotes the same words in
// prose (whitespace runs, line breaks included, read as one space), writes
// the archive's italic span back. A period or comma the quotes held after
// the words moves outside the italics. Quotation marks around any other
// words are left alone, since those may be speech, and so are those inside
// an italic span (ledger B72): italics inside italics show the title no
// different from the words around it, and the quotes were what set it apart.

/**
 One pair of quotation marks a title may be written in.
 */
type QuotePair = {
  readonly open: string;
  readonly close: string;
};

/**
 Pairs read: curly doubles, and straight doubles for a page the quote-style
 pass left straight.
 */
const QUOTE_PAIRS: readonly QuotePair[] = [
  {
    open: '“',
    close: '”',
  },
  {
    open: '"',
    close: '"',
  },
];

/**
 Marks a title's quotes may hold after its words, moved outside the italics.
 */
const TRAILING_MARKS: ReadonlySet<string> = new Set([
  '.',
  ',',
],);

/**
 One quoted span of a text, with its words.
 */
type QuoteSpan = {
  readonly start: number;
  readonly end: number;
  readonly inner: string;
};

/**
 A quoted span with the italic form it takes, empty where it names no title.
 */
type TitleSpan = QuoteSpan & { readonly form: string; };

/**
 Every span of a text one pair of quotation marks encloses, in order.

 @param text - text under scan

 @param pair - quotation marks to read

 @returns Quoted spans, the marks included

 @example
 ```ts
 quoteSpans({ text: 'a “b” c', pair: QUOTE_PAIRS[0], },); // [{ start: 2, end: 5, inner: 'b' }]
 ```
 */
function quoteSpans(
  {
    text,
    pair,
  }: {
    readonly text: string;
    readonly pair: QuotePair;
  },
): readonly QuoteSpan[] {
  /**
   Spans found so far.
   */
  const spans: QuoteSpan[] = [];
  for (let open = text.indexOf(pair.open,); open !== (-1);) {
    /**
     Where this span's closing mark stands.
     */
    const close = text.indexOf(
      pair.close,
      open + 1,
    );
    if (close === (-1))
      break;
    spans.push({
      start: open,
      end: close + 1,
      inner: text.slice(
        open + 1,
        close,
      ),
    },);
    open = text.indexOf(
      pair.open,
      close + 1,
    );
  }
  return spans;
}

/**
 The italic form one quoted span takes, where its words are an archive title.

 @param inner - quoted words

 @param titles - archive italic spans

 @returns The span's italic form, or the empty string where it names no title

 @example
 ```ts
 italicForm({ inner: 'Long Nap.', titles: new Set(['Long Nap',],), },); // '*Long Nap*.'
 ```
 */
function italicForm(
  {
    inner,
    titles,
  }: {
    readonly inner: string;
    readonly titles: ReadonlySet<string>;
  },
): string {
  /**
   Quoted words on one line.
   */
  const words = oneLine({ text: inner, },);
  if (titles.has(words,))
    return `*${words}*`;
  /**
   Mark the quotes hold after the words.
   */
  const last = words.slice(-1,);
  /**
   Words before that mark.
   */
  const head = words.slice(
    0,
    -1,
  );
  return TRAILING_MARKS.has(last,) && titles.has(head,) ? `*${head}*${last}` : '';
}

/**
 Writes each quoted archive title in one slice's text in italics.

 @param text - one slice's text as the page carries it

 @param titles - archive italic spans

 @returns Text with those spans in italics and a note per span written

 @example
 ```ts
 italicizeSlice({ text: 'She loved “Long Nap”.', titles: new Set(['Long Nap',],), },).text; // 'She loved *Long Nap*.'
 ```
 */
function italicizeSlice(
  {
    text,
    titles,
  }: {
    readonly text: string;
    readonly titles: ReadonlySet<string>;
  },
): TextRewrite {
  /**
   Non-prose ranges of the text.
   */
  const ranges = protectedRanges({ text, },);
  /**
   Quoted titles in prose.
   */
  const quotedTitles = QUOTE_PAIRS
    .flatMap(function spansOf(pair,): readonly QuoteSpan[] {
      return quoteSpans({
        text,
        pair,
      },);
    },)
    .map(function withForm(span,): TitleSpan {
      return {
        ...span,
        form: italicForm({
          inner: span.inner,
          titles,
        },),
      };
    },)
    .filter(function isTitleInProse(span,): boolean {
      return (span.form !== '') && inProse({
        ranges,
        start: span.start,
        end: span.end,
      },);
    },);
  // MOST SLICES QUOTE NO TITLE, and those are left before their text is parsed.
  if (quotedTitles.length === 0) {
    return {
      text,
      changed: [],
    };
  }
  /**
   Spans the text already sets in italics.
   */
  const italics = emphasisSpans({ text, },);
  /**
   Quoted titles clear of every italic span, in document order: one inside
   italics stays quoted, since italics there would show it no different from
   the words around it (ledger B72).
   */
  const restorable = quotedTitles
    .filter(function outsideItalics(span,): boolean {
      return !italics.some(function overlaps(italic,): boolean {
        return (italic.start < span.end) && (span.start < italic.end);
      },);
    },)
    .toSorted(function byStart(
      left,
      right,
    ): number {
      return left.start - right.start;
    },);
  // A CURLY AND A STRAIGHT PAIR CAN NEST, so a title quoted inside a quoted
  // title yields two spans sharing text. The one that opens first is written
  // and the other goes: its form was read from quotes the first rewrites.
  // Written one at a time, last first, the inner went in first and the outer
  // then cut at its own end, one unit off wherever the inner had shrunk,
  // which cost a page the mark after the title (ledger B70).
  /**
   Spans written, each clear of the one before it.
   */
  const rewrites: TitleSpan[] = [];
  for (const span of restorable) {
    /**
     Span written before this one, absent for the first.
     */
    const earlier = rewrites.at(-1,);
    if ((earlier !== undefined) && (span.start < earlier.end))
      continue;
    rewrites.push(span,);
  }
  return {
    text: spliceDisjointEdits({
      text,
      edits: rewrites.map(function toEdit(span,): SpliceEdit {
        return {
          start: span.start,
          end: span.end,
          text: span.form,
        };
      },),
    },),
    changed: rewrites.map(function note(span,): string {
      return `${oneLine({ text: span.inner, },)} to ${span.form}`;
    },),
  };
}

/**
 Writes the page's quoted archive titles in the archive's italics, the slices
 no lane replaced included.

 @param slices - prepared pairs, whose archive text stands where no row replaces it

 @param replacements - what the page would write per slice

 @param archiveOriginalSpans - spans sealed as the English original

 @returns Replacements with the titles in italics (a row added for an archive
 slice the pass changed), the rewritten rows alone, and one finding per
 slice changed

 @example
 ```ts
 const page = restoreArchiveItalicTitles({ slices, replacements, archiveOriginalSpans: [], },);
 ```
 */
export function restoreArchiveItalicTitles(
  {
    slices,
    replacements,
    archiveOriginalSpans,
  }: {
    readonly slices: readonly ChunkPair[];
    readonly replacements: readonly SliceReplacement[];
    readonly archiveOriginalSpans: readonly ArchiveOriginalSpan[];
  },
): {
  readonly replacements: readonly SliceReplacement[];
  readonly restored: readonly SliceReplacement[];
  readonly findings: readonly string[];
} {
  /**
   Words the archive sets in italics.
   */
  const titles = archiveItalicSpans({ slices, },);
  if (titles.size === 0) {
    return {
      replacements,
      restored: [],
      findings: [],
    };
  }
  return rewriteEverySlice({
    slices,
    replacements,
    archiveOriginalSpans,
    rewrite: function rewrite({ text, },): TextRewrite {
      return italicizeSlice({
        text,
        titles,
      },);
    },
    findingName: 'archive-italic-title-restored',
  },);
}

//endregion Archive italic title restore
