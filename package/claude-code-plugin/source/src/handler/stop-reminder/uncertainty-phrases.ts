/**
 Phrase lists and word-bounded scans used by the uncertainty detection
 engine. Apostrophe normalisation is centralised here so every consumer
 folds curly quotes to ASCII before lookup.
 
 @module
 */

import { isWordChar, } from '@monochromatic-dev/agent-harness-shared-text-scan/ts';

//region Apostrophe normalisation

/**
 Unicode left single quotation mark (`U+2018`).
 */
const LEFT_SINGLE_QUOTE = '‘';

/**
 Unicode right single quotation mark (`U+2019`).
 */
const RIGHT_SINGLE_QUOTE = '’';

/**
 Folds curly single quotation marks to ASCII apostrophes so phrase lookups
 only need the straight-apostrophe variant of each entry.
 
 @param text - input text
 
 @returns text with curly single quotes replaced by `'`
 
 @example
 ```ts
 normaliseApostrophes('that’s wrong'); // => "that's wrong"
 ```
 */
function normaliseApostrophes(text: string,): string {
  return text
    .replaceAll(
    LEFT_SINGLE_QUOTE,
    "'",
    )
    .replaceAll(
    RIGHT_SINGLE_QUOTE,
    "'",
    );
}

//endregion

//region Uncertainty phrase list

/**
 Lowercase phrases that indicate hedging or uncertain language in prose.
 
 Each entry expands one alternation of the original regex array:
 
 - Modal hedges (probably, maybe, perhaps, possibly, presumably, likely)
 - Epistemic hedges (I think, I believe, I assume, I suspect, ...)
 - Conditional hedges (might be, could be, should be)
 - Uncertainty markers (not sure, hard to say, difficult to tell)
 - Approximation markers (if I recall, from what I recall, as far as I know)
 - Comparative hedges ("worse than most", "more than most", "less than most")
 - Admitted gaps (I didn't check, I haven't measured, I didn't try, test gap, not verified); the bare
   adjectives `untested`, `unverified`, `unexplained` and `unmeasured` are left out because they mostly
   describe a thing, not the agent's own gap

 The `\wer than most` variant is handled separately by
 {@link containsErThanMost} since it requires a word-character prefix the
 word-bounded-phrase scan cannot express.
 */
const UNCERTAINTY_PHRASES: readonly string[] = [
  'probably',
  'maybe',
  'may be',
  'may only be',
  'worth testing',
  'perhaps',
  'possibly',
  'presumably',
  'presume',
  'i think',
  'i believe',
  'i assume',
  'i suspect',
  'i imagine',
  'i guess',
  'i suppose',
  'might be',
  'could be',
  'should be',
  'not sure',
  'not entirely sure',
  'not certain',
  'hard to say',
  'hard to tell',
  'difficult to say',
  'difficult to tell',
  'if i recall',
  'if i remember',
  'from what i recall',
  'from what i remember',
  'as far as i know',
  'as far as i can tell',
  'likely',
  'worse than most',
  'more than most',
  'less than most',
  // Admitted gaps: the agent says it skipped a check, which is an uncertainty it can still close.
  "i didn't check",
  'i did not check',
  "i haven't checked",
  'i have not checked',
  "i didn't verify",
  'i did not verify',
  "i haven't verified",
  'i have not verified',
  "i didn't read",
  'i did not read',
  "i haven't read",
  'i have not read',
  "i didn't test",
  'i did not test',
  "i haven't tested",
  'i have not tested',
  "i didn't confirm",
  'i did not confirm',
  "i haven't confirmed",
  'i have not confirmed',
  "i didn't run",
  'i did not run',
  "i haven't run",
  'i have not run',
  'not verified',
  'not checked',
  'not tested',
  // Admitted gaps in evidence-gathering steps beyond check and verify.
  "i didn't measure",
  'i did not measure',
  "i haven't measured",
  'i have not measured',
  "i didn't try",
  'i did not try',
  "i haven't tried",
  'i have not tried',
  "i didn't look",
  'i did not look',
  "i haven't looked",
  'i have not looked',
  "i didn't reproduce",
  'i did not reproduce',
  "i haven't reproduced",
  'i have not reproduced',
  "i didn't rerun",
  'i did not rerun',
  "i didn't re-run",
  'i did not re-run',
  "i haven't rerun",
  'i have not rerun',
  "i haven't re-run",
  'i have not re-run',
  "i haven't shown",
  'i have not shown',
  // Report labels that name an open gap.
  'test gap',
  'not viewed',
];

//endregion

//region Comparative `\wer than most` scan

/**
 Literal substring scanned for when looking for comparative-suffix `<X>er than most` hedges.
 */
const ER_THAN_MOST_PHRASE = 'er than most';

/**
 Sentinel returned by {@link findErThanMost} when no comparative fragment is found.
 
 A unique symbol rather than `undefined`: callers narrow on identity so the
 matched-fragment string never shares a nullish union with "no match".
 */
const ER_NOT_FOUND: unique symbol = Symbol('uncertainty-phrases/er-not-found',);

/**
 Looks up the first `<word>er than most` occurrence in `text`, returning
 the matched word fragment (e.g. `'bigger than most'`).
 
 Mirrors the original regex `/\wer than most\b/i`: scans for the literal
 `er than most`, checks that the character preceding the `e` is a word
 character (so `'inner than most'` matches but `' er than most'` does
 not), and that the trailing `most` ends at a word boundary.
 
 @param text - input text to scan
 
 @returns matched fragment from `<word>er` through `most`, or `ER_NOT_FOUND` when no match exists
 
 @example
 ```ts
 findErThanMost('this is bigger than most lengths'); // => 'bigger than most'
 findErThanMost('just er than most');                // => ER_NOT_FOUND (no word prefix)
 ```
 */
function findErThanMost(text: string,): string | typeof ER_NOT_FOUND {
  /**
   Lower-cased text used for the case-insensitive substring scan.
   */
  const lower = text.toLowerCase();
  // Walk every `er than most` occurrence in order (monotonic `indexOf`, no
  // rescan of earlier text). The first occurrence with a word-char prefix and a
  // word boundary after `most` yields the matched fragment; others are skipped.
  for (
    let idx = lower.indexOf(
      ER_THAN_MOST_PHRASE,
      0,
    );
    idx !== (-1);
    idx = lower.indexOf(
      ER_THAN_MOST_PHRASE,
      idx + 1,
    )
  ) {
    if (idx === 0)
      continue;
    /**
     Character immediately before the `e`; must be a word char for the match.
     */
    const before = lower.charAt(idx - 1,);
    if (!isWordChar(before,))
      continue;
    /**
     Position one past the trailing `t` of `most`; checked for a word boundary below.
     */
    const endIdx = idx + ER_THAN_MOST_PHRASE
      .length;
    if ((endIdx < lower
      .length) && isWordChar(lower.charAt(endIdx,),))
      continue;
    /**
     Inclusive start of the word that ends in `er`, found by scanning back over word chars.
     */
    let wordStart = idx - 1;
    while ((wordStart > 0) && isWordChar(text.charAt(wordStart - 1,),)) {
      wordStart -= 1;
    }
    return text.slice(
      wordStart,
      endIdx,
    );
  }
  return ER_NOT_FOUND;
}

/**
 Whether `text` contains any `<word>er than most` occurrence.
 
 @param text - input text
 
 @returns whether the comparative hedge appears
 
 @example
 ```ts
 containsErThanMost('this is faster than most options'); // true
 containsErThanMost('all good');                         // false
 ```
 */
function containsErThanMost(text: string,): boolean {
  return findErThanMost(text,)
    !== ER_NOT_FOUND;
}

//endregion

//region Dismissal phrase list

/**
 Lowercase phrases for categorical dismissals that a citation must
 accompany. Apostrophe-bearing variants are paired with their
 no-apostrophe forms (`don't` / `dont`) to mirror the original regex
 `['']?` markers; curly quotes in input are folded via
 {@link normaliseApostrophes} before lookup.
 */
const DISMISSAL_PHRASES: readonly string[] = [
  // \b(?:the )?project doesn['']?t use\b
  "the project doesn't use",
  'the project doesnt use',
  "project doesn't use",
  'project doesnt use',
  // \bwe don['']?t use\b
  "we don't use",
  'we dont use',
  // \b(?:the )?codebase doesn['']?t (?:use|have)\b
  "the codebase doesn't use",
  "the codebase doesn't have",
  'the codebase doesnt use',
  'the codebase doesnt have',
  "codebase doesn't use",
  "codebase doesn't have",
  'codebase doesnt use',
  'codebase doesnt have',
  // \bdoesn['']?t apply here\b
  "doesn't apply here",
  'doesnt apply here',
  // \bis already (?:handled|covered) by\b
  'is already handled by',
  'is already covered by',
];

//endregion

export {
  containsErThanMost,
  DISMISSAL_PHRASES,
  ER_NOT_FOUND,
  findErThanMost,
  normaliseApostrophes,
  RIGHT_SINGLE_QUOTE,
  UNCERTAINTY_PHRASES,
};
