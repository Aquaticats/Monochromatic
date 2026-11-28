/**
 Puts a build's output in place by renaming it there, never by deleting it first.

 Rolldown's `output.cleanDir` deletes every file of the output directory at the start of its
 write phase, and writes each one again after the render phase,
 so every path under that directory is absent for the whole phase
 (issue 6668:
 an `mvm` started in that window ended with
 `Error: Cannot find module '<checkout>/package/mvm/cli/dist/final/node/cli.mjs'`,
 and `package/mvm/cli/dist/final/node/cli.mjs` is what the `node_modules/.bin/mvm` shim an
 older install wrote runs).
 A file written in place also shows a reader its first bytes before its last ones.

 This plugin takes the cleaning away from the bundler and does the replacement itself:
 the build writes into a staging directory beside the output directory,
 and one rename per file moves each into place, so the name a reader opens always resolves
 to one whole file,
 either the previous build's or this build's.
 What no build emits any more is removed after the new files are in place,
 which keeps the end state `cleanDir` produced:
 the consumer turns that removal off the same way it turned `cleanDir` off.

 @module
 */

import {
  mkdir,
  readdir,
  rename,
  rm,
  rmdir,
  unlink,
} from 'node:fs/promises';
import {
  basename,
  dirname,
  join,
} from 'node:path';

import type {
  NormalizedOutputOptions,
  OutputBundle,
  Plugin,
} from 'rolldown';

//region The staging directory

/**
 Name this plugin carries, which a test or a consumer names when it looks for the plugin.
 */
export const ATOMIC_OUTPUT_DIRECTORY_PLUGIN_NAME = 'monochromatic:atomic-output-directory';

/**
 Words appended to the output directory's own name to name the staging directory.

 The staging directory is the output directory's sibling, because a rename only replaces a
 name without a moment of absence inside one filesystem, and a sibling always shares one.
 */
const STAGING_DIRECTORY_SUFFIX = '.building';

/**
 The directory a build writes into before its files are renamed into the output directory.

 The name is derived from the output directory alone, so a build that ended before its write
 phase leaves a directory the next build of the same package clears (`buildStart`),
 and no such directory accumulates.

 @param finalDirectory - Directory the build's files are reachable at when it settles

 @returns Sibling staging directory, hidden by its leading dot

 @example
 ```ts
 stagingDirectoryFor({ finalDirectory: '/repo/package/x/dist/final/node' });
 // '/repo/package/x/dist/final/.node.building'
 ```
 */
export function stagingDirectoryFor({ finalDirectory, }: {
  readonly finalDirectory: string;
},): string {
  return join(
    dirname(finalDirectory,),
    `.${basename(finalDirectory,)}${STAGING_DIRECTORY_SUFFIX}`,
  );
}

//endregion The staging directory

//region The emitted bundle

/**
 Fields of one emitted entry this plugin reads.
 */
type EmittedEntry = {
  readonly type?: string;
  readonly isEntry?: boolean;
};

/**
 The emitted bundle, keyed by output-directory-relative file name.
 */
type EmittedBundle = Readonly<Record<string, EmittedEntry>>;

/**
 Names of the chunks that are build entries rather than shared chunks.

 @param bundle - The emitted bundle

 @returns File names a reader may import directly

 @example
 ```ts
 entryFileNames({ bundle: { 'cli.mjs': { type: 'chunk', isEntry: true } } });
 ```
 */
function entryFileNames({ bundle, }: {
  readonly bundle: EmittedBundle;
},): ReadonlySet<string> {
  return new Set(Object.entries(bundle,)
    .filter(function isEntryChunk([, entry,]: readonly [string, EmittedEntry,],): boolean {
      return (entry.type === 'chunk') && (entry.isEntry === true);
    },)
    .map(function nameOf([fileName,]: readonly [string, EmittedEntry,],): string {
      return fileName;
    },));
}

/**
 Move every emitted file from the staging directory into the output directory.

 Shared chunks and assets go first and entry chunks last,
 so a reader that opens an entry while the move runs finds the files its imports name already
 in place.

 @param stagingDirectory - Directory the bundler wrote into

 @param finalDirectory - Directory the files belong at

 @param bundle - The emitted bundle, which names the files and says which are entries

 @returns Every output-directory-relative name the build moved

 @throws Error when a staged file cannot be renamed into place, leaving the output directory
 as it was for the files not yet moved
 */
async function moveStagedFiles({
  stagingDirectory,
  finalDirectory,
  bundle,
}: {
  readonly stagingDirectory: string;
  readonly finalDirectory: string;
  readonly bundle: EmittedBundle;
},): Promise<readonly string[]> {
  /**
   Every file name this build emitted.
   */
  const fileNames = Object.keys(bundle,);
  /**
   File names whose chunk a reader imports directly, moved once their imports exist.
   */
  const entries = entryFileNames({ bundle, },);
  /**
   Emitted names in the order the move takes them.
   */
  const orderedNames = [
    ...fileNames.filter(function isNotEntry(fileName: string,): boolean {
      return !entries.has(fileName,);
    },),
    ...fileNames.filter(function isEntry(fileName: string,): boolean {
      return entries.has(fileName,);
    },),
  ];
  await mkdir(finalDirectory, { recursive: true, },);
  for (const fileName of orderedNames) {
    /**
     Path the file belongs at.
     */
    const target = join(finalDirectory, fileName,);
    await mkdir(dirname(target,), { recursive: true, },);
    await rename(join(stagingDirectory, fileName,), target,);
  }
  return orderedNames;
}

//endregion The emitted bundle

//region Removing what this build did not emit

/**
 One directory's files and directories, named relative to the directory read.
 */
type DirectoryContents = {
  readonly files: string[];
  readonly directories: string[];
};

/**
 Whether a caught value is a Node error carrying a system error code.

 @param error - Caught value (anything; `try` lifts to `unknown`)

 @returns Whether `error` is an Error with a `code` property

 @example
 ```ts
 isErrnoException(Object.assign(new Error('x'), { code: 'ENOTEMPTY' })); // true
 ```
 */
function isErrnoException(error: unknown,): error is NodeJS.ErrnoException {
  return (Error.isError(error,)) && ('code' in error);
}

/**
 List one directory's files and directories, recursively, naming each below `prefix`.

 @param directory - Directory to read

 @param prefix - Name the entries are relative to, empty for the directory itself

 @returns File names and directory names, each output-directory relative

 @example
 ```ts
 await listDirectoryContents({ directory: '/repo/package/x/dist/final/node' });
 ```
 */
async function listDirectoryContents({
  directory,
  prefix = '',
}: {
  readonly directory: string;
  readonly prefix?: string;
},): Promise<DirectoryContents> {
  /**
   Entries of this directory, files and directories kept apart.
   */
  const entries = await readdir(directory, { withFileTypes: true, },);
  /**
   Files found so far, named relative to `prefix`.
   */
  const files: string[] = [];
  /**
   Directories found so far, named relative to `prefix`.
   */
  const directories: string[] = [];
  for (const entry of entries) {
    /**
     Name this entry has relative to the directory the walk started at.
     */
    const relativeName = prefix === '' ? entry.name : `${prefix}/${entry.name}`;
    if (entry.isDirectory()) {
      /**
       Contents of this subdirectory.
       */
      const nested = await listDirectoryContents({
        directory: join(directory, entry.name,),
        prefix: relativeName,
      },);
      directories.push(relativeName, ...nested.directories,);
      files.push(...nested.files,);
    }
    else {
      files.push(relativeName,);
    }
  }
  return {
    files,
    directories,
  };
}

/**
 Remove every file of the output directory this build did not emit, then its empty directories.

 This is what `cleanDir` ended with: the output directory holds this build's files only.

 @param finalDirectory - Output directory the build moved its files into

 @param keptFileNames - Output-directory-relative names the build emitted

 @returns Nothing; what the build did not emit is gone when it settles

 @example
 ```ts
 await removeFilesThisBuildDidNotEmit({ finalDirectory, keptFileNames: new Set(['cli.mjs']) });
 ```
 */
async function removeFilesThisBuildDidNotEmit({
  finalDirectory,
  keptFileNames,
}: {
  readonly finalDirectory: string;
  readonly keptFileNames: ReadonlySet<string>;
},): Promise<void> {
  /**
   Every file and directory the output directory holds after the move.
   */
  const contents = await listDirectoryContents({ directory: finalDirectory, },);
  for (const fileName of contents.files) {
    if (keptFileNames.has(fileName,)) {
      continue;
    }
    await unlink(join(finalDirectory, fileName,),);
  }
  // Deepest first, so a directory a previous build left holding nothing goes after the
  // directories inside it.
  for (const directoryName of contents.directories.toSorted(function byDepth(one: string, another: string,): number {
    return another.split('/',).length - one.split('/',).length;
  },)) {
    try {
      await rmdir(join(finalDirectory, directoryName,),);
    }
    catch (error) {
      // A directory this build still emits into, or one a reader removed meanwhile, stays.
      if (!isErrnoException(error,)
        || ((error.code !== 'ENOTEMPTY') && (error.code !== 'ENOENT'))) {
        throw error;
      }
    }
  }
}

//endregion Removing what this build did not emit

//region The plugin

/**
 Rolldown plugin that writes nothing over an existing output file and deletes nothing before it
 has the file to put there.

 The build's `output.dir` points at {@link stagingDirectoryFor}; this plugin renames each
 emitted file into the output directory in `writeBundle`, so every path the directory holds
 resolves to one whole file at every instant of a build.

 @param finalDirectory - Directory the build's files must be reachable at when it settles

 @param removeStaleFiles - Whether files this build did not emit are removed afterwards, which a
 consumer asked for by leaving the output directory cleaning on

 @returns Rolldown plugin staging the build's output and renaming it into place

 @example
 ```ts
 plugins: [atomicOutputDirectoryPlugin({ finalDirectory: '/repo/package/x/dist/final/node', removeStaleFiles: true })];
 ```
 */
export function atomicOutputDirectoryPlugin({
  finalDirectory,
  removeStaleFiles,
}: {
  readonly finalDirectory: string;
  readonly removeStaleFiles: boolean;
},): Plugin {
  /**
   Directory the bundler writes this build's files into.
   */
  const stagingDirectory = stagingDirectoryFor({ finalDirectory, },);
  return {
    name: ATOMIC_OUTPUT_DIRECTORY_PLUGIN_NAME,

    /**
     Clear what an earlier build left staged and make the staging directory again.
     */
    async buildStart(): Promise<void> {
      await rm(stagingDirectory, {
        force: true,
        recursive: true,
      },);
      await mkdir(stagingDirectory, { recursive: true, },);
    },

    writeBundle: {
      // `writeBundle` hooks run as one parallel batch, and the shared executable-bit plugin
      // chmods a staged file in its own hook; a rename that raced it would leave that file
      // without the bit or fail the build. The `post` group runs once the plain hooks have
      // settled, which makes this rename the build's final act of writing.
      order: 'post',

      /**
       Rename this build's files into the output directory.

       @param bundle - The emitted bundle, which names the files the move takes

       @throws Error when a staged file cannot be renamed into place
       */
      async handler(
        _options: NormalizedOutputOptions,
        bundle: OutputBundle,
      ): Promise<void> {
        /**
         Names this build moved into the output directory.
         */
        const movedFileNames = await moveStagedFiles({
          stagingDirectory,
          finalDirectory,
          bundle,
        },);
        if (removeStaleFiles) {
          await removeFilesThisBuildDidNotEmit({
            finalDirectory,
            keptFileNames: new Set(movedFileNames,),
          },);
        }
        await rm(stagingDirectory, {
          force: true,
          recursive: true,
        },);
      },
    },
  };
}

//endregion The plugin
