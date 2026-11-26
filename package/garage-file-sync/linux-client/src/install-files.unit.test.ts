import {
  chmod,
  link,
  mkdir,
  mkdtempDisposable,
  readFile,
  rm,
  symlink,
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
  checkInstalled,
  installFiles,
  planCopies,
} from './install-files.ts';

/**
 The package folder whose `systemd/` and `config/` folders the install copies.
 */
const PACKAGE_ROOT = join(import.meta.dirname, '..',);

/**
 Folder in the user's config that holds the installed units, relative to the home folder.
 */
const UNIT_TARGET_FOLDER = join(
  '.config',
  'systemd',
  'user',
);

/**
 The sync unit's name, and so its name in the package and in the user manager's folder.
 */
const SYNC_UNIT_NAME = 'rclone-bisync-garage.service';

/**
 The watcher unit's name, which the plan must place beside the sync unit.
 */
const WATCH_UNIT_NAME = 'rclone-bisync-garage-watch.service';

/**
 Unit file the install must place in the user manager's folder.
 */
const SYNC_UNIT_TARGET = join(UNIT_TARGET_FOLDER, SYNC_UNIT_NAME,);

/**
 Drop-in the install must place beside the unit.
 */
const DROP_IN_TARGET = join(
  UNIT_TARGET_FOLDER,
  `${SYNC_UNIT_NAME}.d`,
  '60-alert-on-failure.conf',
);

/**
 Watcher config the install must place in the watcher's folder.
 */
const WATCHER_CONFIG_TARGET = join('.config', 'directory-watcher', 'garage-plain.yml',);

/**
 Whether the tests run as root, which ignores file modes and so cannot make an installed
 copy unreadable with `chmod`.
 */
const RUNS_AS_ROOT = process.getuid?.() === 0;

/**
 Whether any reported problem names a given text.

 @param problems - texts the check reported

 @param needle - text one of them should contain

 @returns true when one problem mentions the text
 */
function mentions({
  problems,
  needle,
}: Readonly<{
  problems: readonly string[];
  needle: string;
}>): boolean {
  return problems.some(function includes(problem,) {
    return problem.includes(needle,);
  },);
}

/**
 Creates a disposable home for one install.

 @returns the home folder, removed when the caller's scope ends
 */
async function disposableHome() {
  return await mkdtempDisposable(join(scratchRoot(), 'garage-install-',),);
}

/**
 Reads the problems the check reports for one home.

 @param home - home folder the installed files belong to

 @param sourceRoot - package folder the copies come from, the real one unless a test
   needs sources on the fixture's own device

 @returns the problem texts
 */
async function problemsIn({
  home,
  sourceRoot = PACKAGE_ROOT,
}: Readonly<{
  home: string;
  sourceRoot?: string;
}>): Promise<readonly string[]> {
  return (await checkInstalled({
    sourceRoot,
    home,
  },))
    .map(function problemText(problem,) {
      return problem.problem;
    },);
}

/**
 Creates a disposable package folder holding one unit and the watcher config.

 A hard link cannot cross a device, and the real package folder sits on another device
 than the fixture root, so a test that links a source into a home needs its own sources.

 @returns the package folder, removed when the caller's scope ends
 */
async function disposablePackage() {
  /**
   The package folder the check reads its sources from.
   */
  const sourceRoot = await mkdtempDisposable(join(scratchRoot(), 'garage-install-source-',),);
  await mkdir(join(
    sourceRoot.path,
    'systemd',
  ), { recursive: true, },);
  await writeFile(join(
    sourceRoot.path,
    'systemd',
    SYNC_UNIT_NAME,
  ), '[Service]\nExecStart=/bin/true\n',);
  await mkdir(join(
    sourceRoot.path,
    'config',
  ), { recursive: true, },);
  await writeFile(join(
    sourceRoot.path,
    'config',
    'garage-plain.yml',
  ), 'path: Plain\n',);
  return sourceRoot;
}

await describe({
  name: 'install and check',
  children: [
    describe({
      name: planCopies.name,
      children: [
        it({
          name: 'plans every unit, its drop-ins, and the watcher config',
          fn: async function plansEveryFile() {
            /**
             A disposable home to plan against.
             */
            await using home = await disposableHome();
            /**
             The planned copies.
             */
            const copies = await planCopies({ sourceRoot: PACKAGE_ROOT, home: home.path, },);
            /**
             Targets, in plan order.
             */
            const targets = copies.map(function target(copy,) {
              return copy.target;
            },);
            expect(targets,).toContain(join(home.path, SYNC_UNIT_TARGET,),);
            expect(targets,).toContain(join(home.path, DROP_IN_TARGET,),);
            expect(targets,).toContain(join(home.path, WATCHER_CONFIG_TARGET,),);
            expect(targets,).toContain(join(
              home.path,
              UNIT_TARGET_FOLDER,
              WATCH_UNIT_NAME,
            ),);
          },
        },),
      ],
    },),
    describe({
      name: installFiles.name,
      children: [
        it({
          name: 'writes regular copies that check accepts, and check reports a drifted, linked or missing copy by name',
          fn: async function installThenDrift() {
            /**
             A disposable home to install into.
             */
            await using home = await disposableHome();
            /**
             Targets written by the install.
             */
            const written = await installFiles({ sourceRoot: PACKAGE_ROOT, home: home.path, },);
            expect(written.length,).toBeGreaterThan(2,);
            expect(await checkInstalled({ sourceRoot: PACKAGE_ROOT, home: home.path, },),).toEqual([],);
            /**
             A copy that was edited after the install.
             */
            const drifted = join(home.path, SYNC_UNIT_TARGET,);
            await writeFile(drifted, `${await readFile(drifted, 'utf8',)}\n# local edit\n`,);
            expect(mentions({
              problems: await problemsIn({ home: home.path, },),
              needle: `differs: ${drifted}`,
            },),).toBe(true,);
            /**
             A copy replaced by a link into the checkout.
             */
            const linked = join(home.path, WATCHER_CONFIG_TARGET,);
            await rm(linked,);
            await symlink(join(PACKAGE_ROOT, 'config', 'garage-plain.yml',), linked,);
            expect(mentions({
              problems: await problemsIn({ home: home.path, },),
              needle: `is a link: ${linked}`,
            },),).toBe(true,);
            /**
             A copy that was never installed.
             */
            await rm(linked,);
            expect(mentions({
              problems: await problemsIn({ home: home.path, },),
              needle: `missing: ${linked}`,
            },),).toBe(true,);
          },
        },),
        it({
          name: 'replaces a link with a regular file instead of writing through it',
          fn: async function installReplacesLink() {
            /**
             A disposable home whose installed unit is a link to a decoy.
             */
            await using home = await disposableHome();
            await installFiles({ sourceRoot: PACKAGE_ROOT, home: home.path, },);
            /**
             Installed sync unit, removed and replaced by a link.
             */
            const installed = join(home.path, SYNC_UNIT_TARGET,);
            /**
             Decoy file the link points at.
             */
            const decoy = join(home.path, 'decoy.service',);
            await writeFile(decoy, 'decoy\n',);
            await rm(installed,);
            await symlink(decoy, installed,);
            await installFiles({ sourceRoot: PACKAGE_ROOT, home: home.path, },);
            expect(await readFile(decoy, 'utf8',),).toBe('decoy\n',);
            expect(await checkInstalled({ sourceRoot: PACKAGE_ROOT, home: home.path, },),).toEqual([],);
          },
        },),
      ],
    },),
    describe({
      name: checkInstalled.name,
      children: [
        it({
          name: 'reports a unit file the package does not ship, because the user manager still loads it',
          fn: async function unshippedUnitFile() {
            await using home = await disposableHome();
            await installFiles({ sourceRoot: PACKAGE_ROOT, home: home.path, },);
            /**
             The stale unit file, which is the only file of that name the manager can load.
             */
            const stale = join(
              home.path,
              UNIT_TARGET_FOLDER,
              'rclone-bisync-garage-old.service',
            );
            await writeFile(stale, '[Service]\nExecStart=/bin/true\n',);
            /**
             A drop-in folder beside that stale unit, which has no unit of this package to
             act on.
             */
            const staleDropIn = join(
              home.path,
              UNIT_TARGET_FOLDER,
              'rclone-bisync-garage-old.service.d',
              '70-local.conf',
            );
            await mkdir(dirname(staleDropIn,), { recursive: true, },);
            await writeFile(staleDropIn, '[Service]\nEnvironment=OLD=1\n',);
            /**
             Every problem the check reports.
             */
            const problems = await problemsIn({ home: home.path, },);
            expect(problems,).toHaveLength(1,);
            expect(problems[0],).toContain(`not shipped: ${stale}`,);
          },
        },),
        it({
          name: 'reports extra files in a drop-in folder of a shipped unit, whether or not a drop-in ships there',
          fn: async function extraDropIn() {
            await using home = await disposableHome();
            await installFiles({ sourceRoot: PACKAGE_ROOT, home: home.path, },);
            /**
             A drop-in somebody added beside the one this package installs.
             */
            const extra = join(
              home.path,
              UNIT_TARGET_FOLDER,
              `${SYNC_UNIT_NAME}.d`,
              '70-local.conf',
            );
            await writeFile(extra, '[Service]\nEnvironment=LOCAL=1\n',);
            /**
             A drop-in in the timer's folder, which this package installs nothing into.
             */
            const unshippedDropIn = join(
              home.path,
              UNIT_TARGET_FOLDER,
              'rclone-bisync-garage.timer.d',
              '70-local.conf',
            );
            await mkdir(dirname(unshippedDropIn,), { recursive: true, },);
            await writeFile(unshippedDropIn, '[Timer]\nOnCalendar=daily\n',);
            /**
             Every problem the check reports.
             */
            const problems = await problemsIn({ home: home.path, },);
            expect(problems,).toHaveLength(2,);
            expect(mentions({
              problems,
              needle: `extra drop-in: ${extra}`,
            },),).toBe(true,);
            expect(mentions({
              problems,
              needle: `extra drop-in: ${unshippedDropIn}`,
            },),).toBe(true,);
          },
        },),
        it({
          name: 'reports an installed copy it cannot read, naming it',
          skip: RUNS_AS_ROOT ? 'root ignores file modes, so chmod cannot make a copy unreadable' : false,
          fn: async function unreadableCopy() {
            await using home = await disposableHome();
            await installFiles({ sourceRoot: PACKAGE_ROOT, home: home.path, },);
            /**
             Installed sync unit, made unreadable after the install.
             */
            const unreadable = join(home.path, SYNC_UNIT_TARGET,);
            await chmod(unreadable, 0o000,);
            /**
             Every problem the check reports.
             */
            const problems = await problemsIn({ home: home.path, },);
            expect(problems,).toHaveLength(1,);
            expect(problems[0],).toContain(`unreadable: ${unreadable}`,);
          },
        },),
        it({
          name: 'reports a hard link into the checkout as a link, and an install replaces it with a copy',
          fn: async function hardLink() {
            await using packageRoot = await disposablePackage();
            await using home = await disposableHome();
            await installFiles({
              sourceRoot: packageRoot.path,
              home: home.path,
            },);
            /**
             Unit source the home is linked to.
             */
            const source = join(
              packageRoot.path,
              'systemd',
              SYNC_UNIT_NAME,
            );
            /**
             Installed unit, removed and hard-linked to its source.
             */
            const linked = join(
              home.path,
              UNIT_TARGET_FOLDER,
              SYNC_UNIT_NAME,
            );
            await rm(linked,);
            await link(source, linked,);
            /**
             Every problem the check reports.
             */
            const problems = await problemsIn({
              home: home.path,
              sourceRoot: packageRoot.path,
            },);
            expect(problems,).toHaveLength(1,);
            expect(problems[0],).toContain(`is a link: ${linked}`,);
            expect(problems[0],).toContain('hard link',);
            await installFiles({
              sourceRoot: packageRoot.path,
              home: home.path,
            },);
            expect(await problemsIn({
              home: home.path,
              sourceRoot: packageRoot.path,
            },),).toEqual([],);
            expect(await readFile(source, 'utf8',),).toBe('[Service]\nExecStart=/bin/true\n',);
          },
        },),
        it({
          name: 'reports every copy missing, and no unshipped file, when nothing has been installed',
          fn: async function nothingInstalled() {
            await using home = await disposableHome();
            /**
             The copies the package would install.
             */
            const copies = await planCopies({ sourceRoot: PACKAGE_ROOT, home: home.path, },);
            /**
             Every problem the check reports.
             */
            const problems = await problemsIn({ home: home.path, },);
            expect(problems,).toHaveLength(copies.length,);
            expect(problems.every(function isMissing(problem,) {
              return problem.startsWith('missing: ',);
            },),).toBe(true,);
          },
        },),
        it({
          name: 'reports every copy missing, and no unshipped file, when the unit folder is a regular file',
          fn: async function unitFolderIsAFile() {
            await using home = await disposableHome();
            /**
             The user manager's folder, written as a regular file instead.
             */
            const unitFolder = join(
              home.path,
              UNIT_TARGET_FOLDER,
            );
            await mkdir(dirname(unitFolder,), { recursive: true, },);
            await writeFile(unitFolder, 'not a folder\n',);
            /**
             The copies the package would install.
             */
            const copies = await planCopies({ sourceRoot: PACKAGE_ROOT, home: home.path, },);
            /**
             Every problem the check reports.
             */
            const problems = await problemsIn({ home: home.path, },);
            expect(problems,).toHaveLength(copies.length,);
            expect(problems.every(function isMissing(problem,) {
              return problem.startsWith('missing: ',);
            },),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
