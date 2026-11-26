/**
 Tests for the install entry as a command: the files `node src/install.ts --home=<folder>`
 writes, the exit statuses `--check` reports, and the usage line for a command line it
 cannot use.

 @module
 */

import {
  chmod,
  lstat,
  mkdtempDisposable,
  readFile,
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
  type CommandResult,
  runCommand,
} from './run-command.ts';

/**
 The install entry this package's `install` and `check:installed` tasks run.
 */
const INSTALL_ENTRY = join(import.meta.dirname, 'install.ts',);

/**
 The package folder whose `systemd/` and `config/` folders the install copies.
 */
const PACKAGE_ROOT = join(import.meta.dirname, '..',);

/**
 Whether the tests run as root, which ignores file modes and so cannot make an installed
 copy unreadable with `chmod`.
 */
const RUNS_AS_ROOT = process.getuid?.() === 0;

/**
 Names of the unit files the README names, from `systemd/` straight into the user
 manager's folder, the drop-in folder included.
 */
const UNIT_FILE_NAMES: readonly string[] = [
  'rclone-bisync-garage.service',
  join('rclone-bisync-garage.service.d', '60-alert-on-failure.conf',),
  'rclone-bisync-garage.timer',
  'rclone-bisync-garage-after-current.service',
  'rclone-bisync-garage-watch.service',
  'rclone-bisync-garage-alert@.service',
  'rclone-bisync-garage-recovered@.service',
];

/**
 Folder in the package that holds the unit sources, relative to the package folder.
 */
const UNIT_SOURCE_FOLDER = 'systemd';

/**
 Folder the user manager loads the installed units from, relative to the home folder.
 */
const UNIT_TARGET_FOLDER = join(
  '.config',
  'systemd',
  'user',
);

/**
 One file the README promises: where the package keeps it and where it must end up, both
 relative.
 */
type InstalledFile = {
  readonly source: string;
  readonly target: string;
};

/**
 Every file the README promises, from its unit list and its install section.
 */
const INSTALLED_FILES: readonly InstalledFile[] = [
  ...UNIT_FILE_NAMES.map(function unitFile(name,): InstalledFile {
    return {
      source: join(UNIT_SOURCE_FOLDER, name,),
      target: join(UNIT_TARGET_FOLDER, name,),
    };
  },),
  {
    source: join('config', 'garage-plain.yml',),
    target: join('.config', 'directory-watcher', 'garage-plain.yml',),
  },
];

/**
 Creates a disposable home for one run of the entry.

 @returns the home folder, removed when the caller's scope ends
 */
async function disposableHome() {
  return await mkdtempDisposable(join(scratchRoot(), 'garage-install-entry-',),);
}

/**
 Runs the install entry against one home folder.

 @param home - disposable home to install into or check

 @param args - arguments after the home folder, such as `--check`

 @returns what the entry printed and its exit status
 */
async function runInstall({
  home,
  args = [],
}: Readonly<{
  home: string;
  args?: readonly string[];
}>): Promise<CommandResult> {
  return await runCommand({
    command: process.execPath,
    args: [
      INSTALL_ENTRY,
      `--home=${home}`,
      ...args,
    ],
  },);
}

/**
 Creates a disposable home with the package installed into it.

 @returns the home folder, removed when the caller's scope ends
 */
async function installedHome() {
  /**
   The home folder the entry installs into.
   */
  const home = await disposableHome();
  /**
   What the install printed.
   */
  const result = await runInstall({ home: home.path, },);
  if (result.exitCode !== 0) {
    throw new Error(`the install failed before the test could check it: ${result.stderr}`,);
  }
  return home;
}

await describe({
  name: 'the install entry',
  children: [
    //region Installing
    it({
      name: 'installs every file the README names as a regular copy of its source, and exits 0',
      fn: async function installsEveryFile() {
        await using home = await disposableHome();
        /**
         What the install printed.
         */
        const result = await runInstall({ home: home.path, },);
        expect(result.exitCode,).toBe(0,);
        expect(result.problem,).toBeUndefined();
        await Promise.all(INSTALLED_FILES.map(async function installedCopy(file,) {
          /**
           Installed copy of this file.
           */
          const installed = join(
            home.path,
            file.target,
          );
          /**
           Installed copy's metadata, its text and its source's text, read together.
           */
          const [stats, installedText, sourceText,] = await Promise.all([
            lstat(installed,),
            readFile(installed, 'utf8',),
            readFile(join(
              PACKAGE_ROOT,
              file.source,
            ), 'utf8',),
          ],);
          expect(stats.isFile(),).toBe(true,);
          expect(stats.isSymbolicLink(),).toBe(false,);
          expect(result.stdout,).toContain(`installed ${installed}`,);
          expect(installedText,).toBe(sourceText,);
        },),);
      },
    },),
    //endregion Installing

    //region Checking
    it({
      name: 'exits 0 and says every copy matches when the install is current',
      fn: async function currentInstall() {
        await using home = await installedHome();
        /**
         What the check printed.
         */
        const result = await runInstall({
          home: home.path,
          args: ['--check',],
        },);
        expect(result.exitCode,).toBe(0,);
        expect(result.stdout,).toContain(`every installed copy under ${home.path} matches this package`,);
      },
    },),
    it({
      name: 'exits 1 and names a copy that drifted, instead of printing the usage line',
      fn: async function driftedCopy() {
        await using home = await installedHome();
        /**
         Installed sync unit, edited after the install.
         */
        const drifted = join(
          home.path,
          '.config',
          'systemd',
          'user',
          'rclone-bisync-garage.service',
        );
        await writeFile(drifted, `${await readFile(drifted, 'utf8',)}\n# local edit\n`,);
        /**
         What the check printed.
         */
        const result = await runInstall({
          home: home.path,
          args: ['--check',],
        },);
        expect(result.exitCode,).toBe(1,);
        expect(result.stderr,).toContain(`differs: ${drifted}`,);
        expect(`${result.stdout}${result.stderr}`,).not.toContain('usage:',);
      },
    },),
    it({
      name: 'exits 1 and names a copy it cannot read, instead of printing the usage line',
      skip: RUNS_AS_ROOT ? 'root ignores file modes, so chmod cannot make an installed copy unreadable' : false,
      fn: async function unreadableCopy() {
        await using home = await installedHome();
        /**
         Installed watcher config, made unreadable.
         */
        const unreadable = join(
          home.path,
          '.config',
          'directory-watcher',
          'garage-plain.yml',
        );
        await chmod(unreadable, 0o000,);
        /**
         What the check printed.
         */
        const result = await runInstall({
          home: home.path,
          args: ['--check',],
        },);
        expect(result.exitCode,).toBe(1,);
        expect(result.stderr,).toContain(`unreadable: ${unreadable}`,);
        expect(`${result.stdout}${result.stderr}`,).not.toContain('usage:',);
      },
    },),
    it({
      name: 'exits 2 and prints the usage line for an option it cannot use',
      fn: async function unusableOption() {
        await using home = await disposableHome();
        /**
         What the entry printed.
         */
        const result = await runInstall({
          home: home.path,
          args: ['--no-such-option',],
        },);
        expect(result.exitCode,).toBe(2,);
        expect(result.stderr,).toContain('unknown option "--no-such-option"',);
        expect(result.stderr,).toContain('usage: node ',);
      },
    },),
    //endregion Checking
  ],
},);
