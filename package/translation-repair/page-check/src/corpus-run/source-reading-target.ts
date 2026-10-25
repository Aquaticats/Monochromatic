//region Source reading target
// THE TARGET OF A LINK AS A READER OF THE PAGE OR OF THE SOURCE HAS IT, decoded as far as it reads. A percent escape that is
// no text in UTF-8 (`%E0%A4`, a lone `%`) makes `decodeURI` throw, and a page that parses and holds such a link could not be
// compared at all. The account marks where a link stands with its target (`source-reading-account.ts`), and the targets are
// dropped before the two accounts are compared, so what a malformed escape decodes to decides no answer; it decodes as the
// WHATWG decoding of UTF-8 does, a replacement character for bytes that read as no character, and an escape that is none
// (`%zz`, a `%` with fewer than two digits after it) stays as it stands. Where `decodeURI` succeeds the result is the same,
// the escapes of the characters it keeps escaped (`;/?:@&=+$,#`) included.

/**
 Decodes UTF-8 and replaces what is no text with a replacement character; keeps a byte order mark, which `decodeURI` does.
 */
const UTF8: TextDecoder = new TextDecoder(
  'utf-8',
  {
    fatal: false,
    ignoreBOM: true,
  },
);

/**
 Characters `decodeURI` leaves escaped, because they delimit the parts of an address.
 */
const KEPT_ESCAPED = ';/?:@&=+$,#';

/**
 The first value of a byte that is no ASCII character, which no escape of a delimiter reaches.
 */
const FIRST_NON_ASCII = 0x80;

/**
 The digits of a hexadecimal number, in either case.
 */
const HEX_DIGITS = '0123456789abcdefABCDEF';

/**
 What the byte of an escape is when the text at an index starts none.
 */
const NO_ESCAPE = -1;

/**
 The number of characters of an escape: the percent sign and two hexadecimal digits.
 */
const ESCAPE_LENGTH = 3;

/**
 The byte a percent escape at an index stands for.

 @param target - text of a link target

 @param at - index to read from

 @returns Value of the byte; -1 when the text at the index is no percent sign followed by two hexadecimal digits

 @example
 ```ts
 const byte = escapedByteAt({ target: 'a%20b', at: 1, },); // 32
 ```
 */
function escapedByteAt(
  {
    target,
    at,
  }: {
    readonly target: string;
    readonly at: number;
  },
): number {
  if (target.charAt(at,) !== '%')
    return NO_ESCAPE;
  /**
   The two characters after the percent sign.
   */
  const digits = target.slice(
    at + 1,
    at + ESCAPE_LENGTH,
  );
  if (digits.length !== 2)
    return NO_ESCAPE;
  for (const digit of digits) {
    if (!HEX_DIGITS.includes(digit,))
      return NO_ESCAPE;
  }
  return Number.parseInt(
    digits,
    16,
  );
}

/**
 Decodes a target as far as it reads.

 @param target - text of a link target as a parse gives it

 @returns Target with each run of valid escapes decoded as UTF-8, bytes that read as no character as replacement
 characters, the escapes `decodeURI` keeps and the text that is no escape as they stand

 @example
 ```ts
 const decoded = decodedTarget({ target: 'a%20b%E0%A4', },); // 'a b�'
 ```
 */
export function decodedTarget({ target, }: { readonly target: string; },): string {
  /**
   Pieces of the result, joined at the end.
   */
  const pieces: string[] = [];
  /**
   Bytes of the run of escapes being read, decoded together since a character may stand in several.
   */
  const run: number[] = [];

  /**
   Decodes the bytes read so far and starts a new run.

   @example
   ```ts
   endRun();
   ```
   */
  function endRun(): void {
    if (run.length === 0)
      return;
    /**
     The bytes of the run as the decoder takes them.
     */
    const bytes = Uint8Array.from(run,);
    pieces.push(UTF8.decode(bytes,),);
    run.length = 0;
  }

  for (let at = 0; at < target.length;) {
    /**
     The byte an escape here stands for, none where the text is no escape.
     */
    const byte = escapedByteAt({
      target,
      at,
    },);
    if (byte === NO_ESCAPE) {
      endRun();
      pieces.push(target.charAt(at,),);
      at += 1;
      continue;
    }
    if ((byte < FIRST_NON_ASCII) && KEPT_ESCAPED.includes(String.fromCodePoint(byte,),)) {
      endRun();
      pieces.push(target.slice(
        at,
        at + ESCAPE_LENGTH,
      ),);
    }
    else
      run.push(byte,);
    at += ESCAPE_LENGTH;
  }
  endRun();
  return pieces.join('',);
}

//endregion Source reading target
