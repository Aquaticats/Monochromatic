import {
  isTagWhitespace,
  readAttributeName,
  readTagName,
} from '@monochromatic-dev/translation-repair-markup/ts';

//region Tag attributes
// A JSX OR HTML TAG READ FOR ITS QUOTED ATTRIBUTES, by index scan: the name
// after `<` as the grammar reads one, then attribute names as the grammar reads
// them (both through `mdx-tag-name.ts`) with their quoted values up to `>` or
// `/>`. A tag whose attribute is not a quoted string (an expression in braces,
// a bare word) is not read at all, so nothing is restored inside it, and
// neither is a tag whose attribute name the grammar refuses, which fails the
// page's compile.

/**
 Opening of a tag.
 */
const TAG_OPEN = '<';

/**
 Closing of a tag.
 */
const TAG_CLOSE = '>';

/**
 Self-closing mark before the closing.
 */
const SELF_CLOSE = '/';

/**
 Separator between an attribute's name and its value.
 */
const EQUALS = '=';

/**
 Quotes a value may stand in.
 */
const QUOTES: ReadonlySet<string> = new Set([
  '"',
  '\'',
],);

/**
 Offset standing for no reading.
 */
export const NO_READING = -1;

/**
 One quoted value as a tag carries it.
 */
type QuotedValue = {
  /**
   Value between the quotes.
   */
  readonly value: string;

  /**
   Offset of the value's first character in the text.
   */
  readonly valueStart: number;

  /**
   Offset just past the value in the text, `NO_READING` where no quote
   opens or closes.
   */
  readonly valueEnd: number;
};

/**
 One quoted attribute as a tag carries it.
 */
export type AttributeReading = QuotedValue & {
  /**
   Attribute name as the grammar reads it, a prefixed one without the
   whitespace the grammar allows around its colon.
   */
  readonly name: string;
};

/**
 One tag as a text carries it.
 */
export type TagReading = {
  /**
   Tag name after `<`.
   */
  readonly name: string;

  /**
   Tag source from `<` to `>`.
   */
  readonly text: string;

  /**
   Offset of `<` in the text.
   */
  readonly start: number;

  /**
   Offset just past `>` in the text, `NO_READING` where no readable tag
   opens.
   */
  readonly end: number;

  /**
   Quoted attributes in order.
   */
  readonly attributes: readonly AttributeReading[];
};

/**
 Offset of the first character past the whitespace from an offset on.

 @param text - text under the scan

 @param from - offset the whitespace may start at

 @returns Offset of the first non-whitespace character, or the text's end

 @example
 ```ts
 pastWhitespace({ text: 'a  b', from: 1, },); // 3
 ```
 */
function pastWhitespace(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): number {
  for (let at = from; at < text.length; at += 1) {
    if (!isTagWhitespace({ character: text.charAt(at,), },))
      return at;
  }
  return text.length;
}

/**
 Reading of no quoted value.
 */
const NO_QUOTED: QuotedValue = {
  value: '',
  valueStart: NO_READING,
  valueEnd: NO_READING,
};

/**
 Reading of no tag.
 */
const NO_TAG: TagReading = {
  name: '',
  text: '',
  start: NO_READING,
  end: NO_READING,
  attributes: [],
};

/**
 Reads one quoted value opening at an offset.

 @param text - text under the scan

 @param at - offset of the opening quote

 @returns Value with its bounds, `NO_READING` as its end where no quote
 opens or closes

 @example
 ```ts
 readQuoted({ text: 'n="5"', at: 2, },); // { value: '5', valueStart: 3, valueEnd: 4, }
 ```
 */
function readQuoted(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): QuotedValue {
  /**
   Quote the value opens with.
   */
  const quote = text.charAt(at,);
  if (!QUOTES.has(quote,))
    return NO_QUOTED;
  /**
   Offset of the closing quote, -1 for none.
   */
  const close = text.indexOf(
    quote,
    at + 1,
  );
  if (close === (-1))
    return NO_QUOTED;
  return {
    value: text.slice(
      at + 1,
      close,
    ),
    valueStart: at + 1,
    valueEnd: close,
  };
}

/**
 Reads the tag opening at an offset.

 @param text - text under the scan

 @param at - offset of `<`

 @returns Tag reading, `NO_TAG` where no readable tag opens there

 @example
 ```ts
 readTagAt({ text: '<Paw n="V"/>', at: 0, },);
 ```
 */
function readTagAt(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): TagReading {
  /**
   The name the strict grammar reads after the bracket, empty where it reads
   none, where the tag closes an element, or where a character it refuses
   stands in the name.
   */
  const [reading,] = readTagName({
    text,
    at,
  },);
  if ((reading === undefined) || reading.closes)
    return NO_TAG;
  /**
   Offset just past the tag name.
   */
  const nameEnd = reading.end;
  /**
   Attributes read so far.
   */
  const attributes: AttributeReading[] = [];
  for (let cursor = pastWhitespace({
    text,
    from: nameEnd,
  },); cursor < text.length;) {
    /**
     Character under the cursor.
     */
    const character = text.charAt(cursor,);
    if (character === TAG_CLOSE)
      return {
        name: reading.name,
        text: text.slice(
          at,
          cursor + 1,
        ),
        start: at,
        end: cursor + 1,
        attributes,
      };
    if (character === SELF_CLOSE) {
      cursor += 1;
      continue;
    }
    /**
     The attribute name the strict grammar reads at the cursor, none where it
     refuses one there.
     */
    const [attributeName,] = readAttributeName({
      text,
      at: cursor,
    },);
    if (attributeName === undefined)
      return NO_TAG;
    /**
     Offset of the character after the name and any whitespace.
     */
    const afterName = pastWhitespace({
      text,
      from: attributeName.end,
    },);
    if (text.charAt(afterName,) !== EQUALS) {
      cursor = afterName;
      continue;
    }
    /**
     Quoted value after the equals sign.
     */
    const quoted = readQuoted({
      text,
      at: pastWhitespace({
        text,
        from: afterName + 1,
      },),
    },);
    if (quoted.valueEnd === NO_READING)
      return NO_TAG;
    attributes.push({
      name: attributeName.name,
      ...quoted,
    },);
    cursor = pastWhitespace({
      text,
      from: quoted.valueEnd + 1,
    },);
  }
  return NO_TAG;
}

/**
 Every readable tag in a text, in order.

 @param text - text that may carry tags

 @returns Tag readings with their quoted attributes

 @example
 ```ts
 readTags({ text: '<Paw n="V"/>\n\nCat.', },); // one reading, attribute n
 ```
 */
export function readTags({ text, }: { readonly text: string; },): readonly TagReading[] {
  /**
   Tags read so far.
   */
  const tags: TagReading[] = [];
  for (
    let at = text.indexOf(TAG_OPEN,);
    at !== (-1);
    at = text.indexOf(
      TAG_OPEN,
      at + 1,
    )
  ) {
    /**
     Tag opening here, if one does.
     */
    const tag = readTagAt({
      text,
      at,
    },);
    if (tag.end !== NO_READING)
      tags.push(tag,);
  }
  return tags;
}

//endregion Tag attributes
