import { leadingMarkWidth, } from '@monochromatic-dev/translation-repair-document-structure/ts';
import { rendersAsNothing, } from '@monochromatic-dev/translation-repair-typography/ts';

//region Insertion separator
// The whitespace nobody owns.
//
// A slice covers the nodes it covers; the blank line BETWEEN two blocks belongs
// to neither of them, and until now nothing had to care: every replacement went
// into a span that already sat between the right separators, so writing model
// text verbatim preserved them. An insertion has no span. Written verbatim at a
// boundary it concatenates with whatever is there, so a rendering placed before
// a heading produces `...afternoon.## Habits`, which still parses as Markdown
// and is silently wrong.
//
// THE MODEL DOES NOT DECIDE THIS. A prompt asking for correct leading and
// trailing blank lines is a hope rather than a guarantee, it fails silently,
// and it cannot be right anyway: several fragments landing at one boundary each
// carrying their own blank lines would multiply the separators between them.
// Only assembly knows what is on both sides of the boundary, how many fragments
// share it, and what the document uses to separate blocks.
//
// ADD-ONLY. Existing whitespace is preserved byte for byte and topped up to one
// blank line where an insertion creates an adjacency that had none. Nothing
// here removes a separator the archive already had.

/**
 Two line endings, which is what separates blocks in a Markdown document.
 */
const BLOCK_SEPARATOR_LINES = 2;

/**
 What a string search answers with when it finds nothing.
 */
const NOT_FOUND = -1;

/**
 Where one fragment's words stand in the text written at an insertion boundary.

 @example
 ```ts
 const place: FragmentPlace = { fragment: 0, englishAt: 2, from: 4, to: 20, };
 ```
 */
export type FragmentPlace = {
  /**
   Which fragment of the boundary this is, counted from zero in the order the fragments were given.
   */
  readonly fragment: number;

  /**
   The offset where the fragment's first character would stand, so a character past the blank lines cut from the fragment's
   start stands at this offset plus its own; before `from` when blank lines were cut.
   */
  readonly englishAt: number;

  /**
   Where the fragment's body starts in the text.
   */
  readonly from: number;

  /**
   Where it ends, exclusive.
   */
  readonly to: number;
};

/**
 Reads the line ending a document uses.

 FROM THE DOCUMENT rather than from the platform. A translation written on
 Windows carries `\r\n`, and joining its blocks with bare `\n` produces a file
 with two conventions in it, which every later diff reports as a change to
 lines nobody touched.

 @param targetText - document being written into

 @returns Line ending to write, defaulting to `\n` for a document that shows
 no preference

 @example
 ```ts
 const eol = documentLineEnding({ targetText, },);
 ```
 */
export function documentLineEnding(
  { targetText, }: { readonly targetText: string; },
): string {
  return targetText.includes('\r\n',) ? '\r\n' : '\n';
}

/**
 Counts the line endings a text ends with, up to the block separator.

 @param text - text to look at the end of

 @param eol - line ending this document uses

 @returns How many trailing line endings there are, capped at
 {@link BLOCK_SEPARATOR_LINES}

 @example
 ```ts
 const trailing = trailingLineEndings({ text: 'a\n\n', eol: '\n', },);
 ```
 */
function trailingLineEndings(
  {
    text,
    eol,
  }: {
    readonly text: string;
    readonly eol: string;
  },
): number {
  /**
   How many have been counted so far, walking backwards.
   */
  const counted = [
    0,
    1,
  ]
    .filter(function isPresent(depth,): boolean {
      return text.endsWith(eol.repeat(depth + 1,),);
    },);
  return counted.length;
}

/**
 Counts the line endings a text begins with, up to the block separator.

 @param text - text to look at the start of

 @param eol - line ending this document uses

 @returns How many leading line endings there are, capped at
 {@link BLOCK_SEPARATOR_LINES}

 @example
 ```ts
 const leading = leadingLineEndings({ text: '\n\nb', eol: '\n', },);
 ```
 */
function leadingLineEndings(
  {
    text,
    eol,
  }: {
    readonly text: string;
    readonly eol: string;
  },
): number {
  /**
   How many have been counted so far, walking forwards.
   */
  const counted = [
    0,
    1,
  ]
    .filter(function isPresent(depth,): boolean {
      return text.startsWith(eol.repeat(depth + 1,),);
    },);
  return counted.length;
}

/**
 Strips the blank-line material around one fragment, and its trailing spaces.

 INDENTATION SURVIVES, which is the asymmetry between the two ends and why the
 leading side is not a plain trim. A fragment beginning with spaces on its
 first content line is inside a list or a block quote, and cutting that would
 move it out of the structure it belongs to. What is cut there is whitespace
 ending in a line ending, which is blank lines rather than indentation.

 THE TRAILING SIDE TAKES SPACES TOO, including the two that would make a
 Markdown hard break. That is safe HERE and only here: every caller reaches
 this through {@link composedInsertion}, which joins fragments with a blank
 line, and a hard break before a blank line breaks nothing. A join that ever
 put two fragments on consecutive lines would make those spaces meaningful
 again, and this would have to narrow to blank lines alone.

 @param fragment - text a lane produced for one slice

 @returns Same text without leading or trailing blank lines

 @example
 ```ts
 const body = fragmentBody({ fragment: '\n\n  The cat naps.\n\n', },);
 ```
 */
export function fragmentBody(
  { fragment, }: { readonly fragment: string; },): string {
  return fragmentCut({ fragment, },)
    .body;
}

/**
 Cuts the blank-line material from around one fragment and says where its body began.

 @param fragment - text a lane produced for one slice

 @returns The body, which {@link fragmentBody} gives, and the offset in the fragment where it starts, which a reader of the
 page the fragment lands in needs to find the fragment's words

 @example
 ```ts
 const { body, start, } = fragmentCut({ fragment: '\n\n  The cat naps.\n\n', },); // start: 2
 ```
 */
export function fragmentCut(
  { fragment, }: { readonly fragment: string; },
): {
  readonly body: string;
  readonly start: number;
} {
  /**
   How much of the fragment is left once its leading whitespace is gone.
   */
  const bodyLength = fragment.trimStart()
    .length;

  /**
   Where that whitespace run ends, which is where the fragment stops being
   blank.
   */
  const contentStart = fragment.length - bodyLength;

  /**
   Last line ending inside that run, which is where the blank lines stop and
   this fragment's own indentation begins.
   */
  const lastBreak = fragment.lastIndexOf(
    '\n',
    contentStart,
  );

  /**
   Where the body starts: after the blank lines, before any indentation.
   */
  const bodyStart = (lastBreak === NOT_FOUND) ? 0 : (lastBreak + 1);
  return {
    body: fragment.slice(bodyStart,)
      .trimEnd(),
    start: bodyStart,
  };
}

/**
 Builds the text to write at one insertion boundary and says where each fragment's words land in it.

 A FRAGMENT THAT SHOWS A READER NOTHING WRITES NOTHING, asked of
 `rendersAsNothing` (ledger B40): asked of an empty-string comparison, a
 fragment of one zero-width space or Hangul filler opened a paragraph of its
 own at the boundary, with separators around a block no reader sees.

 @param fragments - what the lanes produced for the slices anchored here, in
 document order

 @param before - document text preceding the boundary

 @param after - document text following it, as it will stand

 @param eol - line ending this document uses

 @returns The text to write, separators included, and where the words of each fragment stand in that text, in the order of
 the fragments; a fragment that shows a reader nothing writes nothing and has no entry, and the text is empty exactly when
 no fragment said anything

 @example
 ```ts
 const { text, placed, } = composedInsertion({ fragments, before, after, eol, },);
 ```
 */
export function composedInsertion(
  {
    fragments,
    before,
    after,
    eol,
  }: {
    readonly fragments: readonly string[];
    readonly before: string;
    readonly after: string;
    readonly eol: string;
  },
): {
  readonly text: string;
  readonly placed: readonly FragmentPlace[];
} {
  /**
   Each fragment cut to its body, with the offset in the fragment where the body starts.
   */
  const cuts = fragments.map(function toCut(fragment,): ReturnType<typeof fragmentCut> {
    return fragmentCut({ fragment, },);
  },);
  /**
   Whether each fragment says something to a reader.
   */
  const says = cuts.map(function saysSomething({ body, },): boolean {
    return !rendersAsNothing({ text: body, },);
  },);
  /**
   Fragments reduced to their own text, joined by one blank line.

   Joined here rather than by each fragment carrying its own blank lines,
   which would put two between every pair.
   */
  const spoken = cuts.filter(function isSpoken(
    _cut,
    at,
  ): boolean {
    return says[at] === true;
  },);
  /**
   The bodies of the fragments that say something, joined by one blank line.
   */
  const body = spoken
    .map(function toBody({ body: text, },): string {
      return text;
    },)
    .join(eol.repeat(BLOCK_SEPARATOR_LINES,),);
  // No fragment said anything: every one was filtered out, and the join of
  // none is the empty string exactly.
  if (body === '') {
    return {
      text: '',
      placed: [],
    };
  }

  /**
   Line endings the archive already provides before this boundary, which are
   kept and only topped up. A boundary after nothing but a byte order mark is
   the start of the document, since the mark is no content, so it needs none.
   */
  const kept = (before.slice(leadingMarkWidth({ text: before, },),) === '')
    ? BLOCK_SEPARATOR_LINES
    : trailingLineEndings({
      text: before,
      eol,
    },);

  /**
   Line endings the archive provides after it. A boundary at the very end of
   the document has none, and the text written there terminates the file
   instead, which is a different question from separating two blocks.
   */
  const following = (after === '')
    ? 0
    : leadingLineEndings({
      text: after,
      eol,
    },);

  /**
   What to write before the body, so the block ahead of it is separated.
   */
  const opening = eol.repeat(BLOCK_SEPARATOR_LINES - kept,);

  /**
   What to write after it, which for the end of a document is the single line
   ending a text file ends with.
   */
  const closing = (after === '')
    ? eol
    : eol.repeat(BLOCK_SEPARATOR_LINES - following,);
  /**
   Where the body of the next fragment that says something starts in the text, mutated only inside this function.
   */
  const cursor = { at: opening.length, };
  /**
   Where each fragment's words stand in the text.
   */
  const placed = cuts.flatMap(function placeOf(
    {
      body: text,
      start,
    },
    at,
  ): readonly FragmentPlace[] {
    if (says[at] !== true)
      return [];
    /**
     The offset where this fragment's body starts in the text.
     */
    const bodyAt = cursor.at;
    cursor.at += text.length + (eol.length * BLOCK_SEPARATOR_LINES);
    return [
      {
        fragment: at,
        englishAt: bodyAt - start,
        from: bodyAt,
        to: bodyAt + text.length,
      },
    ];
  },);
  return {
    text: opening
      + body
      + closing,
    placed,
  };
}

//endregion Insertion separator
