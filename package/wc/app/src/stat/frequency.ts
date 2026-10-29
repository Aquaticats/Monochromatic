/**
 Case-insensitive word-frequency computation, over a raw word list such
 as `splitWords`'s output in `./tokenize.ts`.
 */
import type { FrequencyEntry, } from './types.ts';

/**
 Multiplier converting a 0-1 share into a 0-100 percentage.
 */
const PERCENTAGE_SCALE = 100;

/**
 Tallies lowercased word occurrences.
 
 @param words - raw, case-preserved words
 
 @returns lowercased word to occurrence count
 
 @example
 ```ts
 tallyWords(['The', 'the', 'cat']); // Map { 'the' => 2, 'cat' => 1 }
 ```
 */
function tallyWords(words: readonly string[],): Map<string, number> {
  /**
   Lowercased word to occurrence count, built by one pass over words.
   */
  const counts = new Map<string, number>();

  for (const rawWord of words) {
    /**
     Case-folded form of the current word, the bucket key for tallying.
     */
    const word = rawWord.toLowerCase();
    counts.set(
      word,
      (counts.get(word,) ?? 0) + 1,
    );
  }

  return counts;
}

/**
 Computes word-frequency rows from a raw word list, via {@link tallyWords}:
 case-insensitive counts, words appearing once excluded, sorted by count
 descending then alphabetically.
 
 @param words - raw, case-preserved words
 
 @returns frequency rows for words occurring 2 or more times
 
 @example
 ```ts
 computeFrequency(['cat', 'dog', 'cat', 'cat']);
 // [{ word: 'cat', count: 3, percentage: 75 }]
 ```
 */
export function computeFrequency(words: readonly string[],): FrequencyEntry[] {
  /**
   Total word count, the percentage denominator for every entry.
   */
  const total = words.length;
  /**
   Frequency rows for words occurring 2 or more times, collected by one
   pass over {@link tallyWords}'s output.
   */
  const entries: FrequencyEntry[] = [];

  for (const [word, count,] of tallyWords(words,)) {
    if (count >= 2) {
      entries.push(
        {
          word,
          count,
          percentage: total > 0 ? (count / total) * PERCENTAGE_SCALE : 0,
        },
      );
    }
  }

  return entries.toSorted(
    function compareByCountThenWord(
      a: FrequencyEntry,
      b: FrequencyEntry,
    ): number {
      return (b.count - a.count)
        || a.word
          .localeCompare(b.word,);
    },
  );
}
