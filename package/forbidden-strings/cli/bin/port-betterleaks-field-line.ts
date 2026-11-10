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
 Whether one character is an ASCII digit `0` to `9`, the only character `\d` matched.

 @param character - one character, or the empty string where an index falls outside the line

 @returns whether character sits in `0` through `9`

 @example
 ```ts
 isAsciiDigit('4'); // true
 isAsciiDigit('x'); // false
 ```
 */
function isAsciiDigit(character: string,): boolean {
  return (character >= '0') && (character <= '9');
}

/**
 Length of the digit run that starts one text.

 @param text - value part of a field line

 @returns how many ASCII digits the text starts with, which is zero when it starts with anything
 else

 @example
 ```ts
 endOfDigitRun('12abc'); // 2
 endOfDigitRun('abc'); // 0
 ```
 */
function endOfDigitRun(text: string,): number {
  /**
   Cursor into the text, advanced over each digit.
   */
  let at = 0;
  while ((at < text.length) && isAsciiDigit(text.charAt(at,),)) at += 1;
  return at;
}

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

/**
 The number a `field = digits` line carries.

 @param line - one line of the configuration, as the walker's cursor holds it

 @param key - field name the line must carry, compared as literal text

 @returns the digit run after the assignment as a number, or {@link NO_VALUE_AFTER_ASSIGNMENT} when
 the line does not start with that field name followed by an assignment and at least one digit

 @example
 ```ts
 numberAfterAssignment({ line: 'secretGroup = 3', key: 'secretGroup' }); // 3
 numberAfterAssignment({ line: 'secretGroup = x', key: 'secretGroup' }); // NO_VALUE_AFTER_ASSIGNMENT
 ```
 */
export function numberAfterAssignment({
  line,
  key,
}: {
  readonly line: string;
  readonly key: string;
},): number | typeof NO_VALUE_AFTER_ASSIGNMENT {
  /**
   The value part of the line, already past the assignment.
   */
  const value = valueAfterAssignment({
    line,
    key,
  },);
  if ((typeof value) === 'symbol') return NO_VALUE_AFTER_ASSIGNMENT;
  /**
   How many digits the value starts with, which is zero when no digit opens it.
   */
  const digitCount = endOfDigitRun(value,);
  if (digitCount === 0) return NO_VALUE_AFTER_ASSIGNMENT;
  /**
   The digit run itself, as the text the number is read from.
   */
  const digits = value.slice(
    0,
    digitCount,
  );
  return Number(digits,);
}

/**
 Whether a `field = true` line carries that marker.

 The marker is a prefix of the value, not the whole value, because the replaced pattern tested the
 literal `true` without anchoring the end of the line.

 @param line - one line of the configuration, as the walker's cursor holds it

 @param key - field name the line must carry, compared as literal text

 @returns whether the value part of the line starts with `true`

 @example
 ```ts
 assignmentHoldsTrue({ line: 'skipReport = true', key: 'skipReport' }); // true
 assignmentHoldsTrue({ line: 'skipReport = false', key: 'skipReport' }); // false
 ```
 */
export function assignmentHoldsTrue({
  line,
  key,
}: {
  readonly line: string;
  readonly key: string;
},): boolean {
  /**
   The value part of the line, already past the assignment.
   */
  const value = valueAfterAssignment({
    line,
    key,
  },);
  return ((typeof value) === 'string') && value.startsWith('true',);
}
