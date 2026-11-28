/**
 The `mvm` command runs the TypeScript source beside it, never a built copy of it.

 Free page reporting landed in `src/domain-xml-builders.ts` on 2026-11-25 (commit
 `269b812807`, `doc/decision/mvm-guests-report-free-pages.md`), and six guests made two
 days later by rounds in worktrees of their own were defined without it: the
 `node_modules/.bin/mvm` those rounds ran reached `dist/final/node/cli.mjs`, a build
 nothing refreshed when the source moved and every worktree inherited from the checkout
 (issue 6655). Nothing said so, and a decision recorded as shipped did not reach a guest
 for two days.

 So the manifest's `bin` names `src/cli.ts`, which is the file `mise run
 //package/mvm/cli:run` already runs, and Node strips its types on the way in. No build
 output sits between the command and its source, so no build can be older than the code a
 person's `mvm` runs.

 This file holds the control: it fails when the manifest is pointed back at a built copy,
 and it fails when the built entry a shim an older install wrote names stops leading to the
 same source file.

 @module
 */

import {
  readFile,
} from 'node:fs/promises';
import {
  dirname,
  join,
  resolve,
} from 'node:path';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import spawn, { type SubprocessError, } from 'nano-spawn';

//region The package this control reads

/**
 The package folder, which holds the manifest, the source and the build this control reads.

 This file sits in the package's `src` folder, so the package folder is one level above the
 folder holding it.
 */
const PACKAGE_FOLDER = dirname(dirname(new URL(import.meta.url,).pathname,),);

/**
 The command name the manifest's `bin` maps, which is the word a person types.
 */
const COMMAND_NAME = 'mvm';

/**
 The built entry a `node_modules/.bin/mvm` shim written by an install older than the
 manifest's `bin` names, relative to the package folder.

 `node_modules/.bin/mvm` in this checkout was written by pnpm from the `bin` field as it
 stood then, and pnpm rewrites a shim only on install, so this path is still where many a
 `mvm` command starts. The build writes a launcher there for exactly that reason.
 */
const BUILT_ENTRY_PATH = 'dist/final/node/cli.mjs';

/**
 How long one `mvm --help` may take: it prints usage and exits, and a loaded machine
 stretches a Node start-up to a few hundred milliseconds rather than minutes.
 */
const HELP_TIMEOUT_MILLISECONDS = 60_000;

/**
 The first line `mvm --help` prints, which only a started CLI prints.

 Copied from `src/cli.ts`'s `parseCommandLine` call, the `brief` it hands the parser, and
 the same line `src/cli.unit.test.ts` already expects of the built entry.
 */
const HELP_FIRST_LINE = 'mvm - ephemeral VM manager';

//endregion The package this control reads

//region Reading what the manifest declares

/**
 What this control reads of the package manifest.

 @property bin - the manifest's `bin` field, whose values are the files a command starts
 */
type PackageManifest = {
  readonly bin?: Readonly<Record<string, string>>;
};

/**
 Reads the package's manifest.

 @returns the manifest's `bin` map, which names one file per command

 @throws Error when the manifest holds no `bin` map, so the command it declares is unknown

 @example
 ```ts
 const bin = await manifestBin();
 ```
 */
async function manifestBin(): Promise<Readonly<Record<string, string>>> {
  /**
   The manifest as parsed from disk, narrowed to the field this control reads.
   */
  const manifest = JSON.parse(
    await readFile(
      join(
        PACKAGE_FOLDER,
        'package.json',
      ),
      'utf8',
    ),
  ) as PackageManifest;
  if (manifest.bin === undefined) {
    throw new Error(`${join(
      PACKAGE_FOLDER,
      'package.json',
    )} declares no bin map, so no file is named as the one \`${COMMAND_NAME}\` starts`,);
  }
  return manifest.bin;
}

/**
 The absolute path of the file the manifest names as the one a command starts.

 @param bin - the manifest's `bin` map

 @returns the resolved path of the file `mvm` starts

 @throws Error when the manifest names no file for `mvm`

 @example
 ```ts
 const entry = entryOf(await manifestBin());
 ```
 */
function entryOf(bin: Readonly<Record<string, string>>,): string {
  /**
   The path the manifest names for this command.
   */
  const declared = bin[COMMAND_NAME];
  if (declared === undefined) {
    throw new Error(`the manifest's bin map names no ${COMMAND_NAME} command`,);
  }
  return resolve(
    PACKAGE_FOLDER,
    declared,
  );
}

//endregion Reading what the manifest declares

//region Reading and starting the built entry

/**
 Reads the source specifier the built entry imports, when it imports one.

 The built entry is a launcher rather than a bundled program, so its whole body is one
 side-effect import of the source file the command runs.

 @returns the specifier the built entry imports, or nothing when it imports no relative file

 @example
 ```ts
 const specifier = importedSpecifierOf(await readFile(path, 'utf8'));
 ```
 */
function importedSpecifierOf(builtEntryText: string,): string | undefined {
  /**
   The first relative specifier the file imports, which is how the launcher names its source.
   */
  const found = /(?:^|\n)import\s+'(\.[^']+)'/.exec(builtEntryText,);
  return found?.[1];
}

/**
 Runs the file the manifest names as the one `mvm` starts, with the arguments given.

 @param args - the words after `mvm` on the command line

 @returns what the started command printed and how it ended

 @example
 ```ts
 const result = await runDeclaredEntry(['--help']);
 ```
 */
async function runDeclaredEntry(args: readonly string[],): Promise<{
  readonly stdout: string;
  readonly stderr: string;
  readonly exitCode: number;
}> {
  try {
    /**
     What the started command printed before it ended.
     */
    const ended = await spawn(
      process.execPath,
      [entryOf(await manifestBin(),), ...args,],
      {
        cwd: PACKAGE_FOLDER,
        timeout: HELP_TIMEOUT_MILLISECONDS,
      },
    );
    return {
      exitCode: 0,
      stderr: ended.stderr,
      stdout: ended.stdout,
    };
  }
  catch (error: unknown) {
    /**
     The failure a command that did not end successfully answers with.
     */
    const ended = error as SubprocessError;
    return {
      exitCode: ended.exitCode ?? 1,
      stderr: ended.stderr,
      stdout: ended.stdout,
    };
  }
}

//endregion Reading and starting the built entry

await describe({
  name: 'the mvm command',
  children: [
    //region The file the manifest's bin names

    it({
      name: 'names the TypeScript source beside it, so no build output stands between the command and its code',
      fn: async () => {
        /**
         The file the manifest names as the one `mvm` starts.
         */
        const entry = entryOf(await manifestBin(),);
        // A built copy is refreshed by whoever remembers to build, and every worktree
        // inherits whatever the checkout held when it was copied: the source is the only
        // file that cannot be older than the source (issue 6655).
        expect(entry,).toBe(join(
          PACKAGE_FOLDER,
          'src',
          'cli.ts',
        ),);
      },
    },),

    it({
      name: 'starts that source when it is run, and the command answers',
      fn: async () => {
        /**
         What `mvm --help` printed, started from the file the manifest names.
         */
        const ended = await runDeclaredEntry(['--help',],);
        expect(ended.exitCode,).toBe(0,);
        expect(ended.stdout,).toContain(HELP_FIRST_LINE,);
      },
    },),

    //endregion The file the manifest's bin names

    //region The built entry an older install's shim still names

    it({
      name: 'runs that same source through the built entry a shim an older install wrote names',
      fn: async () => {
        /**
         The built entry's text, which is a launcher or a bundle.
         */
        const builtEntryText = await readFile(
          join(
            PACKAGE_FOLDER,
            BUILT_ENTRY_PATH,
          ),
          'utf8',
        );
        /**
         The file inside the package the built entry would run.
         */
        const imported = importedSpecifierOf(builtEntryText,);
        // A bundle written by an older build holds the program itself, and then a source
        // change never reaches it: that is how the six guests lost free page reporting.
        expect(imported,).toBeDefined();
        expect(resolve(
          dirname(join(
            PACKAGE_FOLDER,
            BUILT_ENTRY_PATH,
          ),),
          imported ?? '',
        ),).toBe(entryOf(await manifestBin(),),);
      },
    },),

    //endregion The built entry an older install's shim still names
  ],
},);
