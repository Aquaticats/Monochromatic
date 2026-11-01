import { readFile, } from 'node:fs/promises';

import {
  filesystemReason,
  StatedRefusalError,
} from '@monochromatic-dev/translation-repair-glossary/ts';

//region Score sheet text
// Reads a graded sheet's text for the `score-*` runners, stating a failure in
// words rather than letting the filesystem's own error reach the operator as a
// fault in the command.

/**
 Reads a sheet the operator named or the runs directory implies.

 @param path - sheet to read

 @param label - what the sheet is called in a refusal

 @param remedy - what the operator can do about a sheet that will not read

 @returns The sheet's text

 @throws {@link StatedRefusalError} when the file cannot be read, naming the
 path and the filesystem reason

 @example
 ```ts
 const text = await readSheetText({ path, label: 'graded sheet', remedy: 'name it with --sheet', },);
 ```
 */
export async function readSheetText(
  {
    path,
    label,
    remedy,
  }: {
    readonly path: string;
    readonly label: string;
    readonly remedy: string;
  },
): Promise<string> {
  try {
    return await readFile(
      path,
      'utf8',
    );
  }
  catch (error) {
    throw new StatedRefusalError({
      says: `cannot read the ${label} at ${path} (${filesystemReason({ error, },)}); ${remedy}`,
      cause: error,
    },);
  }
}

//endregion Score sheet text
