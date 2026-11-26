/**
 Tests for the unit's open streak: what a readable state file holds, and every reason the
 read reports it could not use one instead.

 @module
 */

import {
  chmod,
  mkdir,
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
} from 'node:path';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  readStreak,
  stateDirectory,
  streakFilePath,
} from './alert-streak.ts';
import { SYNC_UNIT, } from './units.ts';

/**
 Whether the tests run as root, which ignores file modes and so cannot make a state file
 unreadable with `chmod`.
 */
const RUNS_AS_ROOT = process.getuid?.() === 0;

/**
 ISO 8601 timestamp of the alert a state file records.
 */
const LAST_ALERT_AT = '2026-11-25T20:41:14-05:00';

/**
 Creates a disposable home for one read.

 @returns the home folder, removed when the caller's scope ends
 */
async function disposableHome() {
  return await mkdtempDisposable(join(scratchRoot(), 'garage-alert-streak-',),);
}

/**
 Writes the sync unit's state file into a home.

 @param home - disposable home to write into

 @param text - the text the file holds
 */
async function writeStateFile({
  home,
  text,
}: Readonly<{
  home: string;
  text: string;
}>): Promise<void> {
  await mkdir(stateDirectory({ home, },), { recursive: true, },);
  await writeFile(streakFilePath({
    home,
    unit: SYNC_UNIT,
  },), text, 'utf8',);
}

await describe({
  name: readStreak.name,
  children: [
    //region A usable state file
    it({
      name: 'reads the alert time a state file records',
      fn: async function readsRecordedAlertTime() {
        await using home = await disposableHome();
        await writeStateFile({
          home: home.path,
          text: `${JSON.stringify({
            unit: SYNC_UNIT,
            lastAlertAt: LAST_ALERT_AT,
          },)}\n`,
        },);
        expect(await readStreak({ home: home.path, unit: SYNC_UNIT, },),).toEqual({
          streak: { lastAlertAtMs: Date.parse(LAST_ALERT_AT,), },
        },);
      },
    },),
    it({
      name: 'reports no streak and no problem when the unit has no state file',
      fn: async function missingStateFile() {
        await using home = await disposableHome();
        expect(await readStreak({ home: home.path, unit: SYNC_UNIT, },),).toEqual({},);
      },
    },),
    //endregion A usable state file

    //region A state file that cannot be read
    it({
      name: 'reports the state file it cannot read, naming the file and the reason',
      skip: RUNS_AS_ROOT ? 'root ignores file modes, so chmod cannot make the state file unreadable' : false,
      fn: async function unreadableStateFile() {
        await using home = await disposableHome();
        /**
         The state file this unit records its streak in.
         */
        const path = streakFilePath({
          home: home.path,
          unit: SYNC_UNIT,
        },);
        await writeStateFile({
          home: home.path,
          text: '{}',
        },);
        await chmod(path, 0o000,);
        /**
         What the read found.
         */
        const found = await readStreak({ home: home.path, unit: SYNC_UNIT, },);
        expect(found.streak,).toBeUndefined();
        expect(found.problem,).toContain(`could not read ${path}`,);
        expect(found.problem,).toContain('EACCES',);
      },
    },),
    it({
      name: 'reports the state file under a folder that is a regular file, naming the file',
      fn: async function folderIsAFile() {
        await using home = await disposableHome();
        /**
         Folder the state file would sit in, written as a regular file instead.
         */
        const folder = stateDirectory({ home: home.path, },);
        await mkdir(dirname(folder,), { recursive: true, },);
        await writeFile(folder, 'not a folder\n',);
        /**
         The state file path that folder cannot hold.
         */
        const path = streakFilePath({
          home: home.path,
          unit: SYNC_UNIT,
        },);
        /**
         What the read found.
         */
        const found = await readStreak({ home: home.path, unit: SYNC_UNIT, },);
        expect(found.streak,).toBeUndefined();
        expect(found.problem,).toContain(`could not read ${path}`,);
      },
    },),
    //endregion A state file that cannot be read

    //region A state file that holds no streak
    it({
      name: 'reports a state file that does not hold JSON',
      fn: async function notJson() {
        await using home = await disposableHome();
        await writeStateFile({
          home: home.path,
          text: 'a streak that is not JSON\n',
        },);
        /**
         What the read found.
         */
        const found = await readStreak({ home: home.path, unit: SYNC_UNIT, },);
        expect(found.streak,).toBeUndefined();
        expect(found.problem,).toContain('the streak file the text does not hold JSON',);
      },
    },),
    it({
      name: 'reports a state file that holds no lastAlertAt field',
      fn: async function noLastAlertAt() {
        await using home = await disposableHome();
        await writeStateFile({
          home: home.path,
          text: `${JSON.stringify({ unit: SYNC_UNIT, },)}\n`,
        },);
        /**
         What the read found.
         */
        const found = await readStreak({ home: home.path, unit: SYNC_UNIT, },);
        expect(found.streak,).toBeUndefined();
        expect(found.problem,).toContain('does not hold an object with a lastAlertAt field',);
      },
    },),
    it({
      name: 'reports a lastAlertAt that is not a timestamp string',
      fn: async function lastAlertAtIsNotText() {
        await using home = await disposableHome();
        await writeStateFile({
          home: home.path,
          text: `${JSON.stringify({ lastAlertAt: 0, },)}\n`,
        },);
        /**
         What the read found.
         */
        const found = await readStreak({ home: home.path, unit: SYNC_UNIT, },);
        expect(found.streak,).toBeUndefined();
        expect(found.problem,).toContain("the streak file's lastAlertAt is not a timestamp string",);
      },
    },),
    it({
      name: 'reports a lastAlertAt string that names no date',
      fn: async function lastAlertAtIsNoDate() {
        await using home = await disposableHome();
        /**
         Text that is a string but not a timestamp.
         */
        const notADate = 'last Tuesday';
        await writeStateFile({
          home: home.path,
          text: `${JSON.stringify({ lastAlertAt: notADate, },)}\n`,
        },);
        /**
         What the read found.
         */
        const found = await readStreak({ home: home.path, unit: SYNC_UNIT, },);
        expect(found.streak,).toBeUndefined();
        expect(found.problem,).toContain(`lastAlertAt "${notADate}" is not a timestamp`,);
      },
    },),
    //endregion A state file that holds no streak
  ],
},);
