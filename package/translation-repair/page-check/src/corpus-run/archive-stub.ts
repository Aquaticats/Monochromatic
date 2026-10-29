import { nonNullishOrThrow, } from '@monochromatic-dev/or-throw-core/ts';
import {
  leadingMarkWidth,
  maskHtmlComments,
} from '@monochromatic-dev/translation-repair-document-structure/ts';

//region Archive stub markers
// A PLACEHOLDER IS NOT CONTENT THE ORIGINAL CARRIES. XIEPT2's archive English
// page is a stub: front matter, the line `(To-Do)`, an HTML comment of
// translator hints, and nothing the ORIGINAL says. The pipeline translated the
// whole ORIGINAL below that and, since no slice covered the marker, published
// `(To-Do)` over a finished translation (2026-09-03; ledger A1
// records the stub). The owner: "The pipeline's job
// is to give a good result even when the originals are bad."
// (`doc/decision/translation-repair-good-result-over-bad-original.md`).
//
// MEASURED AGAINST THE PINNED CORPUS, 92 English pages: two stub markers.
// XIEPT2's `(To-Do)`, and XingZ60's ``>>> `Under Construction` `` standing
// where the original has the heading 七句破题 (ledger A1), which shipped on
// all 17 XingZ60 pages until the strip read blockquote markers and a code
// span. The token set is what the corpus shows plus the obvious English
// variants; Chinese placeholders have no instance and are not guessed at.
// "To be continued" is not one: its two instances are a person's own words
// and a translation of 未完待续.
//
// A NOTICE THAT THE PAGE IS UNFINISHED IS A PLACEHOLDER TOO (XingZ60,
// 2026-10-19). The same archive opens with `**Come back later!**`, a warning
// that the version is not the formal one and will be updated after another
// version is finished, and a caution that it is unfinished and the reader
// should see another page. The original has none of them, the archive had
// translated only part of the original, and the pass translated all of it, yet
// the shipped page kept all three: the archive-block review's judges called
// them "a faithful, plainly worded English notice about the page's own
// status", since nothing told them the page would carry every block of the
// original. A paragraph that is nothing but such a notice is stripped here, by
// the sentences the notice is made of, because the page published from the
// pass is complete and the notice is false on it. Measured against the pinned
// corpus, 92 English pages: one carries such a notice, XingZ60's, and no
// other paragraph of any page is made of such sentences. The wording a
// notice takes is open, so the review's sheet says the same
// (`PAGE_CARRIES_EVERY_BLOCK`) for the notices this list does not name.
//
// THE COMMENTS STAY. `entry-notes.ts` reads every archive HTML comment as an
// "ARCHIVE editor comment" line of the identity block, which is where the
// vocabulary and voice notes 22 of those 92 pages carry reach the translators
// and judges, and a reader never sees them. Only the reader-visible marker
// goes, and only where it stands as a paragraph of its own outside front
// matter, comments and code fences.

/**
 Tokens a placeholder paragraph consists of, lowercased and unwrapped.
 */
export const STUB_MARKER_TOKENS: ReadonlySet<string> = new Set([
  'to-do',
  'todo',
  'tbd',
  'wip',
  'under construction',
],);

/**
 One bracket pair a marker may be wrapped in.
 */
type MarkerWrap = {
  /**
   Opening bracket.
   */
  readonly open: string;

  /**
   Closing bracket.
   */
  readonly close: string;
};

/**
 Bracket pairs a marker may be wrapped in, one layer.
 */
const MARKER_WRAPS: readonly MarkerWrap[] = [
  {
    open: '(',
    close: ')',
  },
  {
    open: '[',
    close: ']',
  },
  {
    open: '（',
    close: '）',
  },
];

/**
 Line that opens and closes front matter.
 */
const FRONT_MATTER_FENCE = '---';

/**
 Prefix of a fenced code block's opening and closing line.
 */
const CODE_FENCE = '```';

/**
 One marker the strip removed, for the log and the record.

 @example
 ```ts
 const marker: StrippedStubMarker = { lineNumber: 8, text: '(To-Do)', };
 ```
 */
export type StrippedStubMarker = {
  /**
   One-based line of the archive as read, before any removal.
   */
  readonly lineNumber: number;

  /**
   Line as the archive carried it.
   */
  readonly text: string;
};

/**
 Blockquote marker a placeholder may stand behind, at any depth.
 */
const QUOTE_MARKER = '>';

/**
 Delimiter of the one code span a placeholder may wear.
 */
const CODE_SPAN = '`';

/**
 Text without its leading blockquote markers, however deeply nested.

 ONE PASS over the leading characters, stopping at the first that is neither
 a marker nor the space between markers (ledger A1, a nested blockquote of
 inline code). Every character before that one is ASCII, so its code-point
 index is also its UTF-16 index and the slice is exact.

 @param text - trimmed paragraph

 @returns Text after the last leading marker, trimmed

 @example
 ```ts
 withoutQuoteMarkers({ text: '>> > TBD', },); // 'TBD'
 ```
 */
function withoutQuoteMarkers({ text, }: { readonly text: string; },): string {
  /**
   Index of the first character past the markers, or -1 when none is.
   */
  const contentStart = Array.from(text,)
    .findIndex(function isContent(character,): boolean {
      return (character !== QUOTE_MARKER) && (character !== ' ');
    },);
  return (contentStart === (-1))
    ? ''
    : text.slice(contentStart,)
      .trim();
}

/**
 Text without one code span around the whole of it.

 @param text - paragraph after its quote markers

 @returns Text inside the span, or the text unchanged when it wears none

 @example
 ```ts
 withoutCodeSpan({ text: '`WIP`', },); // 'WIP'
 ```
 */
function withoutCodeSpan({ text, }: { readonly text: string; },): string {
  /**
   Whether one span wraps the whole text with something inside it that is not
   itself a delimiter, so a double-backtick span is left as it is.
   */
  const wears = text.startsWith(CODE_SPAN,)
    && text.endsWith(CODE_SPAN,)
    && (text.length > (CODE_SPAN.length + CODE_SPAN.length))
    && (!text.startsWith(CODE_SPAN.repeat(2,),));
  return wears
    ? text
      .slice(
      CODE_SPAN.length,
      -CODE_SPAN.length,
      )
      .trim()
    : text;
}

/**
 Whether one paragraph is nothing but a placeholder token.

 @param paragraph - paragraph text; whitespace, blockquote markers at any
 depth, one code span and one layer of brackets tolerated

 @returns Whether it names no content

 @example
 ```ts
 isStubMarkerParagraph({ paragraph: '(To-Do)', },);
 ```
 */
export function isStubMarkerParagraph({ paragraph, }: { readonly paragraph: string; },): boolean {
  /**
   Paragraph without its surrounding whitespace, its blockquote markers and
   one code span.
   */
  const trimmed = withoutCodeSpan({ text: withoutQuoteMarkers({ text: paragraph.trim(), },), },);
  /**
   Paragraph without one layer of brackets, when it wore one.
   */
  const unwrapped = MARKER_WRAPS.reduce(
    function unwrap(
      text: string,
      wrap: MarkerWrap,
    ): string {
      /**
       Opening and closing bracket of this pair.
       */
      const {
        open,
        close,
      } = wrap;
      /**
       Whether the text wears this pair with something inside it.
       */
      const wears = text.startsWith(open,)
        && text.endsWith(close,)
        && (text.length > (open.length + close.length));
      if (!wears)
        return text;
      /**
       Text between the brackets.
       */
      const inside = text.slice(
        open.length,
        text.length - close.length,
      );
      return inside.trim();
    },
    trimmed,
  );
  return STUB_MARKER_TOKENS.has(unwrapped.toLowerCase(),);
}

/**
 Sentences that are a notice on their own, lowercased: a reader told to come
 back when the page is done.
 */
const NOTICE_SENTENCES: ReadonlySet<string> = new Set([
  'come back later',
  'please come back later',
  'check back later',
  'please check back later',
],);

/**
 What a notice calls the page it stands on, lowercased.
 */
const NOTICE_SUBJECTS: readonly string[] = [
  'this version',
  'this page',
  'this translation',
  'this entry',
  'this article',
  'this english version',
  'this english page',
  'the english version',
  'the english page',
  'this',
];

/**
 What a notice says of that page, lowercased: it is not the finished one.
 */
const NOTICE_STATES: readonly string[] = [
  'not the formal version',
  'not the final version',
  'not the official version',
  'not the complete version',
  'not finished',
  'not complete',
  'unfinished',
  'incomplete',
  'still being translated',
  'a work in progress',
  'under construction',
  'an unfinished version',
  'an incomplete version',
];

/**
 Words a notice opens with when it says who will finish the page, lowercased.
 */
const NOTICE_UPDATE_OPENERS: readonly string[] = [
  'maintainers will update this',
  'the maintainers will update this',
  'translators will update this',
  'the translators will update this',
  'we will update this',
];

/**
 Words a notice opens with when it sends the reader to another page,
 lowercased.
 */
const NOTICE_REFER_OPENERS: readonly string[] = [
  'please refer to the ',
  'please see the ',
  'please visit the ',
];

/**
 What a notice that sends the reader elsewhere ends with, lowercased.
 */
const NOTICE_REFER_CLOSERS: readonly string[] = [
  ' page',
  ' version',
];

/**
 Most words a notice that sends the reader elsewhere may use, so a sentence
 about a page in the middle of a story is not read as one.
 */
const NOTICE_REFER_MAXIMUM_WORDS = 10;

/**
 Sentences of the form a notice says of its page: a subject, "is", and a state.
 */
const NOTICE_STATEMENTS: ReadonlySet<string> = new Set(NOTICE_SUBJECTS.flatMap(
  function statementsOf(subject,): readonly string[] {
    return NOTICE_STATES.map(function statement(state,): string {
      return `${subject} is ${state}`;
    },);
  },
),);

/**
 Characters that end a sentence, of either width.
 */
const SENTENCE_ENDERS: ReadonlySet<string> = new Set([
  '.',
  '!',
  '?',
  '。',
  '！',
  '？',
],);

/**
 Text without a leading callout label such as `[!Warning]`, which a page
 writes to set a notice apart.

 @param text - paragraph after its quote markers

 @returns Text after the label, trimmed, or the text unchanged when it opens
 with none

 @example
 ```ts
 withoutCalloutLabel({ text: '[!Caution] This is an unfinished version.', },); // 'This is an unfinished version.'
 ```
 */
function withoutCalloutLabel({ text, }: { readonly text: string; },): string {
  if (!text.startsWith('[!',))
    return text;
  /**
   Offset of the bracket closing the label, none as minus one.
   */
  const closeAt = text.indexOf(']',);
  return (closeAt === (-1))
    ? text
    : text.slice(closeAt + 1,)
      .trim();
}

/**
 Whether one character is no emphasis mark.

 @param character - one character

 @returns Whether it is neither an asterisk nor an underscore

 @example
 ```ts
 isNoEmphasisMark('C',); // true
 ```
 */
function isNoEmphasisMark(character: string,): boolean {
  return (character !== '*') && (character !== '_');
}

/**
 Text without the emphasis marks that wrap the whole of it.

 @param text - paragraph after its callout label

 @returns Text with the leading and trailing asterisks and underscores gone

 @example
 ```ts
 withoutEmphasis({ text: '**Come back later!**', },); // 'Come back later!'
 ```
 */
function withoutEmphasis({ text, }: { readonly text: string; },): string {
  /**
   The text's characters.
   */
  const characters = Array.from(text,);
  /**
   Offset of the first character that is no emphasis mark, none as minus one.
   */
  const first = characters.findIndex(isNoEmphasisMark,);
  if (first === (-1))
    return '';
  return characters
    .slice(
    first,
    characters.findLastIndex(isNoEmphasisMark,) + 1,
    )
    .join('',);
}

/**
 The sentences of a paragraph, lowercased, each with its words on single
 spaces.

 @param text - paragraph after its label and emphasis

 @returns Each non-empty sentence, split at full stops, exclamation marks and
 question marks of either width

 @example
 ```ts
 noticeSentences({ text: 'Come back later! Thanks.', },); // ['come back later', 'thanks']
 ```
 */
function noticeSentences({ text, }: { readonly text: string; },): readonly string[] {
  return Array.from(
    text.toLowerCase(),
    function breakAtEnd(character,): string {
      return SENTENCE_ENDERS.has(character,) ? '\n' : character;
    },
  )
    .join('',)
    .split('\n',)
    .map(function folded(sentence,): string {
      return sentence
        .split(' ',)
        .filter(function isWord(word,): boolean {
          return word.length > 0;
        },)
        .join(' ',);
    },)
    .filter(function isSentence(sentence,): boolean {
      return sentence.length > 0;
    },);
}

/**
 Whether one sentence is a notice that the page is unfinished.

 @param sentence - lowercased sentence on single spaces

 @returns Whether it says come back later, that the page is not the finished
 one, who will update it, or which other page to see

 @example
 ```ts
 isNoticeSentence({ sentence: 'this version is not the formal version', },); // true
 ```
 */
function isNoticeSentence({ sentence, }: { readonly sentence: string; },): boolean {
  if (NOTICE_SENTENCES.has(sentence,) || NOTICE_STATEMENTS.has(sentence,))
    return true;
  if (NOTICE_UPDATE_OPENERS.some(function opens(opener,): boolean {
    return sentence.startsWith(opener,);
  },))
    return true;
  /**
   The sentence's words.
   */
  const words = sentence.split(' ',);
  return NOTICE_REFER_OPENERS.some(function opens(opener,): boolean {
    return sentence.startsWith(opener,);
  },)
    && NOTICE_REFER_CLOSERS.some(function closes(closer,): boolean {
      return sentence.endsWith(closer,);
    },)
    && (words.length <= NOTICE_REFER_MAXIMUM_WORDS);
}

/**
 Whether one paragraph is nothing but a notice that the English page is
 unfinished, not the formal version, or waiting on another version.

 @param paragraph - paragraph text; whitespace, blockquote markers at any
 depth, one callout label and emphasis marks around the whole of it tolerated

 @returns Whether every sentence of it is such a notice

 @example
 ```ts
 isStatusNoticeParagraph({ paragraph: '**Come back later!**', },);
 ```
 */
export function isStatusNoticeParagraph({ paragraph, }: { readonly paragraph: string; },): boolean {
  /**
   Sentences of the paragraph without its quote markers, callout label and
   emphasis.
   */
  const sentences = noticeSentences({
    text: withoutEmphasis({
      text: withoutCalloutLabel({ text: withoutQuoteMarkers({ text: paragraph.trim(), },), },),
    },),
  },);
  return (sentences.length > 0) && sentences.every(function isNotice(sentence,): boolean {
    return isNoticeSentence({ sentence, },);
  },);
}

/**
 One retained normalized line with its unchanged pinned-file position.

 @example
 ```ts
 const retained: ArchiveRetainedLine = { text: 'Cat.', lineNumber: 3 };
 ```
 */
export type ArchiveRetainedLine = {
  /**
   Exact line after invisible-character normalization.
   */
  readonly text: string;
  /**
   One-based pinned-file line before any marker or adjacent blank removal.
   */
  readonly lineNumber: number;
};

/**
 Removes every paragraph that is nothing but a placeholder token.

 ONE LINEAR PASS over the lines, with HTML comments masked first so a marker
 inside a comment is left alone: the masked text keeps every newline, so its
 lines index the original's exactly. A marker paragraph is one line whose
 previous kept line is blank or absent and whose next line is blank or
 absent, outside front matter and code fences. The marker goes with one
 adjacent blank line: the following one, or the preceding one at the end of
 the document, so the page keeps single blank lines between blocks.

 @param text - archive text after the invisible-variant fold

 @returns Text without the markers, and each marker removed with its line

 @example
 ```ts
 const { text, stripped, lines, } = stripStubMarkersWithOrigins({ text: archive, },);
 ```
 */
export function stripStubMarkersWithOrigins(
  { text, }: { readonly text: string; },
): {
  readonly text: string;
  readonly stripped: readonly StrippedStubMarker[];
  readonly lines: readonly ArchiveRetainedLine[];
} {
  /**
   Original lines.
   */
  const lines = text.split('\n',);
  /**
   Same lines with every comment blanked, so a comment line differs from its
   original and a marker inside one is never read as a paragraph.
   */
  const maskedLines = maskHtmlComments({ text, },)
    .masked
    .split('\n',);
  /**
   Whether the document opens with front matter.
   */
  const opensWithFrontMatter = lines[0] === `${text.slice(
    0,
    leadingMarkWidth({ text, },),
  )}${FRONT_MATTER_FENCE}`;

  // ONE PASS THAT APPENDS. The scan was a fold that copied every line kept so
  // far at each line, so the pass this function's summary calls linear cost
  // the square of the line count (ledger B70).
  /**
   Lines kept so far.
   */
  const kept: ArchiveRetainedLine[] = [];
  /**
   Markers removed so far.
   */
  const stripped: StrippedStubMarker[] = [];
  /**
   Where the scan stands: inside the leading front matter, inside a fenced
   code block, and whether the next line, if blank, is the blank a removed
   marker owned.
   */
  const scan = {
    inFrontMatter: opensWithFrontMatter,
    inFence: false,
    skipBlank: false,
  };
  for (const [index, line,] of lines.entries()) {
    /**
     Text and original position stay in one record through every keep or removal branch.
     */
    const retainedLine: ArchiveRetainedLine = {
      text: line,
      lineNumber: index + 1,
    };
    /**
     This line as masked, unchanged when no comment touches it.
     */
    const maskedLine = nonNullishOrThrow(maskedLines[index],);
    if (scan.inFrontMatter) {
      kept.push(retainedLine,);
      scan.inFrontMatter = !((index > 0) && (line === FRONT_MATTER_FENCE));
      continue;
    }
    if (maskedLine.trimStart()
      .startsWith(CODE_FENCE,)) {
      kept.push(retainedLine,);
      scan.inFence = !scan.inFence;
      scan.skipBlank = false;
      continue;
    }
    if (scan.inFence) {
      kept.push(retainedLine,);
      continue;
    }
    if (scan.skipBlank && (line.trim() === '')) {
      scan.skipBlank = false;
      continue;
    }
    /**
     Whether no comment covers any of this line.
     */
    const outsideComment = maskedLine === line;
    /**
     Last kept line, or nothing at the document's start.
     */
    const previous = kept
      .at(-1,)
      ?.text
      ?? '';
    /**
     Whether nothing kept stands directly above.
     */
    const previousBlank = previous.trim() === '';
    /**
     Line after this one, absent when the document ends here.
     */
    const next = lines[index + 1];
    /**
     Whether the document ends with this line.
     */
    const atEnd = next === undefined;
    /**
     Whether the paragraph ends with this line.
     */
    const nextBlank = atEnd || (next.trim() === '');
    /**
     Whether this line stands as a paragraph of its own.
     */
    const standsAlone = outsideComment
      && previousBlank
      && nextBlank;
    /**
     Whether this line is a placeholder or a status notice paragraph of its own.
     */
    const isMarker = standsAlone
      && (isStubMarkerParagraph({ paragraph: line, },) || isStatusNoticeParagraph({ paragraph: line, },));
    if (!isMarker) {
      kept.push(retainedLine,);
      scan.skipBlank = false;
      continue;
    }
    /**
     Whether the blank above goes, since no line follows to give up its
     blank instead.
     */
    const dropsBlankAbove = atEnd
      && previousBlank
      && (kept.length > 0);
    if (dropsBlankAbove)
      kept.pop();
    stripped.push({
      lineNumber: index + 1,
      text: line,
    },);
    scan.skipBlank = !atEnd;
  }
  return {
    text: kept
      .map(function lineText(line,): string {
        return line.text;
      },)
      .join('\n',),
    stripped,
    lines: kept,
  };
}

//endregion Archive stub markers
