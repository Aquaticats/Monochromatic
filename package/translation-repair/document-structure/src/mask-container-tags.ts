import {
  inlineContainerTags,
  isTagWhitespace,
  readTagName,
  tagEnd,
} from '@monochromatic-dev/translation-repair-markup/ts';

//region Lone container tag masking
// A container's opening tag is owned by the first block inside it and its
// closing tag by the last (`container-extents.ts`), so a container whose
// blocks fall in different slices puts `<details>` alone at the head of one
// slice and `</details>` alone at the foot of another. The document reader
// sees both and parses; a slice reader handed one half sees an end-tag
// mismatch or an unexpected closing slash and refuses, and both gates that
// consume the slice floor treat a refusal as inadmissible. That stopped the
// Huasheng pass of 2026-09-07 at slices 9 and 12, the class-five pattern for
// containers: 30 of the pinned corpus's pages carry a disclosure element.
//
// Masking a lone tag to same-length whitespace lets the strict grammar read
// the rest of the slice as the document reader did, and reporting the tag lets
// the slice skeleton carry it as an atom, so a candidate that drops the tag
// fails the floor deterministically rather than at the page grammar.
//
// A line is a lone tag only when nothing but the white space Markdown ends a line with (a space, a tab, a carriage return)
// follows the tag's bracket, as the MDX parser reads it. Any other white space after the bracket, a no-break space for
// one, is text: the line is then no tag line, and no mask leaves part of it standing as a character that is not blank to
// CommonMark between two spaces (n156, W5).

/**
 One container tag standing without its partner inside a slice.

 @example
 ```ts
 const tag: LoneContainerTag = { kind: 'close', name: 'details', text: '</details>', startOffset: 12, endOffset: 22, };
 ```
 */
export type LoneContainerTag = {
  /**
   Whether the tag opens or closes its element.
   */
  readonly kind: 'open' | 'close';

  /**
   Element name as written.
   */
  readonly name: string;

  /**
   Tag exactly as the slice writes it.
   */
  readonly text: string;

  /**
   Offset of the tag's first character in the text the mask read.

   CARRIED SO A READER CUTS AT THE TAG ITSELF (ledger B67). The block
   deficit searched the slice for the tag's text instead, and a whole
   element of the same name beside the container moved the cut.
   */
  readonly startOffset: number;

  /**
   Exclusive offset past the tag's last character.
   */
  readonly endOffset: number;
};

/**
 One tag line found while scanning, before pairing: a lone tag once nothing
 partners it.
 */
type TagLine = LoneContainerTag;

/**
 One tag the pairing reads: a whole tag line, carried so it can be masked,
 or an opener or closer inside another line, which only partners one.
 */
type PairingTag = Readonly<{
  /**
   Whether the tag opens or closes its element.
   */
  kind: 'open' | 'close';

  /**
   Element name as written.
   */
  name: string;

  /**
   Offset of the tag's first character.
   */
  startOffset: number;

  /**
   The tag line itself, absent for an inline tag.
   */
  line?: TagLine;
}>;

/**
 Whether a text holds nothing but whitespace the strict grammar steps over.

 @param text - text to read

 @returns True for a text of whitespace alone, the empty text included

 @example
 ```ts
 isWhitespaceOnly({ text: ' \t', },); // true
 ```
 */
function isWhitespaceOnly({ text, }: { readonly text: string; },): boolean {
  return Array.from(text,)
    .every(function isWhitespace(character,): boolean {
      return isTagWhitespace({ character, },);
    },);
}

/**
 The characters Markdown ends a line with, besides its text: a space, a tab and the carriage return of a line break the scan cut
 at its line feed. Any other white space (a no-break space, a wide space, a byte order mark) is text to the MDX parser, so a
 line that ends in one is not a line of one tag.
 */
const LINE_END_SPACE: ReadonlySet<string> = new Set([
  ' ',
  '\t',
  '\r',
],);

/**
 Cuts the white space Markdown ends a line with off the end of a line.

 @param line - line without its line feed

 @returns The line without trailing spaces, tabs and carriage returns; any other white space stays, since it is text

 @example
 ```ts
 trimLineEnd({ line: '<details> \t\r', },); // '<details>'
 ```
 */
function trimLineEnd({ line, }: { readonly line: string; },): string {
  /**
   Where the line ends once the spaces, tabs and carriage returns are cut: the exclusive end of the last other character.
   */
  const end = { at: line.length, };
  while ((end.at > 0) && LINE_END_SPACE.has(line.charAt(end.at - 1,),))
    end.at -= 1;
  return line.slice(
    0,
    end.at,
  );
}

/**
 Whether a text holds an angle bracket, which a tag's own brackets enclose none of.

 @param text - text between a tag's brackets

 @returns True when it holds `<` or `>`

 @example
 ```ts
 holdsBracket({ text: 'details open', },); // false
 ```
 */
function holdsBracket({ text, }: { readonly text: string; },): boolean {
  return text.includes('<',) || text.includes('>',);
}

/**
 Reads one line as a container tag when the line is nothing but one tag.

 A line holding an opening tag with attributes counts; a self-closing tag,
 a comment, a line holding a whole element or any other text does not.

 @param line - line without its newline

 @param lineStart - offset of the line's first character

 @returns Tag on that line as a one-element list, or an empty one

 @example
 ```ts
 tagOnLine({ line: '</details>', lineStart: 120, },);
 ```
 */
function tagOnLine(
  {
    line,
    lineStart,
  }: {
    readonly line: string;
    readonly lineStart: number;
  },
): readonly TagLine[] {
  /**
   Line without trailing white space, which the corpus leaves on lines freely: the white space Markdown ends a line with.
   */
  const trimmed = trimLineEnd({ line, },);

  /**
   Whether the line is bracketed like a tag.
   */
  const bracketed = trimmed.startsWith('<',) && trimmed.endsWith('>',);

  /**
   Whether the tag closes itself, as `<br/>` and a component do.
   */
  const selfClosing = trimmed.endsWith('/>',);
  if (!bracketed)
    return [];
  if (selfClosing)
    return [];

  if (holdsBracket({
    text: trimmed.slice(
      1,
      -1,
    ),
  },))
    return [];

  /**
   The name the strict grammar reads after the opening bracket, empty where it
   reads none (a character it refuses in a name included).
   */
  const [reading,] = readTagName({
    text: trimmed,
    at: 0,
  },);
  if (reading === undefined)
    return [];

  // A closer may carry whitespace before its bracket (`</details >`), which
  // the strict grammar reads as closing the element; anything else past the
  // name names no element.
  if (reading.closes && (!isWhitespaceOnly({
    text: trimmed.slice(
      reading.end,
      -1,
    ),
  },)))
    return [];

  return [{
    kind: reading.closes ? 'close' : 'open',
    name: reading.name,
    text: trimmed,
    startOffset: lineStart,
    endOffset: lineStart + trimmed.length,
  },];
}

/**
 Where a tag that crosses a line break ends, when the line it opens on is the
 head of one: a `<` and a name or a slash and a name, with no bracket
 ending the tag on that line.

 THE STRICT GRAMMAR STEPS OVER A LINE BREAK between a name and an attribute or
 the closing bracket as it does over a space, so `<details` then `  open>` on
 the next line is one tag. The tag must end before any blank line and leave
 nothing but whitespace after its bracket on its last line, or the line is
 prose that opens with a bracket and is read as before.

 @param text - slice to scan

 @param lineStart - offset of the line's first character

 @param lineEnd - exclusive end of that line's content

 @returns Offset just past the tag's bracket, -1 when the line heads no such tag

 @example
 ```ts
 const end = multiLineTagEnd({ text: '<details\n  open>\n', lineStart: 0, lineEnd: 8, },);
 ```
 */
function multiLineTagEnd(
  {
    text,
    lineStart,
    lineEnd,
  }: {
    readonly text: string;
    readonly lineStart: number;
    readonly lineEnd: number;
  },
): number {
  if (text.charAt(lineStart,) !== '<')
    return -1;
  /**
   The name the strict grammar reads after the bracket, empty where it reads none.
   */
  const [reading,] = readTagName({
    text,
    at: lineStart,
  },);
  if (reading === undefined)
    return -1;
  /**
   Offset of the bracket ending the tag, which must stand past this line.
   */
  const close = tagEnd({
    text,
    from: reading.end,
  },);
  if (close < lineEnd)
    return -1;
  /**
   The tag as written, line breaks included.
   */
  const span = text.slice(
    lineStart,
    close + 1,
  );
  if (span.split('\n',)
    .some(function isBlank(line,): boolean {
      return isWhitespaceOnly({ text: line, },);
    },))
    return -1;
  /**
   Offset of the line break ending the tag's last line, the text's end when none.
   */
  const lastLineEnd = text.indexOf(
    '\n',
    close,
  );
  if (trimLineEnd({
    line: text.slice(
      close + 1,
      (lastLineEnd === (-1)) ? text.length : lastLineEnd,
    ),
  },) !== '')
    return -1;
  return close + 1;
}

/**
 Tag lines of a text in document order.

 @param text - slice to scan

 @returns Every line that is exactly one tag

 @example
 ```ts
 const lines = tagLinesOf({ text, },);
 ```
 */
function tagLinesOf({ text, }: { readonly text: string; },): readonly TagLine[] {
  /**
   Tag lines found so far.
   */
  const found: TagLine[] = [];
  for (let lineStart = 0; lineStart <= text.length;) {
    /**
     Offset of the newline ending this line, or the text's end.
     */
    const newlineAt = text.indexOf(
      '\n',
      lineStart,
    );

    /**
     Exclusive end of this line's content.
     */
    const lineEnd = (newlineAt === (-1)) ? text.length : newlineAt;

    /**
     Where a tag opening on this line ends past it, -1 when none does.
     */
    const spanEnd = multiLineTagEnd({
      text,
      lineStart,
      lineEnd,
    },);
    /**
     The tag crossing line breaks, read as a line of its own, empty when the
     lines hold none.
     */
    const crossing = (spanEnd === (-1))
      ? []
      : tagOnLine({
        line: text.slice(
          lineStart,
          spanEnd,
        ),
        lineStart,
      },);
    if (crossing.length > 0) {
      found.push(...crossing,);
      /**
       Offset of the line break ending the tag's last line.
       */
      const lastNewline = text.indexOf(
        '\n',
        spanEnd,
      );
      if (lastNewline === (-1))
        break;
      lineStart = lastNewline + 1;
      continue;
    }

    found.push(...tagOnLine({
      line: text.slice(
        lineStart,
        lineEnd,
      ),
      lineStart,
    },),);
    if (newlineAt === (-1))
      break;
    lineStart = newlineAt + 1;
  }
  return found;
}

/**
 Tag lines left without a partner once openers and closers of one name pair
 up innermost first.

 @param lines - tag lines in document order

 @returns Unpaired tag lines in document order

 @example
 ```ts
 const lone = unpairedOf({ lines, },);
 ```
 */
function unpairedOf<const TagT extends Pick<TagLine, 'kind' | 'name' | 'startOffset'>,>(
  { lines, }: { readonly lines: readonly TagT[]; },
): readonly TagT[] {
  /**
   Openers not yet closed, innermost last.
   */
  const open: TagT[] = [];

  /**
   Closers with no opener before them.
   */
  const strayClosers: TagT[] = [];
  for (const line of lines) {
    if (line.kind === 'open') {
      open.push(line,);
      continue;
    }

    /**
     Index of the innermost opener of the same name, or -1.
     */
    const partner = open.findLastIndex(function sameName(candidate,): boolean {
      return candidate.name === line.name;
    },);
    if (partner === (-1)) {
      strayClosers.push(line,);
      continue;
    }
    open.splice(
      partner,
      1,
    );
  }
  return [
    ...open,
    ...strayClosers,
  ].toSorted(function byOffset(
    left,
    right,
  ): number {
    return left.startOffset - right.startOffset;
  },);
}

/**
 Masks every container tag standing without its partner to same-length
 whitespace and reports each one.

 @param text - slice, comments already masked

 @returns Masked slice and the lone tags in document order

 @example
 ```ts
 const { masked, tags, } = maskLoneContainerTags({ text: slice, },);
 ```
 */
export function maskLoneContainerTags(
  { text, }: { readonly text: string; },
): {
  readonly masked: string;
  readonly tags: readonly LoneContainerTag[];
} {
  /**
   Lines that are one container tag and nothing else, the only ones masked.
   */
  const lines = tagLinesOf({ text, },);
  /**
   Openers and closers of those names inside other lines, which partner a
   tag line as surely as another tag line does (ledger X10).
   */
  const inline = inlineContainerTags({
    text,
    names: new Set(lines.map(function nameOf({ name, },): string {
      return name;
    },),),
    covered: lines.map(function rangeOf({
      startOffset,
      endOffset,
    },): readonly [
      number,
      number,
    ] {
      return [
        startOffset,
        endOffset,
      ] as const;
    },),
  },);
  /**
   Every tag the pairing reads, in document order: the tag lines, each
   carrying itself so an unpaired one can be masked, and the inline tags,
   which only partner one.
   */
  const pairing: readonly PairingTag[] = [
    ...lines.map(function asPairing(line,): PairingTag {
      return {
        kind: line.kind,
        name: line.name,
        startOffset: line.startOffset,
        line,
      };
    },),
    ...inline,
  ].toSorted(function byOffset(
    left: PairingTag,
    right: PairingTag,
  ): number {
    return left.startOffset - right.startOffset;
  },);
  /**
   Tag lines with no partner anywhere in this slice.
   */
  const lone = unpairedOf({ lines: pairing, },)
    .flatMap(function tagLineOf({ line, },): readonly TagLine[] {
      return (line === undefined) ? [] : [line,];
    },);
  if (lone.length === 0)
    return {
      masked: text,
      tags: [],
    };

  /**
   Slice with each lone tag blanked, built from the pieces between them.
   */
  const masked = (function blank(): string {
    /**
     Kept and blanked pieces in source order.
     */
    const parts: string[] = [];

    /**
     Scan position; everything before it is already in parts.
     */
    let cursor = 0;
    for (const tag of lone) {
      parts.push(
        text.slice(
          cursor,
          tag.startOffset,
        ),
        // The line breaks inside a tag stay, so every line is where it stood.
        tag.text
          .split('\n',)
          .map(function blanked(line,): string {
            return ' '.repeat(line.length,);
          },)
          .join('\n',),
      );
      cursor = tag.endOffset;
    }
    parts.push(text.slice(cursor,),);
    return parts.join('',);
  })();

  return {
    masked,
    tags: lone.map(function toTag(tag,): LoneContainerTag {
      return {
        kind: tag.kind,
        name: tag.name,
        text: tag.text,
        startOffset: tag.startOffset,
        endOffset: tag.endOffset,
      };
    },),
  };
}

//endregion Lone container tag masking
