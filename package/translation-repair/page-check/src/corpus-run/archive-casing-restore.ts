import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import {
  rowsChangedBy,
  type SliceReplacement,
  slicesInOrder,
} from '@monochromatic-dev/translation-repair-page-pass/ts';

import {
  applySpanRewrites,
  type SpanRewrite,
} from './span-rewrites.ts';
import { latinWordSpans, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Archive casing restore
// THE ONE HUNDRED AND TWENTY-FIRST CLASS (mikaela17, 2026-09-25). The archive
// writes a romanised station name in capitals, "XIAWAFANG Station", in the
// body and again in the footnote that explains it, and its translator's note
// says why: the city writes its station names that way. A judge chose a
// candidate writing it "Xiawafang" as "standard capitalization ... instead of
// all-caps", so the page carried the name one way in the body and the
// archive's way in the footnote, which no slice sheet saw together. A word the
// archive writes in capitals more than once and never any other way in prose
// is the page's own form; a title-case spelling of it in a shipped slice's
// prose is restored here, where the page is in view. Headings are read past
// on both sides: the archive's own "## From Xiawafang to Tianjin Eye" writes
// the name title case by the heading's style, not as a second spelling.

/**
 Fewest letters a word needs before its capitals read as a name, not an
 emphasis or an acronym the page may also spell out.
 */
const MIN_LETTERS = 4;

/**
 Fewest times the archive writes the word in capitals, so one shouted word
 is not read as the page's spelling.
 */
const MIN_USES = 2;

/**
 Characters that put a word inside a link destination, a path or an
 attribute rather than in prose.
 */
const NON_PROSE_BEFORE: ReadonlySet<string> = new Set([
  '/',
  '.',
  '#',
  '=',
  '_',
  '-',
],);

/**
 Mark a Markdown heading line opens with; a heading writes its words title
 case by style, so its casing says nothing about the page's form of a name.
 */
const HEADING_MARK = '#';

/**
 Whether the line holding one position is a Markdown heading.

 @param text - text under scan

 @param at - position on the line

 @returns Whether the line opens with a heading mark

 @example
 ```ts
 onHeadingLine({ text: '## From Maowu', at: 8, },); // true
 ```
 */
export function onHeadingLine(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): boolean {
  /**
   Where the line holding the position starts.
   */
  const lineStart = text.lastIndexOf(
    '\n',
    at,
  ) + 1;
  return text.slice(lineStart,)
    .trimStart()
    .startsWith(HEADING_MARK,);
}

/**
 Words the archive writes in capitals at least twice and in no other casing
 anywhere.

 @param archiveText - every archive slice joined

 @returns Capital forms, each a word of at least four letters

 @example
 ```ts
 capitalForms({ archiveText: 'MAOWU here, MAOWU there.', },); // Set { 'MAOWU' }
 ```
 */
function capitalForms(
  { archiveText, }: { readonly archiveText: string; },
): ReadonlySet<string> {
  /**
   Every spelling the archive uses for a word, keyed by the word in capitals,
   with how often each spelling occurs.
   */
  const spellings = latinWordSpans({ text: archiveText, },)
    .filter(function inProse({ start, },): boolean {
      return !onHeadingLine({
        text: archiveText,
        at: start,
      },);
    },)
    .reduce(
    function tally(
      byWord,
      { word, },
    ): Map<string, Map<string, number>> {
        /**
         The word in capitals.
         */
      const upper = word.toUpperCase();
        /**
         Spellings seen for it so far.
         */
      const seen = byWord.get(upper,) ?? new Map<string, number>();
      seen.set(
        word,
        (seen.get(word,) ?? 0) + 1,
      );
      byWord.set(
        upper,
        seen,
      );
      return byWord;
    },
    new Map<string, Map<string, number>>(),
    );
  return new Set([...spellings,]
    .filter(function onlyCapitals([
      upper,
      seen,
    ],): boolean {
      return (upper.length >= MIN_LETTERS)
        && (seen.size === 1)
        && ((seen.get(upper,) ?? 0) >= MIN_USES);
    },)
    .map(function toUpper([upper,],): string {
      return upper;
    },),);
}

/**
 Whether a word is written title case: one capital, then lower case only.

 @param word - run of Latin letters

 @returns Whether it is title case

 @example
 ```ts
 isTitleCase({ word: 'Maowu', },); // true
 ```
 */
export function isTitleCase(
  { word, }: { readonly word: string; },
): boolean {
  /**
   First letter.
   */
  const head = word.charAt(0,);
  /**
   The letters after it.
   */
  const tail = word.slice(1,);
  /**
   Whether the first letter is a capital.
   */
  const capitalHead = head === head.toUpperCase();
  /**
   Whether every later letter is lower case.
   */
  const lowerTail = (tail.length > 0) && (tail === tail.toLowerCase());
  return capitalHead && lowerTail;
}

/**
 Rewrites every title-case spelling of an archive capital form in one text.

 @param text - shipped text

 @param forms - the archive's capital forms

 @returns Rewritten text and each spelling it changed

 @example
 ```ts
 recase({ text: 'at Maowu', forms: new Set(['MAOWU']), },);
 ```
 */
function recase(
  {
    text,
    forms,
  }: {
    readonly text: string;
    readonly forms: ReadonlySet<string>;
  },
): {
  readonly text: string;
  readonly changed: readonly string[];
} {
  /**
   Words to rewrite, in order.
   */
  const targets = latinWordSpans({ text, },)
    .filter(function restorable({
      word,
      start,
    },): boolean {
      return forms.has(word.toUpperCase(),)
        && isTitleCase({ word, },)
        && (!NON_PROSE_BEFORE.has(text.charAt(start - 1,),))
        && (!onHeadingLine({
          text,
          at: start,
        },));
    },);
  /**
   Text with the words upper-cased, and the words that were.
   */
  const rebuilt = applySpanRewrites({
    text,
    rewrites: targets.map(function toRewrite({
      word,
      start,
    },): SpanRewrite & { readonly word: string; } {
      return {
        start,
        end: start + word.length,
        to: word.toUpperCase(),
        word,
      };
    },),
  },);
  return {
    text: rebuilt.text,
    changed: rebuilt.applied
      .map(function spelling({ word, },): string {
        return word;
      },),
  };
}

/**
 Restores the archive's all-capitals form of a word wherever a shipped slice
 writes it title case.

 @param slices - prepared pairs, whose archive text carries the forms

 @param replacements - what the page would write per slice

 @returns Replacements with the forms restored, the rewritten rows alone,
 and one finding per distinct spelling changed in a slice

 @example
 ```ts
 const restored = restoreArchiveCasing({ slices, replacements, },);
 ```
 */
export function restoreArchiveCasing(
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
   The archive's capital forms across the whole page.
   */
  const forms = capitalForms({
    archiveText: slicesInOrder({ slices, },)
      .map(function archiveOf(slice,): string {
        return slice.target
          .text;
      },)
      .join('\n',),
  },);
  /**
   Findings, one per distinct spelling changed in a slice.
   */
  const findings: string[] = [];
  /**
   Every replacement, the rewritten ones swapped in.
   */
  const rewritten = replacements.map(function restore(row,): SliceReplacement {
    if (forms.size === 0)
      return row;
    /**
     The row's text with the forms restored.
     */
    const recased = recase({
      text: row.replacementText,
      forms,
    },);
    [...new Set(recased.changed,),].forEach(function report(spelling,): void {
      findings.push(
        `archive-casing-restored (slice ${String(row.sliceIndex,)}: "${spelling}" to "${spelling.toUpperCase()}")`,
      );
    },);
    return {
      sliceIndex: row.sliceIndex,
      replacementText: recased.text,
    };
  },);
  /**
   Rows this pass changed.
   */
  const restored = rowsChangedBy({
    before: replacements,
    after: rewritten,
  },);
  return {
    replacements: rewritten,
    restored,
    findings,
  };
}
//endregion Archive casing restore
