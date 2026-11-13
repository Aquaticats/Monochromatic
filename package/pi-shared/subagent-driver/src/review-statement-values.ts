/**
 Reads one value of a review's statement into the shape it must hold: an object, text, a whole number from zero, an array of
 objects, and the field lists that say which names the shape carries.

 Each refusal names the file, the field's path, and what about the value does not fit, so a reviewer whose block is wrong is told
 which value to replace rather than that the block is wrong.

 @module
 */

import {
  isRecord,
} from './read-pi-fields.ts';
import {
  REVIEW_STATEMENT_LANGUAGE,
  ReviewStatementError,
} from './review-statement-shape.ts';

/**
 Builds the error one unreadable statement answers with.

 @param file - the review file the statement came from

 @param message - what does not fit

 @returns the error the reader throws

 @example
 ```ts
 throw reviewStatementErrorOf({ file: 'a/review.md', message: 'the block is no JSON' });
 ```
 */
export function reviewStatementErrorOf({
  file,
  message,
}: {
  readonly file: string;
  readonly message: string;
},): Error {
  return new ReviewStatementError(`${file}: ${message}`,);
}

/**
 Reads a value that must be a JSON object.

 @param value - the value to read

 @param where - the field's path, for the message

 @param file - the review file, for the message

 @returns the object

 @throws ReviewStatementError when the value is no object

 @example
 ```ts
 const diff = objectIn({ value: kept.diff, where: 'diff', file: 'a/review.md' });
 ```
 */
export function objectIn({
  value,
  where,
  file,
}: {
  readonly value: unknown;
  readonly where: string;
  readonly file: string;
},): Record<string, unknown> {
  if (!isRecord(value,)) {
    throw reviewStatementErrorOf({
      file,
      message: `${where} is no JSON object`,
    });
  }
  return value;
}

/**
 Reads a value that must be non-empty text.

 @param value - the value to read

 @param where - the field's path, for the message

 @param file - the review file, for the message

 @returns the text

 @throws ReviewStatementError when the value is no text or is blank

 @example
 ```ts
 const commit = textIn({ value: kept.commit, where: 'commit', file: 'a/review.md' });
 ```
 */
export function textIn({
  value,
  where,
  file,
}: {
  readonly value: unknown;
  readonly where: string;
  readonly file: string;
},): string {
  if (((typeof value) !== 'string') || (value.trim() === '')) {
    throw reviewStatementErrorOf({
      file,
      message: `${where} is no text, so what it names cannot be read`,
    });
  }
  return value;
}

/**
 Reads a value that must be a whole number from zero.

 @param value - the value to read

 @param where - the field's path, for the message

 @param file - the review file, for the message

 @returns the number

 @throws ReviewStatementError when the value is no whole number from zero

 @example
 ```ts
 const lines = wholeNumberIn({ value: kept.lines, where: 'diff.lines', file: 'a/review.md' });
 ```
 */
export function wholeNumberIn({
  value,
  where,
  file,
}: {
  readonly value: unknown;
  readonly where: string;
  readonly file: string;
},): number {
  if (((typeof value) !== 'number') || (!Number.isInteger(value,))
    || (value < 0)) {
    throw reviewStatementErrorOf({
      file,
      message: `${where} is no whole number from zero, so how many lines it names cannot be read`,
    });
  }
  return value;
}

/**
 Reads a value that must be a JSON array of objects.

 @param value - the value to read

 @param where - the field's path, for the message

 @param file - the review file, for the message

 @returns the entries

 @throws ReviewStatementError when the value is no array, or one entry is no object

 @example
 ```ts
 const files = objectsIn({ value: kept.files, where: 'files', file: 'a/review.md' });
 ```
 */
export function objectsIn({
  value,
  where,
  file,
}: {
  readonly value: unknown;
  readonly where: string;
  readonly file: string;
},): readonly Readonly<Record<string, unknown>>[] {
  if (!Array.isArray(value,)) {
    throw reviewStatementErrorOf({
      file,
      message: `${where} is no JSON array`,
    });
  }
  return value.map(function asAnObject(
    entry,
    at
  ) {
    return objectIn({
      value: entry,
      where: `${where} entry ${String(at + 1,)}`,
      file,
    },);
  },);
}

/**
 Refuses a field the reader would drop, naming the fields it does read.

 @param held - the object as the reviewer wrote it

 @param read - the fields the reader reads

 @param where - the object's path, for the message

 @param file - the review file, for the message

 @throws ReviewStatementError when the object carries a field outside the shape

 @example
 ```ts
 refuseUnreadFields({ held, read: FILE_FIELDS, where: 'files entry 1', file: 'a/review.md' });
 ```
 */
export function refuseUnreadFields({
  held,
  read,
  where,
  file,
}: {
  readonly held: Readonly<Record<string, unknown>>;
  readonly read: readonly string[];
  readonly where: string;
  readonly file: string;
},): void {
  /**
   The fields the object carries that no reader reads.
   */
  const unread = Object.keys(held,)
    .filter(function isUnread(field,) {
      return !read.includes(field,);
    },);
  if (unread.length === 0) {
    return;
  }
  throw reviewStatementErrorOf({
    file,
    message: `${where} names a field this reader does not read: ${unread.join(', ')}; it reads ${read.join(', ')}`,
  });
}

/**
 Reads one file entry.

/**
 Reads a block as JSON, naming the file when it is no JSON.

 @param block - the text inside the fenced block

 @param file - the review file, for the message

 @returns the value the block holds, of any shape

 @throws ReviewStatementError when the block is no JSON, because naming the file is what a reader needs

 @example
 ```ts
 const value = parsedJsonOf({ block: '{"commit":"abc"}', file: 'a/review.md' });
 ```
 */
export function parsedJsonOf({
  block,
  file,
}: {
  readonly block: string;
  readonly file: string;
},): unknown {
  try {
    return JSON.parse(block,);
  }
  catch (error) {
    throw reviewStatementErrorOf({
      file,
      message: `the "${REVIEW_STATEMENT_LANGUAGE}" block is no JSON: ${String(error,)}`,
    },);
  }
}
