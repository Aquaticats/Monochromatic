import { readFile, } from 'node:fs/promises';

import { rethrowUnlessMissingPath, } from './missing-path-error.ts';

//region Read text if present
// A file whose absence is an ordinary state, read as empty text: a ledger
// before its first line, a cache marker before its lane first wrote. Two
// readers held this body word for word once the missing-path check was shared
// (`missing-path-error.ts`), and the duplicate-body guard refuses two copies.

/**
 Reads a file's text, empty when nothing stands at the path.

 @param path - file to read

 @returns Its text, or the empty string where it does not exist; a reader
 that must tell an empty file from an absent one reads the file itself

 @throws Whatever the read raised other than the path's absence, since a
 file that could not be read is not a file that is not there

 @example
 ```ts
 const text = await readTextOrEmptyIfMissing({ path, },);
 ```
 */
export async function readTextOrEmptyIfMissing({ path, }: { readonly path: string; },): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    rethrowUnlessMissingPath({ error, },);
    return '';
  }
}

//endregion Read text if present
