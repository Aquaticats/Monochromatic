import { codePointAt, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Word reading
// Words of a sentence and the neighbours the present-tense rules compare: the
// word after a subject past one run of whitespace, and the word before it. No
// pattern is compiled from a declared name, so a name is compared as text and
// never read as a regular expression. Text is read one whole character at a
// time, so a letter of a script without case, or beyond the first plane, is a
// letter like any other.

/**
 One word of a sentence with the offsets it occupies there.
 */
export type WordToken = {
  /**
   Word as written.
   */
  readonly text: string;

  /**
   Word folded for comparison: lower case, with the typographic apostrophe read as the straight one.
   */
  readonly folded: string;

  /**
   Offset of the first character.
   */
  readonly start: number;

  /**
   Exclusive end offset.
   */
  readonly end: number;
};

/**
 Whether a whole character is a letter of any script, cased or not, or a mark
 that joins one.

 READ BY GENERAL CATEGORY, ONE WHOLE CHARACTER AT A TIME. A letter was once
 whatever has a lower-case form that differs from its upper-case one, which is
 no letter of Han or Hangul and no half of a surrogate pair, so a name written
 in either script, or beyond the first plane, never opened a word.

 @param character - one whole character of text, as `codePointAt` of `code-points.ts` reads it

 @returns Whether the character is a letter or a combining mark

 @example
 ```ts
 const letter = isLetterCharacter({ character: 'm', },);
 ```
 */
function isLetterCharacter({ character, }: { readonly character: string; },): boolean {
  // oxlint-disable-next-line no-restricted-syntax/no-regex -- Unicode letter and mark classes have no string-API equivalent and a declared name may be written in any script; input is ONE character and the pattern carries no quantifier or alternation outside the class, so it cannot backtrack.
  return /^[\p{L}\p{M}]$/u.test(character,);
}

/**
 Whether a character continues a word: a letter, a straight or typographic apostrophe, or a hyphen.

 @param character - one whole character of text, empty past the end

 @returns Whether the character belongs inside a word

 @example
 ```ts
 const inside = isWordCharacter({ character: '’', },);
 ```
 */
function isWordCharacter({ character, }: { readonly character: string; },): boolean {
  if (character.length === 0) {
    return false;
  }
  return isLetterCharacter({ character, },) || ('\'’-'.includes(character,));
}

/**
 Whether a character is whitespace.

 @param character - one character of text, empty past the end

 @returns Whether the character is a space, tab or line break

 @example
 ```ts
 const blank = isWhitespace({ character: ' ', },);
 ```
 */
export function isWhitespace({ character, }: { readonly character: string; },): boolean {
  /**
   The character with surrounding whitespace removed, empty when the character is whitespace.
   */
  const trimmed = character.trim();
  return (character.length > 0) && (trimmed.length === 0);
}

/**
 Folds a word for comparison: lower case, with the typographic apostrophe read as the straight one.

 @param word - word as written

 @returns The folded word

 @example
 ```ts
 const folded = foldWord({ word: 'She’s', },);
 ```
 */
function foldWord({ word, }: { readonly word: string; },): string {
  return word
    .toLowerCase()
    .replaceAll(
    '’',
    '\'',
    );
}

/**
 Builds the word that occupies one range of a sentence.

 @param sentence - sentence the word sits in

 @param start - offset of the first character

 @param end - exclusive end offset

 @returns The word with its text, folded form and offsets

 @example
 ```ts
 const word = wordAt({ sentence: 'Mittens naps.', start: 0, end: 7, },);
 ```
 */
function wordAt(
  {
    sentence,
    start,
    end,
  }: {
    readonly sentence: string;
    readonly start: number;
    readonly end: number;
  },
): WordToken {
  /**
   Word as written.
   */
  const text = sentence.slice(
    start,
    end,
  );
  return {
    text,
    folded: foldWord({ word: text, },),
    start,
    end,
  };
}

/**
 Reads the words of a sentence, in order, with their offsets.

 @param sentence - sentence text, quotes already blanked

 @returns Words with their offsets, in order

 @example
 ```ts
 const words = wordsOf({ sentence: 'Mittens naps.', },);
 ```
 */
export function wordsOf({ sentence, }: { readonly sentence: string; },): readonly WordToken[] {
  return (function readWords(): readonly WordToken[] {
    /**
     Words found so far, in order.
     */
    const words: WordToken[] = [];

    /**
     Offset where the word being read begins, negative between words.
     */
    let wordStart = -1;

    /**
     Offset of the next character to read.
     */
    let index = 0;
    while (index <= sentence.length) {
      /**
       Whole character at this offset, one UTF-16 unit or two, empty past the end.
       */
      const character = codePointAt({
        text: sentence,
        at: index,
      },);

      /**
       Whether that character continues a word; past the end, nothing does.
       */
      const inWord = isWordCharacter({ character, },);
      if (inWord && (wordStart < 0)) {
        wordStart = index;
      }
      if ((!inWord) && (wordStart >= 0)) {
        words.push(wordAt({
          sentence,
          start: wordStart,
          end: index,
        },));
        wordStart = -1;
      }
      // Past the end the character is empty, so the step of one leaves the loop.
      index += Math.max(
        1,
        character.length,
      );
    }
    return words;
  })();
}

/**
 Whether a range of a sentence is one or more whitespace characters and nothing else.

 @param sentence - sentence the range sits in

 @param start - offset of the first character

 @param end - exclusive end offset

 @returns Whether the range is all whitespace

 @throws {@link Error} when the range is empty, which the word reader never produces

 @example
 ```ts
 const blank = isBlankSpan({ sentence: 'did she', start: 3, end: 4, },);
 ```
 */
export function isBlankSpan(
  {
    sentence,
    start,
    end,
  }: {
    readonly sentence: string;
    readonly start: number;
    readonly end: number;
  },
): boolean {
  if (end <= start) {
    throw new Error('unreachable: the two words either side of a span are separated by at least one non-word character, so the span is never empty',);
  }
  for (let index = start; index < end; index += 1) {
    if (!isWhitespace({ character: sentence.charAt(index,), },)) {
      return false;
    }
  }
  return true;
}

/**
 Offset just past the run of whitespace that begins at a given offset.

 @param sentence - sentence the run sits in

 @param start - offset where the run begins

 @returns The offset of the first character after the run, equal to start when no whitespace begins there

 @example
 ```ts
 const after = skipWhitespace({ sentence: 'Mittens  naps', start: 7, },);
 ```
 */
function skipWhitespace(
  {
    sentence,
    start,
  }: {
    readonly sentence: string;
    readonly start: number;
  },
): number {
  return (function scanRun(): number {
    /**
     Offset of the next character to test.
     */
    let next = start;
    while ((next < sentence.length) && isWhitespace({ character: sentence.charAt(next,), },)) {
      next += 1;
    }
    return next;
  })();
}

/**
 Finds the word that follows an offset past one run of whitespace. The result is
 an array so a missing word is an empty array rather than an absent value.

 @param sentence - sentence the words sit in

 @param wordsByStart - words keyed by their start offset

 @param end - offset just past the word before the run of whitespace

 @returns The following word in a one-element array, or an empty array when no whitespace or no word follows

 @example
 ```ts
 const next = wordAfterSpace({ sentence: 'Mittens naps', wordsByStart: new Map(), end: 7, },);
 ```
 */
export function wordAfterSpace(
  {
    sentence,
    wordsByStart,
    end,
  }: {
    readonly sentence: string;
    readonly wordsByStart: ReadonlyMap<number, WordToken>;
    readonly end: number;
  },
): readonly WordToken[] {
  /**
   Offset of the first character after the whitespace run.
   */
  const next = skipWhitespace({
    sentence,
    start: end,
  },);
  if (next === end) {
    return [];
  }
  /**
   Word that starts at the offset past the whitespace, when one does.
   */
  const word = wordsByStart.get(next,);
  if (word === undefined) {
    return [];
  }
  return [word,];
}

/**
 Whether the sentence has the declared subject name at an offset, compared folded.

 @param sentence - sentence the name may begin in

 @param start - offset where the name would begin

 @param name - declared name, trimmed, empty when none is declared

 @returns Whether the name occupies that offset and is not followed by another word character

 @example
 ```ts
 const named = namesAt({ sentence: 'Mittens naps', start: 0, name: 'Mittens', },);
 ```
 */
export function namesAt(
  {
    sentence,
    start,
    name,
  }: {
    readonly sentence: string;
    readonly start: number;
    readonly name: string;
  },
): boolean {
  if (name.length === 0) {
    return false;
  }
  /**
   Offset just past the name in the sentence.
   */
  const end = start + name.length;
  if (isWordCharacter({
    character: codePointAt({
      text: sentence,
      at: end,
    },),
  },)) {
    return false;
  }
  return foldWord({
    word: sentence.slice(
      start,
      end,
    ),
  },) === foldWord({ word: name, },);
}

/**
 Whether a folded word is an -s ending of at least two letters, as a present verb reads.

 @param word - folded word

 @returns Whether the word is letters only and ends in s after at least one letter

 @example
 ```ts
 const verbLike = isSEndingWord({ word: 'naps', },);
 ```
 */
export function isSEndingWord({ word, }: { readonly word: string; },): boolean {
  if ((word.length < 2) || (!word.endsWith('s',))) {
    return false;
  }
  for (const character of word) {
    if (!isLetterCharacter({ character, },)) {
      return false;
    }
  }
  return true;
}
//endregion Word reading
