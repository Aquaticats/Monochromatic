import { codePointAt, } from './code-points.ts';
import {
  isTagWhitespace,
  leavesBracketAsText,
  startsTagName,
} from './mdx-tag-name.ts';

//region MDX tag start
// WHETHER A `<` OPENS A TAG, answered as the MDX compiler the corpus is built
// with answers it (`micromark-extension-mdx-jsx` 3.0.2, `dev/lib/factory-tag.js`,
// the states `startAfter` and `nameBefore`): a space, a tab or a line end right
// after `<` leaves it text; any other whitespace is stepped over; then `/`
// opens a closing tag, `>` a fragment, and a character that can start an
// identifier (`$`, `_`, or Unicode ID_Start of the first plane, Han included)
// an element. Anything else fails the page's compile, so no tag opens there
// either; a letter past U+FFFF is two units to the compiler, neither of them a
// letter (`mdx-tag-name.ts`).
//
// Three scanners kept their own answer (audit area six, 2026-09-28; ledger
// B18): the typography mask took ASCII letters and `/`, the markup atom scan
// the same and `!`, and the prose ranges `/` and any cased letter. None
// matched the compiler: `<猫 title="nap" />` was prose to all three, its quoted
// value curled and respelled, and `<Ⓐ` (a cased symbol no identifier starts
// with) a tag to one. An HTML comment is none of these: the corpus's build
// rewrites `<!--` into a JSX comment before compiling, and each scanner reads
// comments by their own opener.

/**
 Offset past the whitespace the compiler steps over, starting at an offset.

 @param text - text being read

 @param from - offset just after `<`

 @returns Offset of the first code point that is no whitespace, or the text's
 length

 @example
 ```ts
 const at = pastWhitespace({ text: '<\u{00A0}b', from: 1, },); // 2
 ```
 */
function pastWhitespace({
  text,
  from,
}: {
  readonly text: string;
  readonly from: number;
},): number {
  /**
   Offset moved past each whitespace code point.
   */
  let at = from;
  while (isTagWhitespace({
    character: codePointAt({
      text,
      at,
    },),
  },)) {
    at += codePointAt({
      text,
      at,
    },)
      .length;
  }
  return at;
}

/**
 Whether the `<` at an offset opens a tag the MDX compiler reads: a closing
 tag, a fragment or an element, by the first character of its name alone:
 an angle autolink (`<https://cat.example>`) must stay a span the scanners
 protect, and a name the grammar refuses further on leaves the page uncompilable,
 so protecting it to its bracket is the safe side. What a name is, whole, is
 `readTagName` (`mdx-tag-name.ts`).

 @param text - text being read

 @param at - offset of the `<`

 @returns False where the compiler reads `<` as text, and where the first
 character of what follows can start no tag, so it refuses the page

 @example
 ```ts
 opensMdxTag({ text: 'a <猫 /> b', at: 2, },); // true
 opensMdxTag({ text: 'a < b', at: 2, },); // false
 opensMdxTag({ text: 'a <3 b', at: 2, },); // false: the page fails to compile
 ```
 */
export function opensMdxTag({
  text,
  at,
}: {
  readonly text: string;
  readonly at: number;
},): boolean {
  /**
   Character right after the `<`.
   */
  const next = codePointAt({
    text,
    at: at + 1,
  },);
  if ((next === '') || leavesBracketAsText({ character: next, },))
    return false;
  /**
   First character past any whitespace the compiler steps over.
   */
  const first = text.charAt(pastWhitespace({
    text,
    from: at + 1,
  },),);
  return (first === '/') || (first === '>')
    || startsTagName({ character: first, },);
}

//endregion MDX tag start
