import { isHanCharacter, } from '@monochromatic-dev/translation-repair-markup/ts';
import { commentRegionsOf, } from './mask-html-comments.ts';

//region Translators' notes
// A NOTE BETWEEN TRANSLATORS IS A COMMENT NO READER SEES. XingZ60's archive
// ends a sentence with `<!-- 前面这句中文懂什么意思，先这么翻译，大家看一下 -->`
// ("does anyone understand the Chinese before this; translated like this for
// now; everyone take a look"), a note the archive's translators left each
// other in their own language and the original does not carry. The repair
// lane keeps a comment as markup, the translate lane does not, and all three
// lane-contest judges chose the repair rendering for keeping it, one writing
// that the other "drops the editor's comment note ... which is page
// apparatus". A rendering was chosen for text no reader reads.
//
// WHAT COUNTS AS ONE. A closed HTML comment or a closed JSX comment whose
// words hold Han, on a page written in English: the language of the note is
// the evidence that it was written for translators and not rendered from the
// original. Measured over the 92 pinned English archives: ten carry such
// comments, 34 closed ones (33 by a line-based search; `mikaela_khara`,
// `shi_Yumiaoya`, `XIEPT2` and `hakureico` among them), and none of the 34
// appears verbatim in its original. The original's own comments are the other kind: the contributor
// credits (`条目贡献`) are translated into English by the archive, so no Han
// remains in them, and a comment the original carries in Han and a candidate
// repeats verbatim is the original's own markup and stays in every view.
//
// AN OPEN COMMENT IS NEVER ONE. An unterminated opener swallows the rest of
// the page in the parser's mask (`mask-html-comments.ts`), so there is no end
// to cut at.
//
// TWO USES, ONE DEFINITION. `withoutTranslatorsNotes` is what the judging
// sheets show (no stage votes on a note), and `archiveNotesMissingFrom` is
// what the page assembly carries back onto the page (no vote loses one).

/**
 Opening delimiter of a JSX comment.
 */
const JSX_COMMENT_OPEN = '{/*';

/**
 Closing delimiter of a JSX comment.
 */
const JSX_COMMENT_CLOSE = '*/}';

/**
 One comment of a text, in offsets of that text.

 @example
 ```ts
 const note: TranslatorsNote = { startOffset: 12, endOffset: 40, text: '<!-- 大家看一下 -->', };
 ```
 */
export type TranslatorsNote = {
  /**
   Offset of the comment's opening delimiter.
   */
  readonly startOffset: number;

  /**
   Exclusive offset past the comment's closing delimiter.
   */
  readonly endOffset: number;

  /**
   The comment as the text writes it, delimiters included.
   */
  readonly text: string;
};

/**
 The closed JSX comments between two offsets.

 @param text - text read

 @param from - first offset to read

 @param to - exclusive offset past the last one to read

 @returns Each closed comment inside the stretch, in source order

 @example
 ```ts
 const spans = jsxCommentsBetween({ text: 'a {/* b *\/} c', from: 0, to: 13, },); // one, from 2 to 11
 ```
 */
function jsxCommentsBetween(
  {
    text,
    from,
    to,
  }: {
    readonly text: string;
    readonly from: number;
    readonly to: number;
  },
): readonly TranslatorsNote[] {
  /**
   Comments found, in source order.
   */
  const found: TranslatorsNote[] = [];
  /**
   Scan position.
   */
  let cursor = from;
  while (cursor < to) {
    /**
     Next opener at or past the cursor.
     */
    const openAt = text.indexOf(
      JSX_COMMENT_OPEN,
      cursor,
    );
    if ((openAt === (-1)) || (openAt >= to))
      break;
    /**
     Closer past the opener.
     */
    const closeAt = text.indexOf(
      JSX_COMMENT_CLOSE,
      openAt + JSX_COMMENT_OPEN.length,
    );
    if (closeAt === (-1))
      break;
    /**
     Exclusive end of the comment.
     */
    const endOffset = closeAt + JSX_COMMENT_CLOSE.length;
    if (endOffset > to)
      break;
    found.push({
      startOffset: openAt,
      endOffset,
      text: text.slice(
        openAt,
        endOffset,
      ),
    },);
    cursor = endOffset;
  }
  return found;
}

/**
 Every closed comment of a text, HTML and JSX, in source order.

 @param text - text read

 @returns Each closed comment; an open HTML comment, which runs to the end of
 the text, and whatever it hides are left out

 @example
 ```ts
 const comments = closedCommentsOf({ text: 'a<!-- b -->c', },); // one, from 1 to 11
 ```
 */
function closedCommentsOf({ text, }: { readonly text: string; },): readonly TranslatorsNote[] {
  /**
   Comments found, in source order.
   */
  const found: TranslatorsNote[] = [];
  /**
   Offset where the stretch not yet read for JSX comments starts.
   */
  let cursor = 0;
  for (const region of commentRegionsOf({ text, },)) {
    found.push(...jsxCommentsBetween({
      text,
      from: cursor,
      to: region.startOffset,
    },),);
    if (region.terminated)
      found.push({
        startOffset: region.startOffset,
        endOffset: region.endOffset,
        text: text.slice(
          region.startOffset,
          region.endOffset,
        ),
      },);
    cursor = region.endOffset;
  }
  found.push(...jsxCommentsBetween({
    text,
    from: cursor,
    to: text.length,
  },),);
  return found;
}

/**
 Whether a text holds a Han ideograph.

 @param text - text read

 @returns Whether any of its code points is Han

 @example
 ```ts
 holdsHan({ text: '大家看一下', },); // true
 ```
 */
function holdsHan({ text, }: { readonly text: string; },): boolean {
  return Array.from(text,)
    .some(function isHan(character,): boolean {
      return isHanCharacter({ character, },);
    },);
}

/**
 Whether one character is whitespace by the same test `trim` uses.

 @param character - one character

 @returns Whether trimming it alone leaves nothing

 @example
 ```ts
 isWhitespace({ character: '\n', },); // true
 ```
 */
function isWhitespace({ character, }: { readonly character: string; },): boolean {
  /**
   The character with whitespace removed.
   */
  const trimmed = character.trim();
  return trimmed.length === 0;
}

/**
 A comment with every run of whitespace made one space, so a note a lane
 re-wrapped is the same note.

 @param text - comment as a text writes it

 @returns Comment on one line

 @example
 ```ts
 const folded = foldedComment({ text: '<!-- a\n  b -->', },); // '<!-- a b -->'
 ```
 */
function foldedComment({ text, }: { readonly text: string; },): string {
  return Array.from(
    text,
    function toSpace(character,): string {
      return isWhitespace({ character, },) ? ' ' : character;
    },
  )
    .join('',)
    .split(' ',)
    .filter(function isWord(word,): boolean {
      return word.length > 0;
    },)
    .join(' ',);
}

/**
 The notes between translators in a text: closed comments whose words hold Han.

 @param text - text read, an English page or one slice of it

 @returns Each note in source order, with its offsets

 @example
 ```ts
 const notes = translatorsNotesIn({ text: 'The cat slept. <!-- 大家看一下 -->', },); // one, from 15 to 31
 ```
 */
export function translatorsNotesIn({ text, }: { readonly text: string; },): readonly TranslatorsNote[] {
  return closedCommentsOf({ text, },)
    .filter(function isNote(comment,): boolean {
      return holdsHan({ text: comment.text, },);
    },);
}

/**
 A stretch of a text to take out.

 @example
 ```ts
 const cut: Cut = { from: 1, to: 13, };
 ```
 */
export type Cut = {
  /**
   First offset taken out.
   */
  readonly from: number;

  /**
   Exclusive offset where the text goes on.
   */
  readonly to: number;
};

/**
 Whether what stands before a note on its line is nothing but Markdown quote
 markers and the space between them, as `>   > ` is.

 A LINE OF MARKERS ALONE IS A BLANK LINE OF THE QUOTE, the way an empty line
 is of the page, so a note that is the only content of a quoted line must be
 told from one that shares the line with words. The test is on the characters
 before the note and never on the note's own words, so a `>` inside a note
 changes nothing. `provider-stream/src/quote-line.ts` reads the same head
 (`carriesContent`); it is not imported because that package depends on this
 one.

 @param head - what stands between the line's start and the note

 @returns Whether the head holds a quote marker and nothing else but
 whitespace; an empty or blank head holds no marker and is not one

 @example
 ```ts
 isQuoteHead({ head: '>   > ', },); // true
 isQuoteHead({ head: '1 > ', },); // false
 ```
 */
function isQuoteHead({ head, }: { readonly head: string; },): boolean {
  /**
   The head's characters.
   */
  const characters = Array.from(head,);
  return characters.includes('>',)
    && characters.every(function isMarkerOrSpace(character,): boolean {
      return (character === '>') || isWhitespace({ character, },);
    },);
}

/**
 Whether the text between two offsets is nothing but whitespace that is not a
 line feed, or nothing at all.

 @param text - text read

 @param from - first offset of the stretch

 @param to - exclusive offset past the stretch

 @returns Whether the stretch holds no word and no line feed; a stretch whose
 offsets meet is empty and counts

 @example
 ```ts
 isGapOnOneLine({ text: 'a  b', from: 1, to: 3, },); // true
 isGapOnOneLine({ text: 'a\n b', from: 1, to: 3, },); // false
 ```
 */
function isGapOnOneLine(
  {
    text,
    from,
    to,
  }: {
    readonly text: string;
    readonly from: number;
    readonly to: number;
  },
): boolean {
  return Array.from(text.slice(
    from,
    to,
  ),)
    .every(function isSpaceOnTheLine(character,): boolean {
      return (character !== '\n') && isWhitespace({ character, },);
    },);
}

/**
 The stretches of a text that hold notes, a run of notes with nothing but
 whitespace on their line between them being one stretch.

 TWO NOTES SIDE BY SIDE ARE ONE NOTE TO CUT. A stretch is judged from the
 text around it, and the text around the second of two notes on a line is the
 first, which is no word; judged note by note against the cut made so far, the
 second of them read an empty head and took the line break with it, so the
 line after them was joined onto theirs. The stretch is judged against the
 original text, once, and a note with a word or a line break between it and
 the next stands as a stretch of its own.

 @param text - text the notes stand in

 @param notes - the notes to cut, in source order

 @returns One stretch for each run of notes, in source order and apart from
 each other: a word or a line break stands between any two

 @example
 ```ts
 const stretches = noteStretchesOf({ text: 'a <!-- 注 --> {/* 注 *\/} b', notes, },); // one, from 2 to 22
 ```
 */
function noteStretchesOf(
  {
    text,
    notes,
  }: {
    readonly text: string;
    readonly notes: readonly TranslatorsNote[];
  },
): readonly Cut[] {
  /**
   Stretches found, in source order.
   */
  const stretches: Cut[] = [];
  for (const note of notes) {
    /**
     The stretch the previous note ended, or nothing before the first note.
     */
    const previous = stretches.pop();
    if (
      (previous !== undefined)
        && isGapOnOneLine({
          text,
          from: previous.to,
          to: note.startOffset,
        },)
    )
      stretches.push({
        from: previous.from,
        to: note.endOffset,
      },);
    else {
      if (previous !== undefined)
        stretches.push(previous,);
      stretches.push({
        from: note.startOffset,
        to: note.endOffset,
      },);
    }
  }
  return stretches;
}

/**
 Offset a removal of the notes in `stretch` starts from, and the one it ends at.

 A COMMENT ALONE ON ITS LINES TAKES THE LINES WITH IT, so the removal leaves
 neither an empty line that would split a paragraph nor the line's break; a
 comment alone past its line's quote markers (`>   > {/* 注 *\/}`) takes the
 line the same way, since the markers it would leave behind are a blank line
 of the quote that splits the quoted paragraph; a comment ending its line
 takes the spaces before it and after it, but not the carriage return of a
 CRLF break, which would leave a bare line feed among CRLF ones; a comment
 between words takes one of the two
 spaces around it. Notes with only whitespace on their line between them are
 the one comment these rules speak of (`noteStretchesOf`), and the line is judged from the
 original text.

 @param text - text the stretch stands in

 @param stretch - the notes to cut, from the first one's opening delimiter to
 the last one's end

 @returns The stretch to remove, in offsets of the text

 @example
 ```ts
 const cut = cutAround({ text: 'a <!-- 注 -->', stretch: { from: 2, to: 13, }, },); // from 1 to 13
 ```
 */
function cutAround(
  {
    text,
    stretch,
  }: {
    readonly text: string;
    readonly stretch: Cut;
  },
): Cut {
  /**
   Offset where the stretch's first line starts.
   */
  const lineStart = text.lastIndexOf(
    '\n',
    stretch.from - 1,
  ) + 1;
  /**
   First line break at or past the stretch's end, none as minus one.
   */
  const foundBreak = text.indexOf(
    '\n',
    stretch.to,
  );
  /**
   Offset of the line break ending the stretch's last line, or the text's end.
   */
  const breakAt = (foundBreak === (-1)) ? text.length : foundBreak;
  /**
   What stands before the stretch on its line.
   */
  const before = text.slice(
    lineStart,
    stretch.from,
  );
  /**
   What stands after the stretch on its line.
   */
  const after = text.slice(
    stretch.to,
    breakAt,
  );
  if (((before.trim() === '') || isQuoteHead({ head: before, },)) && (after.trim() === ''))
    return {
      from: lineStart,
      to: Math.min(
        breakAt + 1,
        text.length,
      ),
    };
  /**
   What stands before the stretch, without the spaces and tabs that end it.
   */
  const words = before.trimEnd();
  /**
   What stands after the stretch on its line, without the carriage return
   that ends it in a CRLF text, which the cut leaves to its line feed.
   */
  const trailing = after.endsWith('\r',) ? after.slice(
    0,
    -1,
  ) : after;
  if (after.trim() === '')
    return {
      from: stretch.from - (before.length - words.length),
      to: stretch.to + trailing.length,
    };
  if (before.endsWith(' ',) && after.startsWith(' ',))
    return {
      from: stretch.from,
      to: stretch.to + 1,
    };
  return stretch;
}

/**
 The stretches a judge's view of a text takes out: each run of notes between
 translators with the space or line it stood on, the original's own comments
 left in.

 THE ONE PLACE THE CUTS ARE MADE. The text a judge is shown
 (`withoutTranslatorsNotes`) and the map from that text back to the stored one
 (`translators-note-view.ts`) both read these stretches, so they cannot
 disagree about which note was cut.

 @param text - passage whose notes are cut

 @param original - the original passage, whose own comments stay: a note found
 verbatim in it is the original's markup, whatever language it is in

 @returns The stretches to take out, in source order and apart from each other;
 none for a text with no note of its own

 @example
 ```ts
 const cuts = noteCutsOf({ text: 'The cat slept. <!-- 大家看一下 -->', },); // one, from 14 to 29
 ```
 */
export function noteCutsOf(
  {
    text,
    original = '',
  }: {
    readonly text: string;
    readonly original?: string;
  },
): readonly Cut[] {
  /**
   Notes that are the passage's, not the original's.
   */
  const notes = translatorsNotesIn({ text, },)
    .filter(function isNotOriginals(note,): boolean {
      return !original.includes(note.text,);
    },);
  return noteStretchesOf({
    text,
    notes,
  },)
    .map(function cutOf(stretch,): Cut {
      return cutAround({
        text,
        stretch,
      },);
    },);
}

/**
 A text as a judge is shown it: without the notes between translators, but
 with the comments the original itself carries.

 @param text - passage whose wording is compared

 @param original - the original passage, whose own comments stay: a note found
 verbatim in it is the original's markup, whatever language it is in

 @returns Text with each note cut out along with the space or line it stood on

 @example
 ```ts
 const shown = withoutTranslatorsNotes({ text: 'The cat slept. <!-- 大家看一下 -->', },); // 'The cat slept.'
 ```
 */
export function withoutTranslatorsNotes(
  {
    text,
    original = '',
  }: {
    readonly text: string;
    readonly original?: string;
  },
): string {
  /**
   The stretch each run of notes takes out, each starting past the one before.
   */
  const cuts = noteCutsOf({
    text,
    original,
  },);
  if (cuts.length === 0)
    return text;
  return [
    ...cuts.map(function keptBefore(
      cut,
      index,
    ): string {
      return text.slice(
        cuts[index - 1]
          ?.to
          ?? 0,
        cut.from,
      );
    },),
    text.slice(
      cuts.at(-1,)
        ?.to
        ?? 0,
    ),
  ].join('',);
}

/**
 The archive's notes between translators that a page's wording does not carry.

 @param archive - the archive's text for a slice

 @param page - the text the page carries for it

 @returns The archive's notes whose words no comment of the page repeats, in
 archive order; a note re-wrapped across lines counts as carried

 @example
 ```ts
 const missing = archiveNotesMissingFrom({ archive, page, },);
 ```
 */
export function archiveNotesMissingFrom(
  {
    archive,
    page,
  }: {
    readonly archive: string;
    readonly page: string;
  },
): readonly TranslatorsNote[] {
  /**
   Comments the page carries, folded.
   */
  const carried = new Set(closedCommentsOf({ text: page, },)
    .map(function folded(comment,): string {
      return foldedComment({ text: comment.text, });
    },),);
  return translatorsNotesIn({ text: archive, },)
    .filter(function isMissing(note,): boolean {
      return !carried.has(foldedComment({ text: note.text, },),);
    },);
}

//endregion Translators' notes
