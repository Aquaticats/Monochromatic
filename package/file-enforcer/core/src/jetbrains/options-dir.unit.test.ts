import {
  mkdir,
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type JetbrainsOptionsDirectory,
  latestJetbrainsOptionsDirectory,
  NO_JETBRAINS_OPTIONS_DIRECTORY,
} from '../../dist/final/node/index.mjs';

/**
 Product prefixes recognized by IntelliJ IDEA discovery in these tests.
 */
const IDEA_PREFIXES = ['IntelliJIdea', 'IdeaIC',] as const;

/**
 Sets `XDG_CONFIG_HOME` for the scope, restoring the prior value on disposal.
 
 @param configRoot - Directory to expose as the XDG config root.
 
 @returns Disposable that restores the previous environment value.
 
 @example
 ```ts
 using xdg = withXdgConfigHome('/tmp/x');
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
      if (previous === undefined) delete process.env.XDG_CONFIG_HOME;
      else process.env.XDG_CONFIG_HOME = previous;
    },
  };
}

/**
 Owns a throwaway directory, removing it on async disposal.
 
 @param directory - Directory to remove on disposal.
 
 @returns Async disposable that recursively removes the directory.
 
 @example
 ```ts
 await using owned = throwawayDir(await mkdtemp(prefix));
 ```
 */
function throwawayDir(directory: string,): AsyncDisposable {
  return {
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(directory, { recursive: true, force: true, },);
    },
  };
}

/**
 Narrows a discovery result to a present directory, throwing otherwise.
 
 @param latest - Discovery result.
 
 @param label - Identifier surfaced when absent.
 
 @returns Present options directory.
 
 @throws Error when latest is the no-directory sentinel.
 
 @example
 ```ts
 requireDir(latest, 'idea');
 ```
 */
function requireDir(
  latest: JetbrainsOptionsDirectory | typeof NO_JETBRAINS_OPTIONS_DIRECTORY,
  label: string,
): JetbrainsOptionsDirectory {
  if (latest === NO_JETBRAINS_OPTIONS_DIRECTORY) throw new Error(`expected an options directory for '${label}'`,);
  return latest;
}

await describe({
  name: '',
  // Serialized because the suite mutates the shared XDG_CONFIG_HOME environment variable.
  concurrency: 1,
  children: [
    //region latestJetbrainsOptionsDirectory

    describe({
      name: latestJetbrainsOptionsDirectory.name,
      children: [
        it({
          name: 'selects the newest matching product and ignores other products',
          fn: async () => {
            /**
             Throwaway config root seeded with several JetBrains product directories.
             */
            const configRoot = await mkdtemp(join(scratchRoot(), 'fe-jb-latest-',),);
            await using owned = throwawayDir(configRoot,);
            await Promise.all([
              'IntelliJIdea2025.1',
              'IntelliJIdea2026.2',
              'IdeaIC2024.3',
              'PyCharm2026.1',
              'NotAProduct',
            ].map(async function makeProduct(productName,): Promise<void> {
              await mkdir(join(configRoot, 'JetBrains', productName, 'options',), { recursive: true, },);
            },),);
            using xdg = withXdgConfigHome(configRoot,);
            /**
             Latest IDEA options directory discovered under the throwaway root.
             */
            const latest = requireDir(
              await latestJetbrainsOptionsDirectory({ productPrefixes: IDEA_PREFIXES, },),
              'IntelliJIdea2026.2',
            );
            expect(latest.optionsDirectory,)
              .toBe(join(configRoot, 'JetBrains', 'IntelliJIdea2026.2', 'options',),);
            expect(latest.versionParts,).toEqual([2_026, 2,],);
            void owned;
            void xdg;
          },
        },),
        it({
          name: 'returns the no-directory sentinel when no JetBrains config root exists',
          fn: async () => {
            /**
             Throwaway root with no JetBrains directory.
             */
            const configRoot = await mkdtemp(join(scratchRoot(), 'fe-jb-empty-',),);
            await using owned = throwawayDir(configRoot,);
            using xdg = withXdgConfigHome(configRoot,);
            expect(await latestJetbrainsOptionsDirectory({ productPrefixes: IDEA_PREFIXES, },),)
              .toBe(NO_JETBRAINS_OPTIONS_DIRECTORY,);
            void owned;
            void xdg;
          },
        },),
      ],
    },),

    //endregion latestJetbrainsOptionsDirectory
  ],
},);
