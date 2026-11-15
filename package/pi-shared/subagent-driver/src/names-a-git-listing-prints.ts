/**
 The names a Git listing prints, read from a listing that cannot quote and shown as the files they are.

 Git quotes a path its `quotePath` setting speaks for, so a line-oriented listing holds `"package/a/src/\\303\\251"` rather than
 the name: the closing quote defeats every suffix list, and a name copied from it points nowhere. `-z` cannot quote, and reading
 it one character per byte keeps every byte of a name, including bytes that are not UTF-8, which a text reading would replace.

 A brief is text, so a name is shown as its UTF-8 text when its bytes are that, and with reversible `\\x` escapes where they are
 not, or where a character would break the brief's own line.

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

 A name's bytes are the name: bytes that form UTF-8 are shown as that text, and any that do not are shown with `\x` escapes, as
 are the characters that would break the brief's own grammar (a newline, a tab, a backslash, a backtick). The escape is
 reversible, so the name a reviewer reads is the file's name and not a rendering of it that points nowhere.

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
  if (isText) {
    return Array.from(text,)
      .map(function escaped(character,) {
        /**
         The characters that would end or break the brief's own line, and the two the brief quotes a path with.
         */
        const breaksTheBrief = (character < ' ') || (character === '\u007F')
          || (character === '\\')
          || (character === '`');
        return breaksTheBrief ? `\\x${hexOf(character.codePointAt(0,) ?? 0,)}` : character;
      },)
      .join('',);
  }
  // A byte that is not part of a UTF-8 text is shown as itself when it is printable ASCII, and as its number otherwise, so a
  // reviewer sees the name's bytes and can put them back together.
  return Array.from(bytes,)
    .map(function escaped(byte,) {
    /**
     Whether the byte prints as itself: the space through the tilde, but not the backslash or the backtick, which a brief's own
     line and code span use.
     */
      const printable = ((byte >= FIRST_PRINTABLE_BYTE) && (byte <= LAST_PRINTABLE_BYTE)
        && (byte !== BACKSLASH_BYTE)
        && (byte !== BACKTICK_BYTE));
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
