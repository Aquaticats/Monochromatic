import { isAsciiAlphanumeric, } from '@monochromatic-dev/translation-repair-markup/ts';

//region Coverage stretch atoms
// Ledger T8: the tokens the coverage census reads one cold stretch's bundle
// text by, to say whether the stretch is nothing but invariant throws
// (`coverage-invariant-throw.ts` holds that reading and why it reads the
// bundle rather than the source lines).
//
// The reading is one linear pass, no parser and no pattern. A token is a
// word, a string or template literal read whole, a bracketed group read whole
// through everything it nests, or any other single character. Strings,
// templates with their nested substitutions, comments and brackets are read
// exactly.
//
// What one pass cannot read for certain is refused, never guessed: a slash
// that opens no comment is a division or a pattern literal, which cannot be
// told apart without parsing, and a pattern's quotes and brackets would be
// misread as code; so is a string, template, comment or bracket that never
// closes. The reader of the tokens keeps such a stretch cold.

/**
 Characters that are blank space between tokens. Rarer blank characters are
 left unread, which keeps a stretch holding one cold.
 */
const BLANKS: ReadonlySet<string> = new Set([
  ' ',
  '\t',
  '\n',
  '\r',
],);

/**
 Characters that end a line, and with it a line comment or a string that
 never closed.
 */
const LINE_ENDS: ReadonlySet<string> = new Set([
  '\n',
  '\r',
  '\u2028',
  '\u2029',
],);

/**
 Each opening bracket with the character that closes it.
 */
const CLOSER_OF: ReadonlyMap<string, string> = new Map([
  [
    '(',
    ')',
  ],
  [
    '[',
    ']',
  ],
  [
    '{',
    '}',
  ],
],);

/**
 Characters that close a bracket.
 */
const CLOSERS: ReadonlySet<string> = new Set(CLOSER_OF.values(),);

/**
 Characters that open a string literal.
 */
const QUOTES: ReadonlySet<string> = new Set([
  '"',
  '\'',
],);

/**
 Character that opens and closes a template literal.
 */
const TEMPLATE_MARK = '`';

/**
 Characters a name may hold beside ASCII letters and digits.
 */
const NAME_MARKS: ReadonlySet<string> = new Set([
  '_',
  '$',
],);

/**
 Where a read from one offset got to, or that the text cannot be read for
 certain from there.
 */
type Reach = {
  readonly kind: 'reached';

  /**
   Offset one past what was read.
   */
  readonly at: number;
} | { readonly kind: 'unreadable'; };

/**
 One token of a stretch: a word, a string or template literal, a whole
 parenthesised group, any other single character, the end of the text, or
 that the text cannot be read for certain from here.

 @example
 ```ts
 const atom: Atom = { kind: 'word', start: 6, end: 9, };
 ```
 */
export type Atom = {
  readonly kind: 'group' | 'mark' | 'text' | 'word';

  /**
   Offset of its first character.
   */
  readonly start: number;

  /**
   One past its last.
   */
  readonly end: number;
} | { readonly kind: 'end'; } | { readonly kind: 'unreadable'; };

/**
 Skips blank space and comments, which hold no statement.

 @param text - stretch text

 @param from - offset to read from

 @returns Offset of the first character that is neither, the text's length
 when none is left, or that a block comment never closes

 @example
 ```ts
 afterBlank({ text: ' /* nap *\/ throw', from: 0, },); // { kind: 'reached', at: 11, }
 ```
 */
function afterBlank(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): Reach {
  /**
   Offset read so far.
   */
  const cursor = { at: from, };
  while (cursor.at < text.length) {
    if (BLANKS.has(text.charAt(cursor.at,),)) {
      cursor.at += 1;
    }
    else if (text.startsWith(
      '//',
      cursor.at,
    )) {
      while ((cursor.at < text.length) && (!LINE_ENDS.has(text.charAt(cursor.at,),)))
        cursor.at += 1;
    }
    else if (text.startsWith(
      '/*',
      cursor.at,
    )) {
      /**
       Where the comment closes, -1 when it never does.
       */
      const close = text.indexOf(
        '*/',
        cursor.at + 2,
      );
      if (close === (-1))
        return { kind: 'unreadable', };
      cursor.at = close + 2;
    }
    else {
      return {
        kind: 'reached',
        at: cursor.at,
      };
    }
  }
  return {
    kind: 'reached',
    at: text.length,
  };
}

/**
 Reads a string literal to its closing quote.

 @param text - stretch text

 @param from - offset of its opening quote

 @returns Offset one past its closing quote, or that the string never closes
 on its line

 @example
 ```ts
 afterString({ text: '"a \\" cat" + 1', from: 0, },); // { kind: 'reached', at: 10, }
 ```
 */
function afterString(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): Reach {
  /**
   The quote that opened it, which closes it.
   */
  const quote = text.charAt(from,);
  /**
   Offset read so far.
   */
  const cursor = { at: from + 1, };
  while (cursor.at < text.length) {
    /**
     Character there.
     */
    const character = text.charAt(cursor.at,);
    if (character === quote)
      return {
        kind: 'reached',
        at: cursor.at + 1,
      };
    if (LINE_ENDS.has(character,))
      return { kind: 'unreadable', };
    // A backslash takes the character after it with it.
    cursor.at += (character === '\\') ? 2 : 1;
  }
  return { kind: 'unreadable', };
}

/**
 Reads to the character closing what was opened before `from`, a bracket or
 a template, through everything nested inside: brackets, strings, templates
 with their substitutions, and comments. One loop over a stack of the
 closers still owed, so nesting costs no call depth.

 @param text - stretch text

 @param from - offset one past the opening character

 @param closer - character that closes it: a closing bracket, or the
 template mark for a template's text

 @returns Offset one past that closer, or that the text cannot be read for
 certain: a slash that opens no comment, a closer that closes nothing open,
 or something that never closes

 @example
 ```ts
 afterNesting({ text: '(a[0], `b ${c})`) + 1', from: 1, closer: ')', },); // { kind: 'reached', at: 17, }
 ```
 */
function afterNesting(
  {
    text,
    from,
    closer,
  }: {
    readonly text: string;
    readonly from: number;
    readonly closer: string;
  },
): Reach {
  /**
   Closers still owed, the innermost last. A template mark on top means the
   cursor stands in a template's text.
   */
  const owed: string[] = [closer,];
  /**
   Offset read so far.
   */
  const cursor = { at: from, };
  while ((cursor.at < text.length) && (owed.length > 0)) {
    /**
     Character there.
     */
    const character = text.charAt(cursor.at,);
    /**
     Innermost closer owed.
     */
    const innermost = owed.at(-1,);
    if (innermost === TEMPLATE_MARK) {
      if (character === TEMPLATE_MARK) {
        owed.pop();
        cursor.at += 1;
      }
      else if (text.startsWith(
        '${',
        cursor.at,
      )) {
        owed.push('}',);
        cursor.at += 2;
      }
      else {
        // A backslash takes the character after it with it.
        cursor.at += (character === '\\') ? 2 : 1;
      }
    }
    else if (BLANKS.has(character,) || (character === '/')) {
      /**
       Where the blank space and comments from here end.
       */
      const blank = afterBlank({
        text,
        from: cursor.at,
      },);
      // A slash still standing opens no comment: a division or a pattern.
      if ((blank.kind === 'unreadable') || (blank.at === cursor.at))
        return { kind: 'unreadable', };
      cursor.at = blank.at;
    }
    else if (QUOTES.has(character,)) {
      /**
       Where the string closes.
       */
      const closed = afterString({
        text,
        from: cursor.at,
      },);
      if (closed.kind === 'unreadable')
        return closed;
      cursor.at = closed.at;
    }
    else if (CLOSERS.has(character,)) {
      if (character !== innermost)
        return { kind: 'unreadable', };
      owed.pop();
      cursor.at += 1;
    }
    else {
      /**
       Closer the character owes: a template's, a bracket's, or none.
       */
      const owes = (character === TEMPLATE_MARK) ? TEMPLATE_MARK : CLOSER_OF.get(character,);
      if (owes !== undefined)
        owed.push(owes,);
      cursor.at += 1;
    }
  }
  return (owed.length === 0)
    ? {
      kind: 'reached',
      at: cursor.at,
    }
    : { kind: 'unreadable', };
}

/**
 Reads the run of name characters from an offset: ASCII letters and digits,
 the underscore and the dollar sign. A name written with any other letter
 ends there, which keeps its stretch cold.

 @param text - stretch text

 @param from - offset to read from

 @returns Offset one past the run, `from` itself where no name character
 stands

 @example
 ```ts
 afterName({ text: 'Nap_1$ purr', from: 0, },); // 6
 ```
 */
function afterName(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): number {
  /**
   Offset read so far.
   */
  const cursor = { at: from, };
  while (cursor.at < text.length) {
    /**
     Character there.
     */
    const character = text.charAt(cursor.at,);
    if ((!isAsciiAlphanumeric({ character, },)) && (!NAME_MARKS.has(character,)))
      return cursor.at;
    cursor.at += 1;
  }
  return cursor.at;
}

/**
 The token a string, a template or a group makes once read to its close.

 @param kind - which it is

 @param start - offset of its opening character

 @param closed - where it closes, or that it cannot be read for certain

 @returns The token, or that the text cannot be read for certain

 @example
 ```ts
 closedAtom({ kind: 'text', start: 0, closed: { kind: 'reached', at: 5, }, },); // { kind: 'text', start: 0, end: 5, }
 ```
 */
function closedAtom(
  {
    kind,
    start,
    closed,
  }: {
    readonly kind: 'group' | 'text';
    readonly start: number;
    readonly closed: Reach;
  },
): Atom {
  if (closed.kind === 'unreadable')
    return closed;
  return {
    kind,
    start,
    end: closed.at,
  };
}

/**
 Reads the token at or after an offset, past blank space and comments.

 @param text - stretch text

 @param from - offset to read from

 @returns The token with its offsets, the end of the text, or that the text
 cannot be read for certain from there

 @example
 ```ts
 atomAt({ text: 'throw new Error("nap")', from: 5, },); // { kind: 'word', start: 6, end: 9, }
 ```
 */
export function atomAt(
  {
    text,
    from,
  }: {
    readonly text: string;
    readonly from: number;
  },
): Atom {
  /**
   Where the blank space and comments before the token end.
   */
  const blank = afterBlank({
    text,
    from,
  },);
  if (blank.kind === 'unreadable')
    return blank;
  /**
   Offset of the token's first character.
   */
  const { at: start, } = blank;
  if (start >= text.length)
    return { kind: 'end', };
  /**
   That character.
   */
  const character = text.charAt(start,);
  // A slash the blank space left standing opens no comment.
  if (character === '/')
    return { kind: 'unreadable', };
  if (QUOTES.has(character,))
    return closedAtom({
      kind: 'text',
      start,
      closed: afterString({
        text,
        from: start,
      },),
    },);
  if ((character === TEMPLATE_MARK) || (character === '('))
    return closedAtom({
      kind: (character === '(') ? 'group' : 'text',
      start,
      closed: afterNesting({
        text,
        from: start + 1,
        closer: (character === '(') ? ')' : TEMPLATE_MARK,
      },),
    },);
  /**
   One past the name standing there, `start` when none does.
   */
  const nameEnd = afterName({
    text,
    from: start,
  },);
  return (nameEnd === start)
    ? {
      kind: 'mark',
      start,
      end: start + 1,
    }
    : {
      kind: 'word',
      start,
      end: nameEnd,
    };
}

//endregion Coverage stretch atoms
