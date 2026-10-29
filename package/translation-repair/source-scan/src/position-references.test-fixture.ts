/**
 The scan behind `position-references.unit.test.ts` (ledger D33): the
 functions that find a reference by position in a file and match it against
 the guard's exemptions. Split out of the guard so a probe can print every
 reference the guard would name, whole, where a failing assertion shows a
 truncated list; its vocabulary, its reading of a text, how it reads a compared word and where a phrase
 stands (quotes, paragraphs, code spans) sit in the four sibling fixtures.

 @module
 */

import { isAsciiDigit, } from '@monochromatic-dev/translation-repair-markup';
import {
  adjoins,
  readsAsComparedObject,
} from './position-references-compared.test-fixture.ts';
import {
  insideCodeSpan,
  insideQuotes,
  paragraphStart,
} from './position-references-context.test-fixture.ts';
import {
  joinedText,
  type Word,
  wordsOf,
} from './position-references-text.test-fixture.ts';
import {
  COMPARED_OBJECTS,
  NAMING_OPENERS,
  POSITION_MARKERS,
  POSITIONS,
  REFERENCE_VERBS,
  SEQUENCES,
  WINDOW_REACH,
} from './position-references-vocabulary.test-fixture.ts';
import type { PackageText, } from './prose-texts.test-fixture.ts';

//region Scan

/**
 Words in a phrase of one word and the position after it.
 */
const WORD_THEN_POSITION = 2;

/**
 One reference by position.

 @example
 ```ts
 const found: PositionReference = { path: 'src/cat.ts', line: 3, phrase: 'case above', window: '...', };
 ```
 */
type PositionReference = {
  /**
   File it stands in.
   */
  readonly path: string;

  /**
   Line its first word stands on.
   */
  readonly line: number;

  /**
   The phrase as written.
   */
  readonly phrase: string;

  /**
   Text around it, which an exemption is matched against.
   */
  readonly window: string;
};

/**
 One reference by position the guard leaves, with why.

 @example
 ```ts
 const exempt: PositionExemption = { path: 'src/cat.ts', holds: 'note above the bowl', reason: 'page text', };
 ```
 */
export type PositionExemption = {
  /**
   File it stands in.
   */
  readonly path: string;

  /**
   Text beside it that picks it out.
   */
  readonly holds: string;

  /**
   Why it is not a reference by position.
   */
  readonly reason: string;
};

/**
 Whether the words from an index on spell a sequence, each from its set and
 each adjoining the last.

 @param flat - joined text

 @param words - every word of it

 @param at - index of the first word

 @param sets - one set per word of the sequence

 @returns Whether they match

 @example
 ```ts
 spells({ flat, words, at: 0, sets: [DETERMINERS, SEQUENCE_NOUNS,], },);
 ```
 */
function spells(
  {
    flat,
    words,
    at,
    sets,
  }: {
    readonly flat: string;
    readonly words: readonly Word[];
    readonly at: number;
    readonly sets: readonly ReadonlySet<string>[];
  },
): boolean {
  return sets.every(function matches(
    set,
    offset,
  ): boolean {
    /**
     Word this set is matched against.
     */
    const word = words[at + offset];
    if ((word === undefined) || (!set.has(word.text,)))
      return false;
    if (offset === 0)
      return true;
    /**
     Word before it in the sequence.
     */
    const previous = words[(at + offset) - 1];
    return (previous !== undefined) && adjoins({
      flat,
      left: previous,
      right: word,
    },);
  },);
}

/**
 Whether what follows a position makes it a comparison or names its target.

 A NAMING CHARACTER COUNTS ONLY AFTER A SPACE. One written straight after the
 position closes something the position sits in, as the backtick ending a
 printed line's template does ("the credits above`"), rather than opening a
 name. The compared word may stand across a string joined by a sign, whose
 closing quote and sign the joined text keeps ("since below ' the floor").

 @param flat - joined text

 @param words - every word of it

 @param at - index of the position word

 @returns Whether the position compares or names rather than points

 @example
 ```ts
 comparesOrNames({ flat, words, at, },);
 ```
 */
function comparesOrNames(
  {
    flat,
    words,
    at,
  }: {
    readonly flat: string;
    readonly words: readonly Word[];
    readonly at: number;
  },
): boolean {
  /**
   The position word.
   */
  const position = words[at];
  if (position === undefined)
    return false;
  /**
   The text after it.
   */
  const rest = flat.slice(position.end,);
  // A HYPHEN STRAIGHT AFTER MAKES A COMPOUND, "a below-threshold vote", which
  // compares, unless its second half cites, as "the above-mentioned" does.
  if (rest.startsWith('-',))
    return !REFERENCE_VERBS.has(words[at + 1]
      ?.text
      ?? '',);
  /**
   That text from its first non-space character.
   */
  const after = rest.trimStart();
  /**
   That character.
   */
  const opener = after.charAt(0,);
  if ((after.length < rest.length) && (NAMING_OPENERS.has(opener,) || isAsciiDigit({ character: opener, },)))
    return true;
  /**
   The next word, when the text goes on with one.
   */
  const next = words[at + 1];
  return (next !== undefined)
    && adjoins({
      flat,
      left: position,
      right: next,
    },)
    && COMPARED_OBJECTS.has(next.text,)
    && readsAsComparedObject({
      flat,
      words,
      at: at + 1,
    },);
}

/**
 How many words of a phrase begin at an index, zero when none does.

 @param flat - joined text

 @param words - every word of it

 @param at - index of the phrase's first word

 @returns Words in the phrase

 @example
 ```ts
 phraseLength({ flat, words, at, },);
 ```
 */
function phraseLength(
  {
    flat,
    words,
    at,
  }: {
    readonly flat: string;
    readonly words: readonly Word[];
    readonly at: number;
  },
): number {
  /**
   The phrase's first word.
   */
  const word = words[at];
  if (word === undefined)
    return 0;
  /**
   The plain sequence spelled here, when one is.
   */
  const spelled = SEQUENCES.find(function spelledHere(sets,): boolean {
    return spells({
      flat,
      words,
      at,
      sets,
    },);
  },);
  if (spelled !== undefined)
    return spelled.length;
  if (
    POSITIONS.has(word.text,)
      && (flat.charAt(word.end,) === ')')
      && ((flat.charAt(word.start - 1,) === '(')
        || flat.slice(
          0,
          word.start,
        )
          .endsWith('(see ',))
  )
    return 1;
  /**
   The word after this one.
   */
  const next = words[at + 1];
  if (
    (next !== undefined)
      && POSITIONS.has(next.text,)
      && (!POSITION_MARKERS.has(word.text,))
      && adjoins({
        flat,
        left: word,
        right: next,
      },)
      && (!comparesOrNames({
        flat,
        words,
        at: at + 1,
      },))
  )
    return WORD_THEN_POSITION;
  return 0;
}

/**
 Every reference by position in one file.

 @param file - file with its text

 @returns References, in order

 @example
 ```ts
 positionReferences({ file: { path: 'src/cat.ts', text: '// see above', }, },);
 ```
 */
export function positionReferences({ file, }: { readonly file: PackageText; },): readonly PositionReference[] {
  /**
   The file with its wrapped lines joined.
   */
  const {
    flat,
    lineStarts,
  } = joinedText({ text: file.text, },);
  /**
   Its words.
   */
  const words = wordsOf({ flat, },);
  /**
   Its lines as written, read for Markdown code spans.
   */
  const written = file.text
    .split('\n',);
  /**
   Whether the file is Markdown, where a backtick opens a code span.
   */
  const markdown = file.path
    .endsWith('.md',);
  /**
   References found so far.
   */
  const found: PositionReference[] = [];
  /**
   Offset the last phrase found ends at, so a phrase inside it is not found
   twice.
   */
  let coveredUntil = -1;
  for (const [at, word,] of words.entries()) {
    if (word.start < coveredUntil)
      continue;
    /**
     Words in a phrase starting here.
     */
    const length = phraseLength({
      flat,
      words,
      at,
    },);
    if (length === 0)
      continue;
    /**
     The phrase's last word.
     */
    const last = words[(at + length) - 1] ?? word;
    coveredUntil = last.end;
    /**
     Index of the line the phrase starts on.
     */
    const lineIndex = lineStarts.findLastIndex(function startsBefore(start,): boolean {
      return start <= word.start;
    },);
    if (insideQuotes({
      flat,
      lineStart: paragraphStart({
        flat,
        lineStarts,
        lineIndex,
      },),
      offset: word.start,
    },))
      continue;
    /**
     The line the phrase starts on, as written.
     */
    const line = written[lineIndex] ?? '';
    // A JOINED LINE ENDS WHERE ITS WRITTEN LINE ENDS, since joining only strips
    // a line's opening markup, so the phrase's distance from that end is the
    // same in both.
    if (markdown && insideCodeSpan({
      line,
      offset: line.length - ((lineStarts[lineIndex + 1] ?? flat.length) - word.start),
    },))
      continue;
    found.push({
      path: file.path,
      line: lineIndex + 1,
      phrase: flat.slice(
        word.start,
        last.end,
      ),
      window: flat.slice(
        Math.max(
          0,
          word.start - WINDOW_REACH,
        ),
        last.end + WINDOW_REACH,
      ),
    },);
  }
  return found;
}

/**
 Every reference by position across files that no exemption names, as
 `path:line phrase`.

 @param files - files to read

 @param exemptions - references left, each with its reason

 @returns Unexempted references

 @example
 ```ts
 unexempted({ files, exemptions: [], },);
 ```
 */
export function unexempted(
  {
    files,
    exemptions,
  }: {
    readonly files: readonly PackageText[];
    readonly exemptions: readonly PositionExemption[];
  },
): readonly string[] {
  return files
    .flatMap(function inFile(file,): readonly PositionReference[] {
      return positionReferences({ file, },);
    },)
    .filter(function isUnexempted(found,): boolean {
      return !exemptions.some(function names(exemption,): boolean {
        return (exemption.path === found.path)
          && found.window
            .includes(exemption.holds,);
      },);
    },)
    .map(function located(found,): string {
      return `${found.path}:${String(found.line,)} ${found.phrase}`;
    },);
}

/**
 Exemptions that no longer name any reference.

 @param files - files to read

 @param exemptions - references left, each with its reason

 @returns Stale ones' texts

 @example
 ```ts
 staleExemptions({ files, exemptions, },);
 ```
 */
export function staleExemptions(
  {
    files,
    exemptions,
  }: {
    readonly files: readonly PackageText[];
    readonly exemptions: readonly PositionExemption[];
  },
): readonly string[] {
  /**
   Every reference across the files.
   */
  const found = files.flatMap(function inFile(file,): readonly PositionReference[] {
    return positionReferences({ file, },);
  },);
  return exemptions
    .filter(function isStale(exemption,): boolean {
      return !found.some(function isNamed(reference,): boolean {
        return (reference.path === exemption.path)
          && reference.window
            .includes(exemption.holds,);
      },);
    },)
    .map(function held(exemption,): string {
      return `${exemption.path}: ${exemption.holds}`;
    },);
}

//endregion Scan
