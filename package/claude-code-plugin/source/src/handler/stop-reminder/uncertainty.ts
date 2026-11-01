/**
 Uncertainty detection engine for Claude Code response text.
 
 Orchestration module. Phrase lists, citation predicates, and strip
 helpers live in sibling `uncertainty-*.ts` files so this file stays
 within the workspace's `max-lines` budget.
 
 @module
 */

import {
  containsAnyOfWordBounded,
  isWhitespace,
  PHRASE_NOT_FOUND,
} from '@monochromatic-dev/agent-harness-shared-text-scan/ts';
import { lineHasCitation, } from './uncertainty-citations.ts';
import {
  containsErThanMost,
  DISMISSAL_PHRASES,
  ER_NOT_FOUND,
  findErThanMost,
  normaliseApostrophes,
  UNCERTAINTY_PHRASES,
} from './uncertainty-phrases.ts';
//region Types

/**
 Result of scanning text for uncertainty markers.
 */
type UncertaintyMatch = {
  /**
   First matched uncertain phrase.
   */
  phrase: string;
};

/**
 Result of scanning text for trailing questions directed at the user.
 */
type QuestionMatch = {
  /**
   Sentence ending with `?` that was detected.
   */
  sentence: string;
};

/**
 Sentinel returned by the scan functions when no match is found.
 
 A unique symbol rather than `undefined`: each detector narrows on identity
 (`=== NO_MATCH`), keeping the match payload free of a nullish union.
 */
const NO_MATCH: unique symbol = Symbol('stop-reminders/prose-detector-match-absent',);

//endregion

//region Uncertainty detection

/**
 Scans prose text for the first uncertainty marker match.
 
 Tries the phrase list first via {@link containsAnyOfWordBounded}; if no
 literal phrase matches, falls back to {@link containsErThanMost} for the
 comparative `<word>er than most` shape.
 
 @param prose - text with code blocks and quotes already stripped
 
 @returns match details if uncertain language was found, `NO_MATCH` otherwise
 
 @example
 ```ts
 const match = findUncertainty('This probably works');
 // => { phrase: 'probably' }
 ```
 */
function findUncertainty(prose: string,): UncertaintyMatch | typeof NO_MATCH {
  /**
   First word-bounded phrase hit from {@link UNCERTAINTY_PHRASES};
   `PHRASE_NOT_FOUND` when no phrase fires.
   */
  const phraseHit = containsAnyOfWordBounded({
    haystack: normaliseApostrophes(prose,),
    phrases: UNCERTAINTY_PHRASES,
  },);
  if (phraseHit !== PHRASE_NOT_FOUND)
    return { phrase: phraseHit.phrase, };
  if (containsErThanMost(prose,)) {
    /**
     Recovered comparative fragment for the diagnostic ("bigger than most" etc.).
     */
    const fragment = findErThanMost(prose,);
    return { phrase: fragment === ER_NOT_FOUND ? 'er than most' : fragment, };
  }
  return NO_MATCH;
}

//endregion

//region Dismissal detection

/**
 Scans prose line-by-line for categorical-dismissal phrases that are not
 accompanied by a citation on the same line.
 
 A dismissal is treated as grounded when the line carries a file path, a
 `:N` line-number suffix, or names `AGENTS.md` directly; in that case the
 line is allowed and scanning continues. Uncited dismissals are flagged so
 the agent verifies the claim or removes it.
 
 Per-line scope keeps the check practical: a response with citations
 elsewhere does not blanket-authorise uncited dismissals scattered in
 other bullets.
 
 @param prose - text with code blocks and quotes already stripped
 
 @returns match details if an uncited dismissal was found, `NO_MATCH` otherwise
 
 @example
 ```ts
 findCategoricalDismissal('All JSX rules: project doesn\'t use JSX.');
 // => { phrase: "project doesn't use" }
 
 findCategoricalDismissal('Skip; project doesn\'t use X (see tsconfig.json:5)');
 // => NO_MATCH
 ```
 */
function findCategoricalDismissal(prose: string,): UncertaintyMatch | typeof NO_MATCH {
  /**
   Prose split per-line so each dismissal check is scoped to its own citation context.
   */
  const lines = prose.split('\n',);
  for (const line of lines) {
    if (lineHasCitation(line,))
      continue;
    /**
     Normalised line that folds curly apostrophes to ASCII so phrase entries match either shape.
     */
    const normalised = normaliseApostrophes(line,);
    /**
     First dismissal-phrase hit, or `PHRASE_NOT_FOUND`; populates the returned match.
     */
    const hit = containsAnyOfWordBounded({
      haystack: normalised,
      phrases: DISMISSAL_PHRASES,
    },);
    if (hit !== PHRASE_NOT_FOUND)
      return { phrase: hit.phrase, };
  }
  return NO_MATCH;
}

//endregion

//region Trailing-question detection

/**
 Maximum characters from the end of the message to scan for trailing questions.
 */
const TRAILING_QUESTION_SCAN_LENGTH = 500;

/**
 Sentence-terminator characters used to find the start of the trailing sentence.
 */
const SENTENCE_TERMINATORS = '.!?';

/**
 Lowercase prefixes that mark a question as rhetorical/conditional and thus benign.
 */
const RHETORICAL_PREFIXES: readonly string[] = [
  'what if',
  'why does',
  'why would',
  'how does',
  'have you ever',
];

/**
 Checks whether `s` begins with `prefix` and the character following the
 prefix (if any) is a non-word boundary. Mirrors `^<prefix>\b` semantics
 without invoking the regex engine.
 
 @param s - candidate text (typically a lower-cased sentence)
 
 @param prefix - lowercase prefix to test against
 
 @returns whether the prefix appears word-bounded at the start of `s`
 
 @example
 ```ts
 startsWithWordBounded({ s: 'what if?', prefix: 'what if' }); // true
 startsWithWordBounded({ s: 'whatifelse?', prefix: 'what if' }); // false
 ```
 */
function startsWithWordBounded(
  {
    s,
    prefix,
  }: {
    readonly s: string;
    readonly prefix: string;
  },
): boolean {
  if (!s.startsWith(prefix,))
    return false;
  if (s.length
    === prefix
      .length)
    return true;
  /**
   Character immediately after the prefix; must not be alphanumeric to mark a boundary.
   */
  const next = s.charAt(prefix.length,);
  return ((next < 'a') || (next > 'z'))
    && ((next < 'A') || (next > 'Z'))
    && ((next < '0') || (next > '9'))
    && (next !== '_');
}

/**
 Locates the start of the last sentence in `text` by walking backward
 from the trailing `?` character. A sentence starts after the most
 recent terminator-then-whitespace boundary, or at index `0` when none
 is found.
 
 @param text - prose to inspect (trimmed at the end)
 
 @returns inclusive index of the last sentence's first character
 
 @example
 ```ts
 findLastSentenceStart('I finished. Want me to run tests?'); // 12 ('W')
 ```
 */
function findLastSentenceStart(text: string,): number {
  /**
   Skips whitespace from `idx` forward.
   
   @param idx - candidate scan offset
   
   @returns first index whose character is not whitespace, capped at text length
   
   @example
   ```ts
   skipWs(0); // 2 for '  text'
   ```
   */
  function skipWs(idx: number,): number {
    /**
     Cursor advanced over the whitespace run; returned as the helper-shape binding.
     */
    let at = idx;
    while ((at < text
      .length) && isWhitespace(text.charAt(at,),)) {
      at += 1;
    }
    return at;
  }
  /**
   Walks backward from `at` looking for a terminator-then-whitespace
   transition that marks the start of the next sentence.
   
   @param at - candidate scan offset
   
   @returns inclusive start of the trailing sentence
   
   @example
   ```ts
   walk(text.length - 1); // first index after the last `.|!|? ` boundary
   ```
   */
  function walk(at: number,): number {
    // Walk backward from `at`; the first terminator-then-whitespace boundary
    // marks the start of the trailing sentence (after its leading whitespace).
    for (let cursor = at; cursor > 0; cursor -= 1) {
      /**
       Character just before the cursor; checked for sentence-terminator membership.
       */
      const prev = text.charAt(cursor - 1,);
      if (SENTENCE_TERMINATORS.includes(prev,)
        && isWhitespace(text.charAt(cursor,),))
        return skipWs(cursor,);
    }
    return 0;
  }
  return walk(text.length
    - 1,);
}

/**
 Detects when prose ends with a literal question to the user, ignoring
 common rhetorical patterns. Used by the stop-reminders hook to nudge
 the agent toward `AskUserQuestion` when it forgot to use the structured
 tool.
 
 Mirrors the legacy regex `(?:^|[.!?]\s+)([A-Z][^.!?]*\?)\s*$`: walks
 backwards from the final `?` to find the start of the trailing sentence,
 checks the sentence begins with an uppercase letter, and excludes the
 configured rhetorical prefixes.
 
 @param prose - assistant message text to scan
 
 @returns matched question sentence, or `NO_MATCH` if none qualifies
 
 @example
 ```ts
 findTrailingQuestion('Done. Want me to commit?'); // { sentence: 'Want me to commit?' }
 findTrailingQuestion('What if cats ruled?');      // NO_MATCH (rhetorical)
 ```
 */
function findTrailingQuestion(prose: string,): QuestionMatch | typeof NO_MATCH {
  /**
   Last `TRAILING_QUESTION_SCAN_LENGTH` chars; trailing questions live at the end of a turn.
   */
  const tail = prose.slice(-TRAILING_QUESTION_SCAN_LENGTH,);
  /**
   Tail trimmed of trailing whitespace; the trailing `?` lives at the very end after this.
   */
  const trimmed = tail.trimEnd();
  if (!trimmed.endsWith('?',))
    return NO_MATCH;
  /**
   Inclusive start index of the trailing sentence within `trimmed`.
   */
  const sentenceStart = findLastSentenceStart(trimmed,);
  /**
   Trailing sentence text including the terminating `?`.
   */
  const sentence = trimmed.slice(sentenceStart,);
  if (sentence.length
    === 0)
    return NO_MATCH;
  /**
   Sentence-leading character; must be an uppercase ASCII letter to qualify.
   */
  const firstChar = sentence.charAt(0,);
  if ((firstChar < 'A') || (firstChar > 'Z'))
    return NO_MATCH;
  /**
   Lower-cased sentence used for rhetorical-prefix matching.
   */
  const lower = sentence.toLowerCase();
  for (const prefix of RHETORICAL_PREFIXES) {
    if (startsWithWordBounded({
      s: lower,
      prefix,
    },)) {
      return NO_MATCH;
    }
  }
  return { sentence, };
}

//endregion

export {
  findCategoricalDismissal,
  findTrailingQuestion,
  findUncertainty,
  NO_MATCH,
};

export { stripNonProseRegions, } from './uncertainty-strip.ts';

export type {
  QuestionMatch,
  UncertaintyMatch,
};
