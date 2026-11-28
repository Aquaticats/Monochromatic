import {
  nodeConfig,
  type NodeFlavorConfig,
} from '@monochromatic-dev/config-rolldown/.node.ts';

//region Constants

/**
 The name of the emitted file a person's `mvm` command runs.

 `package.json`'s `bin` names the TypeScript source instead, so this file is what a
 `node_modules/.bin/mvm` shim an older install wrote reaches. It is written as a launcher,
 never as a copy of the program: a launcher cannot be older than the source it names.
 */
const CLI_ENTRY_FILE_NAME = 'cli.mjs';

/**
 The source the built `cli.mjs` runs, written relative to where the build puts `cli.mjs`.

 `dist/final/node/cli.mjs` sits three folders below `package/mvm/cli`, so the specifier is
 `../../../src/cli.ts`. Node strips the types of the file it names, which is how the command
 runs the source that sits beside it in whatever checkout or worktree holds this build.
 */
const CLI_SOURCE_FROM_THE_BUILT_ENTRY = '../../../src/cli.ts';

/**
 What the built `cli.mjs` holds instead of a bundled program.
 */
const CLI_ENTRY_LAUNCHER = `#!/usr/bin/env node
// This file is not the command: it runs the TypeScript source beside this build, so a
// build can never be older than the code a person's \`mvm\` runs. It is here because a
// \`node_modules/.bin/mvm\` shim written by an install older than the manifest's \`bin\`
// still names this path; \`mise run //package/mvm/cli:build\` refreshes it.
import '${CLI_SOURCE_FROM_THE_BUILT_ENTRY}';
`;

//endregion Constants

//region Types

/**
 The fields this build reads of one emitted bundle entry.

 Only what the launcher needs: whether the entry is a chunk, and its code.
 */
type BuiltBundleEntry = {
  readonly type?: string;
  code?: string;
};

/**
 The one hook this build adds, shaped as the build tool's plugin hook takes it.

 `@monochromatic-dev/config-rolldown/.node.ts` does not re-export the build tool's
 `Plugin` type, and this package does not depend on the build tool itself, so the hook is
 declared here and checked against the config's `extraPlugins` where it is passed.
 */
type TheCommandRunsTheSourceBesideItPlugin = {
  readonly name: string;
  readonly generateBundle: (
    options: unknown,
    bundle: Record<string, unknown>,
  ) => void;
};

//endregion Types

//region Narrowing an emitted entry

/**
 Says whether a bundle entry is the chunk carrying the CLI's code.

 @param entry - one value of the emitted bundle, of unknown shape

 @returns whether the entry is a chunk this build may rewrite

 @example
 ```ts
 const isCliChunk = isChunkEntry(bundle['cli.mjs']);
 ```
 */
function isChunkEntry(entry: unknown,): entry is BuiltBundleEntry {
  if (((typeof entry) !== 'object') || (entry === null)) {
    return false;
  }
  return ((entry as BuiltBundleEntry).type === 'chunk');
}

//endregion Narrowing an emitted entry

//region The launcher plugin

/**
 Replaces the bundled CLI entry with a launcher that runs the source beside the build.

 The shared shebang plugin reads the manifest's `bin` targets, which now name `src/cli.ts`,
 so it no longer touches the emitted chunk: the launcher's own text carries the shebang, and
 the shared executable-bit plugin still sees that shebang on the written chunk and marks it
 runnable. The CLI stays in the build's input so the chunk is emitted under the name an older
 shim names, and so `cli.d.mts` is still produced beside the library entry, which stays a real
 bundle.

 @returns Rolldown plugin rewriting the CLI entry chunk before it is written

 @example
 ```ts
 const config = nodeConfig({ input: ['./src/cli.ts'] },);
 ```
 */
function theCommandRunsTheSourceBesideItPlugin(): TheCommandRunsTheSourceBesideItPlugin {
  return {
    name: 'mvm:the-command-runs-the-source-beside-it',

    /**
     Puts the launcher's text in the CLI entry chunk's place.

     @param bundle - the emitted bundle, keyed by output file name

     @throws Error when the build emitted no CLI entry chunk, which would leave the command
     running whatever an older build left at that path
     */
    generateBundle(
      _options: unknown,
      bundle: Record<string, unknown>,
    ): void {
      /**
       The emitted entry this build writes the launcher over.
       */
      const entry = bundle[CLI_ENTRY_FILE_NAME];
      if (!isChunkEntry(entry,)) {
        throw new Error(
          `the build emitted no ${CLI_ENTRY_FILE_NAME} chunk, so the \`mvm\` command would run whatever an older build left at that path`,
        );
      }
      entry.code = CLI_ENTRY_LAUNCHER;
    },
  };
}

//endregion The launcher plugin

/**
 Node-side rolldown build for `cli-mvm`.

 Two entries: the library re-exports (`src/index.ts`) and the CLI (`src/cli.ts`). Both land
 under `dist/final/node/` as `.mjs`. The VM manager uses
 `node:fs/promises`/`node:path`/`node:child_process` and
 TODO: deprecate Optique
 the bin uses `@optique/run`, so the bundle is node-only.

 `package.json#bin` points at `src/cli.ts`, so `mvm` runs TypeScript the way `mise run
 //package/mvm/cli:run` already does and no build can leave the command running code older
 than the source beside it. The build still emits `dist/final/node/cli.mjs`, as a launcher
 running that source, because a `node_modules/.bin/mvm` shim written by an older install
 names that path.
 */
const config: NodeFlavorConfig = nodeConfig({
  input: [
    './src/index.ts',
    './src/cli.ts',
  ],
  extraPlugins: [theCommandRunsTheSourceBesideItPlugin(),],
},);

export default config;
