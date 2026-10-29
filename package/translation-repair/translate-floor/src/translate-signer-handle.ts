import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import { signaturesOf, } from '@monochromatic-dev/translation-repair-rendering-audit/ts';
import {
  handleReading,
  withoutGloss,
} from '@monochromatic-dev/translation-repair-page-pass/ts';
import {
  carriesHan,
  isAsciiLetter,
} from '@monochromatic-dev/translation-repair-markup/ts';
import type { DeclaredNamePair, } from '@monochromatic-dev/translation-repair-house-rule/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Signer handle floor
// LEDGER A17. The house rule (owner, 2026-09-22) romanizes a handle the
// ORIGINAL writes in Han, with no declared or archive rendering, as it is
// read, with its literal meaning in parentheses at its first appearance.
// The page passes place that meaning (`handle-gloss-place.ts`) but cannot
// invent one, so a meaning no writer wrote never reaches the page: over the
// stored artifacts, 45 of the 51 such page signers shipped bare everywhere
// they were rendered, shihai4h's runs of 2026-09-26 among them, after the
// sheet already stated the rule. And a signer the archive also left in Han
// passed the Han residue floor, whose page excuse keeps Han the page already
// carries.
//
// WHAT IS REFUSED, at a signature line the original signs in Han: the signer
// left in Han, or written as the handle's own reading (letters alone, so a
// spaced, capitalized or tone-marked spelling is the same reading) with no
// meaning in parentheses. Every candidate is asked for the meaning, since no
// slice knows which appearance comes first; the page keeps the first and
// strips the rest.
//
// WHAT IS LEFT ALONE: a signer the archive renders in Latin letters at the
// aligned signature, or a declared pair renders, whose rendering governs; any
// other Latin rendering, which a floor cannot tell from a rendering the
// person is known by; and a candidate whose signatures do not line up with
// the original's, which the block floors read.

/**
 Names a text signs with, in line order.

 @param text - slice text to read

 @returns Each signature's name, trimmed

 @example
 ```ts
 signerNames({ text: '——Whiskers, 2021', },); // ['Whiskers']
 ```
 */
function signerNames({ text, }: { readonly text: string; },): readonly string[] {
  return signaturesOf({ text, },)
    .map(function nameOf({ name, },): string {
      return name.trim();
    },);
}

/**
 Letters of a rendering alone, lowercased, so spacing, capitals and tone
 marks do not tell two spellings of one reading apart.

 @param rendering - rendering as a text writes it

 @returns Its Latin letters, lowercased

 @example
 ```ts
 readingLetters({ rendering: 'Jú Māo', },); // 'jumao'
 ```
 */
function readingLetters({ rendering, }: { readonly rendering: string; },): string {
  // BY UTF-16 UNIT, deliberately: only basic Latin letters are kept, and
  // every tone mark NFD splits off is a unit of its own.
  return rendering
    .normalize('NFD',)
    .split('',)
    .filter(function isLetter(character,): boolean {
      return isAsciiLetter({ character, },);
    },)
    .join('',)
    .toLowerCase();
}

/**
 Letters of the parenthetical a writer may echo instead of a meaning: the
 words a finding or the house rule uses for one. The one wrong meaning that
 can be predicted, so the one refused; any other text is the writer's
 reading of the handle.
 */
const PLACEHOLDER_MEANINGS: ReadonlySet<string> = new Set([
  'meaning',
  'literalmeaning',
  'itsmeaning',
  'itsliteralmeaning',
  'literalmeaninginenglish',
  'itsliteralmeaninginenglish',
  'meaninginenglish',
],);

/**
 Finding for one signer a candidate left in Han, signed with nothing a
 reader sees, wrote with no meaning, or glossed with a placeholder, or nothing.

 @param name - signer as the original signs it

 @param rendered - signer as the candidate signs it

 @returns Finding, or none

 @example
 ```ts
 signerFinding({ name: '橘猫', rendered: 'Jumao', },);
 ```
 */
function signerFinding(
  {
    name,
    rendered,
  }: {
    readonly name: string;
    readonly rendered: string;
  },
): readonly string[] {
  /**
   The handle as the house rule reads it.
   */
  const reading = handleReading({ name, },);
  /**
   What the candidate wrote, without any meaning after it.
   */
  const bare = withoutGloss({ rendering: rendered, },);
  // AN INSTRUCTION WITH THE HOUSE RULE'S OWN EXAMPLE, never a template: a
  // quoted "(its literal meaning)" is the text a writer copies back.
  /**
   What to write, said in every finding.
   */
  const instruction = `Write ${reading}, followed by what ${name} means in English in parentheses, as the house `
    + 'rule writes Jinxin (Brocade Heart); the page keeps the meaning at its first appearance and drops it after that.';
  // WHAT SHOWS A READER NOTHING SIGNS NO ONE (ledger B40): a name of one
  // zero-width space or Hangul filler has no Latin letters to compare with the
  // reading, so it fell through to the reading-letters test, which passes a
  // signer that is not the handle's reading.
  if (rendersAsNothing({ text: rendered, },))
    return [`The signature names ${name} and your translation signs it with nothing a reader can see. ${instruction}`,];
  if (carriesHan({ text: bare, },))
    return [`The signature names ${name} and your translation leaves it in Han. ${instruction}`,];
  if (readingLetters({ rendering: bare, },) !== readingLetters({ rendering: reading, },))
    return [];
  if (bare === rendered)
    return [`The signature names ${name} and your translation writes "${bare}" with no literal meaning. ${instruction}`,];
  if (PLACEHOLDER_MEANINGS.has(readingLetters({ rendering: rendered.slice(bare.length,), },),))
    return [`The signature names ${name} and the parentheses after "${bare}" hold a placeholder, not a meaning. ${instruction}`,];
  return [];
}

/**
 Findings for every signer a candidate leaves in Han or romanizes with no
 literal meaning (ledger A17).

 @param sourceText - original slice

 @param candidateText - rendering under the floors

 @param pageText - page as it stands, whose Latin rendering of a signer
 governs it

 @param declared - name pairs the front matter declares, whose rendering
 governs a signer they name

 @returns One finding per such signer, empty when none

 @example
 ```ts
 const findings = signerHandleFindings({ sourceText, candidateText, pageText, declared, },);
 ```
 */
export function signerHandleFindings(
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
   Signers in the original, in line order.
   */
  const sourceNames = signerNames({ text: sourceText, },);
  /**
   Signers in the candidate, read only where they line up with the original's.
   */
  const candidateNames = signerNames({ text: candidateText, },);
  if ((sourceNames.length === 0) || (candidateNames.length !== sourceNames.length))
    return [];
  /**
   Signers on the page as it stands, lined up the same way or not at all.
   */
  const pageNames = signerNames({ text: pageText, },);
  /**
   Whether the page's signers line up with the original's.
   */
  const pageAligned = pageNames.length === sourceNames.length;
  return sourceNames.flatMap(function findingsAt(
    name,
    at,
  ): readonly string[] {
    if (!carriesHan({ text: name, },))
      return [];
    /**
     Page's rendering of this signer, where the page lines up; lined up, the
     page signs as often as the original does, so it has one at this place.
     */
    const pageName = pageAligned ? nonNullishOrThrow(pageNames[at],) : '';
    // A PAGE SIGNER THAT SHOWS NOTHING RENDERS NO ONE, so it governs nothing.
    if ((pageName !== '')
      && (!carriesHan({ text: pageName, },))
      && (!rendersAsNothing({ text: pageName, },)))
      return [];
    if (declared.some(function declares({ source, },): boolean {
      return source === name;
    },))
      return [];
    // The candidate signs as often as the original, checked on entry, so it
    // has a signer at this place.
    return signerFinding({
      name,
      rendered: nonNullishOrThrow(candidateNames[at],),
    },);
  },);
}

//endregion Signer handle floor
