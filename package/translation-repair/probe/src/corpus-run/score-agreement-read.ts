import { readFile, } from 'node:fs/promises';

import { isMissingPathError, } from '@monochromatic-dev/translation-repair-typography/ts';
import {
  filesystemReason,
  StatedRefusalError,
} from '@monochromatic-dev/translation-repair-glossary/ts';

//region Score agreement read
// Reads the optional files that sit beside a graded detection sheet.

/**
 File name pattern for the blind pre-grades of one draw.

 @param seed - draw seed the pre-grades belong to

 @returns File name beside the sheet

 @example
 ```ts
 const name = preGradeName({ seed: DEFAULT_SAMPLE_SEED, },);
 ```
 */
export function preGradeName({ seed, }: { readonly seed: string; },): string {
  return `pre-grades-${seed}.json`;
}

/**
 What reading an optional file found.

 @example
 ```ts
 const reading: FileReading = { found: false, };
 ```
 */
export type FileReading =
  | {
    /**
     File is not there, which for pre-grades is the ordinary case.
     */
    readonly found: false;
  }
    | {
    /**
     File was read.
     */
      readonly found: true;

    /**
     Its contents.
     */
      readonly text: string;
    };

/**
 Reads a file, naming its absence rather than returning nothing.

 @param path - file to read

 @param label - what the file is called in a refusal

 @returns Contents, or a named absence when the file does not exist

 @throws {@link StatedRefusalError} when the read failed for any reason but a
 plain absence, naming the path and the filesystem reason, because a
 permissions or IO fault must not read as "no pre-grades recorded"

 @example
 ```ts
 const reading = await readOptional({ path, label: 'pre-grades file', },);
 ```
 */
export async function readOptional(
  {
    path,
    label,
  }: {
    readonly path: string;
    readonly label: string;
  },
): Promise<FileReading> {
  try {
    return {
      found: true,
      text: await readFile(
        path,
        'utf8',
      ),
    };
  }
  catch (error) {
    // An absent pre-grade file is the ordinary case before calibration starts;
    // anything else is a real fault and must surface, in words.
    if (isMissingPathError({ error, },))
      return { found: false, };
    throw new StatedRefusalError({
      says: `cannot read the ${label} at ${path} (${filesystemReason({ error, },)})`,
      cause: error,
    },);
  }
}

/**
 Reads a file that is optional beside a sheet but required where the operator
 named it.

 A flag that names a file the report then does not read, and says nothing
 about, scores without the file the operator typed the flag to supply.

 @param path - file to read, named by the operator or derived beside the sheet

 @param label - what the file is called in a refusal

 @param flag - flag that names the file, without its dashes

 @param named - whether the operator wrote the flag

 @returns Contents, or a named absence when nothing was named and nothing is there

 @throws {@link StatedRefusalError} when the operator named a file that is not
 there, or the read failed for any reason but a plain absence

 @example
 ```ts
 const reading = await readNamedOrBeside({ path, label: 'sample manifest', flag: 'manifest', named: true, },);
 ```
 */
export async function readNamedOrBeside(
  {
    path,
    label,
    flag,
    named,
  }: {
    readonly path: string;
    readonly label: string;
    readonly flag: string;
    readonly named: boolean;
  },
): Promise<FileReading> {
  /**
   What reading the file found.
   */
  const reading = await readOptional({
    path,
    label,
  },);
  if (named && (!reading.found))
    throw new StatedRefusalError({
      says: `--${flag} names ${path}, which is not there; name a ${label} that exists, or leave the flag out to look `
        + 'beside the sheet',
    },);
  return reading;
}

//endregion Score agreement read
