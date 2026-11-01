import {
  proseNamePairs,
  type ProseNamePair,
  respelledRendering,
} from '@monochromatic-dev/translation-repair-rendering-audit/ts';
import type { DeclaredNamePair, } from '@monochromatic-dev/translation-repair-house-rule/ts';

//region Prose name floor
// A HAN NAME THE ARCHIVE SPELLS INSIDE A SENTENCE (the zheermao16 pass, slice
// 13, 2026-10-20). The house rule romanizes a handle with no declared, archive
// or corpus rendering as it is read, grouped, with its meaning in parentheses;
// the archive's own spelling of a name is a rendering, so the rule does not
// reach it. A judged polish read the rule as if it did, and shipped "Han Lian
// Yi (Cold Ripples)" over the archive's "Han Lianyi" for a cited author. The
// signer floor (`translate-signer-handle.ts`) reads signature lines only, and
// the cited author stood in a sentence.
//
// WHAT IS REFUSED: a candidate that writes the letters of the archive's
// spelling of a Han name the original writes, under another spelling (regrouped,
// joined, spaced differently, with or without a meaning after it). The finding
// names the original's name, the archive's spelling and the candidate's.
//
// WHAT IS LEFT ALONE: a name the front matter declares, whose declared form
// governs; a name the page spells two ways, since neither is the page's; a
// candidate that writes the archive's spelling anywhere, with or without a
// meaning; a candidate that does not write the name; and a slice with no page
// text, which has no spelling to keep. Hyphenated, lower-case and all-capital
// spellings are not read, so they pass; the spelling index reads capitalised
// words one space apart.

/**
 Whether a text holds another.

 @param outer - text that may hold it

 @param inner - text looked for

 @returns Whether `inner` stands in `outer`

 @example
 ```ts
 within({ outer: 'Xue Tuanzi', inner: 'Tuanzi', },); // true
 ```
 */
function within(
  {
    outer,
    inner,
  }: {
    readonly outer: string;
    readonly inner: string;
  },
): boolean {
  return outer.includes(inner,);
}

/**
 Whether a declared pair covers a name, by either text holding the other.

 @param declared - name pairs the front matter declares

 @param source - Han name under test

 @returns Whether a declared pair names it or a text around it

 @example
 ```ts
 isDeclared({ declared: [{ source: '雪团子', rendering: 'Snow', },], source: '雪团子', },); // true
 ```
 */
function isDeclared(
  {
    declared,
    source,
  }: {
    readonly declared: readonly DeclaredNamePair[];
    readonly source: string;
  },
): boolean {
  return declared.some(function covers(pair,): boolean {
    return within({
      outer: source,
      inner: pair.source,
    },) || within({
      outer: pair.source,
      inner: source,
    },);
  },);
}

/**
 Finding for one name a candidate respells.

 @param pair - the original's name and the archive's spelling of it

 @param spellings - spellings of the same letters the candidate writes

 @returns The sentence the floor reports

 @example
 ```ts
 const finding = respellFinding({ pair, spellings: ['Xuetuan Zi',], },);
 ```
 */
function respellFinding(
  {
    pair,
    spellings,
  }: {
    readonly pair: ProseNamePair;
    readonly spellings: readonly string[];
  },
): string {
  /**
   Spellings quoted, in the order the candidate writes them.
   */
  const written = spellings.map(function quoted(spelling,): string {
    return `"${spelling}"`;
  },);
  return `The original names ${pair.source} and the page already spells it "${pair.rendering}"; `
    + `your translation spells it ${written.join(', ',)}. Write "${pair.rendering}" exactly as the page does: `
    + 'the house rule leaves a name the archive has already rendered as the archive spells it.';
}

/**
 Findings for a candidate that respells a Han name the archive spells inside
 its sentences.

 @param sourceText - original slice

 @param candidateText - rendering under the floors

 @param pageText - text this candidate would replace, empty where the slice has none

 @param declared - name pairs the front matter declares

 @returns One finding per respelled name, in the order the original writes
 them; none where the page carries no spelling

 @example
 ```ts
 const findings = proseNameFindings({ sourceText, candidateText, pageText, declared, },);
 ```
 */
export function proseNameFindings(
  {
    sourceText,
    candidateText,
    pageText,
    declared,
  }: {
    readonly sourceText: string;
    readonly candidateText: string;
    readonly pageText: string;
    readonly declared: readonly DeclaredNamePair[];
  },
): readonly string[] {
  /**
   The original's Han names and the page's spelling of each.
   */
  const pairs = proseNamePairs({
    sourceText,
    targetText: pageText,
  },);
  return pairs.flatMap(function findingFor(pair,): readonly string[] {
    if (isDeclared({
      declared,
      source: pair.source,
    },))
      return [];
    // A NAME THE PAGE SPELLS TWO WAYS has no one spelling to keep.
    if (pairs.some(function spelledOtherwise(other,): boolean {
      return (other.source === pair.source) && (other.rendering !== pair.rendering);
    },))
      return [];
    /**
     Spellings of the same letters the candidate writes instead.
     */
    const spellings = respelledRendering({
      text: candidateText,
      rendering: pair.rendering,
    },);
    if (spellings.length === 0)
      return [];
    return [
      respellFinding({
        pair,
        spellings,
      },),
    ];
  },);
}

//endregion Prose name floor
