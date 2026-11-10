/**
 Shared regex-free text-scanning primitives for agent harness integrations.
 
 @module
 */

export {
  isAlphaNum,
  isDigit,
  isLowerAlpha,
  isUpperAlpha,
  isWhitespace,
  isWordChar,
} from './characters.ts';
export {
  EmptyDelimiterError,
  stripBetweenDelims,
  stripLinesStartingWith,
} from './delimiters.ts';
export { splitWhitespace, } from './splitting.ts';
export {
  containsAnyOfWordBounded,
  containsWordBoundedPhrase,
  PHRASE_NOT_FOUND,
} from './word-boundary.ts';
