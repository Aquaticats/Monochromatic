import {
  isAsciiDigit,
  isAsciiLetter,
} from './ascii-letters.ts';

//region Latin letters
// What a word of English prose is made of, for the scanners that split one:
// the glossary's form edges (`glossary-match.ts`, which set this definition
// when ledger C1 put Latin forms at word boundaries) and the Canadian
// spelling pass's combining marks. Every other prose scanner tested ASCII
// letters only (audit area six, 2026-09-28), so an accented letter ended a
// word: `Pokémon` read as `Pok` and `mon`, and a tone-marked pinyin syllable
// as two fragments. The ledger's B18 entry lists each scanner switched here
// and what it changed over the pinned corpus.
//
// LATIN SCRIPT, NOT EVERY CASED LETTER. A kaomoji an English page keeps
// (`(・ω・)`) carries Greek and Cyrillic letters that are no part of a word
// beside them. The blocks here are Latin-1's letters, Latin Extended-A and
// -B (every pinyin tone vowel sits in one of them) and Latin Extended
// Additional (Vietnamese); the two Latin-1 signs among them are excluded.

/**
 First code point of the Latin-1 letters and the Latin Extended blocks.
 */
const LATIN_EXTENDED_FIRST = '\u{00C0}';

/**
 Last code point of Latin Extended-B.
 */
const LATIN_EXTENDED_LAST = '\u{024F}';

/**
 First code point of Latin Extended Additional.
 */
const LATIN_ADDITIONAL_FIRST = '\u{1E00}';

/**
 Last code point of Latin Extended Additional.
 */
const LATIN_ADDITIONAL_LAST = '\u{1EFF}';

/**
 Signs inside the Latin-1 range that are not letters.
 */
const LATIN_SIGNS: ReadonlySet<string> = new Set([
  '\u{00D7}',
  '\u{00F7}',
],);

/**
 First combining diacritical mark, which continues the letter before it.
 */
const COMBINING_FIRST = '\u{0300}';

/**
 Last combining diacritical mark.
 */
const COMBINING_LAST = '\u{036F}';

/**
 Whether one character is a Latin letter: ASCII, or one of the Latin-1,
 Latin Extended-A, -B or Additional letters.

 @param character - one UTF-16 unit, empty past a text's edge

 @returns True for a Latin letter, accented or not

 @example
 ```ts
 isLatinLetter({ character: 'é', },); // true
 isLatinLetter({ character: 'ω', },); // false
 ```
 */
export function isLatinLetter({ character, }: { readonly character: string; },): boolean {
  if (isAsciiLetter({ character, },))
    return true;
  if ((character >= LATIN_ADDITIONAL_FIRST) && (character <= LATIN_ADDITIONAL_LAST))
    return true;
  return (character >= LATIN_EXTENDED_FIRST)
    && (character <= LATIN_EXTENDED_LAST)
    && (!LATIN_SIGNS.has(character,));
}

/**
 Whether one character is a capital Latin letter, accented or not.

 @param character - one UTF-16 unit, empty past a text's edge

 @returns True for a Latin letter its lower case changes

 @example
 ```ts
 isLatinCapital({ character: 'É', },); // true
 isLatinCapital({ character: 'é', },); // false
 ```
 */
export function isLatinCapital({ character, }: { readonly character: string; },): boolean {
  return isLatinLetter({ character, },) && (character.toLowerCase() !== character);
}

/**
 Whether one character belongs to a Latin word: a Latin letter or an ASCII
 digit.

 @param character - one UTF-16 unit, empty past a text's edge

 @returns True for a letter or digit a word may continue with

 @example
 ```ts
 isLatinWordCharacter({ character: '7', },); // true
 isLatinWordCharacter({ character: '，', },); // false
 ```
 */
export function isLatinWordCharacter({ character, }: { readonly character: string; },): boolean {
  return isLatinLetter({ character, },) || isAsciiDigit({ character, },);
}

/**
 Whether a character is a combining diacritical mark, which continues the
 letter before it (`idée` written with a separate accent).

 @param character - one UTF-16 unit

 @returns True inside the combining diacritical marks block

 @example
 ```ts
 isCombiningMark({ character: '\u{0301}', },); // true
 ```
 */
export function isCombiningMark({ character, }: { readonly character: string; },): boolean {
  return (character >= COMBINING_FIRST) && (character <= COMBINING_LAST);
}

/**
 Whether one character continues a run of Latin letters already begun: a
 Latin letter, or a combining mark. Digits end such a run, for the scanners
 that read words and not tokens.

 @param character - one UTF-16 unit, empty past a text's edge

 @returns True for a letter or a combining mark

 @example
 ```ts
 isLatinLetterOrMark({ character: '\u{030C}', },); // true
 isLatinLetterOrMark({ character: '2', },); // false
 ```
 */
export function isLatinLetterOrMark({ character, }: { readonly character: string; },): boolean {
  return isLatinLetter({ character, },) || isCombiningMark({ character, },);
}

/**
 Whether one character continues a Latin word already begun: a word
 character, or a combining mark. It reads the character as written, so a
 compatibility character (a mathematical or fullwidth letter, a ligature, `™`,
 `²`) does not continue a word here; the consistency pass alone reads a compatibility letter or digit as one
 (`continuesLatinWordOrCompatibilityForm`), and never a symbol or a number that is no digit (`™`, `²`).

 @param character - one UTF-16 unit, empty past a text's edge

 @returns True for a character a word runs on through

 @example
 ```ts
 continuesLatinWord({ character: '\u{0301}', },); // true
 continuesLatinWord({ character: '\u{FF21}', },); // false, a fullwidth capital A
 ```
 */
export function continuesLatinWord({ character, }: { readonly character: string; },): boolean {
  return isLatinWordCharacter({ character, },) || isCombiningMark({ character, },);
}

/**
 First code point past ASCII, so a character below it that is no word character is none under any form.
 */
const FIRST_PAST_ASCII = '\u{0080}';

/* oxlint-disable no-restricted-syntax/no-regex -- the input is one code point, anchored at both ends with no quantifier, so the test is bounded and cannot backtrack; the Unicode general categories for letters and decimal digits have no string API */
/**
 A whole character that is a letter or a decimal digit of any script.
 */
const LETTER_OR_DIGIT_OF_ANY_SCRIPT = /^[\p{L}\p{Nd}]$/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/**
 Whether a whole character is a letter or a decimal digit of any script. It is the category test, and no test of a Latin word:
 a word's edge or a mail address needs it as it is, and the neighbour test of the consistency pass
 (`continuesLatinWordOrCompatibilityForm`) asks it of a compatibility character so that a symbol or a number that is no digit
 (`™`, `²`, `①`) is no letter of a word.

 @param character - one whole character, empty past an edge of a text

 @returns Whether it is a letter or a decimal digit

 @example
 ```ts
 const letter = isLetterOrDigitOfAnyScript({ character: '猫', },); // true
 ```
 */
export function isLetterOrDigitOfAnyScript({ character, }: { readonly character: string; },): boolean {
  return LETTER_OR_DIGIT_OF_ANY_SCRIPT.test(character,);
}

/* oxlint-disable no-restricted-syntax/no-regex -- the input is one code point, anchored at both ends with no quantifier, so the test is bounded and cannot backtrack; the Unicode general categories and scripts have no string API */
/**
 A whole character that is a combining mark of any script, or a letter, mark or decimal digit of a script written without spaces
 between words (Han, Hiragana, Katakana): the two groups apart, since the second is no part of a word a space ends.
 */
const MARK_OF_ANY_SCRIPT = /^\p{M}$/u;

/**
 A whole character of a script whose words are not set apart by spaces.
 */
const UNSPACED_SCRIPT_CHARACTER = /^[\p{Script=Han}\p{Script=Hiragana}\p{Script=Katakana}]$/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/**
 The characters that are letters by category but that the consistency pass reads as a single mark, never as a letter of a word:
 the modifier letter apostrophe (`cats\u{02BC}` is a plural and a mark, as `cats’` is).
 */
const SINGLE_MARK_LETTERS: ReadonlySet<string> = new Set(['\u{02BC}',],);

/**
 Whether a whole character is a letter, a combining mark or a decimal digit of a script that sets its words apart by spaces, so a
 run of them is one word to a reader: Latin, Cyrillic, Greek, Hebrew, Arabic, Hangul and the rest. Han, Hiragana and Katakana are
 not, since a word of any other script beside them is a word of its own (`看kittens好`), as an apostrophe does not bind into Han
 either (`quote-neighbours.ts`, class ninety-six).

 @param character - one whole character, empty past an edge of a text

 @returns Whether a word runs on through it

 @example
 ```ts
 const letter = isSpacedScriptWordCharacter({ character: 'П', },); // true
 const han = isSpacedScriptWordCharacter({ character: '猫', },); // false
 ```
 */
export function isSpacedScriptWordCharacter({ character, }: { readonly character: string; },): boolean {
  if (character === '')
    return false;
  if (SINGLE_MARK_LETTERS.has(character,) || UNSPACED_SCRIPT_CHARACTER.test(character,))
    return false;
  return isLetterOrDigitOfAnyScript({ character, },) || MARK_OF_ANY_SCRIPT.test(character,);
}

/**
 Whether one character is the neighbour of a mark that a word runs on through, as the consistency pass reads it:
 a character `continuesLatinWord` admits, or a whole character that is none of them as written but whose NFKC form
 is made of them alone and that is itself a letter or a decimal digit (`Ａ`, `１`, `𝓠`, `𝐀`, `𝟏`, `ﬁ`). A symbol or a number
 that is no decimal digit is none (`™`, `²`, `①`, `Ⅻ`): a mark beside one has no word beside it.

 THE ONE NEIGHBOUR TEST OF THE CONSISTENCY PASS, AND WHY IT DIFFERS FROM `continuesLatinWord`. The pass keeps a word
 and the apostrophes, quotation marks and brackets beside it together, and a page may spell a word with
 mathematical script letters (`𝓠𝓾𝓪𝓷’s`) or fullwidth ones, which a reader sees as the Latin word. Every other
 reader in the package (glossary word bounds, quote locating, alignment, preservation tokens, ballots, archive name
 runs) is measured against the text as written and keeps `continuesLatinWord`, so a compatibility character
 changes nothing outside the pass.

 THE COMPATIBILITY FORM IS READ ONLY FOR A WHOLE CHARACTER. A caller that reads by code point hands over a
 mathematical letter beyond the first plane whole and it counts; a caller that reads by UTF-16 unit hands over
 half of one, a lone surrogate, whose form is itself, and it does not count. A form of several characters counts when
 every one of them is a Latin letter, an ASCII digit or a combining mark (`ﬁ` is `fi`), and a form with any other
 character in it does not (`½` is `1⁄2`, `℃` is `°C`, `µ` is a Greek letter). A character that is no letter and no decimal
 digit does not count whatever its form is (`Ⅻ` is `XII`, `™` is `TM`, `²` is `2`, `①` is `1`). ASCII and the
 characters the plain test admits are decided before any form is computed; only a non-ASCII character the plain test
 refuses is normalised.

 @param character - one whole character, or one UTF-16 unit, empty past a text's edge

 @returns True for a character a word runs on through, as written or by its compatibility form

 @example
 ```ts
 continuesLatinWordOrCompatibilityForm({ character: '\u{0301}', },); // true
 continuesLatinWordOrCompatibilityForm({ character: '\u{FF21}', },); // true, a fullwidth capital A
 ```
 */
export function continuesLatinWordOrCompatibilityForm({ character, }: { readonly character: string; },): boolean {
  if (continuesLatinWord({ character, },))
    return true;
  if (character < FIRST_PAST_ASCII)
    return false;
  /**
   The whole characters of the text it was given.
   */
  const points = Array.from(character,);
  if (points.length !== 1)
    return false;
  // A symbol or a number that is no decimal digit is no letter of a word, whatever its form is made of.
  if (!isLetterOrDigitOfAnyScript({ character, },))
    return false;
  /**
   What the character is under NFKC.
   */
  const form = character.normalize('NFKC',);
  if (form === character)
    return false;
  /**
   The whole characters of that form.
   */
  const formPoints = Array.from(form,);
  return formPoints.every(function isWordCharacterAsWritten(point,): boolean {
    return continuesLatinWord({ character: point, },);
  },);
}

/**
 One run of Latin letters and where it starts.
 */
export type LatinWord = {
  readonly word: string;
  readonly start: number;
};

/**
 Every run of Latin letters in a text, with where each starts, by one index
 scan. A run opens on a letter, accented or not, and goes on through letters
 and combining marks, so `Château` is one word whether its accent is composed
 or not. The opening letter is taken before the run's loop, so the scan always
 advances (ledger M50).

 The casing restores and the address and suicide-method floors each kept a
 copy of this scan (ledger B18).

 @param text - text under scan

 @returns Runs in order, as written

 @example
 ```ts
 latinWordSpans({ text: 'MAOWU Station', },); // [{ word: 'MAOWU', start: 0 }, { word: 'Station', start: 6 }]
 ```
 */
export function latinWordSpans({ text, }: { readonly text: string; },): readonly LatinWord[] {
  /**
   Runs read so far.
   */
  const words: LatinWord[] = [];
  for (let at = 0; at < text.length;) {
    if (!isLatinLetter({ character: text.charAt(at,), },)) {
      at += 1;
      continue;
    }
    /**
     Where this run starts.
     */
    const start = at;
    at += 1;
    while ((at < text.length) && isLatinLetterOrMark({ character: text.charAt(at,), },))
      at += 1;
    words.push({
      word: text.slice(
        start,
        at,
      ),
      start,
    },);
  }
  return words;
}

/**
 Every run of Latin letters in a text, lower-cased, in order, so a word is
 matched whole and never inside another.

 @param text - text under scan

 @returns Lower-cased words in order

 @example
 ```ts
 lowerCaseLatinWords({ text: 'May you, yes you!', },); // ['may', 'you', 'yes', 'you']
 ```
 */
export function lowerCaseLatinWords({ text, }: { readonly text: string; },): readonly string[] {
  return latinWordSpans({ text, },)
    .map(function lowered({ word, },): string {
      return word.toLowerCase();
    },);
}

/**
 A Latin word folded for comparison across writers: decomposed, its
 combining marks dropped, lower-cased. `Mikä`, `Mika\u{0308}` and `Mika` fold
 alike, as a handle carried across a translation is written with its accent,
 with a separate one, or without.

 @param word - run of Latin letters and marks

 @returns Folded word

 @example
 ```ts
 foldLatinWord({ word: 'Mikä', },); // 'mika'
 ```
 */
export function foldLatinWord({ word, }: { readonly word: string; },): string {
  return Array.from(word.normalize('NFD',),)
    .filter(function unmarked(character,): boolean {
      return !isCombiningMark({ character, },);
    },)
    .join('',)
    .toLowerCase();
}

/**
 Every Latin word of a text, folded (`foldLatinWord`), in order: the form in
 which two writers' vocabularies are compared, so an accent written composed,
 combining or not at all neither splits a word nor makes it another word, and
 a word's length counts its letters and not its marks.

 @param text - text under scan

 @returns Folded words in order

 @example
 ```ts
 foldedLatinWords({ text: 'Château and Cha\u{0302}teau', },); // ['chateau', 'and', 'chateau']
 ```
 */
export function foldedLatinWords({ text, }: { readonly text: string; },): readonly string[] {
  return latinWordSpans({ text, },)
    .map(function folded({ word, },): string {
      return foldLatinWord({ word, },);
    },);
}

//endregion Latin letters
