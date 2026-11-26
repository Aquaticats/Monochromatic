/**
 Copies this package's units and watcher config into the user's config folders, and
 reports an installed copy that drifted.

 The user manager reads a unit while loading it, and a copy stays loadable whatever
 happens to the checkout, so every installed file is a regular copy and never a link
 into the repository. The check names each file that is missing, is a link, or differs.

 @module
 */

import type {
  Dirent,
  Stats,
} from 'node:fs';
import {
  copyFile,
  lstat,
  mkdir,
  readdir,
  readFile,
  readlink,
  rename,
} from 'node:fs/promises';
import {
  dirname,
  join,
  relative,
} from 'node:path';
import { UNIT_NAME_PREFIX, } from './units.ts';

/**
 Folder in the package that holds the unit sources.
 */
const UNIT_SOURCE_FOLDER = 'systemd';

/**
 Folder in the user's config that holds the installed units.
 */
const UNIT_TARGET_FOLDER = join(
  '.config',
  'systemd',
  'user',
);

/**
 Watcher config source, relative to the package folder.
 */
const WATCHER_CONFIG_SOURCE = join(
  'config',
  'garage-plain.yml',
);

/**
 Watcher config target, relative to the user's home.
 */
const WATCHER_CONFIG_TARGET = join(
  '.config',
  'directory-watcher',
  'garage-plain.yml',
);

/**
 One file to install: where it comes from and where it goes.

 Both paths are absolute.
 */
export type PlannedCopy = {
  readonly source: string;
  readonly target: string;
};

/**
 One installed file that is not a copy of its source.
 */
export type InstalledProblem = {
  readonly target: string;
  readonly problem: string;
};

/**
 Metadata of one installed file, or the reason it has none.

 @param target - absolute path of the installed file

 @returns the file's own metadata, or `missing` when nothing is there
 */
async function readInstalledMetadata(target: string,): Promise<{
  readonly entry?: Stats;
  readonly problem?: string;
}> {
  try {
    return { entry: await lstat(target,), };
  } catch (error) {
    return { problem: String(error,), };
  }
}

/**
 Lists the files under a folder, one level of subfolders deep.

 The units sit in the folder itself and in its `<unit>.d/` drop-in folders, and nothing
 deeper, so two levels are enough and a stray deep folder is not silently installed.

 @param folder - absolute folder to list

 @returns absolute paths of the files found, in directory order

 @example
 ```ts
 await listFiles({ folder: '/repo/package/garage-file-sync/linux-client/systemd', },);
 ```
 */
export async function listFiles({ folder, }: Readonly<{ folder: string; }>,): Promise<readonly string[]> {
  /**
   Entries of the top folder.
   */
  const entries = await readdir(
    folder,
    { withFileTypes: true, },
  );
  /**
   Files that sit in the folder itself.
   */
  const ownFiles = entries
    .filter(function isFile(entry,) {
      return entry.isFile();
    },)
    .map(function ownPath(entry,) {
      return join(
        folder,
        entry.name,
      );
    },);
  /**
   Files of each drop-in folder, read in parallel.
   */
  const dropInFiles = await Promise.all(entries
    .filter(function isFolder(entry,) {
      return entry.isDirectory();
    },)
    .map(async function dropInPaths(entry,): Promise<readonly string[]> {
      /**
       Entries of this drop-in folder.
       */
      const nestedEntries = await readdir(
        join(
          folder,
          entry.name,
        ),
        { withFileTypes: true, },
      );
      return nestedEntries
        .filter(function isNestedFile(nested,) {
          return nested.isFile();
        },)
        .map(function nestedPath(nested,) {
          return join(
            folder,
            entry.name,
            nested.name,
          );
        },);
    },));
  return [
    ...ownFiles,
    ...dropInFiles.flat(),
  ];
}

/**
 Plans every copy the install makes.

 @param sourceRoot - the package folder that holds `systemd/` and `config/`

 @param home - home folder to install into

 @returns one source and target per installed file

 @example
 ```ts
 const copies = await planCopies({ sourceRoot: packageRoot, home: '/home/example', },);
 copies[0]; // { source: '.../systemd/rclone-bisync-garage.service', target: '.../.config/systemd/user/...' }
 ```
 */
export async function planCopies({
  sourceRoot,
  home,
}: Readonly<{
  sourceRoot: string;
  home: string;
}>): Promise<readonly PlannedCopy[]> {
  /**
   Absolute folder that holds the unit sources.
   */
  const unitSourceFolder = join(
    sourceRoot,
    UNIT_SOURCE_FOLDER,
  );
  /**
   Unit sources, drop-ins included.
   */
  const unitSources = await listFiles({ folder: unitSourceFolder, },);
  return [
    ...unitSources.map(function planUnit(source,) {
      return {
        source,
        target: join(
          home,
          UNIT_TARGET_FOLDER,
          relative(
            unitSourceFolder,
            source,
          ),
        ),
      };
    },),
    {
      source: join(
        sourceRoot,
        WATCHER_CONFIG_SOURCE,
      ),
      target: join(
        home,
        WATCHER_CONFIG_TARGET,
      ),
    },
  ];
}

/**
 Copies every planned file into place.

 Each file is written beside its target and renamed over it, so a copy is either the
 old file or the new one and never half of either. A rename replaces a link with a
 regular file instead of writing through it.

 @param sourceRoot - the package folder that holds `systemd/` and `config/`

 @param home - home folder to install into

 @returns the targets written, in plan order

 @example
 ```ts
 await installFiles({ sourceRoot: packageRoot, home: '/home/example', },);
 ```
 */
export async function installFiles({
  sourceRoot,
  home,
}: Readonly<{
  sourceRoot: string;
  home: string;
}>): Promise<readonly string[]> {
  /**
   The copies this install makes.
   */
  const copies = await planCopies({
    sourceRoot,
    home,
  },);
  return await Promise.all(copies.map(async function installCopy(copy,) {
    await mkdir(
      dirname(copy.target,),
      { recursive: true, },
    );
    /**
     Temporary copy beside the target, renamed over it once complete.
     */
    const staged = `${copy.target}.new`;
    await copyFile(
      copy.source,
      staged,
    );
    await rename(
      staged,
      copy.target,
    );
    return copy.target;
  },),);
}

/**
 Reads one installed file's text.

 @param target - absolute path of the installed file

 @returns the text, or the reason it could not be read
 */
async function readInstalledText(target: string,): Promise<{
  readonly text?: string;
  readonly problem?: string;
}> {
  try {
    return {
      text: await readFile(
        target,
        'utf8',
      ),
    };
  } catch (error) {
    return { problem: String(error,), };
  }
}

/**
 Entries of one folder.

 @param folder - absolute folder to list

 @returns the entries, or none when the folder is absent

 @example
 ```ts
 await entriesIn('/home/example/.config/systemd/user',);
 ```
 */
async function entriesIn(folder: string,): Promise<readonly Dirent[]> {
  /**
   Folder's own metadata, absent when nothing is there at all.
   */
  const {entry} = await readInstalledMetadata(folder,);
  if (entry === undefined) return [];
  if (!entry.isDirectory()) return [];
  return await readdir(
    folder,
    { withFileTypes: true, },
  );
}

/**
 Reports the files the user manager loads although this package does not ship them.

 The user manager loads every unit file whose name matches this package's prefix, and
 every file in a drop-in folder of a unit it ships, whatever the install wrote; a unit
 file the package stopped shipping, or a drop-in somebody added, keeps acting beside the
 current ones. Only the unit folder is searched: the watcher config is never loaded by
 name, and a drop-in folder of a unit the package does not ship has no unit to act on.

 @param copies - the copies this package would make

 @param home - home folder the installed files belong to

 @returns one problem per file this package does not ship, empty when it ships none
 */
async function unshippedProblems({
  copies,
  home,
}: Readonly<{
  copies: readonly PlannedCopy[];
  home: string;
}>): Promise<readonly InstalledProblem[]> {
  /**
   Folder that holds the installed units and their drop-in folders.
   */
  const unitFolder = join(
    home,
    UNIT_TARGET_FOLDER,
  );
  /**
   Targets this package installs, so any other file of the folder is not one of them.
   */
  const planned = new Set(copies.map(function targetPath(copy,) {
    return copy.target;
  },),);
  /**
   Entries of the unit folder itself.
   */
  const entries = await entriesIn(unitFolder,);
  /**
   Unit files in the folder itself, whose name this package owns.
   */
  const staleUnits = entries
    .filter(function isPackageUnitFile(entry,) {
      /**
       Name of this entry inside the unit folder.
       */
      const {name} = entry;
      return (!entry.isDirectory()) && name.startsWith(UNIT_NAME_PREFIX,);
    },)
    .map(function unitPath(entry,) {
      return join(
        unitFolder,
        entry.name,
      );
    },)
    .filter(function unplanned(path,) {
      return !planned.has(path,);
    },)
    .map(function staleUnitProblem(path,) {
      return {
        target: path,
        problem: `not shipped: ${path} is a unit file this package does not ship;`
          + ' the user manager still loads it, so remove it or ship it',
      };
    },);
  /**
   Drop-in folders of the units this package ships, which the user manager reads.
   */
  const dropInFolders = copies
    .map(function targetPath(copy,) {
      return copy.target;
    },)
    .filter(function isUnitFile(path,) {
      return dirname(path,) === unitFolder;
    },)
    .map(function dropInPath(path,) {
      return `${path}.d`;
    },);
  /**
   Extra files of every drop-in folder, read in parallel.
   */
  const extraDropIns = await Promise.all(dropInFolders.map(async function extraInFolder(folder,): Promise<readonly InstalledProblem[]> {
    return (await entriesIn(folder,))
      .filter(function isDropInFile(entry,) {
        return !entry.isDirectory();
      },)
      .map(function dropInPath(entry,) {
        return join(
          folder,
          entry.name,
        );
      },)
      .filter(function unplanned(path,) {
        return !planned.has(path,);
      },)
      .map(function extraDropInProblem(path,) {
        return {
          target: path,
          problem: `extra drop-in: ${path} sits in ${folder}, a drop-in folder of a unit this package ships,`
            + ' but this package does not ship it; the user manager still loads it, so remove it',
        };
      },);
  },),);
  return [
    ...staleUnits,
    ...extraDropIns.flat(),
  ];
}

/**
 Checks every planned file against the installed copy.

 @param sourceRoot - the package folder that holds `systemd/` and `config/`

 @param home - home folder the installed copies belong to

 @returns one problem per installed file that is missing, is a link, cannot be read or
   differs, and one per unit file or drop-in this package does not ship; empty when
   every copy matches and the user manager loads nothing else

 @example
 ```ts
 const problems = await checkInstalled({ sourceRoot: packageRoot, home, },);
 problems.length; // 0 when the install is current
 ```
 */
export async function checkInstalled({
  sourceRoot,
  home,
}: Readonly<{
  sourceRoot: string;
  home: string;
}>): Promise<readonly InstalledProblem[]> {
  /**
   The copies the install would make.
   */
  const copies = await planCopies({
    sourceRoot,
    home,
  },);
  /**
   One entry per copy: the problem it reported, or nothing when it matches.
   */
  const reported = await Promise.all(copies.map(async function inspectCopy(copy,): Promise<readonly InstalledProblem[]> {
    /**
     Installed file's own metadata, or the reason it could not be read.
     */
    const metadata = await readInstalledMetadata(copy.target,);
    /**
     Installed file's own metadata, absent when nothing is there.
     */
    const stats = metadata.entry;
    if (stats === undefined) {
      return [{
        target: copy.target,
        problem: `missing: ${copy.target} was never installed; run mise run //package/garage-file-sync/linux-client:install`,
      },];
    }
    if (stats.isSymbolicLink()) {
      return [{
        target: copy.target,
        problem: `is a link: ${copy.target} -> ${await readlink(copy.target,)};`
          + ' the user manager would load nothing once the checkout is away, so install a copy',
      },];
    }
    if (!stats.isFile()) {
      return [{
        target: copy.target,
        problem: `is not a regular file: ${copy.target}; install a copy of ${copy.source}`,
      },];
    }
    /**
     Source file's own metadata, which a hard link into the checkout shares.
     */
    const sourceStats = await lstat(copy.source,);
    if ((stats.dev === sourceStats.dev) && (stats.ino === sourceStats.ino)) {
      return [{
        target: copy.target,
        problem: `is a link: ${copy.target} and ${copy.source} are one file, a hard link;`
          + ' the user manager would load nothing once the checkout is away, so install a copy',
      },];
    }
    /**
     Installed text, or the reason it could not be read.
     */
    const installed = await readInstalledText(copy.target,);
    if (installed.problem !== undefined) {
      return [{
        target: copy.target,
        problem: `unreadable: ${copy.target} could not be read, so its text cannot be compared with ${copy.source}: ${installed.problem};`
          + ' install a copy of it again',
      },];
    }
    /**
     Source's current text.
     */
    const sourceText = await readFile(
      copy.source,
      'utf8',
    );
    if (installed.text !== sourceText) {
      return [{
        target: copy.target,
        problem: `differs: ${copy.target} is not the current text of ${copy.source}; install it again`,
      },];
    }
    return [];
  },),);
  return [
    ...reported.flat(),
    ...await unshippedProblems({
      copies,
      home,
    },),
  ];
}
