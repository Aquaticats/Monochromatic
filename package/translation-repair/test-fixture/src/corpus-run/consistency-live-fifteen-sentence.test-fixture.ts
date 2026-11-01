/**
 The four live judges' ballots for the maximal repeat of the live artifact of
 generation fifteen (the entry TianqiChen666): two quote the repeated sentence
 with its closing full stop and two without, and the last judge chooses the
 place the other three do not. The judges' reasons are abridged to the sentences
 that decide, since the reader checks a reason only for blankness.

 @module
 */

/**
 The quotes the four live judges gave for the maximal repeat, two with the closing full stop and two without.
 */
export const SENTENCE_BALLOTS = [
  {
    seat: 'hf:Qwen/Qwen3.8-27B',
    reason: 'The two English renderings differ only in wording, not meaning. Occurrence 1 is the more faithful because it keeps the idea of being by people\'s side, looking cute, and bringing sunshine to more people. The "also" in Occurrence 1 belongs to the preceding 也是, so the repeated wording itself should follow Occurrence 1 without that adverb.',
    quotes: [
      {
        label: 'OCCURRENCE 1',
        quote: 'She also wanted to stay by more people\'s side looking cute and bring sunshine to more people',
      },
      {
        label: 'OCCURRENCE 2',
        quote: 'She wanted to accompany more people with her cuteness and bring sunshine to their lives',
      },
    ],
    choice: 'OCCURRENCE 1',
    keep: [],
  },
  {
    seat: 'hf:moonshotai/Kimi-K3',
    reason: 'The two renderings differ only in wording, not meaning. OCCURRENCE 1 is the most faithful, since it keeps both occurrences of 更多人 and renders 陪伴在身边 as \'stay by more people\'s side\', while OCCURRENCE 2 glosses away the second 更多人.',
    quotes: [
      {
        label: 'OCCURRENCE 1',
        quote: 'She also wanted to stay by more people\'s side looking cute and bring sunshine to more people',
      },
      {
        label: 'OCCURRENCE 2',
        quote: 'She wanted to accompany more people with her cuteness and bring sunshine to their lives',
      },
    ],
    choice: 'OCCURRENCE 1',
    keep: [],
  },
  {
    seat: 'deepseek-v4.1-flash',
    reason: 'The two renderings carry the same meaning but differ in wording: OCCURRENCE 1 is idiomatic English that keeps both halves of the wording, while OCCURRENCE 2 is a word-for-word calque and drops the second 更多人 into \'their lives\'. Neither occurrence\'s surroundings force a different wording, so nothing is kept.',
    quotes: [
      {
        label: 'OCCURRENCE 1',
        quote: 'She also wanted to stay by more people\'s side looking cute and bring sunshine to more people.',
      },
      {
        label: 'OCCURRENCE 2',
        quote: 'She wanted to accompany more people with her cuteness and bring sunshine to their lives.',
      },
    ],
    choice: 'OCCURRENCE 1',
    keep: [],
  },
  {
    seat: 'inception/mercury-2.5',
    reason: 'The occurrences render the repeated phrase differently in wording and style. OCCURRENCE 2 is more faithful to the ORIGINAL, avoiding the added \'also\' and keeping a smoother phrasing that matches the memorial tone.',
    quotes: [
      {
        label: 'OCCURRENCE 1',
        quote: 'She also wanted to stay by more people\'s side looking cute and bring sunshine to more people.',
      },
      {
        label: 'OCCURRENCE 2',
        quote: 'She wanted to accompany more people with her cuteness and bring sunshine to their lives.',
      },
    ],
    choice: 'OCCURRENCE 2',
    keep: [],
  },
] as const;
