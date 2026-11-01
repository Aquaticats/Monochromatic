//region Fenced code line flags
// Says, for each line of a body, whether that line belongs to a fenced code
// block. Exists so that masking passes can leave code alone.
//
// Inside a fence, a line holding a zero-width space is CONTENT. Blanking it
// rewrites the document being repaired, which is the one thing a
// length-preserving mask exists to avoid.
//
// Ambiguity resolves toward reporting a line as fenced wherever there is a
// choice. A line wrongly reported fenced merely goes unmasked, which is the
// behaviour that shipped before masking existed; a line wrongly reported
// unfenced can have its content silently rewritten. The two errors are not
// comparable.
//
// That is a preference, not a guarantee, and one gap is known. CommonMark
// measures fence indentation from the enclosing CONTAINER, while this reads it
// from the line start, so a fence inside a list item whose content column is
// four or more reads as unfenced here. An invisible-only line inside that block
// would then be masked, which is the corrupting direction. Container tracking
// is what closes it, and the pinned corpus contains no fenced code blocks at
// all, so nothing is bought by building it yet.

/**
 Fewest marker characters a fence needs.
 */
const MIN_FENCE_LENGTH = 3;

/**
 Most leading spaces a fence line may carry before it stops being a fence.

 Four columns of indent is an indented code block in CommonMark, so a line
 indented that far cannot open or close a fence. A leading TAB is four columns
 on its own, which is why tab-indented lines never read as fences here.
 */
const MAX_FENCE_INDENT = 3;

/**
 Marker run a line carries, when the line is shaped like a fence.

 A `length` of zero is the sentinel for a line that carries no marker at all,
 which {@link NOT_A_FENCE} names. Zero is in-domain rather than a stand-in for
 absence: a fence IS a run of marker characters, and a line that is not one
 carries a run of none. Every real fence runs at least
 {@link MIN_FENCE_LENGTH}, so the two can never be confused.

 @example
 ```ts
 const marker: FenceMarker = { marker: '`', length: 3, info: 'ts', };
 ```
 */
type FenceMarker = {
  /**
   Character the run is built from, either a backtick or a tilde, and empty
   when the run holds nothing.
   */
  readonly marker: string;

  /**
   How many marker characters the run holds, zero when the line carries none.
   */
  readonly length: number;

  /**
   Everything after the run, which is an info string on an opening fence and
   must be empty on a closing one.
   */
  readonly info: string;
};

/**
 Run carried by a line that is not fence-shaped, and the state outside any
 fenced block.
 */
const NOT_A_FENCE: FenceMarker = {
  marker: '',
  length: 0,
  info: '',
};

/**
 Reads a line as a fence marker.

 Written as an index scan rather than a pattern: the rule is a run of one
 character after a bounded indent, and a body line is arbitrary text that must
 not be able to make the scan backtrack.

 @param line - one line without its terminator

 @returns Marker the line carries, {@link NOT_A_FENCE} when it carries none

 @example
 ```ts
 const fence = readFence({ line: '```ts', },);
 ```
 */
function readFence({ line, }: { readonly line: string; },): FenceMarker {
  return (function scan(): FenceMarker {
    /**
     Cursor over the line.
     */
    let index = 0;
    while ((index < line.length) && (line.charAt(index,) === ' '))
      index += 1;

    if (index > MAX_FENCE_INDENT)
      return NOT_A_FENCE;

    /**
     Character the run would be built from.
     */
    const marker = line.charAt(index,);
    if ((marker !== '`') && (marker !== '~'))
      return NOT_A_FENCE;

    /**
     Where the marker run starts, so its length can be measured.
     */
    const runStart = index;
    while ((index < line.length) && (line.charAt(index,) === marker))
      index += 1;

    /**
     How long the run turned out to be.
     */
    const length = index - runStart;
    if (length < MIN_FENCE_LENGTH)
      return NOT_A_FENCE;

    return {
      marker,
      length,
      info: line.slice(index,),
    };
  })();
}

/**
 Whether text is made only of spaces and tabs.

 Written as a scan rather than `trim()`, because ECMAScript trims far more
 than CommonMark counts as blank. U+FEFF, U+00A0, U+2028 and U+2029 all
 vanish under `trim()`, so a line spelled ```` ```<U+FEFF> ```` would read as
 a closing fence where CommonMark calls it code content, and the
 invisible-only line after it would then be exposed to masking. That is the
 same trap `mask-invisible-lines.ts` exists to document, one file over.

 @param text - text after a fence marker run

 @returns Whether CommonMark would accept it after a closing fence

 @example
 ```ts
 const bare = isSpacesAndTabs({ text: '  ', },);
 ```
 */
function isSpacesAndTabs({ text, }: { readonly text: string; },): boolean {
  for (const character of text) {
    if ((character !== ' ') && (character !== '\t'))
      return false;
  }

  return true;
}

/**
 Whether a fence-shaped line can open a block.

 A backtick fence may not carry a backtick in its info string, because that
 ambiguity is how inline code spans are told apart from blocks.

 @param fence - marker the line carries

 @returns Whether it opens a fenced block

 @example
 ```ts
 const opens = canOpen({ fence, },);
 ```
 */
function canOpen({ fence, }: { readonly fence: FenceMarker; },): boolean {
  if (fence.length === 0)
    return false;

  if (fence.marker !== '`')
    return true;

  /**
   Info string this fence carries, which may not hold a backtick.
   */
  const { info, } = fence;

  return !info.includes('`',);
}

/**
 Whether a fence-shaped line closes the block a given opening started.

 Must use the same marker, run at least as long, and carry no info string.

 @param fence - marker the line carries

 @param open - opening currently in force

 @returns Whether it closes that block

 @example
 ```ts
 const closes = canClose({ fence, open, },);
 ```
 */
function canClose(
  {
    fence,
    open,
  }: {
    readonly fence: FenceMarker;
    readonly open: FenceMarker;
  },
): boolean {
  if (fence.length === 0)
    return false;

  if (fence.marker !== open.marker)
    return false;

  if (fence.length < open.length)
    return false;

  /**
   Info string the candidate carries, which a closing fence must not have.
   */
  const { info, } = fence;

  return isSpacesAndTabs({ text: info, },);
}

/**
 Flags every line that belongs to a fenced code block.

 Fence lines themselves count as fenced, so a caller that skips flagged lines
 never rewrites a marker either. An unclosed fence runs to the end of the
 body, exactly as CommonMark reads it.

 @param lines - body split on newlines, terminators removed

 @returns One flag per line, in the same order

 @example
 ```ts
 const flags = fencedLineFlags({ lines: body.split('\n',), },);
 ```
 */
export function fencedLineFlags(
  { lines, }: { readonly lines: readonly string[]; },
): readonly boolean[] {
  return (function scan(): readonly boolean[] {
    /**
     Opening currently in force, {@link NOT_A_FENCE} while outside a fence.
     */
    let open: FenceMarker = NOT_A_FENCE;

    /**
     One flag per line, filled in document order.
     */
    const flags: boolean[] = [];
    for (const line of lines) {
      /**
       Marker this line carries, if it is fence-shaped at all.
       */
      const fence = readFence({ line, },);
      if (open.length === 0) {
        if (canOpen({ fence, },)) {
          open = fence;
          flags.push(true,);
          continue;
        }
        flags.push(false,);
        continue;
      }

      // Inside a fence every line is content, including the line that ends it.
      flags.push(true,);
      if (canClose({
        fence,
        open,
      },))
        open = NOT_A_FENCE;
    }

    return flags;
  })();
}

//endregion Fenced code line flags
