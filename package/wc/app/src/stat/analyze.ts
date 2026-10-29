/**
 Aggregate text-statistics computation, combining every tokenizer in
 {@link ./tokenize.ts} into a single {@link TextStats} snapshot.
 */
import {
  computeMaxLength,
  countBytes,
  countGraphemes,
  isBlankLine,
  splitGraphemes,
  splitLines,
  splitParagraphs,
  splitSentences,
  splitWords,
} from './tokenize.ts';
import type { TextStats, } from './types.ts';

/**
 Word count of a sentence, the length function {@link analyzeText} passes
 to {@link computeMaxLength} for {@link TextStats.maxSentenceLength}.
 
 @param sentence - sentence to measure
 
 @returns word count within sentence
 */
function countWordsInSentence(sentence: string,): number {
  return splitWords(sentence,)
    .length;
}

/**
 Sentence count of a paragraph, the length function {@link analyzeText}
 passes to {@link computeMaxLength} for {@link TextStats.maxParagraphLength}.
 
 @param paragraph - paragraph to measure
 
 @returns sentence count within paragraph
 */
function countSentencesInParagraph(paragraph: string,): number {
  return splitSentences(paragraph,)
    .length;
}

/**
 Computes every {@link TextStats} field for text, in one pass over each
 tokenizer's output.
 
 @param text - text to analyze
 
 @returns aggregate statistics
 
 @example
 ```ts
 analyzeText('Hi there.\nBye.');
 // { bytes: 14, chars: 14, maxCharLength: 1, lines: 2, maxLineLength: 9,
 //   words: 3, maxWordLength: 5, sentences: 2, maxSentenceLength: 2,
 //   paragraphs: 1, maxParagraphLength: 2 }
 ```
 */
export function analyzeText(text: string,): TextStats {
  /**
   Grapheme clusters, via {@link splitGraphemes}, reused for both the
   char count and {@link TextStats.maxCharLength}.
   */
  const graphemes = splitGraphemes(text,);
  /**
   Non-blank lines, via {@link splitLines} filtered through
   {@link isBlankLine}, reused for both the line count and
   {@link TextStats.maxLineLength}. Blank lines still separate
   paragraphs (see {@link splitParagraphs}); they just don't count as
   lines here.
   */
  const lines = splitLines(text,)
    .filter(function isCountable(line,): boolean {
      return !isBlankLine(line,);
    },);
  /**
   Word-like segments, via {@link splitWords}, reused for both the word
   count and {@link TextStats.maxWordLength}.
   */
  const words = splitWords(text,);
  /**
   Sentences, via {@link splitSentences}, reused for both the sentence
   count and {@link TextStats.maxSentenceLength}.
   */
  const sentences = splitSentences(text,);
  /**
   Paragraphs, via {@link splitParagraphs}, reused for both the paragraph
   count and {@link TextStats.maxParagraphLength}.
   */
  const paragraphs = splitParagraphs(text,);

  return {
    bytes: countBytes(text,),
    chars: graphemes.length,
    maxCharLength: computeMaxLength({
      items: graphemes,
      lengthOf: countBytes,
    },),
    lines: lines.length,
    maxLineLength: computeMaxLength({
      items: lines,
      lengthOf: countGraphemes,
    },),
    words: words.length,
    maxWordLength: computeMaxLength({
      items: words,
      lengthOf: countGraphemes,
    },),
    sentences: sentences.length,
    maxSentenceLength: computeMaxLength({
      items: sentences,
      lengthOf: countWordsInSentence,
    },),
    paragraphs: paragraphs.length,
    maxParagraphLength: computeMaxLength({
      items: paragraphs,
      lengthOf: countSentencesInParagraph,
    },),
  };
}
