/**
 A file nothing may open, for the cases that read what a sink says of a file that will not open: the artifact file
 every such case makes unreadable, the artifact that parses, the refusal text the guarded reader builds for it, and the
 mode-000 file itself, proved unopenable before a case relies on it.

 TEST SUPPORT, NOT PACKAGE SOURCE. The sinks belong to different packages (the attribution reader of the probe
 package, the standing-read command of the bench package), so the pieces they share are held here once.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  chmod,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import { SEAT_SYNTHETIC_TEXT_EVERYWHERE, } from '../roster-seats.test-fixture.ts';

/**
 Artifact file every case makes unreadable.
 */
export const UNREADABLE = 'Basket.json';

/**
 Opening a filesystem error used to print, which must appear nowhere.

 Kept as its own constant so each case asserts against the exact shape that
 leaked rather than against a paraphrase of it.
 */
export const LEAKED_OPENING = "permission denied, open '";

/**
 Refusal every sink is expected to report for a file that will not open.

 BUILT PER FILE rather than shared as one constant, because the lock case
 cannot choose its file name: `lockRunsDir` competes for `pass.lock` and takes
 only the directory.

 @param file - base name the sink should report, never a path

 @returns Sentence the guarded reader builds for an unopenable file

 @example
 ```ts
 expect(reason,).toBe(namedRefusal({ file: UNREADABLE, },),);
 ```
 */
export function namedRefusal(
  { file, }: { readonly file: string; },
): string {
  return `could not read ${file} as JSON (EACCES)`;
}

/**
 Hexadecimal digits of a full commit name.
 */
const COMMIT_DIGITS = 40;

/**
 Hexadecimal digits of a tree digest.
 */
const TREE_DIGITS = 64;

/**
 Pipeline commit the sound fixture records.
 */
const SHARED_TIP = 'f'.repeat(COMMIT_DIGITS,);

/**
 Built pipeline the sound fixture records.
 */
const SHARED_GENERATION = `sha256-tree-v1:${'f'.repeat(TREE_DIGITS,)}`;

/**
 One artifact that parses, so a reader has a pool to place it in.

 The attribution reader throws rather than returning an empty pool, so a
 directory holding only the broken fixture never reaches the sink at all.
 */
export const SOUND_ARTIFACT: string = JSON.stringify({
  tip: SHARED_TIP,
  pipelineDigest: SHARED_GENERATION,
  artifactSchemaVersion: 1,
  id: 'Whiskers',
  chunkCritics: [
    {
      chunkIndex: 0,
      heardCriticIds: [SEAT_SYNTHETIC_TEXT_EVERYWHERE,],
      claimAttributions: [],
    },
  ],
  issues: [],
},);

/**
 Writes a file nothing may open, and proves it cannot be opened.

 @param dir - directory to write into

 @param name - file to write

 @param body - contents, which no case should ever get to see

 @returns Path written

 @throws Error where the file opened anyway, which a run as root would do

 @example
 ```ts
 const path = await unopenable({ dir, name: UNREADABLE, body: '{}', },);
 ```
 */
export async function unopenable(
  {
    dir,
    name,
    body,
  }: {
    readonly dir: string;
    readonly name: string;
    readonly body: string;
  },
): Promise<string> {
  /**
   Path of the fixture.
   */
  const path = join(
    dir,
    name,
  );

  await writeFile(
    path,
    body,
    'utf8',
  );
  // Removing a file needs write permission on its DIRECTORY rather than on
  // the file, so the scratch directory's disposer takes this mode-000 file
  // out without chmoding it back.
  await chmod(
    path,
    0o000,
  );

  try {
    await readFile(
      path,
      'utf8',
    );
  } catch (refused) {
    // THE REFUSAL IS THE PROOF, so it is read rather than discarded. A file
    // that declined for some other reason would send a different class to the
    // sink, and every assertion on the sink's report would be about a case nobody chose.
    if (Error.isError(refused,)
      && ('code' in refused)
      && (refused.code === 'EACCES'))
      return path;

    throw new Error(
      `${name} declined to open, but not for want of permission, so this case `
        + 'would exercise a refusal it was not written for',
      { cause: refused, },
    );
  }

  throw new Error(
    `${name} opened at mode 000, so this case would test a readable file while `
      + 'reporting a pass. Run the suite as an ordinary user rather than as root.',
  );
}
