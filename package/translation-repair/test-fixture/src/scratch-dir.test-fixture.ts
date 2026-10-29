/**
 Throwaway directories for tests that write files, removed when their scope
 ends.

 ONE COPY. Test files that kept their own copy of this helper, under names
 such as `throwawayCacheDir` or `scratchDirectory`, with only the directory
 prefix and the field name on the returned handle differing, now import it
 from here. Helpers that make a directory and never remove it are a different
 behaviour and stay in their files until they gain a disposer of their own.

 `scratchDirWith` is for a helper whose setup runs more than the one call
 that makes the directory: `git init`, a write, a commit, each its own
 `await` the directory must survive a throw from, until the helper's own
 return. Binding `scratchDir`'s result with `await using` inside such a
 helper is wrong, since it disposes before the caller ever sees the
 directory; `scratchDirWith` runs the setup under a `try` instead, so a
 throw partway through still removes the directory before it escapes.
 `scratchDirPrepared` is the same for a setup that hands back no fields of
 its own, so no caller spells an empty result type.

 @module
 */

import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

/**
 A directory removed with everything in it when its `await using` scope ends.

 @example
 ```ts
 await using scratch = await scratchDir({ prefix: 'whiskers-cache-', },);
 ```
 */
type ScratchDir = {
  /**
   Absolute path of the fresh directory.
   */
  readonly path: string;

  /**
   Removes the directory and everything under it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Makes a fresh directory under the platform temp root.

 @param prefix - start of the directory's name, so a leftover shows which test
 made it

 @returns The directory, removed on scope exit

 @example
 ```ts
 await using scratch = await scratchDir({ prefix: 'whiskers-cache-', },);
 ```
 */
export async function scratchDir({ prefix, }: { readonly prefix: string; },): Promise<ScratchDir> {
  /**
   Fresh directory under the platform temp root.
   */
  const path = await mkdtemp(join(
    scratchRoot(),
    prefix,
  ),);
  return {
    path,
    [Symbol.asyncDispose]: async function removeScratch(): Promise<void> {
      await rm(
        path,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Makes a fresh directory under the platform temp root, runs `setup` against
 it, and carries `setup`'s own result alongside the directory, removed on
 scope exit; a throw anywhere in `setup` removes the directory before it
 escapes, since nothing outside this function ever held a reference to it
 yet.

 REFUSED AT THE TYPE LEVEL: `setup`'s result may not declare `path` or
 `[Symbol.asyncDispose]` of its own. Both come from the directory this
 function already made, and a `setup` that redeclared either would have its
 own value silently overwritten by one call ordered after the spread; a
 runtime refusal could only catch this once a case actually ran, where a
 compile error catches it at the one place this helper is itself edited.

 @param prefix - start of the directory's name, so a leftover shows which
 test made it

 @param setup - fills the directory, reading only its path, never its
 disposer, so it cannot remove the directory ahead of its own caller; returns
 the fields a caller reads alongside that path

 @returns `setup`'s fields, the directory's path, and its disposer

 @throws whatever `setup` throws, after removing the directory

 @example
 ```ts
 await using corpus = await scratchDirWith({
   prefix: 'whiskers-corpus-',
   setup: async function seeded({ path, },): Promise<{ readonly commitSha: string; }> {
     await writeFile(join(path, 'page.md',), 'Whiskers naps.\n',);
     return { commitSha: 'deadbeef', };
   },
 },);
 ```
 */
export async function scratchDirWith<
  const SetupT extends Record<PropertyKey, unknown> & {
    readonly path?: never;
    readonly [Symbol.asyncDispose]?: never;
  },
  >(
  {
    prefix,
    setup,
  }: {
    readonly prefix: string;
    readonly setup: (dir: Pick<ScratchDir, 'path'>) => Promise<SetupT>;
  },
  ): Promise<SetupT & ScratchDir> {
  /**
   Directory this call owns until `setup` finishes or throws.
   */
  const scratch = await scratchDir({ prefix, },);
  try {
    /**
     Fields `setup` hands back, carrying no `path` or disposer of its own.
     */
    const result = await setup(scratch,);
    /**
     The directory alone, spread in concrete rather than generic so its two
     fields are never read as overlapping `SetupT`'s.
     */
    const handle: ScratchDir = {
      path: scratch.path,
      [Symbol.asyncDispose]: scratch[Symbol.asyncDispose],
    };
    return {
      ...result,
      ...handle,
    };
  }
  catch (error) {
    await scratch[Symbol.asyncDispose]();
    throw error;
  }
}

/**
 Makes a fresh directory under the platform temp root and runs `prepare`
 against it, for a helper whose setup fills the directory and hands nothing
 back beside it; a throw anywhere in `prepare` removes the directory before
 it escapes, as `scratchDirWith` does for a setup with fields to return.

 @param prefix - start of the directory's name, so a leftover shows which
 test made it

 @param prepare - fills the directory, reading only its path, never its
 disposer, so it cannot remove the directory ahead of its own caller

 @returns The directory, removed on scope exit

 @throws whatever `prepare` throws, after removing the directory

 @example
 ```ts
 await using runs = await scratchDirPrepared({
   prefix: 'whiskers-runs-',
   prepare: async function seeded({ path, },): Promise<void> {
     await mkdir(join(path, 'artifacts',),);
   },
 },);
 ```
 */
export async function scratchDirPrepared(
  {
    prefix,
    prepare,
  }: {
    readonly prefix: string;
    readonly prepare: (dir: Pick<ScratchDir, 'path'>) => Promise<void>;
  },
): Promise<ScratchDir> {
  /**
   Directory this call owns until `prepare` finishes or throws.
   */
  const scratch = await scratchDir({ prefix, },);
  try {
    await prepare(scratch,);
    return scratch;
  }
  catch (error) {
    await scratch[Symbol.asyncDispose]();
    throw error;
  }
}

/**
 Makes a fresh directory holding exactly the named files, each the empty
 JSON object, for cases about which names a directory reader takes as
 records.

 @param prefix - start of the directory's name, so a leftover shows which
 test made it

 @param names - file names to write, verbatim, so a case can write a name no
 reader should take

 @returns The directory, also as `dir`, removed on scope exit

 @throws whatever a write throws, after removing the directory

 @example
 ```ts
 await using cache = await scratchDirOfEmptyRecords({ prefix: 'whiskers-cache-', names: ['Mittens.json',], },);
 ```
 */
export async function scratchDirOfEmptyRecords(
  {
    prefix,
    names,
  }: {
    readonly prefix: string;
    readonly names: readonly string[];
  },
): Promise<ScratchDir & { readonly dir: string; }> {
  return await scratchDirWith({
    prefix,
    setup: async function seeded({ path: dir, },): Promise<{ readonly dir: string; }> {
      await Promise.all(names.map(async function writeOne(name,): Promise<void> {
        await writeFile(
          join(
            dir,
            name,
          ),
          '{}',
          'utf8',
        );
      },),);
      return { dir, };
    },
  },);
}
