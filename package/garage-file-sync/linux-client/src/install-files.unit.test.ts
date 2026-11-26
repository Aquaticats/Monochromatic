import {
  mkdtempDisposable,
  readFile,
  rm,
  symlink,
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
  checkInstalled,
  installFiles,
  planCopies,
} from './install-files.ts';

/**
 The package folder whose `systemd/` and `config/` folders the install copies.
 */
const PACKAGE_ROOT = join(import.meta.dirname, '..',);

/**
 Unit file the install must place in the user manager's folder.
 */
const SYNC_UNIT_TARGET = join('.config', 'systemd', 'user', 'rclone-bisync-garage.service',);

/**
 Drop-in the install must place beside the unit.
 */
const DROP_IN_TARGET = join(
  '.config',
  'systemd',
  'user',
  'rclone-bisync-garage.service.d',
  '60-alert-on-failure.conf',
);

/**
 Watcher config the install must place in the watcher's folder.
 */
const WATCHER_CONFIG_TARGET = join('.config', 'directory-watcher', 'garage-plain.yml',);

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
            expect(targets,).toContain(join(home.path, '.config', 'systemd', 'user', 'rclone-bisync-garage-watch.service',),);
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
              problems: (await checkInstalled({ sourceRoot: PACKAGE_ROOT, home: home.path, },))
                .map(function problemText(problem,) {
                  return problem.problem;
                },),
              needle: `differs: ${drifted}`,
            },),).toBe(true,);
            /**
             A copy replaced by a link into the checkout.
             */
            const linked = join(home.path, WATCHER_CONFIG_TARGET,);
            await rm(linked,);
            await symlink(join(PACKAGE_ROOT, 'config', 'garage-plain.yml',), linked,);
            expect(mentions({
              problems: (await checkInstalled({ sourceRoot: PACKAGE_ROOT, home: home.path, },))
                .map(function problemText(problem,) {
                  return problem.problem;
                },),
              needle: `is a link: ${linked}`,
            },),).toBe(true,);
            /**
             A copy that was never installed.
             */
            await rm(linked,);
            expect(mentions({
              problems: (await checkInstalled({ sourceRoot: PACKAGE_ROOT, home: home.path, },))
                .map(function problemText(problem,) {
                  return problem.problem;
                },),
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
  ],
},);
