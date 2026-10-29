import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { latestJetbrainsOptionsDirectory, } from '../../dist/final/node/index.mjs';

/**
 Product prefixes recognized by IntelliJ IDEA discovery in these tests.
 */
const IDEA_PREFIXES = ['IntelliJIdea', 'IdeaIC',] as const;

//region Environment and fixture helpers

/**
 Sets `XDG_CONFIG_HOME` for scoped JetBrains discovery tests.
 
 @param configRoot - Directory exposed as XDG config root.
 
 @returns Disposable that restores prior environment state.
 
 @example
 ```ts
 using xdg = withXdgConfigHome('/tmp/config');
 ```
 */
function withXdgConfigHome(configRoot: string,): Disposable {
  /**
   Prior XDG config root, restored on disposal.
   */
  const previous = process.env.XDG_CONFIG_HOME;
  process.env.XDG_CONFIG_HOME = configRoot;
  return {
    [Symbol.dispose](): void {
      if (previous === undefined)
        delete process.env.XDG_CONFIG_HOME;
      else
        process.env.XDG_CONFIG_HOME = previous;
    },
  };
}

/**
 Owns temporary directory cleanup for JetBrains discovery tests.
 
 @param directory - Directory to remove on disposal.
 
 @returns Async disposable that removes directory recursively.
 
 @example
 ```ts
 await using owned = throwawayDir('/tmp/config');
 ```
 */
function throwawayDir(directory: string,): AsyncDisposable {
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        directory,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

//endregion Environment and fixture helpers

//region Error capture helpers

/**
 Captures latest JetBrains options discovery failure for assertion.
 
 @returns Caught discovery error, or undefined when discovery resolves.
 
 @example
 ```ts
 const error = await latestJetbrainsOptionsDirectoryError();
 ```
 */
async function latestJetbrainsOptionsDirectoryError(): Promise<unknown> {
  try {
    await latestJetbrainsOptionsDirectory({ productPrefixes: IDEA_PREFIXES, },);
  }
  catch (error: unknown) {
    return error;
  }

  return undefined;
}

//endregion Error capture helpers

await describe({
  name: latestJetbrainsOptionsDirectory.name,
  // Serialized because the suite mutates shared process.env.XDG_CONFIG_HOME.
  concurrency: 1,
  children: [
    it({
      name: 'propagates corrupt JetBrains config root read errors',
      fn: async function rejectsRegularFileConfigRoot(): Promise<void> {
        /**
         Throwaway XDG root whose JetBrains entry is a file, not directory.
         */
        const configRoot = await mkdtemp(join(
          scratchRoot(),
          'fe-jb-hardening-',
        ),);
        await using owned = throwawayDir(configRoot,);
        using xdg = withXdgConfigHome(configRoot,);
        await writeFile(
          join(
            configRoot,
            'JetBrains',
          ),
          'not a directory',
        );

        const caught = await latestJetbrainsOptionsDirectoryError();
        expect(caught,).toMatchObject({
          code: 'ENOTDIR',
        },);
        void owned;
        void xdg;
      },
    },),
  ],
},);
