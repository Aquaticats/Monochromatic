//region Verdict letter reading
// Deciding when a letter a grader typed is a VERDICT and when it merely began a
// word.
//
// Both sheets ask for `Y` or `N` and both invite prose after it, so
// `N. The original does quote this.` is a verdict with a note while
// `Not enough context to grade` is a refusal that happens to start with the
// same letter. The rule that separates them is one character wide: a letter
// counts only when the sheet ends or a delimiter follows it.
//
// Shared rather than copied. The detection reader had this privately and the
// repair reader needs the identical rule; two implementations of "what did the
// human actually say" would drift, and the drift would show up as a silently
// different denominator rather than as a failure.

/**
 Characters that may follow a verdict letter and still leave it a verdict;
 anything else means the letter merely began a word.
 */
export const VERDICT_DELIMITERS: ReadonlySet<string> = new Set(
  ',.;: ',
);

/**
 Whether an answer opens with a given verdict letter used as a verdict.

 @param answer - grader's answer

 @param letter - verdict letter to test

 @returns True when the letter stands alone or is followed by a delimiter

 @example
 ```ts
 const isYes = opensWithVerdict({ answer: 'Y, but softer', letter: 'Y', },);
 ```
 */
export function opensWithVerdict(
  {
    answer,
    letter,
  }: {
    readonly answer: string;
    readonly letter: string;
  },
): boolean {
  if (!answer.startsWith(letter,))
    return false;
  if (answer.length === letter.length)
    return true;
  return VERDICT_DELIMITERS.has(answer.charAt(letter.length,),);
}

/**
 Drops the punctuation and spacing separating a verdict letter from the prose
 after it.

 A linear scan rather than a pattern: the rule is "skip while the character is
 a delimiter", and the delimiter set is already named.

 @param text - answer remainder after the verdict letter

 @returns Remainder with leading delimiters and surrounding space removed

 @example
 ```ts
 const note = trimLeadingDelimiters({ text: ', anchored to the wrong text', },);
 ```
 */
export function trimLeadingDelimiters({ text, }: { readonly text: string; },): string {
  for (let index = 0; index < text.length; index += 1)
    if (!VERDICT_DELIMITERS.has(text.charAt(index,),))
      return text.slice(index,)
        .trim();
  return '';
}

//endregion Verdict letter reading
