import { readTagName, } from './mdx-tag-name.ts';

//region Inline container tags
// LEDGER X10 (NIGHT81473140 slice 22). The lone-tag masker
// (`mask-container-tags.ts`) paired only lines that are one tag and nothing
// else, so an element whose opener shares its line with content
// (`<blockquote><span>...</span>` then `</blockquote>` on the next line) had
// its closer blanked as unpartnered, and the slice read as unparseable while
// the document read the element whole. This finds the openers and closers of
// the tag-line names that stand inside other lines, so the pairing sees the
// whole slice; only a whole tag line is ever masked.

/**
 One opener or closer of a container name found inside a line.
 */
export type InlineContainerTag = {
  /**
   Whether the tag opens or closes its element.
   */
  readonly kind: 'open' | 'close';

  /**
   Element name as written.
   */
  readonly name: string;

  /**
   Offset of the tag's first character.
   */
  readonly startOffset: number;
};

/**
 Offset of the `>` ending a tag, stepping over quoted attribute values and
 braced expressions, or -1 when the tag never ends.

 @param text - slice to read

 @param from - offset just past the tag's name

 @returns Offset of the closing `>`, or -1

 @example
 ```ts
 tagEnd({ text: '<a title="x>y">', from: 2, },); // 14
 ```
 */
export function tagEnd(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): number {
  return (function scan(): number {
    /**
     Quote mark the scan is inside, empty for none.
     */
    let quote = '';
    /**
     Depth of braced expressions the scan is inside.
     */
    let braces = 0;
    for (let at = from; at < text.length; at += 1) {
      /**
       Character at the scan position.
       */
      const character = text.charAt(at,);
      if (quote !== '') {
        if (character === quote)
          quote = '';
        continue;
      }
      if ((character === '"') || (character === '\'')) {
        quote = character;
        continue;
      }
      if (character === '{') {
        braces += 1;
        continue;
      }
      if ((character === '}') && (braces > 0)) {
        braces -= 1;
        continue;
      }
      if ((character === '>') && (braces === 0))
        return at;
    }
    return -1;
  })();
}

/**
 The inline opener or closer of one name at one `<`, or none.

 @param text - slice to read

 @param at - offset of a `<`

 @param name - element name to read

 @returns The tag as a one-element list, empty when there is none there or it closes itself

 @example
 ```ts
 tagAt({ text: 'x<blockquote>', at: 1, name: 'blockquote', },);
 ```
 */
function tagAt(
  {
    text,
    at,
    name,
  }: {
    readonly text: string;
    readonly at: number;
    readonly name: string;
  },
): readonly InlineContainerTag[] {
  /**
   The name the grammar reads after this `<`, empty where it reads none.
   */
  const [reading,] = readTagName({
    text,
    at,
  },);
  if ((reading === undefined) || (reading.name !== name))
    return [];
  /**
   Offset of the `>` ending the tag.
   */
  const end = tagEnd({
    text,
    from: reading.end,
  },);
  if (end === (-1))
    return [];
  if ((!reading.closes) && (text.charAt(end - 1,) === '/'))
    return [];
  return [{
    kind: reading.closes ? 'close' : 'open',
    name,
    startOffset: at,
  },];
}

/**
 Openers and closers of the given names that stand outside the given ranges,
 in document order.

 @param text - slice to read, comments already masked

 @param names - element names the slice's tag lines carry

 @param covered - ranges already read as whole tag lines, start inclusive and end exclusive

 @returns Inline tags of those names, self-closing ones left out

 @example
 ```ts
 const inline = inlineContainerTags({ text, names: new Set(['blockquote',],), covered: [], },);
 ```
 */
export function inlineContainerTags(
  {
    text,
    names,
    covered,
  }: {
    readonly text: string;
    readonly names: ReadonlySet<string>;
    readonly covered: readonly (readonly [
      number,
      number,
    ])[];
  },
): readonly InlineContainerTag[] {
  /**
   Tags found so far, in document order.
   */
  const found: InlineContainerTag[] = [];
  /**
   UTF-16 offset of every `<`, which is what the tag lines' ranges count in.
   */
  const openings = (function offsetsOfOpening(): readonly number[] {
    /**
     Offsets found so far.
     */
    const offsets: number[] = [];
    /**
     Offset of the next `<`, -1 past the last.
     */
    let at = text.indexOf('<',);
    while (at !== (-1)) {
      offsets.push(at,);
      at = text.indexOf(
        '<',
        at + 1,
      );
    }
    return offsets;
  })();
  for (const position of openings) {
    if (covered.some(function inside([start, end,],): boolean {
      return (position >= start) && (position < end);
    },))
      continue;
    for (const name of names)
      found.push(...tagAt({
        text,
        at: position,
        name,
      },),);
  }
  return found;
}

//endregion Inline container tags
