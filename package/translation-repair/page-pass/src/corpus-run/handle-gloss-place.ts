import type { ChunkPair, } from '@monochromatic-dev/translation-repair-slice-reading/ts';
import type { SliceReplacement, } from '../splice-slices.ts';
import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  pageTextBySlice,
  pageTextOf,
  slicesInOrder,
  withRewrittenText,
} from './assembly-page-text.ts';
import { nameAuthorities, } from './contributor-name-authorities.ts';
import { continuesLatinWord, } from '@monochromatic-dev/translation-repair-markup/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';
import type { ReadonlyEntry, } from '@monochromatic-dev/translation-repair-shared-contract/ts';

//region Handle gloss place
// CLASS EIGHTY-EIGHT (XingZ624, 2026-09-23). The house rule (class
// eighty-three) puts a romanised handle's literal meaning in parentheses
// at its first appearance on the page and the romanisation alone after
// that. Every slice is written on its own and no writer sees the page, so
// the Part Ten heading shipped "Jinxin" bare and the signature under it
// "Jinxin (Brocade Heart)", and the first song credit "Yuli" bare with
// "Yuli (Rain Fox)" on the next. THE PAGE DECIDES ONCE, HERE, where every
// appearance in the rows the lanes wrote is in view (the archive's own text is
// read by no one here, and a handle it rendered is not placed): for each
// handle the archive never rendered, the
// first appearance in a replaced slice takes the gloss the bench wrote
// anywhere on the page (the earliest one where they differ) and every later
// appearance drops its gloss. A handle the bench never glossed is left as it
// is, since the pass cannot invent a meaning; the archive's own text is
// never rewritten.

/**
 Gloss opener after a rendering.
 */
const GLOSS_OPEN = ' (';

/**
 Gloss closer.
 */
const GLOSS_CLOSE = ')';

/**
 What a literal meaning never holds: link and image syntax, a nested
 parenthesis, a code span, a tag, a web address (ledger A6).
 */
const GLOSS_BREAKERS: readonly string[] = [
  '[',
  ']',
  '(',
  '`',
  '<',
  '://',
];

/**
 One appearance of a rendering on the page.
 */
type Appearance = {
  /**
   Slice whose text carries it.
   */
  readonly sliceIndex: number;

  /**
   Offset of the rendering's first character in the slice text.
   */
  readonly start: number;

  /**
   Offset just past the rendering and its gloss, if any.
   */
  readonly end: number;

  /**
   Literal meaning written in parentheses after the rendering, empty for
   none.
   */
  readonly gloss: string;
};

/**
 Whether a character continues a word, so a rendering touching it is part
 of a longer word and no appearance.

 @param character - character beside the rendering, empty at a text edge

 @returns True for a Latin letter, a digit, or a combining mark on the
 rendering's last letter. The test once took any cased letter, so a kaomoji's
 Greek or Cyrillic letter beside a handle made it part of a word, while a
 combining accent on the handle's last letter did not (ledger B18).

 @example
 ```ts
 continuesWord({ character: 's', },); // true
 ```
 */
function continuesWord({ character, }: { readonly character: string; },): boolean {
  return continuesLatinWord({ character, },);
}

/**
 Every whole-word appearance of a rendering in one slice's text that the
 pass may write at, with the gloss each carries. An appearance followed by a
 parenthesis that is no gloss (empty or showing a reader nothing, unclosed on
 its line, or holding markup) is left out, so the pass writes nothing there.

 @param sliceIndex - slice whose text is read

 @param text - page text of the slice

 @param rendering - handle as the page renders it

 @returns Appearances in text order, bare or glossed

 @example
 ```ts
 const found = appearancesIn({ sliceIndex: 0, text: '——Jinmao (Brocade Cat), today', rendering: 'Jinmao', },);
 ```
 */
function appearancesIn(
  {
    sliceIndex,
    text,
    rendering,
  }: {
    readonly sliceIndex: number;
    readonly text: string;
    readonly rendering: string;
  },
): readonly Appearance[] {
  /**
   Appearances found so far.
   */
  const found: Appearance[] = [];
  /**
   Offset the scan has reached.
   */
  let from = 0;
  while (from <= text.length) {
    /**
     Where the rendering next stands, -1 for nowhere.
     */
    const start = text.indexOf(
      rendering,
      from,
    );
    if (start === (-1))
      break;
    /**
     Offset just past the rendering.
     */
    const past = start + rendering.length;
    from = past;
    if (continuesWord({ character: text.slice(
      Math.max(
        0,
        start - 1,
      ),
      start,
    ), },))
      continue;
    if (continuesWord({ character: text.slice(
      past,
      past + 1,
    ), },))
      continue;
    if (!text.startsWith(
      GLOSS_OPEN,
      past,
    )) {
      found.push({
        sliceIndex,
        start,
        end: past,
        gloss: '',
      },);
      continue;
    }
    /**
     Where the gloss closes, -1 for an unclosed one.
     */
    const close = text.indexOf(
      GLOSS_CLOSE,
      past + GLOSS_OPEN.length,
    );
    /**
     Whether the parenthetical is a gloss: closed on the same line, with
     something inside a reader sees. What a reader sees is asked of
     `rendersAsNothing` (ledger B40): asked of the parenthesis's length, a
     heading's `( )` or a zero-width space in parentheses was read as the
     handle's gloss, and the well-formed gloss on its signature was stripped.
     */
    const isGloss = (close !== (-1))
      && (!text.slice(
        past,
        close,
      )
        .includes('\n',))
      && (!rendersAsNothing({
        text: text.slice(
          past + GLOSS_OPEN.length,
          close,
        ),
      },));
    // A parenthesis that is no gloss leaves its appearance as the writer
    // left it, neither the first nor a later one: empty or showing a reader
    // nothing, never closed, or closed only by a mark on a later line. Read
    // as an appearance with no
    // gloss, a first one took the handle's gloss in front of the parenthesis
    // it already opened (`### Ten: Jinmao (Brocade Cat) (Brocade Cat`), and
    // the signature under it, whose gloss was well formed, lost its own. The
    // scan goes on from the handle's end, since the mark that closed the
    // parenthesis may belong to a later appearance's gloss.
    if (!isGloss)
      continue;
    // A PARENTHESIS HOLDING MARKUP IS NO GLOSS (ledger A6). The first close
    // parenthesis ends a link's destination, so `Jinmao ([her song](url))`
    // read the link as a gloss, moved it onto the heading and stripped it
    // from its sentence, leaving "Jinmao) woke." That appearance is neither
    // the first nor a later one: it stands as the writer left it.
    if (GLOSS_BREAKERS.some(function breaks(breaker,): boolean {
      return text.slice(
        past + GLOSS_OPEN.length,
        close,
      )
        .includes(breaker,);
    },)) {
      from = close + 1;
      continue;
    }
    found.push({
      sliceIndex,
      start,
      end: close + 1,
      gloss: text.slice(
        past + GLOSS_OPEN.length,
        close,
      ),
    },);
    from = close + 1;
  }
  return found;
}

/**
 Places one handle's gloss at its first appearance among the replaced
 slices and strips it from the later ones.

 @param rendering - handle as the page renders it

 @param appearances - every appearance in page order

 @param texts - page text per replaced slice, rewritten in place

 @returns One finding per rewritten appearance

 @example
 ```ts
 const findings = placeOne({ rendering: 'Jinmao', appearances, texts, },);
 ```
 */
function placeOne(
  {
    rendering,
    appearances,
    texts,
  }: {
    readonly rendering: string;
    readonly appearances: readonly Appearance[];
    readonly texts: Map<number, string>;
  },
): readonly string[] {
  /**
   Earliest appearance the bench glossed, if any.
   */
  const glossed = appearances.find(function carriesGloss(appearance,): boolean {
    return appearance.gloss !== '';
  },);
  if (glossed === undefined)
    return [];
  /**
   Gloss the bench wrote first.
   */
  const { gloss, } = glossed;
  /**
   Findings, one per rewritten appearance.
   */
  const findings: string[] = [];
  // Later appearances are rewritten from the end of the page backwards so
  // the offsets of the earlier ones in the same slice stay valid.
  appearances.toReversed()
    .forEach(function rewrite(
      appearance,
      fromEnd,
    ): void {
      /**
       Whether this is the first appearance on the page.
       */
      const first = fromEnd === (appearances.length - 1);
      /**
       What the appearance should read.
       */
      const wanted = first ? `${rendering}${GLOSS_OPEN}${gloss}${GLOSS_CLOSE}` : rendering;
      /**
       Slice text as it stands.
       */
      const text = nonNullishOrThrow(texts.get(appearance.sliceIndex,),);
      /**
       What the appearance reads now.
       */
      const written = text.slice(
        appearance.start,
        appearance.end,
      );
      if (written === wanted)
        return;
      texts.set(
        appearance.sliceIndex,
        `${text.slice(
          0,
          appearance.start,
        )}${wanted}${text.slice(appearance.end,)}`,
      );
      /**
       Why the appearance was rewritten.
       */
      const why = first
        ? 'the literal meaning at the handle\'s first appearance'
        : 'the romanisation alone after the first appearance';
      findings.push(
        `handle-gloss-placed (slice ${String(appearance.sliceIndex,)}: "${written}" to "${wanted}"; ${why})`,
      );
    },);
  return findings.toReversed();
}

/**
 Carries every romanised handle's literal meaning at its first appearance
 on the page alone.

 @param slices - prepared pairs, whose original signs the handles

 @param replacements - what the page would write per slice

 @returns Replacements with the glosses placed, the rewritten rows alone,
 and one finding per rewritten appearance

 @example
 ```ts
 const placed = placeHandleGlosses({ slices, replacements, },);
 ```
 */
export function placeHandleGlosses(
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
   Slices in order.
   */
  const ordered = slicesInOrder({ slices, },);
  /**
   Page text per slice.
   */
  const pageText = pageTextBySlice({
    slices,
    replacements,
  },);
  /**
   Rendering per original name.
   */
  const authorities = nameAuthorities({
    slices: ordered,
    pageText,
  },);
  /**
   Slice indices a lane replaced, in page order; the archive's own text
   stays as it is.
   */
  const replaced = ordered
    .map(function indexOf(slice,): number {
      return slice.target
        .sliceIndex;
    },)
    .filter(function isReplaced(sliceIndex,): boolean {
      return replacements.some(function names(replacement,): boolean {
        return replacement.sliceIndex === sliceIndex;
      },);
    },);
  /**
   Page text per replaced slice, rewritten as the glosses are placed.
   */
  const texts = new Map<number, string>(replaced.map(function toEntry(sliceIndex,): readonly [
    number,
    string,
  ] {
    return [
      sliceIndex,
      pageTextOf({
        pageText,
        sliceIndex,
      },),
    ];
  },),);
  /**
   Handles the archive never rendered, one rendering each.
   */
  const renderings = [...new Set([...authorities.values(),]
    .filter(function unarchived(authority,): boolean {
      return authority.origin !== 'the archive\'s signature rendering';
    },)
    .map(function renderingOf(authority,): string {
      return authority.rendering;
    },),),];
  /**
   One finding per rewritten appearance.
   */
  const findings: string[] = [];
  for (const rendering of renderings) {
    /**
     Every appearance of this rendering among the replaced slices, in page
     order.
     */
    const appearances = replaced.flatMap(function inSlice(sliceIndex,): readonly Appearance[] {
      return appearancesIn({
        sliceIndex,
        // Every replaced slice has its text here, entered when `texts` was built.
        text: nonNullishOrThrow(texts.get(sliceIndex,),),
        rendering,
      },);
    },);
    findings.push(...placeOne({
      rendering,
      appearances,
      texts,
    },),);
  }
  /**
   Slices whose text the pass changed.
   */
  const rewritten = new Map<number, string>([...texts.entries(),]
    .filter(function changed([sliceIndex, text,]: ReadonlyEntry<number, string>,): boolean {
      return text !== pageTextOf({
        pageText,
        sliceIndex,
      },);
    },),);
  return {
    ...withRewrittenText({
      replacements,
      rewritten,
    },),
    findings,
  };
}

//endregion Handle gloss place
