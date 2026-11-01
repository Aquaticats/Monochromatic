import {
  countSpan,
  grownSpans,
  indexWindows,
} from './assembly-repetition-span.ts';
import { wordForCount, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Assembly repetition
// DAMAGE THAT IS ONLY VISIBLE IN THE WHOLE DOCUMENT, which every per-slice
// instrument in this package is structurally blind to.
//
// The introduced-defect probe was measured against two defects known to have
// shipped and it named neither. One was an ordinary miss. The other,
// `lintong`'s duplicated farewell, could not have been caught by that probe at
// any setting: it compares one edited region against itself before the edit,
// and the duplication is inside no single region. Slice 3 kept wording it was
// not asked to remove, slice 2 wrote a fresh rendering of the same thing, and
// each edit is defensible alone. Only the assembled document shows it.
//
// NO MODEL, NO ROSTER, NO QUOTA. The comparison that makes this decidable is
// the ARCHIVE, which the artifact stores whole beside its preparation hash.
// A phrase the archive says once and the shipped document says twice is a
// repetition this pipeline introduced. That is a fact about two strings.
//
// WHY THE ARCHIVE IS THE BASELINE RATHER THAN A RULE ABOUT PROSE. Real writing
// repeats: refrains, names, deliberate echoes. A standalone "says it twice"
// check would fire on all of them. Counting against the archive asks the only
// question worth asking, which is whether the pipeline ADDED a repetition, and
// it inherits the author's own judgement about acceptable repetition for free.

/**
 Shortest phrase worth reporting, in words.

 Below four words the matches are ordinary English collocations rather than
 passages, and every document produces them by the dozen.
 */
const MIN_PHRASE_WORDS = 4;

/**
 Letters a word needs before it counts as carrying content.

 Function words are short and they cluster, so a run of them says nothing
 about whether a passage repeated.
 */
const CONTENT_WORD_LETTERS = 5;

/**
 Content words a phrase needs before it is worth reporting.

 MEASURED RATHER THAN CHOSEN, on the five settled artifacts. Counting by words
 alone, this returned five findings, and their content-word counts separate
 them sharply: the duplication established by reading `lintong`'s
 finished text carries THREE content words in 33 characters, while two of the
 others carry ZERO and ONE. Six words with no word longer than four letters is
 an ordinary English collocation that any two paragraphs may share, not a
 passage a document said twice.

 WHAT TWO ACTUALLY COSTS, measured rather than predicted, because the first
 version of this comment guessed and guessed wrong. It takes the five findings
 to two: `lintong`'s documented duplication survives at 3 content words, and
 `saurikissa`'s translate-lane repeat survives at 3. Dropped are two runs of
 function words at 0 and 1, and `dogesir_`'s invented-and-repeated passage,
 which carries only 1 content word in 21 characters.

 DROPPING `dogesir_` IS ACCEPTED rather than worked around. A five-word phrase
 with one substantial word is short enough that two paragraphs sharing it is
 unremarkable, and a stage emitting on every run should be quiet enough that a
 finding means something. If document-scale damage is later found that this
 threshold hid, lower it and say so here.
 */
const MIN_CONTENT_WORDS = 2;

/**
 Longest phrase considered, in words.

 A longer repeat is still reported, and reported ONCE. Growth stops here, so
 a passage longer than this spans several windows of exactly this length, and
 a suppression rule that only drops a phrase contained in a longer one cannot
 merge them: they are all the same length, so no one of them contains
 another. That cost was measured: an 877-word duplication reported as
 866 findings, and `assembly-repetition-span.ts` now grows this layer's
 windows into the passages they belong to before any of them is reported.
 */
const MAX_PHRASE_WORDS = 12;

/**
 Word start marking that the tokenizer stands between words.
 */
const NO_WORD = -1;

/**
 One passage the shipped document repeats more than the archive did.

 @example
 ```ts
 const finding: RepetitionFinding = { phrase: 'the same thing twice', archiveCount: 1, shippedCount: 2, };
 ```
 */
export type RepetitionFinding = {
  /**
   Repeated wording, normalised to single spaces.
   */
  readonly phrase: string;

  /**
   Times the archive said it, which may be zero for wording the pipeline wrote.
   */
  readonly archiveCount: number;

  /**
   Times the shipped document says it.
   */
  readonly shippedCount: number;
};

/**
 Whether one space-joined phrase holds another as whole words.

 PADDED ON BOTH SIDES before the substring test, so a phrase that is a
 character substring of a longer one across a word boundary (`at the garden
 gate` inside `cat the garden gates`) is not taken for a part of it. Both
 phrases come from `whitespaceTokensOf` joined with single spaces, which is what makes
 the space a word boundary here.

 @param longer - phrase that may hold the other

 @param phrase - phrase looked for as whole words

 @returns Whether every word of `phrase` appears in `longer` in order, as words

 @example
 ```ts
 holdsPhrase({ longer: 'cat the garden gates', phrase: 'at the garden gate', },); // false
 ```
 */
export function holdsPhrase(
  {
    longer,
    phrase,
  }: {
    readonly longer: string;
    readonly phrase: string;
  },
): boolean {
  return ` ${longer} `.includes(` ${phrase} `,);
}

/**
 Splits text into words, collapsing every run of whitespace.

 LINE STRUCTURE IS DELIBERATELY DISCARDED. Shipped text is wrapped
 semantically, so the same sentence carries different newlines before and
 after the pipeline runs, and a comparison that kept them would report every
 rewrapped paragraph as a change.

 PUNCTUATION RIDES ON ITS TOKEN, deliberately. Splitting it off would make
 `soon.` and `soon,` the same word, which merges a sentence ending with one
 continuing, and the repeats this looks for are whole passages rather than
 near-matches. The cost is that a passage repeated with different closing
 punctuation is two phrases; the benefit is that nothing is reported as
 repeated which is not repeated verbatim.

 @param text - document or passage

 @returns Words in order

 @example
 ```ts
 const words = whitespaceTokensOf({ text: 'the kitten dozes', },);
 ```

 Shared with the adjacency check; not part of the lane contract.

 @internal
 */
export function whitespaceTokensOf({ text, }: { readonly text: string; },): readonly string[] {
  /**
   Words closed so far.
   */
  const words: string[] = [];

  /**
   Where the word being read starts, `NO_WORD` between words.
   */
  let start = NO_WORD;

  /**
   Offset of the character being read, in UTF-16 units.
   */
  let at = 0;

  // A SINGLE LINEAR PASS rather than a split on a whitespace pattern, per `RG1`:
  // the rule is "break on whitespace", which an index scan states directly, and
  // this runs in O(n) with no backtracking to reason about on document-sized
  // input. Each word is sliced once where it ends rather than grown a
  // character at a time, per `RG2` (ledger B29).
  for (const character of text) {
    if (character.trim() === '') {
      if (start !== NO_WORD) {
        words.push(text.slice(
          start,
          at,
        ),);
      }
      start = NO_WORD;
    }
    else if (start === NO_WORD) {
      start = at;
    }
    at += character.length;
  }
  if (start !== NO_WORD)
    words.push(text.slice(start,),);
  return words;
}

/**
 Whether a phrase carries enough substantial words to be worth reporting.

 Letters are counted with an index scan rather than a pattern, per `RG1`: the
 rule is "how many letters does this word have", which a scan states directly.

 @param phrase - candidate repeated wording

 @returns Whether it clears {@link MIN_CONTENT_WORDS}

 @example
 ```ts
 const worth = carriesContent({ phrase: 'the tabby waits by the gate', },);
 ```
 */
function carriesContent({ phrase, }: { readonly phrase: string; },): boolean {
  /**
   Words long enough to carry meaning rather than grammar.

   LENGTH RATHER THAN A LETTER COUNT, and the reason is a rule conflict rather
   than a preference. Counting letters needs the word walked character by
   character, and this package's linters refuse both spellings of that:
   spreading a string is refused for splitting graphemes, and `Array.from` is
   refused in favour of spread. `LN1` says an apparent conflict gets a
   structural answer rather than one rule's surface reshaped to quiet the
   other, so the character walk goes away.

   WHAT IT COSTS is punctuation counting toward length, so a four-letter word
   carrying a comma reads as five. Measured on the five settled artifacts this
   changes nothing: the same two findings survive and the same three are
   dropped.
   */
  const substantial = phrase
    .split(' ',)
    .filter(function longEnough(word,): boolean {
      return word.length >= CONTENT_WORD_LETTERS;
    },);

  return substantial.length >= MIN_CONTENT_WORDS;
}


/**
 Counts every phrase of one length in a word list.

 @param words - words to walk

 @param length - phrase length in words

 @returns Occurrence count per phrase

 @example
 ```ts
 const counts = countPhrases({ words, length: 4, },);
 ```

 Shared with the adjacency check; not part of the lane contract.

 @internal
 */
export function countPhrases(
  {
    words,
    length,
  }: {
    readonly words: readonly string[];
    readonly length: number;
  },
): ReadonlyMap<string, number> {
  /**
   Occurrences seen so far, by phrase.
   */
  const counts = new Map<string, number>();
  for (let at = 0; (at + length) <= words.length; at += 1) {
    /**
     This window, as one comparable string.
     */
    const phrase = words
      .slice(
      at,
      at + length,
      )
      .join(' ',);

    /**
     Times this phrase has been seen before now.
     */
    const seen = counts.get(phrase,) ?? 0;
    counts.set(
      phrase,
      seen + 1,
    );
  }
  return counts;
}

/**
 Names passages the shipped document repeats more often than the archive did.

 MAXIMAL MATCHES ONLY. A repeated twelve-word passage also repeats as nine
 four-word ones, and reporting all of them would bury the finding in its own
 substrings. Lengths are walked longest first and a shorter phrase contained
 in one already reported is dropped.

 @param archiveText - translation as it stood before the pipeline ran, from
 the artifact's stored archive

 @param shippedText - assembled document a lane produced

 @returns Introduced repetitions, longest first

 @example
 ```ts
 const findings = findIntroducedRepetitions({ archiveText, shippedText, },);
 ```
 */
export function findIntroducedRepetitions(
  {
    archiveText,
    shippedText,
  }: {
    readonly archiveText: string;
    readonly shippedText: string;
  },
): readonly RepetitionFinding[] {
  /**
   Archive as a word list, so phrase counting sees the same units on both
   sides.
   */
  const archiveWords = whitespaceTokensOf({ text: archiveText, },);

  /**
   {@inheritDoc archiveWords}
   */
  const shippedWords = whitespaceTokensOf({ text: shippedText, },);

  /**
   Findings so far, longest first.
   */
  const found: RepetitionFinding[] = [];

  /**
   Passages that suppress their own substrings, which is NOT the same list.

   Every finding suppresses, but not everything that suppresses is a finding.
   A grown span whose occurrences are all inside a longer span already
   reported describes no second duplication, so it earns no finding; the
   shorter phrases inside it are derivative for exactly the same reason, so it
   still has to suppress them. Keeping one list for both was measured to
   report one duplication as eleven findings, the pieces reappearing at every
   shorter length once the span holding them stopped being reported.
   */
  const covering: string[] = [];

  /**
   Windows of the longest length considered, indexed both ways.

   THIS LAYER IS HANDLED APART FROM THE REST, because it is the only one that
   can over-report. Growth stops at {@link MAX_PHRASE_WORDS}, so a passage
   longer than that spans several windows of this length, and containment
   cannot suppress one same-length window with another. Every shorter layer is
   already covered by the containment rule, since a shorter phrase inside a
   reported passage is contained in it.
   */
  const longest = indexWindows({
    words: shippedWords,
    length: MAX_PHRASE_WORDS,
  },);

  /**
   Archive counts at that same length, for deciding which windows are ours.
   */
  const archiveLongest = countPhrases({
    words: archiveWords,
    length: MAX_PHRASE_WORDS,
  },);

  /**
   Offsets whose own window is an introduced repetition.

   JUDGED PER WINDOW, BEFORE ANY MERGING. A span is grown only from windows
   that each earn a finding on their own, so growing can join what would have
   been reported anyway and can never promote a window this test rejected.
   */
  const admitted = new Set<number>();
  for (const [phrase, positions,] of longest.byPhrase) {
    if (positions.length < 2)
      continue;
    if (positions.length <= (archiveLongest.get(phrase,) ?? 0))
      continue;
    if (!carriesContent({ phrase, },))
      continue;
    for (const at of positions)
      admitted.add(at,);
  }

  for (
    const span of grownSpans({
      words: shippedWords,
      length: MAX_PHRASE_WORDS,
      index: longest,
      admitted,
    },)
  ) {
    // THE SHIPPED-OVER-ARCHIVE TEST IS NOT REPEATED HERE, because it cannot
    // fail. A grown span contains the window it started from, so the archive
    // says the span no more often than it says that window, which it already
    // says less often than the shipped document does.
    covering.push(span.phrase,);
    if (span.accountedFor)
      continue;
    found.push({
      phrase: span.phrase,
      archiveCount: countSpan({
        words: archiveWords,
        phrase: span.phrase,
      },),
      shippedCount: span.count,
    },);
  }

  for (let length = MAX_PHRASE_WORDS - 1; length >= MIN_PHRASE_WORDS; length -= 1) {
    /**
     Every phrase of this length in the shipped document.
     */
    const shipped = countPhrases({
      words: shippedWords,
      length,
    },);

    /**
     Every phrase of this length in the archive.
     */
    const archive = countPhrases({
      words: archiveWords,
      length,
    },);

    for (const [phrase, shippedCount,] of shipped) {
      if (shippedCount < 2)
        continue;

      /**
       Times the archive said the same thing, zero when it never did.
       */
      const archiveCount = archive.get(phrase,) ?? 0;

      // The archive's own repetition is the author's, not ours. Only an
      // INCREASE is something this pipeline did.
      if (shippedCount <= archiveCount)
        continue;
      if (!carriesContent({ phrase, },))
        continue;
      if (covering.some(function contains(longer,): boolean {
        return holdsPhrase({
          longer,
          phrase,
        },);
      },))
        continue;
      covering.push(phrase,);
      found.push({
        phrase,
        archiveCount,
        shippedCount,
      },);
    }
  }

  return found;
}

/**
 Renders introduced repetitions as scorecard-stable findings.

 THE PHRASE ITSELF IS NOT IN THE FINDING, and that is deliberate. Findings are
 short tokens that get counted and compared across runs, and a passage of
 corpus prose inside one would make every tally depend on the text it happened
 to find. The shape is enough to locate it: rerun the check over the artifact
 and the phrase is right there.

 @param archiveText - translation as it stood before the lane ran

 @param shippedText - assembled document the lane produced

 @returns One finding per introduced repetition, longest first

 @example
 ```ts
 const findings = repetitionFindings({ archiveText, shippedText, },);
 ```
 */
export function repetitionFindings(
  {
    archiveText,
    shippedText,
  }: {
    readonly archiveText: string;
    readonly shippedText: string;
  },
): readonly string[] {
  return findIntroducedRepetitions({
    archiveText,
    shippedText,
  },)
    .map(function toFinding(found,): string {
      return `introduced-repetition (${
        String(found.phrase
          .split(' ',)
          .length,)
      } ${
        wordForCount({
          count: found.phrase
            .split(' ',)
            .length,
          one: 'word',
          many: 'words',
        },)
      }, archive ${String(found.archiveCount,)}, shipped ${String(found.shippedCount,)})`;
    },);
}

//endregion Assembly repetition
