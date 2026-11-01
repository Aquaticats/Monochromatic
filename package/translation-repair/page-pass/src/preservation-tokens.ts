import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';

import {
  codePointAt,
  continuesLatinWord,
  isHanCharacter,
  isLatinCapital,
} from '@monochromatic-dev/translation-repair-markup/ts';

//region Preservation tokens
// Tokenizing and proper-noun scanning for the preservation gate, as index
// scans rather than regexes. The rules here are positional (is this character a
// letter, is this capital the first word of a sentence), which an index scan
// states directly and a regex only encodes.

/**
 Words too common to carry content, so their loss says nothing about whether
 an edit deleted anything.

 Deliberately short. A long stop list would start removing words whose
 disappearance IS the damage, and the gate's whole job is noticing loss.
 */
const STOP_WORDS: ReadonlySet<string> = new Set([
  'a',
  'an',
  'the',
  'and',
  'or',
  'but',
  'of',
  'to',
  'in',
  'on',
  'at',
  'for',
  'with',
  'is',
  'was',
  'were',
  'are',
  'be',
  'been',
  'as',
  'it',
  'its',
  'she',
  'her',
  'he',
  'his',
  'they',
  'them',
  'their',
  'this',
  'that',
  'these',
  'those',
  'from',
  'by',
  'not',
  'no',
  'so',
  'if',
  'then',
  'than',
  'very',
  'also',
  'had',
  'have',
  'has',
  'do',
  'did',
  'done',
  'you',
  'your',
  'we',
  'our',
  'my',
  'me',
],);

/**
 Characters that end a sentence, in both ASCII and full-width forms.

 A COLON IS NOT HERE, deliberately. A name following a colon is still a proper
 noun, and treating the colon as a terminator is what made a deleted
 contributor name invisible to an earlier draft of this gate.
 */
const SENTENCE_ENDS = '.!?。！？';

/**
 Characters that may sit between a sentence end and its first word.
 */
const LEADING_MARKS = ' \t\n\r"“‘\'(>[-*';

/**
 Shortest capitalized run treated as a possible name.

 Two-letter capitals are overwhelmingly initials and abbreviations, whose
 disappearance an ordinary rewrite explains.
 */
const MIN_NAME_LENGTH = 3;

/**
 Reports whether a character belongs to a word: a Latin letter (accented or
 not), an ASCII digit, a combining mark, an apostrophe or a hyphen. With
 ASCII letters only, `Émile` lost its first letter and `Château` split in two
 (ledger B18).

 @param character - single character

 @returns True for a character a word runs on through

 @example
 ```ts
 const isWord = isWordCharacter('a',);
 ```
 */
function isWordCharacter(character: string,): boolean {
  return continuesLatinWord({ character, },)
    || (character === '\'')
    || (character === '-');
}

/**
 Reports whether a character is a CJK ideograph, which tokenizes one per
 character rather than by word run: the floors' own Han test
 (`han-only-text.ts`), so the tokenizer and the floors read one page one way.

 @param character - single character

 @returns True for a CJK ideograph

 @example
 ```ts
 const isHan = isIdeograph('家',);
 ```
 */
export function isIdeograph(character: string,): boolean {
  return isHanCharacter({ character, },);
}

/**
 Splits text into comparable content tokens.

 Stop words drop out, one-character words drop out, and ideographs survive
 individually because a one-character Chinese token is a full word.

 @param text - text to tokenize

 @returns Lowercased content tokens in document order

 @example
 ```ts
 const tokens = contentTokens({ text: 'She kept the Klipper videos', },);
 ```
 */
export function contentTokens(
  {
    text,
  }: {
    readonly text: string;
  },
): readonly string[] {
  /**
   Tokens accumulated across the scan.
   */
  const tokens: string[] = [];

  /**
   Start of the word run being read, or -1 between runs.
   */
  let runStart = -1;

  // BY CODE POINT, NOT BY UTF-16 UNIT (ledger B21): an ideograph beyond the
  // first plane is two units, neither of which passes the Han test, so a scan
  // by unit dropped it. Offsets stay UTF-16 offsets, so a word run slices as
  // before; the terminator past the end steps one to close the loop.
  for (let index = 0; index <= text.length;) {
    /**
     Whole character at this position, or a terminator past the end.
     */
    const character = codePointAt({
      text,
      at: index,
    },);

    /**
     Offset of the next character.
     */
    const next = index + Math.max(
      character.length,
      1,
    );

    if ((index < text.length) && isWordCharacter(character,)) {
      if (runStart < 0)
        runStart = index;
      index = next;
      continue;
    }

    if (runStart >= 0) {
      /**
       Completed word run, lowercased for comparison.
       */
      const word = text
        .slice(
        runStart,
        index,
        )
        .toLowerCase();
      if ((word.length > 1) && (!STOP_WORDS.has(word,)))
        tokens.push(word,);
      runStart = -1;
    }

    if ((index < text.length) && isIdeograph(character,))
      tokens.push(character,);
    index = next;
  }

  return tokens;
}

/**
 Finds capitalized words that are NOT the first word of a sentence.

 Sentence-initial capitals are ordinary words wearing a capital, and treating
 them as names made an earlier draft reject an edit for losing "Yet" and
 "Moreover".

 SCANNED ON THE ORIGINAL TEXT, never on the text left after removing the
 licensed defect quote. Removing a quote can leave a real name sitting at what
 looks like the start of a sentence, which is exactly how a deleted
 contributor name escaped an earlier draft.

 @param text - original text, with its sentence structure intact

 @returns Lowercased proper nouns

 @example
 ```ts
 const names = properNouns({ text: 'Contributor: Bilibi - the archive', },);
 ```
 */
export function properNouns(
  {
    text,
  }: {
    readonly text: string;
  },
): ReadonlySet<string> {
  /**
   Names found so far.
   */
  const names = new Set<string>();

  for (let index = 0; index < text.length; index += 1) {
    /**
     Character at this position.
     */
    const character = nonNullishOrThrow(text[index],);
    if (!isLatinCapital({ character, },))
      continue;
    if ((index > 0) && isWordCharacter(nonNullishOrThrow(text[index - 1],),))
      continue;

    /**
     End of this capitalized run.
     */
    let end = index;
    while ((end < text.length) && isWordCharacter(nonNullishOrThrow(text[end],),))
      end += 1;
    if ((end - index) < MIN_NAME_LENGTH) {
      index = end;
      continue;
    }

    /**
     Position of the last meaningful character before this word.
     */
    let back = index - 1;
    while ((back >= 0) && LEADING_MARKS.includes(nonNullishOrThrow(text[back],),))
      back -= 1;

    // Start of text, or straight after a sentence end, means this capital is
    // positional rather than a name.
    if ((back >= 0) && (!SENTENCE_ENDS.includes(nonNullishOrThrow(text[back],),))) {
      names.add(text
        .slice(
        index,
        end,
        )
        .toLowerCase(),);
    }
    index = end;
  }

  return names;
}

//endregion Preservation tokens
