//region Markdown fence walk
// WHICH LINES OF A MARKDOWN TEXT LIE OUTSIDE A FENCED CODE BLOCK, AS COMMONMARK READS ONE (ledger B1376, B1413). A fence
// opens with a run of three or more backticks or tildes, indented at most three columns past the container it sits in; a
// backtick fence's info string holds no backtick; it closes at a run of the same character at least as long, indented at
// most three columns past the same container and followed by nothing but spaces; and it ends with its container. The
// installed reader says the same in code (`micromark-core-commonmark@2.0.4`, `lib/code-fenced.js`: the opener needs a run
// of three, the closing fence takes a line prefix of at most three columns before its sequence, and a fence that meets a
// line the container does not continue ends).
//
// THE INDENT IS COUNTED FROM THE CONTAINER. A line of four columns and three backticks is indented code at the top of a
// document (or the lazy continuation of a paragraph, since indented code cannot interrupt one), and is a fence inside a
// numbered item whose text starts at the fourth column, which is how the runbook of the corpus pass writes its commands.
// The containers read are the list item, whose content column is its marker and the spaces after it, and the block
// quote, whose `>` markers come off the front of the line.
//
// THE LIMITS. A container is read from its first line: the marker of a list item and the quote markers. A list item is
// ended by a line indented less than its content column unless that line continues a paragraph (a lazy line), which is
// read as a line that opens no block. Indented code, an HTML block and a table are not read, so a fence run inside one of
// them is read as a fence (ledger B1413 compares this reading with the one before it over the family's docs). A tab
// counts to the next multiple of four columns.

/**
 Columns of indent that make a line indented code and not a fence: one past the most a fence may be indented.
 */
const FENCE_INDENT_LIMIT = 3;

/**
 Shortest run of fence characters that opens or closes a fence.
 */
const FENCE_RUN_MINIMUM = 3;

/**
 Columns a tab advances to, as a multiple.
 */
const TAB_STOP = 4;

/**
 Spaces after a list marker at and past which the item's text is indented code and its content column is one past the
 marker.
 */
const CODE_SPACES = 5;

/**
 Most digits a numbered list marker may have.
 */
const MARKER_DIGITS_LIMIT = 9;

/**
 Characters a fence is made of.
 */
const FENCE_CHARACTERS: ReadonlySet<string> = new Set([
  '`',
  '~',
]);

/**
 Characters that mark a bulleted list item.
 */
const BULLET_CHARACTERS: ReadonlySet<string> = new Set([
  '-',
  '*',
  '+',
]);

/**
 Characters that close the number of a numbered list item.
 */
const NUMBER_CLOSERS: ReadonlySet<string> = new Set([
  '.',
  ')',
]);

/**
 Characters a thematic break is made of.
 */
const BREAK_CHARACTERS: ReadonlySet<string> = new Set([
  '-',
  '*',
  '_',
]);

/**
 The digits a numbered marker is made of.
 */
const DIGITS = '0123456789';

/**
 A fence that is open.
 */
type Fence = {
  /**
   The backtick or the tilde the fence is made of.
   */
  readonly character: string;

  /**
   Length of the run that opened it, which the closing run must match or pass.
   */
  readonly length: number;

  /**
   Number of block quote markers the fence sits behind.
   */
  readonly quoteDepth: number;

  /**
   Column the container's content starts at, from which the fence's indent is counted; none for a fence at the top level.
   */
  readonly contentColumn: number;
};

/**
 A line with its indent measured and taken off.
 */
type Indented = {
  /**
   Columns of indent.
   */
  readonly columns: number;

  /**
   The line after its indent.
   */
  readonly body: string;
};

/**
 A list item the line opens: the column its text starts at, and the text after the marker.
 */
type ListItem = {
  /**
   Column the item's content starts at.
   */
  readonly contentColumn: number;

  /**
   Text after the marker and the spaces that belong to it, or the empty text where the item holds none.
   */
  readonly content: string;

  /**
   Whether the spaces after the marker leave the content as indented code, five columns or more.
   */
  readonly contentIsCode: boolean;
};

/**
 What a line is read as where it opens no fence.
 */
const NO_FENCE: unique symbol = Symbol('the line opens no fence',);

/**
 What a line is read as where it opens no list item.
 */
const NOT_AN_ITEM: unique symbol = Symbol('the line opens no list item',);

/**
 Measures the indent of a line and takes it off.

 @param line - one line of Markdown

 @returns Columns of indent, a tab advancing to the next multiple of four, and the rest of the line

 @example
 ```ts
 const { columns, body, } = indentedOf({ line: '    ```sh', },);
 ```
 */
function indentedOf({ line, }: { readonly line: string; },): Indented {
  return (function measure(): Indented {
    /**
     Columns counted so far.
     */
    let columns = 0;
    for (let at = 0; at < line.length; at += 1) {
      /**
       Character read.
       */
      const character = line.charAt(at,);
      if (character === ' ')
        columns += 1;
      else if (character === '\t')
        columns += TAB_STOP - (columns % TAB_STOP);
      else {
        return {
          columns,
          body: line.slice(at,),
        };
      }
    }
    return {
      columns,
      body: '',
    };
  })();
}

/**
 Takes the block quote markers off the front of a line.

 @param line - one line of Markdown

 @param limit - most markers to take

 @returns How many were taken and the rest of the line

 @example
 ```ts
 const { depth, rest, } = afterQuoteMarkers({ line: '> > text', limit: 2, },);
 ```
 */
function afterQuoteMarkers(
  {
    line,
    limit,
  }: {
    readonly line: string;
    readonly limit: number;
  },
): {
  readonly depth: number;
  readonly rest: string
} {
  return (function take(): {
    readonly depth: number;
    readonly rest: string
  } {
    /**
     Markers taken so far.
     */
    let depth = 0;
    /**
     What is left of the line.
     */
    let rest = line;
    while (depth < limit) {
      /**
       The left of the line without its indent.
       */
      const {
        columns,
        body,
      } = indentedOf({ line: rest, },);
      if ((columns > FENCE_INDENT_LIMIT) || (!body.startsWith('>',)))
        break;
      /**
       What follows the marker.
       */
      const afterMarker = body.slice(1,);
      // ONE SPACE AFTER THE MARKER BELONGS TO IT.
      rest = afterMarker.startsWith(' ',) ? afterMarker.slice(1,) : afterMarker;
      depth += 1;
    }
    return {
      depth,
      rest,
    };
  })();
}

/**
 The run of one fence character a body opens with.

 @param body - line without its indent

 @returns Character, length of its run and the text after it, or the mark of a body that does not open with one

 @example
 ```ts
 const run = fenceRunOf({ body: '~~~~text', },);
 ```
 */
function fenceRunOf(
  { body, }: { readonly body: string; },
): {
  readonly character: string;
  readonly length: number;
  readonly rest: string
} | typeof NO_FENCE {
  /**
   Its first character.
   */
  const character = body.charAt(0,);
  if (!FENCE_CHARACTERS.has(character,))
    return NO_FENCE;
  /**
   Length of the run of that character.
   */
  const length = Array.from(body,)
    .findIndex(function endsRun(next,): boolean {
      return next !== character;
    },);
  /**
   The run's length, which is the whole body where nothing else follows.
   */
  const runLength = (length === (-1)) ? body.length : length;
  return {
    character,
    length: runLength,
    rest: body.slice(runLength,),
  };
}

/**
 Whether a body opens a fence, apart from its indent: three or more backticks or tildes, and for a backtick fence an
 info string with no backtick in it.

 @param body - line without its indent

 @returns Character and length of the run that opens it, or the mark of a body that opens none

 @example
 ```ts
 const opened = fenceOpenedBy({ body: '````ts', },);
 ```
 */
function fenceOpenedBy(
  { body, }: { readonly body: string; },
): {
  readonly character: string;
  readonly length: number
} | typeof NO_FENCE {
  /**
   Run the body opens with.
   */
  const run = fenceRunOf({ body, },);
  if ((run === NO_FENCE) || (run.length < FENCE_RUN_MINIMUM))
    return NO_FENCE;
  if ((run.character === '`')
    && run.rest
      .includes('`',))
    return NO_FENCE;
  return {
    character: run.character,
    length: run.length,
  };
}

/**
 Whether a thematic break is written in a body: three or more of one of `-`, `*` and `_`, with spaces among them.

 @param body - line without its indent

 @returns True when it is a thematic break, which a list marker is not

 @example
 ```ts
 const isBreak = isThematicBreak({ body: '- - -', },);
 ```
 */
function isThematicBreak({ body, }: { readonly body: string; },): boolean {
  /**
   The body without its spaces and tabs.
   */
  const marks = body.replaceAll(
    ' ',
    '',
  )
    .replaceAll(
    '\t',
    '',
    );
  /**
   Its first character.
   */
  const first = marks.charAt(0,);
  return (marks.length >= FENCE_RUN_MINIMUM)
    && BREAK_CHARACTERS.has(first,)
    && Array.from(marks,)
      .every(function isFirst(mark,): boolean {
        return mark === first;
      },);
}

/**
 Reads the list item a body opens.

 @param body - line without its indent

 @param indent - columns of that indent

 @returns Column the item's text starts at and that text, or the mark of a body that opens no item

 @example
 ```ts
 const item = listItemOf({ body: '1.  Confirm.', indent: 0, },);
 ```
 */
function listItemOf(
  {
    body,
    indent,
  }: {
    readonly body: string;
    readonly indent: number;
  },
): ListItem | typeof NOT_AN_ITEM {
  if (isThematicBreak({ body, },))
    return NOT_AN_ITEM;
  /**
   Length of the marker: one for a bullet, the digits and the closing mark for a number.
   */
  const markerLength = (function measureMarker(): number {
    if (BULLET_CHARACTERS.has(body.charAt(0,),))
      return 1;
    /**
     Digits the body opens with.
     */
    const digits = Array.from(body,)
      .findIndex(function endsDigits(next,): boolean {
        return !DIGITS.includes(next,);
      },);
    return ((digits > 0) && (digits <= MARKER_DIGITS_LIMIT)
      && NUMBER_CLOSERS.has(body.charAt(digits,),)) ? digits + 1 : 0;
  })();
  if (markerLength === 0)
    return NOT_AN_ITEM;
  /**
   Text after the marker.
   */
  const after = body.slice(markerLength,);
  if (after === '') {
    return {
      contentColumn: indent + markerLength
        + 1,
      content: '',
      contentIsCode: false,
    };
  }
  /**
   Spaces between the marker and the text.
   */
  const spaced = indentedOf({ line: after, },);
  if ((spaced.columns === 0) || ((!after.startsWith(' ',)) && (!after.startsWith('\t',))))
    return NOT_AN_ITEM;
  /**
   Whether five spaces or more leave the text as indented code.
   */
  const contentIsCode = spaced.columns >= CODE_SPACES;
  return {
    contentColumn: indent + markerLength
      + (contentIsCode ? 1 : spaced.columns),
    content: contentIsCode ? after.slice(1,) : spaced.body,
    contentIsCode,
  };
}

/**
 Whether a line closes the open fence: a run of the same character at least as long as the opening one, indented at most
 three columns past the container, and nothing but spaces after it.

 @param body - line without its indent

 @param relative - columns of its indent past the container's content column

 @param fence - fence that is open

 @returns True when the line closes it

 @example
 ```ts
 const closes = closesFence({ body: '```', relative: 0, fence, },);
 ```
 */
function closesFence(
  {
    body,
    relative,
    fence,
  }: {
    readonly body: string;
    readonly relative: number;
    readonly fence: Fence;
  },
): boolean {
  if (relative > FENCE_INDENT_LIMIT)
    return false;
  /**
   Run the line opens with.
   */
  const run = fenceRunOf({ body, },);
  if (run === NO_FENCE)
    return false;
  return (run.character === fence.character)
    && (run.length >= fence.length)
    && (run.rest
      .trim()
      === '');
}

/**
 The list items a line is still inside.

 A LINE INDENTED LESS THAN AN ITEM'S CONTENT ENDS THE ITEM, unless it continues the paragraph the item holds (a lazy
 line), which a line that opens no block does.

 @param items - content columns of the open items, outermost first

 @param columns - indent of the line

 @param endsLazyLines - whether the line is no lazy continuation, because it opens a block or no paragraph is open

 @returns Items the line does not end, outermost first

 @example
 ```ts
 const open = itemsStillOpen({ items: [4,], columns: 0, endsLazyLines: true, },);
 ```
 */
function itemsStillOpen(
  {
    items,
    columns,
    endsLazyLines,
  }: {
    readonly items: readonly number[];
    readonly columns: number;
    readonly endsLazyLines: boolean;
  },
): readonly number[] {
  if (!endsLazyLines)
    return items;
  return items.filter(function containsLine(contentColumn,): boolean {
    return columns >= contentColumn;
  },);
}

/**
 The one-based numbers of the lines of a Markdown text that lie outside fenced code blocks, a fence's own lines left out.

 @param text - Markdown text

 @returns Line numbers in order

 @example
 ```ts
 // The fence and what it holds are left out; the line after it is read.
 const outside = linesOutsideFences({ text: '```\ncode\n```\nprose\n', },);
 ```
 */
export function linesOutsideFences({ text, }: { readonly text: string; },): readonly number[] {
  /**
   The fence the walk is inside, once it is inside one.
   */
  let fence: Fence | typeof NO_FENCE = NO_FENCE;
  /**
   Content columns of the list items the walk is inside, outermost first.
   */
  let items: readonly number[] = [];
  /**
   Block quote markers the previous line sat behind.
   */
  let quoteDepth = 0;
  /**
   Whether a paragraph is open, so that a line which opens no block continues it.
   */
  let paragraphOpen = false;
  /**
   Line numbers outside fences.
   */
  const outside: number[] = [];
  for (const [at, line,] of text.split('\n',)
    .entries()) {
    if (fence !== NO_FENCE) {
      /**
       The line behind the quote markers the fence sits behind.
       */
      const quoted = afterQuoteMarkers({
        line,
        limit: fence.quoteDepth,
      },);
      /**
       Its indent.
       */
      const {
        columns,
        body,
      } = indentedOf({ line: quoted.rest, },);
      // A FENCE ENDS WITH ITS CONTAINER: a line behind fewer quote markers, or a text line indented less than the item's
      // content column, belongs to no container the fence sits in, and is read again as an ordinary line.
      /**
       Whether a container of the fence ended at this line.
       */
      const containerEnded = (quoted.depth < fence.quoteDepth) || ((body !== '') && (columns < fence.contentColumn));
      if (!containerEnded) {
        if (closesFence({
          body,
          relative: columns - fence.contentColumn,
          fence,
        },)) {
          fence = NO_FENCE;
          paragraphOpen = false;
        }
        continue;
      }
      fence = NO_FENCE;
    }
    /**
     The line behind its quote markers.
     */
    const quoted = afterQuoteMarkers({
      line,
      limit: Number.POSITIVE_INFINITY,
    },);
    if (quoted.depth !== quoteDepth) {
      items = [];
      quoteDepth = quoted.depth;
      paragraphOpen = false;
    }
    /**
     Its indent.
     */
    const {
      columns,
      body,
    } = indentedOf({ line: quoted.rest, },);
    if (body.trim() === '') {
      paragraphOpen = false;
      outside.push(at + 1,);
      continue;
    }
    /**
     Whether the line opens a block of its own, which ends a list item it is not indented into.
     */
    const opensBlock = (fenceOpenedBy({ body, },) !== NO_FENCE) || (listItemOf({
      body,
      indent: columns,
    },) !== NOT_AN_ITEM);
    items = itemsStillOpen({
      items,
      columns,
      endsLazyLines: opensBlock || (!paragraphOpen),
    },);
    /**
     Column the innermost container's content starts at.
     */
    const contentColumn = items.at(-1,) ?? 0;
    if ((columns - contentColumn) <= FENCE_INDENT_LIMIT) {
      /**
       The fence this line opens, when it opens one.
       */
      const opened = fenceOpenedBy({ body, },);
      if (opened !== NO_FENCE) {
        fence = {
          ...opened,
          quoteDepth,
          contentColumn,
        };
        paragraphOpen = false;
        continue;
      }
      /**
       The list item this line opens, when it opens one.
       */
      const item = listItemOf({
        body,
        indent: columns,
      },);
      if (item !== NOT_AN_ITEM) {
        items = [
          ...items,
          item.contentColumn,
        ];
        /**
         The fence the marker's own line opens, when its text does.
         */
        const onMarkerLine: ReturnType<typeof fenceOpenedBy> = item.contentIsCode
          ? NO_FENCE
          : fenceOpenedBy({ body: item.content, },);
        if (onMarkerLine !== NO_FENCE) {
          fence = {
            ...onMarkerLine,
            quoteDepth,
            contentColumn: item.contentColumn,
          };
          paragraphOpen = false;
          continue;
        }
      }
    }
    paragraphOpen = true;
    outside.push(at + 1,);
  }
  return outside;
}

//endregion Markdown fence walk
