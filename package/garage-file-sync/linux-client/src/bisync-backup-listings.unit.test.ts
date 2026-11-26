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
  backupListingNames,
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
 Writes empty files into a home's bisync working folder.

 @param home - home folder whose `.cache/rclone/bisync` receives the names

 @param names - file names to create

 @returns nothing once every file exists
 */
async function writeListings({
  home,
  names,
}: Readonly<{
  home: string;
  names: readonly string[];
}>): Promise<void> {
  /**
   bisync's working folder in that home.
   */
  const folder = join(
    home,
    '.cache',
    'rclone',
    'bisync',
  );
  await mkdir(folder, { recursive: true, },);
  await Promise.all(names.map(async function writeEmpty(name,) {
    await writeFile(
      join(
        folder,
        name,
      ),
      '',
    );
  },),);
}

await describe({
  name: backupListingsExist.name,
  children: [
    it({
      name: 'finds both backups when both files of this home\'s own pair exist',
      fn: async function bothBackups() {
        await using home = await emptyHome();
        /**
         The two names this home's pair leaves behind.
         */
        const names = backupListingNames({ home: home.path, },);
        await writeListings({
          home: home.path,
          names: [names.path1, names.path2,],
        },);
        expect(await backupListingsExist({ home: home.path, },),).toEqual({ exist: true, },);
      },
    },),
    it({
      name: 'one backup of the pair is not enough, whichever side it names',
      fn: async function oneSide() {
        await using firstSide = await emptyHome();
        /**
         The two names of the first home's pair.
         */
        const firstNames = backupListingNames({ home: firstSide.path, },);
        await writeListings({
          home: firstSide.path,
          names: [firstNames.path1,],
        },);
        expect((await backupListingsExist({ home: firstSide.path, },)).exist,).toBe(false,);
        await using secondSide = await emptyHome();
        /**
         The two names of the second home's pair.
         */
        const secondNames = backupListingNames({ home: secondSide.path, },);
        await writeListings({
          home: secondSide.path,
          names: [secondNames.path2,],
        },);
        expect((await backupListingsExist({ home: secondSide.path, },)).exist,).toBe(false,);
      },
    },),
    it({
      name: 'the listings of another local folder paired with the Garage path are not this home\'s pair',
      fn: async function anotherLocalFolder() {
        await using home = await emptyHome();
        /**
         The two names another local folder leaves behind when bisync pairs it with the
         same Garage path.
         */
        const otherFolder = backupListingNames({ home: join(home.path, 'Old Plain',), },);
        await writeListings({
          home: home.path,
          names: [otherFolder.path1, otherFolder.path2,],
        },);
        expect((await backupListingsExist({ home: home.path, },)).exist,).toBe(false,);
      },
    },),
    it({
      name: 'the backups of another pair are not enough',
      fn: async function otherPair() {
        await using home = await emptyHome();
        await writeListings({
          home: home.path,
          names: ['other_pair..x.path1.lst-old', 'other_pair..x.path2.lst-old',],
        },);
        expect((await backupListingsExist({ home: home.path, },)).exist,).toBe(false,);
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

await describe({
  name: backupListingNames.name,
  children: [
    it({
      name: 'names both files after the Garage path and the home folder\'s Plain',
      fn: async function exactNames() {
        expect(backupListingNames({ home: '/home/example', },),).toEqual({
          path1: 'garage_files_Plain..home_example_Plain.path1.lst-old',
          path2: 'garage_files_Plain..home_example_Plain.path2.lst-old',
        },);
      },
    },),
    it({
      name: 'replaces the characters bisync cannot keep, and strips a canonical hex span',
      fn: async function canonicalAndHex() {
        expect(backupListingNames({ home: '/home/example/My Plain', },),).toEqual({
          path1: 'garage_files_Plain..home_example_My_Plain_Plain.path1.lst-old',
          path2: 'garage_files_Plain..home_example_My_Plain_Plain.path2.lst-old',
        },);
        expect(backupListingNames({ home: '/home/example{deadbeef}', },),).toEqual({
          path1: 'garage_files_Plain..home_example_Plain.path1.lst-old',
          path2: 'garage_files_Plain..home_example_Plain.path2.lst-old',
        },);
      },
    },),
  ],
},);
