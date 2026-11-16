/**
 The names a queue gives the things it writes, and the text each name gives back: one function that turns an entry's name into a file name,
 one that turns it into a unit's name, and the inverse of each.

 A name is text a person wrote, so it may hold a slash, `..`, a newline, a quote, a leading dash, a percent sign, a dollar sign, characters
 outside ASCII, or a character no name may hold at all. Every destination has its own grammar, and text is escaped for that grammar at the
 single point it is put into a name, so no path is ever built from a raw name and no name can reach outside the folder it belongs to.

 The file name and the unit's name are one escaped form, because the form is spelled in characters both a file name and a systemd unit name
 accept: letters, digits, `_`, `-`, `.`, and a backslash only in systemd's own `\xHH` escape. systemd keeps such a name as it is instead of
 escaping it again, which is what lets the place a start asked for be counted back from the unit's name.

 What the form refuses, and says so naming the entry:

 - text that is not Unicode: a lone surrogate cannot be written to a file or a unit, so the name is refused rather than escaped into
   something no reader could give back;
 - a name whose escaped form would outgrow the longest file name the file system and systemd accept between them.

 @module
 */

import { QueueFileError, } from './queue-file-error.ts';

/**
 The characters a name keeps as they are, which every file name and every unit name takes: the underscore and the hyphen.
 */
const KEPT_PUNCTUATION = '_-';

/**
 The letters a name keeps as they are, both cases, which every file name and every unit name takes.
 */
const KEPT_LETTERS = 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ';

/**
 The digits a name keeps as they are, which every file name and every unit name takes.
 */
const KEPT_DIGITS = '0123456789';

/**
 The radix the two digits of an escape are read in.
 */
const HEXADECIMAL = 16;

/**
 How many characters one escape takes: the backslash, the `x` and two digits.
 */
const ESCAPE_LENGTH = 4;

/**
 The letter one escape begins with, after the backslash.
 */
const ESCAPE_LETTER = 'x';

/**
 The digits the two hexadecimal digits of an escape are written in.
 */
const HEXADECIMAL_DIGITS = '0123456789abcdef';

/**
 Says whether one character is one a name keeps as it is.

 @param character - the character

 @returns whether a file name and a unit name both take it

 @example
 ```ts
 const fine = isKept('a');
 ```
 */
function isKept(character: string,): boolean {
  return KEPT_LETTERS.includes(character,)
    || KEPT_DIGITS.includes(character,)
    || KEPT_PUNCTUATION.includes(character,);
}

/**
 The longest escaped name a queue writes, in characters, which leaves room for every ending a name takes on: `.json`, `.claim.json`, the
 `-set-aside.json` of a record, the `.being-written-` of a file on its way to its place, and the `launch-batch-<stamp>-<token>-queue-entry-`
 of a unit.

 The file system accepts 255 characters in a name and systemd accepts 255 in a unit's name; the longest of a queue's own names is a unit's,
 which adds 75 characters around the escaped name, and the longest file name adds about 60 around it, so 120 leaves both inside their own
 limit with room for a process number and a random part.
 */
export const LONGEST_ESCAPED_NAME = 120;

/**
 Reading a name that is no text this package can write.
 */
export class QueueNameError extends QueueFileError {
  /**
   The name printed with the message.
   */
  override readonly name: string = 'QueueNameError';
}

/**
 Says whether one escape this reader has just passed is one: an `x` and two hexadecimal digits.

 @param written - the characters after the backslash

 @returns whether they are an escape

 @example
 ```ts
 const fine = isAnEscape({ written: 'x2f' });
 ```
 */
function isAnEscape({ written, }: { readonly written: string },): boolean {
  return (written.length === (ESCAPE_LENGTH - 1))
    && (written.startsWith(ESCAPE_LETTER,))
    && (written.slice(1,)
      .split('',)
      .every(function isAHexadecimalDigit(digit,) {
        return HEXADECIMAL_DIGITS.includes(digit,);
      },));
}

/**
 Reads the UTF-8 bytes of one string, which fails on a string that is no Unicode.

 @param text - the text

 @returns its bytes

 @throws QueueNameError when the text holds a lone surrogate

 @example
 ```ts
 const bytes = utf8BytesOf({ text: 'a', where: 'the entry a' });
 ```
 */
function utf8BytesOf({
  text,
  where,
}: {
  readonly text: string;
  readonly where: string;
},): readonly number[] {
  try {
    return [...Buffer.from(
      text,
      'utf8',
    ),];
  }
  catch (error) {
    throw new QueueNameError(
      `${where} holds a character that is no Unicode, so it cannot be written to a file or to a unit: ${String(error,)}`,
    );
  }
}

/**
 Reads whether one string is Unicode text, which every name must be.

 @param text - the text

 @returns whether it can be written as UTF-8 and read back

 @example
 ```ts
 const fine = isTexToOneCanWrite({ text: 'a' });
 ```
 */
function isTexToOneCanWrite({ text, }: { readonly text: string },): boolean {
  // A string that round-trips through UTF-8 is text; one holding a lone surrogate becomes U+FFFD and does not.
  return Buffer.from(
    text,
    'utf8',
  )
    .toString(
    'utf8',
    )
    === text;
}

/**
 This text was written from no name, as a symbol so no text can be mistaken for one.
 */
export const NO_NAME_IN_AN_ESCAPED_FORM: unique symbol = Symbol('this text was written from no name of a queue',);

/**
 Reads the escaped form of one name: every character a file name or a unit name does not keep is written as the bytes of its UTF-8 form,
 one `\xHH` for each byte.

 A name that begins with a dot keeps no dot at its start, because a file named with one is a hidden file and a name of two dots is the folder
 above.

 @param name - the entry's name

 @returns the escaped form

 @throws QueueNameError when the name is no Unicode, or its escaped form would outgrow the longest name a queue writes

 @example
 ```ts
 const escaped = anEscapedName({ name: 'a b' });
 ```
 */
export function anEscapedName({ name, }: { readonly name: string },): string {
  if (!isTexToOneCanWrite({ text: name, },)) {
    throw new QueueNameError(
      `the entry named ${JSON.stringify(name,)} holds a character that is no Unicode, so it cannot be written to a file or to a unit; `
        + 'give it a name that is text',
    );
  }
  /**
   The escaped text so far, which the loop adds to one character at a time.
   */
  const escaping = { escaped: '', };
  for (let place = 0; place < name.length; place += 1) {
    /**
     The character at this place of the name.
     */
    const character = name.slice(
      place,
      place + 1,
    );
    /**
     Whether this character is one the name keeps: a first character that is no dash, or any later one.
     */
    const keep = (place === 0)
      ? (isKept(character,) && (character !== '-'))
      : isKept(character,);
    if (keep || ((character === '.') && (place > 0))) {
      escaping.escaped += character;
      continue;
    }
    /**
     The bytes of this character's UTF-8 form, one `\xHH` each.
     */
    const written = utf8BytesOf({
      text: character,
      where: `the entry named ${JSON.stringify(name,)}`,
    },)
      .map(function asAnEscape(byte,) {
        return `\\x${byte.toString(16,)
          .padStart(
          2,
          '0',
        )}`;
      },)
      .join('',);
    escaping.escaped += written;
  }
  /**
   The escaped text.
   */
  const { escaped, } = escaping;
  if (escaped.length > LONGEST_ESCAPED_NAME) {
    throw new QueueNameError(
      `the entry named ${JSON.stringify(name,)} needs ${String(escaped.length,)} characters escaped, and a queue writes a name of at `
        + `${String(LONGEST_ESCAPED_NAME,)} at most, so its file and its unit would outgrow the longest name this machine takes; give it a `
        + 'shorter name',
    );
  }
  return escaped;
}

/**
 Reads the text one escaped name was written from, which is the inverse of {@link anEscapedName}.

 @param escaped - the escaped form

 @returns the text, or nothing when the form is not one this module wrote

 @example
 ```ts
 const name = aNameInAnEscapedName({ escaped: 'a\\x20b' });
 ```
 */
export function aNameInAnEscapedName({ escaped, }: { readonly escaped: string },): string | typeof NO_NAME_IN_AN_ESCAPED_FORM {
  /**
   The bytes of the text so far, which every kept character contributes as itself and every escape as one byte.
   */
  const bytes: number[] = [];
  /**
   Where the reading stands, which the reading moves itself.
   */
  const reading = { place: 0, };
  while (reading.place < escaped.length) {
    /**
     The character at this place.
     */
    const character = escaped.slice(
      reading.place,
      reading.place + 1,
    );
    /**
     What follows it.
     */
    const next = escaped.slice(
      reading.place + 1,
      reading.place + ESCAPE_LENGTH,
    );
    if ((character === '\\') && isAnEscape({ written: next, },)) {
      bytes.push(Number.parseInt(
        next.slice(1,),
        HEXADECIMAL,
      ),);
      reading.place += ESCAPE_LENGTH;
      continue;
    }
    if ((character === '\\') || ((!isKept(character,)) && (character !== '.'))) {
      return NO_NAME_IN_AN_ESCAPED_FORM;
    }
    bytes.push(...utf8BytesOf({
      text: character,
      where: 'an escaped name',
    },),);
    reading.place += 1;
  }
  /**
   The text the bytes spell.
   */
  const text = Buffer.from(bytes,)
    .toString(
    'utf8',
    );
  // A form that does not spell the same text is one no name of this module was written from, and a name whose first character is
  // escaped reads back with that character at its start, which is what the escaper wrote.
  return (Buffer.from(
    text,
    'utf8',
  )
    .toString(
    'utf8',
    )
    === text) ? text : NO_NAME_IN_AN_ESCAPED_FORM;
}

/**
 The part of a unit's name that names the entry, after the words that name the list and the attempt.
 */
const ENTRY_PART_OF_A_UNIT_NAME = 'queue-entry-';

/**
 Reads the part of a unit's name that names one entry, which is what a batch list's name and its unit's name both carry.

 It is the escaped name with one word before it, so a unit's name is text systemd keeps as it is, a batch list's file name is the same
 text, and the place a start asked for can be counted back from the unit the manager lists.

 @param name - the entry's name

 @returns the part of the unit's name

 @throws QueueNameError when the name is no Unicode, or would outgrow the longest name a queue writes

 @example
 ```ts
 const part = aUnitNameOf({ name: 'review-a' });
 ```
 */
export function aUnitNameOf({ name, }: { readonly name: string },): string {
  return `${ENTRY_PART_OF_A_UNIT_NAME}${anEscapedName({ name, },)}`;
}

/**
 Reads the entry's name out of the part of a unit's name that names it, which is the inverse of {@link aUnitNameOf}.

 @param unitName - a unit's name, or the part of one that names the entry

 @returns the entry's name, or nothing when the text is not one this module wrote

 @example
 ```ts
 const name = anEntryNameInAUnitName({ unitName: 'launch-batch-x-queue-entry-review-a' });
 ```
 */
export function anEntryNameInAUnitName({ unitName, }: { readonly unitName: string },): string | typeof NO_NAME_IN_AN_ESCAPED_FORM {
  /**
   Where the part that names the entry begins, when the unit's name holds one.
   */
  const beginsAt = unitName.indexOf(ENTRY_PART_OF_A_UNIT_NAME,);
  if (beginsAt === (-1)) {
    return NO_NAME_IN_AN_ESCAPED_FORM;
  }
  return aNameInAnEscapedName({
    escaped: unitName.slice(beginsAt + ENTRY_PART_OF_A_UNIT_NAME.length,),
  },);
}
