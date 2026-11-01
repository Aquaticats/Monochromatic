import { access, } from 'node:fs/promises';
import { join, } from 'node:path';

import { rethrowUnlessMissingPath, } from '@monochromatic-dev/translation-repair-typography/ts';
import { StatedRefusalError, } from '../stated-refusal.ts';
import { writeFileAtomic, } from './atomic-write.ts';

//region Sheet write
// Writing a grading sheet beside its manifest.
//
// A SHEET IS A GRADER'S WORK IN PROGRESS the moment it lands, so a rerun into
// the same runs directory must not replace it: the two writers this serves
// used to write both files in place, which replaced a sheet a grader may have
// been partway through, and a crash between the two writes left a sheet and a
// manifest from different runs beside each other. Both files are refused while
// either exists, and each is written atomically.

/**
 Whether a file sits at a path.

 @param path - file to look for

 @returns Whether it is there

 @throws Whatever `access` raised other than the path's absence, since a sheet
 that could not be looked for may be a grader's work the write would replace

 @example
 ```ts
 const taken = await exists({ path, },);
 ```
 */
async function exists({ path, }: { readonly path: string; },): Promise<boolean> {
  try {
    await access(path,);
    return true;
  }
  catch (error) {
    // Absent is the ordinary answer. Anything else (a permission refused, a
    // file where a directory belongs) is raised as it came, not read as
    // absence: this comment once said the opposite while the code raised it.
    rethrowUnlessMissingPath({ error, },);
    return false;
  }
}

/**
 Refuses when either file of a sheet pair is already in the directory.

 Callable before the work that fills the pair, so a rerun that would be
 refused at the write is refused before that work is paid for.

 @param dir - directory the pair would land in

 @param sheetName - sheet file name

 @param manifestName - manifest file name

 @throws {@link StatedRefusalError} when either file is already there, naming
 the first found, the sheet before the manifest

 @example
 ```ts
 await assertSheetPairFree({ dir, sheetName: 'damage-sheet.md', manifestName: 'damage-manifest.json', },);
 ```
 */
export async function assertSheetPairFree(
  {
    dir,
    sheetName,
    manifestName,
  }: {
    readonly dir: string;
    readonly sheetName: string;
    readonly manifestName: string;
  },
): Promise<void> {
  for (const path of [
    join(
      dir,
      sheetName,
    ),
    join(
      dir,
      manifestName,
    ),
  ]) {
    /* oxlint-disable no-await-in-loop -- two files, checked in order so the refusal names the first one found */
    if (await exists({ path, },))
      throw new StatedRefusalError({
        says: `${path} already exists; grade or move it before rerunning, since a rerun would replace a grader's work`,
      },);
    /* oxlint-enable no-await-in-loop */
  }
}

/**
 Writes a sheet and its manifest, refusing to replace either.

 @param dir - directory both land in

 @param sheetName - sheet file name

 @param manifestName - manifest file name

 @param sheet - sheet text

 @param manifest - manifest text

 @returns Path the sheet landed at

 @throws {@link StatedRefusalError} when either file is already there

 @example
 ```ts
 const at = await writeSheetPair({ dir, sheetName: 'damage-sheet.md', manifestName: 'damage-manifest.json', sheet, manifest, },);
 ```
 */
export async function writeSheetPair(
  {
    dir,
    sheetName,
    manifestName,
    sheet,
    manifest,
  }: {
    readonly dir: string;
    readonly sheetName: string;
    readonly manifestName: string;
    readonly sheet: string;
    readonly manifest: string;
  },
): Promise<string> {
  /**
   Where the sheet lands.
   */
  const sheetPath = join(
    dir,
    sheetName,
  );

  /**
   Where the manifest lands.
   */
  const manifestPath = join(
    dir,
    manifestName,
  );
  await assertSheetPairFree({
    dir,
    sheetName,
    manifestName,
  },);
  await writeFileAtomic({
    path: manifestPath,
    text: manifest,
  },);
  await writeFileAtomic({
    path: sheetPath,
    text: sheet,
  },);
  return sheetPath;
}

//endregion Sheet write
