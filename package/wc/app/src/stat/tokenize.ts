/**
 Pure text-tokenization primitives shared by {@link analyzeText} and
 {@link computeFrequency}: grapheme, word, sentence, line, and paragraph
 splitting, plus a small max-length reducer. No regular expressions;
 Unicode segmentation runs through `Intl.Segmenter`, line/paragraph
 splitting runs through plain string methods.
 */

import type { ForeignBorrowed, } from '@monochromatic-dev/ownership-marker-foreign-borrowed/ts';

/**
 Shared segmenter for grapheme-cluster splitting, via {@link splitGraphemes}.
 */
const GRAPHEME_SEGMENTER = new Intl.Segmenter(
  undefined,
  { granularity: 'grapheme', },
);

/**
 Shared segmenter for word-like splitting, via {@link splitWords}.
 */
const WORD_SEGMENTER = new Intl.Segmenter(
  undefined,
  { granularity: 'word', },
);

/**
 Shared segmenter for sentence splitting, via {@link splitSentences}.
 */
const SENTENCE_SEGMENTER = new Intl.Segmenter(
  undefined,
  { granularity: 'sentence', },
);

/**
 Reads the raw substring off a segment produced by `Intl.Segmenter`, the
 mapper {@link splitGraphemes} passes to `Array.from`.
 
 @param segment - segment to read
 
 @returns substring covered by segment
 */
function segmentText(segment: Readonly<Intl.SegmentData>,): string {
  return segment.segment;
}

/**
 Splits text into grapheme clusters, what a person perceives as a single
 character (correctly handles combining marks and multi-codepoint emoji).
 
 @param text - text to split
 
 @returns grapheme clusters in order
 
 @example
 ```ts
 splitGraphemes('á'); // ['á'] (a + combining acute accent collapses to one grapheme)
 ```
 */
export function splitGraphemes(text: string,): string[] {
  return Array.from(
    GRAPHEME_SEGMENTER.segment(text,),
    segmentText,
  );
}

/**
 Counts grapheme clusters in text, via {@link splitGraphemes}.
 
 @param text - text to measure
 
 @returns grapheme cluster count
 
 @example
 ```ts
 countGraphemes('👨‍👩‍👧'); // 1 (single multi-codepoint emoji grapheme)
 ```
 */
export function countGraphemes(text: string,): number {
  return splitGraphemes(text,)
    .length;
}

/**
 Counts UTF-8 encoded bytes in text.
 
 @param text - text to measure
 
 @returns byte length
 
 @example
 ```ts
 countBytes('café'); // 5 (e-acute encodes as 2 bytes in UTF-8)
 ```
 */
export function countBytes(text: string,): number {
  return new TextEncoder()
    .encode(text,)
    .length;
}

/**
 Splits text into word-like segments, via Unicode word segmentation.
 Punctuation-only and whitespace-only segments are dropped.
 
 @param text - text to split
 
 @returns word-like segments in order
 
 @example
 ```ts
 splitWords('こんにちは、世界'); // ['こんにちは', '世界'] (no spaces needed between CJK words)
 ```
 */
export function splitWords(text: string,): string[] {
  /**
   Word-like segments collected by one pass over the segmenter output.
   */
  const words: string[] = [];

  for (const segment of WORD_SEGMENTER.segment(text,)) {
    if (segment.isWordLike === true) {
      words.push(segment.segment,);
    }
  }

  return words;
}

/**
 Splits text into sentences, via Unicode sentence segmentation. Trims
 surrounding whitespace and drops empty results.
 
 @param text - text to split
 
 @returns trimmed, non-empty sentences in order
 
 @example
 ```ts
 splitSentences('She left. She returned.'); // ['She left.', 'She returned.']
 ```
 */
export function splitSentences(text: string,): string[] {
  /**
   Trimmed, non-empty sentences collected by one pass over the segmenter output.
   */
  const sentences: string[] = [];

  for (const segment of SENTENCE_SEGMENTER.segment(text,)) {
    /**
     Trimmed substring for the current sentence segment.
     */
    const sentence = segment.segment
      .trim();

    if (sentence.length > 0) {
      sentences.push(sentence,);
    }
  }

  return sentences;
}

/**
 Splits text into lines, using editor-style counting: a single trailing
 newline does not add a phantom empty line, but a blank line that exists
 before end-of-text still counts.
 
 @param text - text to split
 
 @returns lines in order, without line terminators
 
 @example
 ```ts
 splitLines('a\nb\n'); // ['a', 'b'] (trailing newline doesn't add a third line)
 splitLines('a\n\n'); // ['a', ''] (blank line before end-of-text still counts)
 ```
 */
export function splitLines(text: string,): string[] {
  if (text.length === 0) {
    return [];
  }

  /**
   Text with every line terminator normalized to a plain `\n`.
   */
  const normalized = text
    .replaceAll(
    '\r\n',
    '\n',
    )
    .replaceAll(
    '\r',
    '\n',
    );
  /**
   Lines split on `\n`, still carrying a trailing empty segment when
   {@link normalized} ends with a newline.
   */
  const segments = normalized.split(
    '\n',
  );

  if (
    normalized.endsWith(
      '\n',
    )
  ) {
    segments.pop();
  }

  return segments;
}

/**
 Reports whether a line is blank (empty or whitespace-only), the
 predicate {@link splitParagraphs} and {@link analyzeText} share to
 detect paragraph breaks and to exclude blank lines from the line count.
 
 @param line - line to test
 
 @returns true when line has no non-whitespace content
 
 @example
 ```ts
 isBlankLine('   '); // true
 ```
 */
export function isBlankLine(line: string,): boolean {
  /**
   Line with surrounding whitespace removed.
   */
  const trimmed = line.trim();

  return trimmed.length === 0;
}

/**
 Splits text into paragraphs, where paragraphs are separated by one or
 more blank (whitespace-only) lines.
 
 @param text - text to split
 
 @returns paragraphs in order, each joining its source lines with a newline
 
 @example
 ```ts
 splitParagraphs('one\ntwo\n\nthree'); // ['one\ntwo', 'three']
 ```
 */
export function splitParagraphs(text: string,): string[] {
  /**
   Paragraphs completed so far, each joining its source lines with a newline.
   */
  const paragraphs: string[] = [];
  /**
   Lines collected for the paragraph currently being built.
   */
  let current: string[] = [];

  for (const line of splitLines(text,)) {
    if (isBlankLine(line,)) {
      if (current.length > 0) {
        paragraphs.push(current.join('\n',),);
        current = [];
      }
    } else {
      current.push(line,);
    }
  }

  if (current.length > 0) {
    paragraphs.push(current.join('\n',),);
  }

  return paragraphs;
}

/**
 Computes the largest length among items, via a caller-supplied length function.
 
 @param items - items to measure
 
 @param lengthOf - length function applied to each item
 
 @returns largest length found, or 0 when items is empty
 
 @example
 ```ts
 computeMaxLength({ items: ['a', 'bb', 'ccc'], lengthOf: (item) => item.length }); // 3
 ```
 */
export function computeMaxLength<T,>(
  {
    items,
    lengthOf,
  }: ForeignBorrowed<Readonly<{
    items: readonly T[];
    lengthOf: (item: T) => number;
  }>>,
): number {
  /**
   Largest length seen so far.
   */
  let max = 0;

  for (const item of items) {
    max = Math.max(
      max,
      lengthOf(item,),
    );
  }

  return max;
}
