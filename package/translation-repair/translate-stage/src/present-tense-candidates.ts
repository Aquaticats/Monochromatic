import {
  type MdxParseError,
  parsedTopLevelBlocks,
} from '@monochromatic-dev/translation-repair-document-structure/ts';
import {
  BARE_VERBS,
  INVERTING_AUXILIARIES,
  PAST_PARTICIPLES,
  PRESENT_AUXILIARIES,
  PRONOUNS,
  SKIPPED_S_WORDS,
} from './present-tense-vocabulary.ts';
import {
  maskQuotedSpeech,
  sentenceSpans,
  visibleWords,
} from './present-tense-text.ts';
import {
  isBlankSpan,
  isSEndingWord,
  namesAt,
  wordAfterSpace,
  wordsOf,
  type WordToken,
} from './present-tense-words.ts';

//region Present-tense candidates
// LINE-LEVEL CANDIDATES FOR THE HOUSE TENSE RULE, shown to judges as sentences
// to check against that rule and never as a refusal or as evidence against a
// text. The house rule tells the narrative of a life that has ended in the past
// tense; these rules flag sentences that read as present narration of a dead
// subject. Measured on the pinned corpus, about two in three flagged sentences
// are a trait, a standing fact or a quote the rule does not reach, so a
// candidate is a question for the judge and not a finding.
//
// THE READING STARTS FROM THE PARSE. Blockquotes, headings, fenced code and HTML
// blocks are top-level blocks of their own, and only paragraphs are read.
// Quoted speech is blanked inside a paragraph, so a line the archive quotes is
// never narration of the subject.
//
// A PAGE THAT RECORDS NO DEATH IS NOT READ. The rule applies to lives that have
// ended, so a page without that record returns nothing rather than guessing.

/**
 Rule that fired for one candidate.
 */
export type PresentTenseRule = 'contraction' | 'copula-or-auxiliary' | 's-ending-verb' | 'bare-verb';

/**
 One sentence the detector flags, with the words that matched and the rule that fired.

 @example
 ```ts
 const candidate: PresentTenseCandidate = {
   sentence: 'Mittens naps by the door.',
   matched: 'Mittens naps',
   rule: 's-ending-verb',
 };
 ```
 */
export type PresentTenseCandidate = {
  /**
   Sentence as the page reads it, links unwrapped and quotes kept.
   */
  readonly sentence: string;

  /**
   Words the rule matched, as they stand in the sentence.
   */
  readonly matched: string;

  /**
   Rule that fired.
   */
  readonly rule: PresentTenseRule;
};

/**
 One rule hit, with the words it matched.
 */
type RuleHit = {
  /**
   Words the rule matched, as they stand in the sentence.
   */
  readonly matched: string;

  /**
   Rule that fired.
   */
  readonly rule: PresentTenseRule;
};

/**
 Rule hits for one subject, given the word after it.

 @param sentence - sentence the subject sits in

 @param start - offset where the subject begins

 @param verbs - the word after the subject, empty when none follows

 @param pronoun - whether the subject is a pronoun rather than the declared name

 @returns At most one hit, for the first rule the following word satisfies

 @example
 ```ts
 const hits = ruleForSubject({ sentence: 'She naps', start: 0, verbs: [], pronoun: true, },);
 ```
 */
function ruleForSubject(
  {
    sentence,
    start,
    verbs,
    pronoun,
  }: {
    readonly sentence: string;
    readonly start: number;
    readonly verbs: readonly WordToken[];
    readonly pronoun: boolean;
  },
): readonly RuleHit[] {
  return verbs.flatMap(function hitForVerb(verb,): readonly RuleHit[] {
    /**
     Text from the subject through the following word, as the page reads it.
     */
    const matched = sentence.slice(
      start,
      verb.end,
    );
    if (PRESENT_AUXILIARIES.includes(verb.folded,)) {
      return [{
        matched,
        rule: 'copula-or-auxiliary',
      },];
    }
    if (isSEndingWord({ word: verb.folded, },) && (!SKIPPED_S_WORDS.includes(verb.folded,))) {
      return [{
        matched,
        rule: 's-ending-verb',
      },];
    }
    if (pronoun && BARE_VERBS.includes(verb.folded,)) {
      return [{
        matched,
        rule: 'bare-verb',
      },];
    }
    return [];
  },);
}

/**
 Rule hits for a contraction: a present `they're`, or `she's` or `he's` not followed
 by a past participle.

 @param sentence - sentence the word sits in

 @param word - the word to read

 @param wordsByStart - words keyed by their start offset

 @returns At most one hit

 @example
 ```ts
 const hits = contractionHits({ sentence: "She's a cat", word, wordsByStart, },);
 ```
 */
function contractionHits(
  {
    sentence,
    word,
    wordsByStart,
  }: {
    readonly sentence: string;
    readonly word: WordToken;
    readonly wordsByStart: ReadonlyMap<number, WordToken>;
  },
): readonly RuleHit[] {
  if (word.folded === 'they\'re') {
    return [{
      matched: word.text,
      rule: 'contraction',
    },];
  }
  if ((word.folded !== 'she\'s') && (word.folded !== 'he\'s')) {
    return [];
  }
  /**
   The word after the contraction, when whitespace and a word follow it.
   */
  const following = wordAfterSpace({
    sentence,
    wordsByStart,
    end: word.end,
  },);
  if (following.some(function isPastParticiple(next,): boolean {
    return PAST_PARTICIPLES.includes(next.folded,);
  },)) {
    return [];
  }
  return [{
    matched: word.text,
    rule: 'contraction',
  },];
}

/**
 Rule hits for a subject, given the offsets just past the subject and the words after it.

 @param sentence - sentence the subject sits in

 @param start - offset where the subject begins

 @param end - offset just past the subject

 @param wordsByStart - words keyed by their start offset

 @param pronoun - whether the subject is a pronoun rather than the declared name

 @returns At most one hit, for the first rule the word after the subject satisfies

 @example
 ```ts
 const hits = subjectHits({ sentence: 'She naps', start: 0, end: 3, wordsByStart, pronoun: true, },);
 ```
 */
function subjectHits(
  {
    sentence,
    start,
    end,
    wordsByStart,
    pronoun,
  }: {
    readonly sentence: string;
    readonly start: number;
    readonly end: number;
    readonly wordsByStart: ReadonlyMap<number, WordToken>;
    readonly pronoun: boolean;
  },
): readonly RuleHit[] {
  return ruleForSubject({
    sentence,
    start,
    verbs: wordAfterSpace({
      sentence,
      wordsByStart,
      end,
    },),
    pronoun,
  },);
}

/**
 Every rule hit that starts at one word of a sentence, with the earlier rule first.

 @param sentence - sentence the word sits in, quotes already blanked

 @param word - the word to read

 @param previous - the word before it, empty for the first word

 @param wordsByStart - words keyed by their start offset

 @param names - declared names, trimmed, longest first

 @returns At most one hit; none when an inverted auxiliary comes before the word

 @example
 ```ts
 const hits = hitsAtWord({ sentence, word, previous, wordsByStart, names: ['Mittens',], },);
 ```
 */
function hitsAtWord(
  {
    sentence,
    word,
    previous,
    wordsByStart,
    names,
  }: {
    readonly sentence: string;
    readonly word: WordToken;
    readonly previous: readonly WordToken[];
    readonly wordsByStart: ReadonlyMap<number, WordToken>;
    readonly names: readonly string[];
  },
): readonly RuleHit[] {
  /**
   Whether the word before this one is an inverting auxiliary separated from it by whitespace alone.
   */
  const inverted = previous.some(function invertsThisWord(before,): boolean {
    /**
     Whether the earlier word is an inverting auxiliary.
     */
    const auxiliary = INVERTING_AUXILIARIES.includes(before.folded,);

    /**
     Whether only whitespace separates the auxiliary from this word.
     */
    const adjacent = isBlankSpan({
      sentence,
      start: before.end,
      end: word.start,
    },);
    return auxiliary && adjacent;
  },);
  if (inverted) {
    return [];
  }
  /**
   Hits for the word as a contraction subject.
   */
  const contraction = contractionHits({
    sentence,
    word,
    wordsByStart,
  },);

  /**
   Hits for the word as a pronoun subject.
   */
  const pronoun = PRONOUNS.includes(word.folded,)
    ? subjectHits({
      sentence,
      start: word.start,
      end: word.end,
      wordsByStart,
      pronoun: true,
    },)
    : [];

  /**
   Declared name that opens at this word, the longest where several do.
   */
  const opening = names.find(function opensHere(name,): boolean {
    return namesAt({
      sentence,
      start: word.start,
      name,
    },);
  },);

  /**
   Hits for the word as a declared subject name.
   */
  const named = (opening === undefined)
    ? []
    : subjectHits({
      sentence,
      start: word.start,
      end: word.start + opening.length,
      wordsByStart,
      pronoun: false,
    },);

  /**
   Every hit the word starts, before the first-rule cut.
   */
  const hits = [
    ...contraction,
    ...pronoun,
    ...named,
  ];
  return hits.slice(
    0,
    1,
  );
}

/**
 Candidates of one sentence, one per start position at most.

 @param sentence - sentence as the page reads it, for display

 @param masked - the same sentence with quoted speech blanked, read by the rules

 @param names - declared names, trimmed, longest first, none empty

 @returns Candidates of the sentence, in the order their matches start

 @example
 ```ts
 const found = candidatesOfSentence({ sentence: 'Mittens naps.', masked: 'Mittens naps.', names: ['Mittens',], },);
 ```
 */
function candidatesOfSentence(
  {
    sentence,
    masked,
    names,
  }: {
    readonly sentence: string;
    readonly masked: string;
    readonly names: readonly string[];
  },
): readonly PresentTenseCandidate[] {
  /**
   Words of the masked sentence, in order.
   */
  const words = wordsOf({ sentence: masked, },);

  /**
   Words keyed by their start offset, for finding the word after a subject.
   */
  const wordsByStart = new Map(words.map(function entryOf(word,) {
    return [
      word.start,
      word,
    ] as const;
  },),);
  return words.flatMap(function candidatesAtWord(
    word,
    index,
  ): readonly PresentTenseCandidate[] {
    /**
     Offset of the word before this one, zero for the first word.
     */
    const previousStart = Math.max(
      0,
      index - 1,
    );

    /**
     The word before this one, empty for the first word.
     */
    const previous = words.slice(
      previousStart,
      index,
    );

    /**
     Hits the word starts, at most one.
     */
    const hits = hitsAtWord({
      sentence: masked,
      word,
      previous,
      wordsByStart,
      names,
    },);
    return hits.map(function candidateOf(hit,): PresentTenseCandidate {
      return {
        sentence,
        matched: hit.matched,
        rule: hit.rule,
      };
    },);
  },);
}

/**
 Candidates of one paragraph, sentence by sentence. A sentence that ends in a
 question mark is an inversion or a question, so it yields nothing.

 @param paragraph - visible words of one paragraph, not yet masked

 @param names - declared names, trimmed, longest first, none empty

 @returns Candidates of the paragraph, in page order

 @example
 ```ts
 const found = candidatesOfParagraph({ paragraph: 'Mittens naps.', names: ['Mittens',], },);
 ```
 */
function candidatesOfParagraph(
  {
    paragraph,
    names,
  }: {
    readonly paragraph: string;
    readonly names: readonly string[];
  },
): readonly PresentTenseCandidate[] {
  /**
   Paragraph with quoted speech blanked, the same length as the visible words.
   */
  const masked = maskQuotedSpeech({ text: paragraph, },);
  /**
   Offsets of each sentence of the masked paragraph.
   */
  const spans = sentenceSpans({ text: masked, },);
  return spans.flatMap(function candidatesOfSpan(span,): readonly PresentTenseCandidate[] {
    /**
     This sentence with quoted speech blanked, untrimmed so its offsets match the paragraph.
     */
    const maskedSentence = masked.slice(
      span.start,
      span.end,
    );
    /**
     The masked sentence with trailing whitespace removed, read for its final mark.
     */
    const trimmed = maskedSentence.trimEnd();
    if (trimmed.endsWith('?',)) {
      return [];
    }
    /**
     This sentence as the page reads it, trimmed for display.
     */
    const displayed = paragraph.slice(
      span.start,
      span.end,
    );
    return candidatesOfSentence({
      sentence: displayed.trim(),
      masked: maskedSentence,
      names,
    },);
  },);
}

/**
 Lists the present-tense candidates of a memorial page, for judges to check
 against the house tense rule. Returns none unless the page records a death.

 Only paragraphs are read. A blockquote, heading, fenced code block or HTML
 block yields nothing, and so does quoted speech inside a paragraph.

 @param text - body of the page, with front matter already removed

 @param subjectNames - declared names of the subject, the display name and its aliases, empty when none is declared; the longest is tried first

 @param died - whether the page records a death; a page without one is not read

 @returns Candidates in page order, each with its sentence, matched words and rule

 @throws {@link MdxParseError} when the shared slice grammar refuses this text

 @example
 ```ts
 const found = presentTenseCandidates({ text: 'Mittens naps by the door.', subjectNames: ['Mittens',], died: true, },);
 ```
 */
export function presentTenseCandidates(
  {
    text,
    subjectNames,
    died,
  }: {
    readonly text: string;
    readonly subjectNames: readonly string[];
    readonly died: boolean;
  },
): readonly PresentTenseCandidate[] {
  if (!died) {
    return [];
  }
  /**
   Declared names with their surrounding whitespace removed and the longest
   first, so a name that begins another is not taken for the other. A blank
   name matches nothing (`namesAt`), so none is dropped here.
   */
  const names = subjectNames
    .map(function trimmed(name,): string {
      return name.trim();
    },)
    .toSorted(function longestFirst(
      left,
      right,
    ): number {
      return right.length - left.length;
    },);
  return parsedTopLevelBlocks({ text, },)
    .filter(function isParagraph(block,): boolean {
      return block.kind === 'paragraph';
    },)
    .flatMap(function candidatesOfBlock(block,): readonly PresentTenseCandidate[] {
      return candidatesOfParagraph({
        paragraph: visibleWords({ text: block.text, },),
        names,
      },);
    },);
}
//endregion Present-tense candidates
