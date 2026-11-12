/**
 Stage one's line readers: what one line of the vendored betterleaks configuration carries, read by walking its
 characters.

 {@link parseRules} in `port-betterleaks-parse.ts` steps through the configuration one line at a time, and these
 readers answer what a single line is: the `[[rules]]` table header, a `[[rules.<name>]]` sub-table header, an
 `id` field, a `description` field. Each walk names the literal characters the line must hold, so what a reader
 accepts is visible where it is written.

 Lines arrive from `toml.split('\n')`, so no line carries a `\n`; a `\r` from a CRLF file ends one, which is why
 the walks count `\r` as the whitespace it is.
 */

//region Constants

/**
 The characters ECMAScript's `\s` names: the ASCII whitespace characters, the Unicode space separators and the
 byte order mark. These walks accept exactly these where a field line may carry spaces.
 */
const SPACE_CHARACTERS: string = ' \t\n\v\f\r\u00a0\u1680\u2000\u2001\u2002\u2003\u2004\u2005\u2006\u2007\u2008\u2009'
  + '\u200a\u2028\u2029\u202f\u205f\u3000\ufeff';

/**
 The line terminators `.` matches none of, which is what bounds one escaped character.
 */
const LINE_TERMINATORS: string = '\n\r\u2028\u2029';

/**
 Returned when the line carries no such quoted value or no such position.
 */
const NO_QUOTED_VALUE: number = -1;

/**
 The top-level table header every rule entry opens with.
 */
const RULES_TABLE_HEADER: string = '[[rules]]';

/**
 The prefix of a sub-table header inside a rule entry.
 */
const RULES_SUB_TABLE_PREFIX: string = '[[rules.';

/**
 The characters that close a sub-table header.
 */
const SUB_TABLE_CLOSE: string = ']]';

/**
 The quote that opens and closes a field value.
 */
const QUOTE: string = '"';

//endregion Constants

//region Character predicates

/**
 Whether the character is one ECMAScript's `\s` names.
 
 @param character - one character of the line, or the empty string past its end
 
 @returns whether the character is whitespace
 */
function isSpaceCharacter({ character, }: { readonly character: string; },): boolean {
  return (character !== '') && SPACE_CHARACTERS.includes(character,);
}

/**
 Whether the character is one `\w` names: an ASCII letter, a digit or an underscore.
 
 @param character - one character of the line, or the empty string past its end
 
 @returns whether the character is a word character
 */
function isWordCharacter({ character, }: { readonly character: string; },): boolean {
  return ((character >= 'a') && (character <= 'z'))
    || ((character >= 'A') && (character <= 'Z'))
    || ((character >= '0') && (character <= '9'))
    || (character === '_');
}

/**
 Whether the character ends a line.
 
 @param character - one character of the line, or the empty string past its end
 
 @returns whether the character is a line terminator
 */
function isLineTerminator({ character, }: { readonly character: string; },): boolean {
  return (character !== '') && LINE_TERMINATORS.includes(character,);
}

//endregion Character predicates

//region Line walks

/**
 Position of the first character from `from` that is not whitespace, or the line's length when the rest is all
 whitespace.
 
 @param line - one line of the configuration
 
 @param from - position the walk starts at
 
 @returns position of the first non-space character, or the line's length
 */
function skipSpaces({ line, from, }: { readonly line: string; readonly from: number; },): number {
  for (let at = from; at < line
    .length; at += 1) {
    if (!isSpaceCharacter({ character: line.charAt(at,), },))
      return at;
  }
  return line.length;
}

/**
 Position of the first character from `from` that is not a word character, or the line's length when the rest is
 all word characters.
 
 @param line - one line of the configuration
 
 @param from - position the walk starts at
 
 @returns position of the first character no name may hold, or the line's length
 */
function wordRunEnd({ line, from, }: { readonly line: string; readonly from: number; },): number {
  for (let at = from; at < line
    .length; at += 1) {
    if (!isWordCharacter({ character: line.charAt(at,), },))
      return at;
  }
  return line.length;
}

/**
 Position of the opening quote of the `key = "` field the line opens with, after any spaces around the `=`.
 
 @param line - one line of the configuration
 
 @param key - the field name the line must open with
 
 @returns position of the opening quote, or {@link NO_QUOTED_VALUE} when the line opens with no such field
 */
function quotedValueStart({ line, key, }: { readonly line: string; readonly key: string; },): number {
  if (!line.startsWith(key,))
    return NO_QUOTED_VALUE;
  /**
   Position of the `=` the key must reach, once the spaces after the key are behind it.
   */
  const equalsAt = skipSpaces({
    line,
    from: key.length,
  },);
  if (line.charAt(equalsAt,) !== '=')
    return NO_QUOTED_VALUE;
  /**
   Position of the quote the value must open with, once the spaces after the `=` are behind it.
   */
  const quoteAt = skipSpaces({
    line,
    from: equalsAt + 1,
  },);
  return (line.charAt(quoteAt,) === QUOTE) ? quoteAt : NO_QUOTED_VALUE;
}

/**
 Position of the quote that closes the value opening at `from`, honouring backslash escapes: either one character
 or a `\` and the character it escapes, so an escaped quote opens no new value.
 
 @param line - one line of the configuration
 
 @param from - position just past the opening quote
 
 @returns position of the closing quote, or {@link NO_QUOTED_VALUE} when the value never closes
 */
function quotedValueEnd({ line, from, }: { readonly line: string; readonly from: number; },): number {
  for (let at = from; at < line
    .length;) {
    /**
     The character the walk is on.
     */
    const character = line.charAt(at,);
    if (character === '\\') {
      // `.` in the pattern this walk replaces matched no line terminator, so an escape onto one closes nothing.
      if (isLineTerminator({ character: line.charAt(at + 1,), },))
        return NO_QUOTED_VALUE;
      at += 2;
      continue;
    }
    if (character === QUOTE)
      return at;
    at += 1;
  }
  return NO_QUOTED_VALUE;
}

//endregion Line walks

//region One line of a rule body

/**
 What one line of a rule body carried.
 
 A line carries one field at most, so a member names the field that arrived. `no-field` is every line the rule
 body walk reads and drops, and every line inside a sub-table, which the walk skips unread.
 */
export type RuleBodyLine =
  | { readonly kind: 'no-field'; }
  | { readonly kind: 'sub-table'; readonly name: string; }
  | { readonly kind: 'id'; readonly value: string; }
  | { readonly kind: 'description'; readonly value: string; };

/**
 Whether the line is the top-level `[[rules]]` table header, with nothing after it but whitespace.
 
 @param line - one line of the configuration
 
 @returns whether the line opens one rule entry
 
 @example
 ```ts
 rulesTableHeaderPresent({ line: '[[rules]]' }); // true
 ```
 */
export function rulesTableHeaderPresent({ line, }: { readonly line: string; },): boolean {
  return line.startsWith(RULES_TABLE_HEADER,)
    && (skipSpaces({
      line,
      from: RULES_TABLE_HEADER.length,
    },) === line.length);
}

/**
 What a line that opens with the sub-table prefix carries: the header, or no field when the rest of the line is
 not one.
 
 @param line - one line of the configuration
 
 @returns the sub-table the line opens, or `no-field`
 */
function subTableLine({ line, }: { readonly line: string; },): RuleBodyLine {
  /**
   Position just past the sub-table name, walked over the name's word characters.
   */
  const nameEnd = wordRunEnd({
    line,
    from: RULES_SUB_TABLE_PREFIX.length,
  },);
  /**
   Position just past the `]]` the header closes with.
   */
  const headerEnd = nameEnd + SUB_TABLE_CLOSE.length;
  if ((nameEnd > RULES_SUB_TABLE_PREFIX.length)
    && line.startsWith(SUB_TABLE_CLOSE, nameEnd,)
    && (skipSpaces({
      line,
      from: headerEnd,
    },) === line.length)) {
    return {
      kind: 'sub-table',
      name: line.slice(
        RULES_SUB_TABLE_PREFIX.length,
        nameEnd,
      ),
    };
  }
  return { kind: 'no-field', };
}

/**
 What one line of a rule body carries, walked from its first character.
 
 A sub-table header is `[[rules.` then one or more word characters then `]]` then whitespace only. The `id` and
 `description` fields are the key, spaces, `=`, spaces and a quoted value; the `description` value may hold escaped
 characters, which the caller decodes. The remaining field lines of a rule body (`regex`, `path`, `secretGroup`,
 `skipReport`) keep their own readers.
 
 @param line - one line of the configuration
 
 @returns the field the line carried, in the order the rule body walk reads them
 
 @example
 ```ts
 readRuleBodyLine({ line: 'description = "a key"' }); // { kind: 'description', value: 'a key' }
 ```
 */
export function readRuleBodyLine({ line, }: { readonly line: string; },): RuleBodyLine {
  if (line.startsWith(RULES_SUB_TABLE_PREFIX,))
    return subTableLine({ line, },);
  /**
   Position of the quote that opens the `id` value.
   */
  const idValueStart = quotedValueStart({
    line,
    key: 'id',
  },);
  if (idValueStart !== NO_QUOTED_VALUE) {
    /**
     Position of the quote that closes the `id` value, which holds no escaped character.
     */
    const idValueEnd = line.indexOf(
      QUOTE,
      idValueStart + 1,
    );
    if (idValueEnd !== NO_QUOTED_VALUE) {
      return {
        kind: 'id',
        value: line.slice(
          idValueStart + 1,
          idValueEnd,
        ),
      };
    }
  }
  /**
   Position of the quote that opens the `description` value.
   */
  const descriptionValueStart = quotedValueStart({
    line,
    key: 'description',
  },);
  if (descriptionValueStart !== NO_QUOTED_VALUE) {
    /**
     Position of the quote that closes the `description` value, read past its escapes.
     */
    const descriptionValueEnd = quotedValueEnd({
      line,
      from: descriptionValueStart + 1,
    },);
    if (descriptionValueEnd !== NO_QUOTED_VALUE) {
      return {
        kind: 'description',
        value: line.slice(
          descriptionValueStart + 1,
          descriptionValueEnd,
        ),
      };
    }
  }
  return { kind: 'no-field', };
}

//endregion One line of a rule body
