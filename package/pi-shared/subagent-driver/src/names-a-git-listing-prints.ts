/**
 The names a Git listing prints, read from a listing that cannot quote and shown as the files they are.

 Git quotes a path its `quotePath` setting speaks for, so a line-oriented listing holds `"package/a/src/\\303\\251"` rather than
 the name: the closing quote defeats every suffix list, and a name copied from it points nowhere. `-z` cannot quote, and reading
 it one character per byte keeps every byte of a name, including bytes that are not UTF-8, which a text reading would replace.

 A brief is text, so a name is shown as its UTF-8 text when its bytes are that, and escaped where they are not or where a
 character would break the brief's own line: a byte that prints is shown as itself, the backslash as `\\` and the apostrophe as
 `\'`, so the name is a `$'...'` word a shell gives back byte for byte.

 @module
 */

/**
 The byte a space is, which is the first that prints as itself.
 */
const FIRST_PRINTABLE_BYTE = 32;

/**
 The byte a tilde is, which is the last that prints as itself.
 */
const LAST_PRINTABLE_BYTE = 126;

/**
 The byte a backslash is, which a brief's line would take for the start of an escape.
 */
const BACKSLASH_BYTE = 92;

/**
 Says whether a character is one a brief cannot carry as it is.

 @param character - one character of a name's text

 @returns whether it is a control character, the delete character, the backslash, the apostrophe or the backtick, each of which
 would end or change the brief's own line, its code span or the shell word it names

 @example
 ```ts
 const breaksTheBrief = breaksTheBrief('a',);
 ```
 */
function breaksTheBrief(character: string,): boolean {
  return (character < ' ')
    || (character === '\u007F')
    || (character === '\\')
    || (character === '\'')
    || (character === '`');
}

/**
 The byte an apostrophe is, which would end the `$'...'` word a brief sends a reader to.
 */
const APOSTROPHE_BYTE = 39;

/**
 The byte a backtick is, which would end the code span a brief quotes a path with.
 */
const BACKTICK_BYTE = 96;

/**
 Reads the paths of a listing that cannot quote, which Git prints with `-z`: one name per NUL byte, and no quoting at all.

 Git quotes a path its `quotePath` setting speaks for, and a line-oriented listing then holds `"package/a/src/\303\251"` rather
 than the name: its quotes defeat every suffix list, and the name in the brief points nowhere. A listing it cannot quote is read
 one character per byte, so each name comes back as the bytes Git printed, which is what the file is.

 @param listing - what Git printed with `-z`, read one character per byte

 @returns each name as the brief shows it, in the order Git printed them

 @example
 ```ts
 const paths = pathsInAListing({ listing: 'src/a.unit.test.ts\0src/fixture/a.json\0', },);
 ```
 */
export function pathsInAListing({ listing }: { readonly listing: string }): readonly string[] {
  return listing.split('\0',)
    .filter(function isAName(name,) {
      return name !== '';
    },)
    .map(function shown(name,) {
      return shownNameOf({ byteCharacters: name, },);
    },);
}

/**
 The bytes of one name, read as text when they are text.

 A name's bytes are the name. Bytes that form UTF-8 are shown as that text; a name whose bytes are not UTF-8, or that holds a
 character the brief's own line or code span cannot carry, is shown escaped: a byte that prints as itself is shown as itself, the
 backslash as `\\`, the apostrophe as `\'`, and every other byte as `\xNN`.

 The escapes obey the grammar the brief sends a reader to: `$'...'`, where the apostrophe and the backslash are the two characters
 that would otherwise end or change the word. A name shown escaped is therefore the file's own bytes when a shell reads it as that
 word, for every byte a name may hold.

 @param byteCharacters - the name as Git printed it, one character per byte

 @returns the name as the brief shows it

 @example
 ```ts
 const shown = shownNameOf({ byteCharacters: 'package/a/src/\303\251.unit.test.ts', },);
 ```
 */
function shownNameOf({ byteCharacters }: { readonly byteCharacters: string }): string {
  /**
   The name's bytes, which one character per byte keeps whole.
   */
  const bytes = Buffer.from(
    byteCharacters,
    'latin1',
  );
  /**
   The name as text, when its bytes are UTF-8.
   */
  const text = bytes.toString('utf8',);
  /**
   Whether those bytes are that text, which they are when writing the text back gives the same bytes.
   */
  const isText = Buffer.from(
    text,
    'utf8',
  )
    .equals(bytes,);
  /**
   Whether the name is text and holds nothing the brief cannot carry, which is the one case it needs no shell word for.
   */
  const needsNoShellWord = isText && (!Array.from(text,)
    .some(breaksTheBrief,));
  if (needsNoShellWord) {
    // A name of text with nothing the brief cannot carry is its own name, and needs no shell word to be read.
    return text;
  }
  if (isText) {
    // Every character the brief cannot carry is escaped, and so are the two a `$'...'` word would end or change.
    return Array.from(text,)
      .map(function escaped(character,) {
        if (character === '\\') {
          return String.raw`\\`;
        }
        if (character === '\'') {
          return `\\'`;
        }
        return breaksTheBrief(character,) ? `\\x${hexOf(character.codePointAt(0,) ?? 0,)}` : character;
      },)
      .join('',);
  }
  // A byte that is not part of a UTF-8 text is shown as itself when it prints, and as its number otherwise, so a reviewer sees
  // the name's bytes and a shell gives them back.
  return Array.from(bytes,)
    .map(function escaped(byte,) {
      if (byte === BACKSLASH_BYTE) {
        return String.raw`\\`;
      }
      if (byte === APOSTROPHE_BYTE) {
        return `\\'`;
      }
      /**
       Whether the byte prints as itself: the space through the tilde, but not the backtick, which is escaped above.
       */
      const printable = (byte >= FIRST_PRINTABLE_BYTE)
        && (byte <= LAST_PRINTABLE_BYTE)
        && (byte !== BACKTICK_BYTE);
      return printable ? String.fromCodePoint(byte,) : `\\x${hexOf(byte,)}`;
    },)
    .join('',);
}

/**
 Reads one byte as the two hex digits the brief shows it with.

 @param byte - the byte, from 0 to 255

 @returns two lowercase hex digits

 @example
 ```ts
 const shown = hexOf(0xe9,);
 ```
 */
function hexOf(byte: number,): string {
  return byte.toString(16,)
    .padStart(
    2,
    '0',
    );
}
