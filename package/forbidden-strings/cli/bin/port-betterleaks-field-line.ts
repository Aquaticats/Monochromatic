/**
 The field-line readers the betterleaks TOML walker uses in place of pattern matching.

 Each reader answers one `key = value` shape the vendored configuration writes, from the single
 line the walker holds at its cursor, and each answers {@link NO_VALUE_AFTER_ASSIGNMENT} for a line
 that does not carry its shape, so the walker falls through to the next reader. Every reader is one
 linear pass over the line: it skips a whitespace run, then a literal, then a character run, with
 no state carried between characters.
 */

//region What a replaced pattern skipped

/**
 Characters the `\s` class matches, exactly the set the replaced patterns skipped: the ASCII
 control spaces, space, no-break space, Ogham space mark, the U+2000 to U+200A typographic spaces,
 line and paragraph separators, narrow no-break space, medium mathematical space, ideographic
 space, and the byte-order mark.
 */
const SPACE_CHARACTERS = [
  '\t\n\v\f\r ',
  '\u00A0\u1680\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009\u200A',
  '\u2028\u2029\u202F\u205F\u3000\uFEFF',
].join('',);

/**
 The same characters as a membership set, so a scan tests one character without a pattern.
 */
const SPACE_CHARACTER_SET: ReadonlySet<string> = new Set(SPACE_CHARACTERS,);

/**
 The separator between a field name and its value on a TOML field line.
 */
const ASSIGNMENT = '=';

/**
 The marker that opens and closes a triple-quoted field value.
 */
const TRIPLE_QUOTE = "'''";

//endregion What a replaced pattern skipped

/**
 The answer for a line that does not carry the field a reader was asked for.

 A field line can legitimately hold an empty value, so the absence of the field is its own value
 rather than an empty one.

 @example
 ```ts
 const body = tripleQuotedBodyAfterAssignment({ line: 'id = "one"', key: 'regex' });
 typeof body === 'symbol'; // true
 ```
 */
export const NO_VALUE_AFTER_ASSIGNMENT: unique symbol = Symbol(
  'the line does not carry that field assignment',
);

/**
 Either the value a reader found, or the marker that says the line carries no such field.
 */
export type FieldValue = string | typeof NO_VALUE_AFTER_ASSIGNMENT;

/**
 Index of the first character at or after one index that `\s` does not match.

 @param text - line being walked

 @param from - index to start at, which may lie past the end of `text`

 @returns the index of the first character the `\s` class rejects, or the text length when every
 character from `from` on is whitespace

 @example
 ```ts
 endOfWhitespace({ text: 'a  = b', from: 1 }); // 3
 ```
 */
function endOfWhitespace({
  text,
  from,
}: {
  readonly text: string;
  readonly from: number;
},): number {
  /**
   Cursor into the line, advanced over each whitespace character.
   */
  let at = from;
  while (
    (at < text.length) && SPACE_CHARACTER_SET.has(text.charAt(at,),)
  ) {
    at += 1;
  }
  return at;
}

/**
 The value part of one `key = value` field line: everything after the field name, the optional
 whitespace, the assignment and the optional whitespace.

 @param line - one line of the configuration, as the walker's cursor holds it

 @param key - field name the line must carry, compared as literal text

 @returns the rest of the line after the assignment, or {@link NO_VALUE_AFTER_ASSIGNMENT} when the
 line does not start with that field name followed by an assignment

 @example
 ```ts
 valueAfterAssignment({ line: 'regex = x', key: 'regex' }); // 'x'
 valueAfterAssignment({ line: 'path = x', key: 'regex' }); // NO_VALUE_AFTER_ASSIGNMENT
 ```
 */
export function valueAfterAssignment({
  line,
  key,
}: {
  readonly line: string;
  readonly key: string;
},): FieldValue {
  if (!line.startsWith(key,)) return NO_VALUE_AFTER_ASSIGNMENT;
  /**
   Index of the character the assignment must sit at, past any whitespace after the field name.
   */
  const atNameEnd = endOfWhitespace({
    text: line,
    from: key.length,
  },);
  if (line.charAt(atNameEnd,) !== ASSIGNMENT) return NO_VALUE_AFTER_ASSIGNMENT;
  return line.slice(endOfWhitespace({
    text: line,
    from: atNameEnd + 1,
  },),);
}

/**
 The body a triple-quoted field line opens, with its `'''` marker removed.

 @param line - one line of the configuration, as the walker's cursor holds it

 @param key - field name the line must carry, compared as literal text

 @returns the text after the opening `'''`, or {@link NO_VALUE_AFTER_ASSIGNMENT} when the line does
 not open a triple-quoted value for that field

 @example
 ```ts
 tripleQuotedBodyAfterAssignment({ line: "regex = '''cat'''", key: 'regex' }); // "cat'''"
 tripleQuotedBodyAfterAssignment({ line: 'regex = cat', key: 'regex' }); // NO_VALUE_AFTER_ASSIGNMENT
 ```
 */
export function tripleQuotedBodyAfterAssignment({
  line,
  key,
}: {
  readonly line: string;
  readonly key: string;
},): FieldValue {
  /**
   The value part of the line, already past the assignment.
   */
  const value = valueAfterAssignment({
    line,
    key,
  },);
  if (((typeof value) === 'symbol') || (!value.startsWith(TRIPLE_QUOTE,))) {
    return NO_VALUE_AFTER_ASSIGNMENT;
  }
  return value.slice(TRIPLE_QUOTE.length,);
}
