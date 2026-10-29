/**
 Aggregate text statistics computed from a single input string.
 */
export type TextStats = {
  /**
   UTF-8 encoded byte length of the input.
   */
  readonly bytes: number;
  /**
   Grapheme cluster count of the input.
   */
  readonly chars: number;
  /**
   UTF-8 encoded byte length of the widest grapheme cluster.
   */
  readonly maxCharLength: number;
  /**
   Line count, using editor-style counting: a single trailing newline
   does not add a phantom empty line.
   */
  readonly lines: number;
  /**
   Grapheme cluster length of the longest line.
   */
  readonly maxLineLength: number;
  /**
   Word count, using Unicode word segmentation.
   */
  readonly words: number;
  /**
   Grapheme cluster length of the longest word.
   */
  readonly maxWordLength: number;
  /**
   Sentence count, using Unicode sentence segmentation.
   */
  readonly sentences: number;
  /**
   Word count of the longest sentence.
   */
  readonly maxSentenceLength: number;
  /**
   Paragraph count, where paragraphs are separated by one or more blank lines.
   */
  readonly paragraphs: number;
  /**
   Sentence count of the longest paragraph.
   */
  readonly maxParagraphLength: number;
};

/**
 Single row in the word-frequency table.
 */
export type FrequencyEntry = {
  /**
   Lowercased word this entry counts.
   */
  readonly word: string;
  /**
   Number of times the word occurs in the input, case-insensitively.
   */
  readonly count: number;
  /**
   Share of all words in the input this word accounts for, as a
   percentage between 0 and 100.
   */
  readonly percentage: number;
};
