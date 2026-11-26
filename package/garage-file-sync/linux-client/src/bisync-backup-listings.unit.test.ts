import {
  mkdir,
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  BISYNC_LISTING_PREFIX,
  backupListingsExist,
} from './bisync-backup-listings.ts';

/**
 Makes a disposable home with no bisync working folder.

 @returns the disposable home
 */
async function emptyHome() {
  return await mkdtempDisposable(join(scratchRoot(), 'garage-bisync-listings-',),);
}

/**
 Makes a disposable home with bisync's working folder holding the given file names.

 @param names - file names to create in `.cache/rclone/bisync`

 @returns the disposable home
 */
async function homeWith(names: readonly string[],) {
  /**
   The disposable home.
   */
  const home = await emptyHome();
  /**
   bisync's working folder in that home.
   */
  const folder = join(
    home.path,
    '.cache',
    'rclone',
    'bisync',
  );
  await mkdir(folder, { recursive: true, },);
  await Promise.all(names.map(async function writeEmpty(name,) {
    await writeFile(join(folder, name,), '',);
  },),);
  return home;
}

await describe({
  name: backupListingsExist.name,
  children: [
    it({
      name: 'finds both backups of the Garage pair',
      fn: async function bothBackups() {
        await using home = await homeWith([
          `${BISYNC_LISTING_PREFIX}home_example_Plain.path1.lst-old`,
          `${BISYNC_LISTING_PREFIX}home_example_Plain.path2.lst-old`,
        ],);
        expect(await backupListingsExist({ home: home.path, },),).toEqual({ exist: true, },);
      },
    },),
    it({
      name: 'one backup, or backups of another pair, are not enough',
      fn: async function partialBackups() {
        await using oneSide = await homeWith([`${BISYNC_LISTING_PREFIX}x.path1.lst-old`,],);
        expect((await backupListingsExist({ home: oneSide.path, },)).exist,).toBe(false,);
        await using otherPair = await homeWith(['other_pair..x.path1.lst-old', 'other_pair..x.path2.lst-old',],);
        expect((await backupListingsExist({ home: otherPair.path, },)).exist,).toBe(false,);
      },
    },),
    it({
      name: 'a missing working folder reports no backups and names the folder',
      fn: async function missingFolder() {
        await using home = await emptyHome();
        /**
         What the check found.
         */
        const found = await backupListingsExist({ home: home.path, },);
        expect(found.exist,).toBe(false,);
        expect(found.problem,).toContain(join('.cache', 'rclone', 'bisync',),);
      },
    },),
  ],
},);
