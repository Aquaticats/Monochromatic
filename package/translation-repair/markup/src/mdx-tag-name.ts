//region MDX tag name
// WHAT A TAG'S NAME IS, written once from the grammar the corpus is built with
// (`micromark-extension-mdx-jsx` 3.0.2, `dev/lib/factory-tag.js`, the states
// `nameBefore` to `localNameAfter`, and `estree-util-is-identifier-name` 3.0.0
// for the character classes). A name starts with `$`, `_` or a Unicode
// ID_Start character of the Basic Multilingual Plane, Han included, and goes on
// with ID_Continue characters of it, `$`, `_`, `-`, the zero-width non-joiner
// and joiner. It may be followed by `.` and member names, or by `:` and one
// local name, with whitespace stepped over on either side of the separator. A
// name that ends on any other character (a zero-width space, a bidirectional
// control, a soft hyphen, an `@`) fails the page's compile, which is the
// verdict a reader of a tag has to give too.
//
// THE COMPILER READS UTF-16 UNITS, not code points: `micromark` hands the
// grammar one unit at a time, and a unit of a surrogate pair is no identifier
// character. A name holding a letter past U+FFFF (an astral Han ideograph, a
// mathematical letter, an emoji) therefore fails the compile, and so does a
// name starting with one. Measured with `parseMdxBody` over every code point
// of the first two planes and a sample of the next two.
//
// Three readers of a tag kept a name of their own: the lone container tag
// masker took an ASCII letter and then everything up to the first whitespace,
// the inline reader took the name the masker found and looked at one character
// past it, and the attribute restorer (`corpus-run/tag-attributes.ts`) took an
// ASCII letter and then ASCII letters, digits, `-`, `.` and `:`. A lone tag
// whose name held a character the grammar refuses was therefore masked as an
// element of that odd name, where the grammar would have refused the slice,
// and a tag named in any other script than ASCII was never read as a tag at
// all. Two more kept names of their own after that: the nesting count
// (`nesting-inline-count.ts`) opened a tag at `<` and an ASCII letter, and the
// attribute restorer read an attribute's name as ASCII letters, digits, `-`,
// `.` and `:`. Both read through this module now: a tag's name through
// `readTagName`, an attribute's through `readAttributeName`, which takes the
// characters a tag name's parts take, one `:` and its local name, and no `.`.

/* oxlint-disable no-restricted-syntax/no-regex -- the input is one UTF-16 unit, anchored at both ends with one class and no quantifier, so the test is bounded and cannot backtrack; Unicode ID_Start has no string API */
/**
 A unit that can start a tag name, as `estree-util-is-identifier-name`
 3.0.0 reads one for the MDX compiler (`startRe`), which the compiler asks of
 each UTF-16 unit it is handed.
 */
const NAME_START = /^[$_\p{ID_Start}]$/u;

/**
 A unit that can continue a tag name, as `estree-util-is-identifier-name`
 3.0.0 reads one for the MDX compiler with its JSX option (`contReJsx`), asked
 of each UTF-16 unit.
 */
const NAME_CONTINUE = /^[-$_\u{200C}\u{200D}\p{ID_Continue}]$/u;
/* oxlint-enable no-restricted-syntax/no-regex */

/**
 Whitespace the strict grammar steps over inside a tag, between its name and
 an attribute or its closing bracket: `markdownLineEndingOrSpace` or
 `unicodeWhitespace` of `micromark-util-character` in
 `micromark-extension-mdx-jsx` 3.0.2 (`factory-tag.js`), the second being the
 ECMAScript `\s`. Measured against `parseMdxBody` over every code point of the
 Basic Multilingual Plane, and seven beyond it, as the separator of an opener
 and of a closer, the grammar steps over exactly these twenty-five and refuses
 every other, a zero-width space, a next line mark and a Hangul filler among
 them.
 */
const TAG_WHITESPACE: ReadonlySet<string> = new Set([
  '\t',
  '\n',
  '\u{000B}',
  '\u{000C}',
  '\r',
  ' ',
  '\u{00A0}',
  '\u{1680}',
  '\u{2000}',
  '\u{2001}',
  '\u{2002}',
  '\u{2003}',
  '\u{2004}',
  '\u{2005}',
  '\u{2006}',
  '\u{2007}',
  '\u{2008}',
  '\u{2009}',
  '\u{200A}',
  '\u{2028}',
  '\u{2029}',
  '\u{202F}',
  '\u{205F}',
  '\u{3000}',
  '\u{FEFF}',
],);

/**
 Characters right after `<` that leave it text: markdown's space, tab and
 line endings.
 */
const LEAVES_TEXT: ReadonlySet<string> = new Set([
  ' ',
  '\t',
  '\n',
  '\r',
],);

/**
 Whether a character is whitespace to the strict grammar inside a tag: the one
 test every container tag reader shares, so none steps over fewer or more
 than the grammar does.

 THE SAME WHITESPACE ENDS A BARE ADDRESS. The autolink literal's path and
 trail (`micromark-extension-gfm-autolink-literal` 2.1.0) end at the same pair
 of `micromark-util-character` tests, so the destination scan
 (`corpus-run/dropped-destinations.ts`) ends a run where this answers true
 rather than keeping a set of its own.

 @param character - one UTF-16 unit of a tag

 @returns True for whitespace the grammar steps over between a name and the
 rest of the tag

 @example
 ```ts
 isTagWhitespace({ character: '\u{00A0}', },); // true
 ```
 */
export function isTagWhitespace({ character, }: { readonly character: string; },): boolean {
  return TAG_WHITESPACE.has(character,);
}

/**
 Whether a character right after `<` leaves it text: markdown's space, tab or
 line ending, which the compiler does not step over as it does other whitespace.

 @param character - the character right after the bracket, empty at the text's end

 @returns True for markdown's space, tab and line endings

 @example
 ```ts
 leavesBracketAsText({ character: ' ', },); // true
 ```
 */
export function leavesBracketAsText({ character, }: { readonly character: string; },): boolean {
  return LEAVES_TEXT.has(character,);
}

/**
 Whether a character can start a tag name, so one test serves every reader of
 where a tag opens and of what its name is.

 @param character - one UTF-16 unit, empty past a text's edge

 @returns True for `$`, `_` and a Unicode ID_Start character of the first
 plane; false for the empty string and for a letter past it, which the
 compiler reads as two units neither of which is a letter

 @example
 ```ts
 startsTagName({ character: '猫', },); // true
 startsTagName({ character: '3', },); // false
 ```
 */
export function startsTagName({ character, }: { readonly character: string; },): boolean {
  return (character.length === 1) && NAME_START.test(character,);
}

/**
 Offset past the whitespace the grammar steps over.

 @param text - text being read

 @param from - offset to start at

 @returns Offset of the first character that is no tag whitespace, or the
 text's length

 @example
 ```ts
 const at = pastTagWhitespace({ text: 'a  b', from: 1, },); // 3
 ```
 */
function pastTagWhitespace(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): number {
  /**
   Offset moved past each whitespace unit.
   */
  let at = from;
  while (isTagWhitespace({ character: text.charAt(at,), },))
    at += 1;
  return at;
}

/**
 Offset past one name part: a starting unit and every continuing one.

 @param text - text being read

 @param from - offset of the part's first character, which starts a name

 @returns Offset of the first character that does not continue the part

 @example
 ```ts
 const end = pastNamePart({ text: 'Paw-x y', from: 0, },); // 5
 ```
 */
function pastNamePart(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): number {
  /**
   Offset moved past each unit of the part.
   */
  let at = from + 1;
  while (NAME_CONTINUE.test(text.charAt(at,),))
    at += 1;
  return at;
}

/**
 Whether the character after a name part ends it in the grammar: a separator,
 a bracket, a brace, a slash or whitespace. Anything else, the text's end
 included, fails the page's compile.

 @param character - character right after the part, empty at the text's end

 @param separators - separators that may follow this kind of part

 @returns Whether the part may end here

 @example
 ```ts
 endsNamePart({ character: '>', separators: ['.', ':',], },); // true
 ```
 */
function endsNamePart(
  {
    character,
    separators,
  }: {
    readonly character: string;
    readonly separators: readonly string[];
  },
): boolean {
  return separators.includes(character,)
    || (character === '/')
    || (character === '>')
    || (character === '{')
    || isTagWhitespace({ character, },);
}

/**
 Whether what follows a whole name is something the grammar reads there: the
 end of the tag, a self-closing slash, an attribute expression or the first
 character of an attribute name.

 @param text - text being read

 @param at - offset past the name and the whitespace after it

 @returns Whether the tag goes on from a name in a way the grammar accepts

 @example
 ```ts
 const goesOn = namedTagGoesOn({ text: 'Paw open>', at: 4, },); // true
 ```
 */
function namedTagGoesOn(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): boolean {
  /**
   Character where the tag goes on.
   */
  const next = text.charAt(at,);
  return (next === '/')
    || (next === '>')
    || (next === '{')
    || startsTagName({ character: next, },);
}

/**
 One tag read at its opening bracket.

 @example
 ```ts
 const reading: TagNameReading = { closes: false, name: 'details', end: 8, };
 ```
 */
export type TagNameReading = {
  /**
   Whether the tag closes its element.
   */
  readonly closes: boolean;

  /**
   Element name without the whitespace the grammar allows around its
   separators, so two spellings of one name compare equal.
   */
  readonly name: string;

  /**
   Offset just past the name's last character, before any whitespace.
   */
  readonly end: number;
};

/**
 Reads the tag name after the `<` at an offset, as the MDX compiler reads it,
 or reads none where the compiler reads the `<` as text or refuses the page.

 `<` followed by markdown's space, tab or line ending is text; any other
 whitespace is stepped over before an opener's name, and every whitespace
 before a closer's. The name is a primary name, then `.` and member names or
 `:` and one local name, whitespace allowed around a separator; what follows
 it must be whitespace, `/`, `>` or `{`, or an attribute's first character.

 @param text - text being read

 @param at - offset of the `<`

 @returns The tag as a one-element list, empty where no tag with a name opens
 there

 @example
 ```ts
 const [reading,] = readTagName({ text: '<details open>', at: 0, },);
 ```
 */
export function readTagName(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): readonly TagNameReading[] {
  /**
   Whether the tag closes its element.
   */
  const closes = text.charAt(at + 1,) === '/';

  /**
   Offset after the bracket, and the slash of a closer.
   */
  const afterBracket = at + (closes ? 2 : 1);
  if ((!closes) && leavesBracketAsText({ character: text.charAt(afterBracket,), },))
    return [];

  /**
   Offset the name's first character stands at.
   */
  const nameStart = pastTagWhitespace({
    text,
    from: afterBracket,
  },);
  if (!startsTagName({
    character: text.charAt(nameStart,),
  },))
    return [];

  /**
   Offset past the primary name.
   */
  const primaryEnd = pastNamePart({
    text,
    from: nameStart,
  },);
  if (!endsNamePart({
    character: text.charAt(primaryEnd,),
    separators: [
      '.',
      ':',
    ],
  },))
    return [];

  /**
   Name as read so far, and the offset just past it.
   */
  const reading = {
    name: text.slice(
      nameStart,
      primaryEnd,
    ),
    end: primaryEnd,
  };

  /**
   Offset of the first character past the name and the whitespace after it.
   */
  const afterPrimary = pastTagWhitespace({
    text,
    from: primaryEnd,
  },);
  if (text.charAt(afterPrimary,) === ':') {
    /**
     Offset the local name's first character stands at.
     */
    const localStart = pastTagWhitespace({
      text,
      from: afterPrimary + 1,
    },);
    if (!startsTagName({
      character: text.charAt(localStart,),
    },))
      return [];
    reading.end = pastNamePart({
      text,
      from: localStart,
    },);
    if (!endsNamePart({
      character: text.charAt(reading.end,),
      separators: [],
    },))
      return [];
    reading.name = `${reading.name}:${
      text.slice(
        localStart,
        reading.end,
      )
    }`;
    return namedTagGoesOn({
      text,
      at: pastTagWhitespace({
        text,
        from: reading.end,
      },),
    },)
      ? [{
        closes,
        name: reading.name,
        end: reading.end,
      },]
      : [];
  }

  /**
   Offset of the character a member separator or the tag's going on stands at.
   */
  const cursor = { at: afterPrimary, };
  while (text.charAt(cursor.at,) === '.') {
    /**
     Offset the member name's first character stands at.
     */
    const memberStart = pastTagWhitespace({
      text,
      from: cursor.at + 1,
    },);
    if (!startsTagName({
      character: text.charAt(memberStart,),
    },))
      return [];
    reading.end = pastNamePart({
      text,
      from: memberStart,
    },);
    if (!endsNamePart({
      character: text.charAt(reading.end,),
      separators: ['.',],
    },))
      return [];
    reading.name = `${reading.name}.${
      text.slice(
        memberStart,
        reading.end,
      )
    }`;
    cursor.at = pastTagWhitespace({
      text,
      from: reading.end,
    },);
  }
  return namedTagGoesOn({
    text,
    at: cursor.at,
  },)
    ? [{
      closes,
      name: reading.name,
      end: reading.end,
    },]
    : [];
}

/**
 One attribute name read where an attribute starts inside a tag.

 @example
 ```ts
 const reading: AttributeNameReading = { name: 'xml:lang', end: 13, }; // read in '<Cat xml:lang="en"/>' at 5
 ```
 */
export type AttributeNameReading = {
  /**
   Attribute name without the whitespace the grammar allows around its colon,
   so two spellings of one name compare equal.
   */
  readonly name: string;

  /**
   Offset just past the name's last character, before any whitespace.
   */
  readonly end: number;
};

/**
 Reads the attribute name that starts at an offset inside a tag, as the MDX
 compiler reads it (`micromark-extension-mdx-jsx` 3.0.2, `factory-tag.js`, the
 states `attributeBefore` to `attributeLocalNameAfter`), or reads none where
 the compiler refuses the tag there.

 An attribute's name is a primary name, then optionally `:` and one local
 name, whitespace allowed around the colon; its parts take the characters a
 tag name's parts take, and no `.` member follows. What follows the name, past
 whitespace, must be `=`, `/`, `>`, `{` or the first character of the next
 attribute's name.

 @param text - text being read

 @param at - offset of the name's first character

 @returns The name as a one-element list, empty where the grammar reads no
 attribute name starting there

 @example
 ```ts
 const [reading,] = readAttributeName({ text: '<Cat xml : lang="en"/>', at: 5, },); // name 'xml:lang', end 15
 ```
 */
export function readAttributeName(
  {
    text,
    at,
  }: {
    readonly text: string;
    readonly at: number;
  },
): readonly AttributeNameReading[] {
  if (!startsTagName({ character: text.charAt(at,), },))
    return [];

  /**
   Offset past the primary name.
   */
  const primaryEnd = pastNamePart({
    text,
    from: at,
  },);
  if (!endsNamePart({
    character: text.charAt(primaryEnd,),
    separators: [
      ':',
      '=',
    ],
  },))
    return [];

  /**
   Offset of the first character past the primary name and the whitespace
   after it.
   */
  const afterPrimary = pastTagWhitespace({
    text,
    from: primaryEnd,
  },);
  if (text.charAt(afterPrimary,) !== ':') {
    return ((text.charAt(afterPrimary,) === '=') || namedTagGoesOn({
      text,
      at: afterPrimary,
    },))
      ? [{
        name: text.slice(
          at,
          primaryEnd,
        ),
        end: primaryEnd,
      },]
      : [];
  }

  /**
   Offset the local name's first character stands at.
   */
  const localStart = pastTagWhitespace({
    text,
    from: afterPrimary + 1,
  },);
  if (!startsTagName({ character: text.charAt(localStart,), },))
    return [];

  /**
   Offset past the local name.
   */
  const localEnd = pastNamePart({
    text,
    from: localStart,
  },);
  if (!endsNamePart({
    character: text.charAt(localEnd,),
    separators: ['=',],
  },))
    return [];

  /**
   Offset of the first character past the local name and the whitespace after it.
   */
  const afterLocal = pastTagWhitespace({
    text,
    from: localEnd,
  },);
  return ((text.charAt(afterLocal,) === '=') || namedTagGoesOn({
    text,
    at: afterLocal,
  },))
    ? [{
      name: `${
        text.slice(
          at,
          primaryEnd,
        )
      }:${
        text.slice(
          localStart,
          localEnd,
        )
      }`,
      end: localEnd,
    },]
    : [];
}

//endregion MDX tag name
