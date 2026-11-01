import {
  BECAUSE_CONNECTIVE,
  MAX_INSIGNIFICANT_WORD_LENGTH,
} from './constants.ts';
import { lowerWords, } from './tokenize.ts';

/**
 Tests whether a word contains at least one cased letter.
 
 @param word - single word to inspect
 
 @returns whether word has a cased letter
 
 @example
 ```ts
 wordHasLetter({ word: '304' });   // false
 wordHasLetter({ word: 'HTTP' });  // true
 ```
 */
function wordHasLetter({ word, }: { readonly word: string; },): boolean {
  return word.toLowerCase() !== word.toUpperCase();
}

/**
 Tests whether every cased letter in a word is uppercase; non-letters ignored.
 
 @param word - single word to inspect
 
 @returns whether all cased letters are uppercase
 
 @example
 ```ts
 wordLettersAllUppercase({ word: 'HTTP304' }); // true
 wordLettersAllUppercase({ word: 'Http' });    // false
 ```
 */
function wordLettersAllUppercase({ word, }: { readonly word: string; },): boolean {
  return word === word.toUpperCase();
}

/**
 Tests whether every word that contains letters is fully uppercase, with at
 least one lettered word present. Catches constant-style `NO_REFS`,
 `VALUE IS MISSING`.
 
 @param words - words in original casing
 
 @returns whether all alphabetic words are uppercase
 
 @example
 ```ts
 allAlphabeticWordsUppercase({ words: ['STATE', 'IS', 'UNKNOWN'] }); // true
 allAlphabeticWordsUppercase({ words: ['HTTP', '304', 'modified'] }); // false
 ```
 */
export function allAlphabeticWordsUppercase(
  { words, }: { readonly words: readonly string[]; },
): boolean {
  /**
   Words that contain at least one cased letter.
   */
  const letteredWords = words.filter(function pickLettered(word,): boolean {
    return wordHasLetter({ word, },);
  },);
  if (letteredWords.length === 0)
    return false;
  return letteredWords.every(function allUpper(word,): boolean {
    return wordLettersAllUppercase({ word, },);
  },);
}

/**
 Reduces words to meaningful ones for repetition checks: drops words at or
 under {@link MAX_INSIGNIFICANT_WORD_LENGTH} and, after lowercasing via
 {@link lowerWords}, words repeated from the namespace prefix.
 
 @param words - tokenized words of description
 
 @param namespaceWords - words of namespace prefix
 
 @returns lowercased meaningful words in source order
 
 @example
 ```ts
 meaningfulWords({ words: ['tsdoc', 'no', 'tag'], namespaceWords: ['tsdoc'] }); // ['tag']
 ```
 */
function meaningfulWords(
  {
    words,
    namespaceWords,
  }: {
    readonly words: readonly string[];
    readonly namespaceWords: readonly string[];
  },
): readonly string[] {
  /**
   Lowercased namespace words for prefix-repeat removal.
   */
  const namespaceSet = new Set(lowerWords({ words: namespaceWords, },),);
  return lowerWords({ words, },)
    .filter(function isMeaningful(word,): boolean {
      if (word.length <= MAX_INSIGNIFICANT_WORD_LENGTH)
        return false;
      if (namespaceSet.has(word,))
        return false;
      return true;
    },);
}

/**
 Detects a repeated meaningful word, after reducing via
 {@link meaningfulWords}; repetition adds no debugging information.
 
 @param words - tokenized words of description
 
 @param namespaceWords - words of namespace prefix
 
 @returns whether any meaningful word repeats
 
 @example
 ```ts
 repeatedMeaningfulWord({ words: ['file', 'file', 'exists'], namespaceWords: [] }); // true
 ```
 */
export function repeatedMeaningfulWord(
  {
    words,
    namespaceWords,
  }: {
    readonly words: readonly string[];
    readonly namespaceWords: readonly string[];
  },
): boolean {
  /**
   Meaningful words after dropping short and namespace-repeated words.
   */
  const meaningful = meaningfulWords({
    words,
    namespaceWords,
  },);
  return meaningful.length !== new Set(meaningful,).size;
}

/**
 Detects the narrow `because` failure: the same meaningful phrase appears on
 both sides, as in `file absent because file absent`. Locates the
 {@link BECAUSE_CONNECTIVE} after lowercasing via {@link lowerWords}, then
 compares each side after reducing via {@link meaningfulWords}. Presence of
 `because` alone never exempts a description.
 
 @param words - tokenized words of description
 
 @param namespaceWords - words of namespace prefix
 
 @returns whether both sides of `because` repeat the same phrase
 
 @example
 ```ts
 repeatsSamePhraseAcrossBecause({
   words: ['file', 'absent', 'because', 'file', 'absent'],
   namespaceWords: [],
 }); // true
 ```
 */
export function repeatsSamePhraseAcrossBecause(
  {
    words,
    namespaceWords,
  }: {
    readonly words: readonly string[];
    readonly namespaceWords: readonly string[];
  },
): boolean {
  /**
   Lowercased words for locating the connective.
   */
  const lowered = lowerWords({ words, },);
  /**
   Index of the `because` connective, or -1 when absent.
   */
  const becauseIndex = lowered.indexOf(BECAUSE_CONNECTIVE,);
  if (becauseIndex === (-1))
    return false;
  /**
   Meaningful words left of the connective.
   */
  const leftWords = meaningfulWords({
    words: words.slice(
      0,
      becauseIndex,
    ),
    namespaceWords,
  },);
  /**
   Meaningful words right of the connective.
   */
  const rightWords = meaningfulWords({
    words: words.slice(becauseIndex + 1,),
    namespaceWords,
  },);
  if (leftWords.length !== rightWords.length)
    return false;
  return leftWords.every(function sameWord(
    word,
    index,
  ): boolean {
    return word === rightWords[index];
  },);
}
