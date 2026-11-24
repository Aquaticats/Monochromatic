/**
 Reads the text a file holds before a tool call changes it.
 
 Judging a change needs the file as it stands:
 an edit's own new text says nothing about the violation an older line already
 holds,
 and a rewrite that keeps one is only a repair if the file really held it.
 A file that does not exist yet holds no text,
 and a file that cannot be read is treated the same way,
 so a change to it is judged against nothing rather than against a guess.
 
 @module
 */

import { readFile, } from 'node:fs/promises';

import { caughtValueText, } from '@monochromatic-dev/caught-value-core/ts';
import { tagged, } from '@monochromatic-dev/logger-core/ts';

//region Logging

/**
 Logger root for the literal-`rm`-path guard.
 */
const parentLogger = tagged({ tag: 'literal-rm-paths', },);

//endregion Logging

//region Constants

/**
 Error code a missing file reports, which is not a failure for a file a call creates.
 */
const MISSING_FILE_CODE = 'ENOENT';

//endregion Constants

//region Predicates

/**
 Reports whether a caught value carries the errno code of a missing file.
 
 @param error - value a filesystem call threw
 
 @returns whether the failure is that the file does not exist
 
 @example
 ```ts
 isMissingFile(Object.assign(new Error('x'), { code: 'ENOENT' })); // true
 ```
 */
function isMissingFile(error: unknown,): boolean {
  return ((typeof error) === 'object')
    && (error !== null)
    && ('code' in error)
    && (error.code === MISSING_FILE_CODE);
}

//endregion Predicates

//region Reads

/**
 Reads the text a file holds now, empty when it cannot be read.
 
 @param path - absolute path of the file a call changes
 
 @returns the file's text, or empty text when it does not exist or cannot be read
 
 @example
 ```ts
 await textBeforeAFileChange({ path: '/repo/clean.sh', });
 ```
 */
async function textBeforeAFileChange({ path, }: { readonly path: string; },): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error: unknown) {
    if (isMissingFile(error,))
      return '';
    /**
     Function-tagged logger reporting a file whose earlier text is unknown.
     */
    const innerLogger = tagged({
      tag: textBeforeAFileChange.name,
      l: parentLogger,
    },);
    innerLogger.debug(
      `text before the change not read, judging against empty text: ${path}: ${caughtValueText(error,)}`,
    );
    return '';
  }
}

//endregion Reads

export {
  isMissingFile,
  textBeforeAFileChange,
};
