import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';
import { tokenStarts, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Neutral pronoun rendering
// A TRANSLATION THAT LEAVES THE CORPUS'S NEUTRAL PRONOUN IN LATIN LETTERS has
// not translated it. The sources write `TA`, `Ta` or `ta` for a person who did
// not specify a pronoun, and English has a rendering for that: singular they.
//
// WHY THIS IS A FLOOR RULE AND NOT A PROMPT ALONE. The SS3B_0016 run of
// 2026-09-04 (13:51 UTC) shipped "a small room for Ta, to give Ta's memorial a
// little warmth" on a page that says "they" for the same person everywhere
// else, and three of five contest ballots plus the consolidation gate kept it
// as "the original's neutral Ta". Judges reading a passage cannot tell an
// untranslated word from a preserved choice when the house rule names TA as a
// pronoun the original uses and never says what English makes of it. The rule
// now says so, and this refuses the candidate before any judge is asked.
//
// WHAT THE CORPUS SAYS. Measured over the pinned corpus on 2026-09-04: sources
// write the pronoun as `TA` in 2 entries, `Ta` in 7 and `ta` in 8, every
// occurrence a pronoun; of those entries' archives, one (a rewrite) keeps a
// bare `TA`, and the rest render it "they". An archive that kept it fails this
// rule as a standing text, which is what the owner's ineligible-standing
// decision provides for: the slate prefers a valid proposal. Remeasured on
// 2026-09-29 under the token reading (`pronoun-entry-census.mjs`): the same
// figures, the archive XingZ60's. The old list read `Ta` in 6 sources,
// missing qianyuanakg's.
//
// THE PRONOUN IS A TOKEN OF ITS OWN (ledger B23), read by `tokenStarts`
// (`word-bounds.ts`): `DATA`, `STATION`, `meta`, a romanised handle, a path
// segment and an address all contain the letters and none is the pronoun.
// The floor used to count an occurrence only between listed marks, whitespace
// or characters at or above U+2E80, compared by UTF-16 unit. That list missed
// a doubled Chinese dash and a slash after han in three originals
// (XingZ60, Mizuki_Yuuki, qianyuanakg), so the floor never applied to those
// slices, and an English em dash beside a kept `TA` in stored renderings
// (XingZ60, noname), so it passed them (`pronoun-disagreement-census.mjs`,
// 2026-09-29). The listed 「」 was redundant: it and 『』 sit above U+2E80.
//
// WHAT THE TOKEN READING DOES NOT SEE: `he/TA` and `Well...Ta` read as one
// token each, as an address would, so neither counts. The old list counted
// neither; the census found neither in any original or stored rendering.

/**
 Spellings the sources give the neutral pronoun, and so the spellings an
 untranslated one keeps.
 */
const PRONOUN_SPELLINGS = [
  'TA',
  'Ta',
  'ta',
] as const;

/**
 What English makes of the pronoun, told to the model that left it.
 */
const RENDERING_RULE: string = 'the ORIGINAL writes its neutral pronoun as TA, Ta or ta, and English renders it as '
  + 'singular they (they, them, their), with TA 们 as plural they; a Ta left standing in the English is an '
  + 'untranslated word, not a preserved choice.';

/**
 Counts how often one spelling stands as a token of its own.

 @param text - original or candidate translation

 @param spelling - fixed form to count

 @returns Occurrences standing as tokens

 @example
 ```ts
 countSpelling({ text: 'Ta smiled. DATA', spelling: 'Ta', },);
 // => 1
 ```
 */
function countSpelling(
  {
    text,
    spelling,
  }: {
    readonly text: string;
    readonly spelling: string;
  },
): number {
  /**
   Starts where the spelling stands as a token.
   */
  const starts = tokenStarts({
    text,
    needle: spelling,
  },);
  return starts.length;
}

/**
 Whether the original writes the neutral pronoun at all, in any spelling.

 @param sourceText - original slice

 @returns True where it stands as a token at least once

 @example
 ```ts
 originalWritesPronoun({ sourceText: 'TA 睡了。', },);
 // => true
 ```
 */
function originalWritesPronoun({ sourceText, }: { readonly sourceText: string; },): boolean {
  return PRONOUN_SPELLINGS.some(function written(spelling,): boolean {
    return countSpelling({
      text: sourceText,
      spelling,
    },) > 0;
  },);
}

/**
 Names an untranslated neutral pronoun in a candidate, written for the model
 that wrote the candidate.

 ONLY WHERE THE ORIGINAL WRITES THE PRONOUN (ledger F-9): an English TA (a
 teaching assistant, for 助教) or a "Ta!" of thanks in a passage whose
 original writes no TA, Ta or ta is English, not the pronoun left standing.

 @param sourceText - original slice

 @param candidateText - translation as the model returned it

 @returns One finding naming each spelling found with its count, or nothing
 when the original writes no neutral pronoun or the candidate carries none

 @example
 ```ts
 neutralPronounFindings({ sourceText: '给 Ta 一个房间。', candidateText: 'We set up a room for Ta.', },);
 // => ['Your translation carries the pronoun untranslated as "Ta" (1 time): ...']
 ```
 */
export function neutralPronounFindings(
  {
    sourceText,
    candidateText,
  }: {
    readonly sourceText: string;
    readonly candidateText: string;
  },
): readonly string[] {
  if (!originalWritesPronoun({ sourceText, },))
    return [];
  /**
   Each spelling the candidate keeps, with its count.
   */
  const kept = PRONOUN_SPELLINGS
    .map(function counted(spelling,): {
      readonly spelling: string;
      readonly count: number;
    } {
      return {
        spelling,
        count: countSpelling({
          text: candidateText,
          spelling,
        },),
      };
    },)
    .filter(function found(entry,): boolean {
      return entry.count > 0;
    },);
  if (kept.length === 0)
    return [];

  /**
   Spellings and counts as one phrase.
   */
  const named = kept
    .map(function phrase(entry,): string {
      return `"${entry.spelling}" (${String(entry.count,)} ${
        wordForCount({
          count: entry.count,
          one: 'time',
          many: 'times',
        },)
      })`;
    },)
    .join(' and ',);
  return [`Your translation carries the pronoun untranslated as ${named}: ${RENDERING_RULE}`,];
}

//endregion Neutral pronoun rendering
