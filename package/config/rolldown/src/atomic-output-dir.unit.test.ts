/**
 Guards that a Node flavor build never leaves an emitted path absent or half written.

 The `mvm` command a checkout serves runs
 `package/mvm/cli/dist/final/node/cli.mjs`,
 and a build of that package used to delete the file at the start of its write phase
 and write it again a moment later.
 A command started in that moment ended with
 `Error: Cannot find module '<checkout>/package/mvm/cli/dist/final/node/cli.mjs'`
 (issue 6668,
 seen twice on 2026-11-28:
 `doc/troubleshooting/mvm-cli-a-build-never-leaves-its-command-file-absent.md`).

 These checks read the path at the build's own lifecycle hooks,
 so nothing here is a race won:
 a hook is a point the build states it has reached,
 and every build passes it.
 A build that replaces its output by writing beside it and renaming it into place
 still holds the previous file at every hook,
 and a build that deletes its output first holds nothing there.

 @module
 */

import {
  existsSync,
  mkdtempSync,
  readFileSync,
  rmSync,
  statSync,
  writeFileSync,
} from 'node:fs';
import { join, } from 'node:path';

import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  type OutputOptions,
  type Plugin,
  rolldown,
} from 'rolldown';

import {
  type NodeFlavorConfig,
  nodeConfig,
} from './index.node.ts';

//region Fixtures

/**
 Output directory every Node flavor build writes into, package-root relative.
 */
const OUTPUT_DIRECTORY_NAME = 'dist/final/node';

/**
 Name of the chunk the fixture's `entry.ts` input emits.
 */
const ENTRY_FILE_NAME = 'entry.mjs';

/**
 Name of the chunk the fixture's `sibling.ts` input emits.
 */
const SIBLING_FILE_NAME = 'sibling.mjs';

/**
 Disposable package fixture one Node flavor build writes into.
 */
type NodeBuildFixture = {
  readonly path: string;
  readonly outputDirectory: string;
  [Symbol.dispose]: () => void;
};

/**
 Create a disposable package directory with two entry sources and no manifest.

 The manifest is left out on purpose:
 `packageExternals` reads the build cwd's manifest for the external list,
 and these checks are about where output lands, not about what stays external.

 @returns Disposable fixture outside repository state.

 @example
 ```ts
 using fixture = createNodeBuildFixture();
 ```
 */
function createNodeBuildFixture(): NodeBuildFixture {
  /**
   Fresh fixture directory under the test harness scratch root.
   */
  const path = mkdtempSync(join(scratchRoot(), 'atomic-output-dir-',),);
  writeFileSync(join(path, 'sibling.ts',), 'export const sibling = 2;\n',);
  return {
    path,
    outputDirectory: join(path, OUTPUT_DIRECTORY_NAME,),
    [Symbol.dispose]: function removeFixture(): void {
      rmSync(path, {
        force: true,
        recursive: true,
      },);
    },
  };
}

/**
 Write the fixture's entry source, marked so two builds differ.

 @param fixture - Fixture whose `entry.ts` this writes

 @param marker - Text the emitted chunk carries, which tells one build's output from another's

 @example
 ```ts
 writeEntry({ fixture, marker: 'second build' });
 ```
 */
function writeEntry({
  fixture,
  marker,
}: {
  readonly fixture: NodeBuildFixture;
  readonly marker: string;
},): void {
  writeFileSync(
    join(fixture.path, 'entry.ts',),
    `export const marker = ${JSON.stringify(marker)};\n`,
  );
}

//endregion Fixtures

//region Observing the output path

/**
 What one path held at one moment: absent, or a file with an inode and text.
 */
type OutputObservation = {
  readonly exists: boolean;
  readonly inode: number | undefined;
  readonly text: string | undefined;
};

/**
 Read what the path holds now.

 @param path - Path a build writes

 @returns Whether the path resolves, and when it does, its inode and text

 @example
 ```ts
 const before = observeOutput({ path: join(fixture.outputDirectory, ENTRY_FILE_NAME) });
 ```
 */
function observeOutput({ path, }: {
  readonly path: string;
},): OutputObservation {
  if (!existsSync(path,)) {
    return {
      exists: false,
      inode: undefined,
      text: undefined,
    };
  }
  return {
    exists: true,
    inode: statSync(path,).ino,
    text: readFileSync(path, 'utf8',),
  };
}

/**
 Observations the probe plugin recorded, keyed by the hook that recorded them.
 */
type HookObservations = {
  renderStart: OutputObservation[];
  generateBundle: OutputObservation[];
};

/**
 Build a plugin that records what the path holds at two hooks of the build.

 `renderStart` is the first hook of the render and write phase,
 which is where the deleted-before-written shape showed itself,
 and `generateBundle` is the last one before the output is written.

 @param observations - Record object the plugin appends to, one list per hook

 @param path - Path each hook observes

 @returns Plugin carrying no other behaviour

 @example
 ```ts
 const observations = { renderStart: [], generateBundle: [] };
 plugins: [probeOutputAtHooks({ observations, path })];
 ```
 */
function probeOutputAtHooks({
  observations,
  path,
}: {
  readonly observations: HookObservations;
  readonly path: string;
},): Plugin {
  return {
    name: 'monochromatic-test:observes-the-output-path-at-each-hook',
    renderStart(): void {
      observations.renderStart.push(observeOutput({ path, },),);
    },
    generateBundle(): void {
      observations.generateBundle.push(observeOutput({ path, },),);
    },
  };
}

//endregion Observing the output path

//region Running one Node flavor build

/**
 Run one Node flavor build of the fixture through {@link nodeConfig}.

 @param fixture - Fixture the build reads and writes

 @param inputs - Entry sources, package-root relative, defaulting to `entry.ts`

 @param plugins - Plugins appended after the config's own, as a probe or a test subject

 @param outputOverrides - Output options the consuming config sets, merged after the shared ones

 @returns Nothing; the build's output is on disk when it settles

 @example
 ```ts
 await buildNodeConfig({ fixture });
 ```
 */
async function buildNodeConfig({
  fixture,
  inputs = ['entry.ts',],
  plugins = [],
  outputOverrides = {},
}: {
  readonly fixture: NodeBuildFixture;
  readonly inputs?: readonly string[];
  readonly plugins?: readonly Plugin[];
  readonly outputOverrides?: OutputOptions;
},): Promise<void> {
  /**
   Config the package's own `rolldown.node.config.ts` would hand the bundler.
   */
  const config: NodeFlavorConfig = nodeConfig({
    input: inputs.map(function toSourcePath(input: string,): string {
      return join(fixture.path, input,);
    },),
    outputDir: fixture.outputDirectory,
    outputOverrides,
  },);
  /**
   Build reading the fixture and writing its output.
   */
  const build = await rolldown({
    ...config,
    plugins: [
      ...(config.plugins ?? []) as readonly Plugin[],
      ...plugins,
    ],
  },);
  /**
   Output options the config carries, which a single-output config holds as one object.
   */
  const outputOptions: OutputOptions = Array.isArray(config.output,)
    ? config.output[0] ?? {}
    : config.output ?? {};
  try {
    await build.write(outputOptions,);
  }
  finally {
    await build.close();
  }
}

//endregion Running one Node flavor build

await describe({
  name: 'a Node flavor build of a package whose output is a command',
  // One build at a time: `nodeConfig` shares one declaration plugin instance across every
  // config in this process, and two builds running at once make it read another build's
  // declarations (rolldown-plugin-dts 0.30.2, `planInlineExports`).
  concurrency: 1,
  children: [
    it({
      name: 'holds the previous build at every hook of the next one, then replaces it by rename',
      fn: async function holdsThePreviousBuild(): Promise<void> {
        using fixture = createNodeBuildFixture();
        writeEntry({
          fixture,
          marker: 'first build',
        },);
        await buildNodeConfig({ fixture, },);
        /**
         Path the `mvm` command's shim names: the entry chunk of this build.
         */
        const outputPath = join(fixture.outputDirectory, ENTRY_FILE_NAME,);
        /**
         What the first build left there.
         */
        const before = observeOutput({ path: outputPath, },);
        expect(before.exists,).toBe(true,);

        writeEntry({
          fixture,
          marker: 'second build',
        },);
        /**
         Observations of the second build's own hooks.
         */
        const observations: HookObservations = {
          renderStart: [],
          generateBundle: [],
        };
        await buildNodeConfig({
          fixture,
          plugins: [probeOutputAtHooks({ observations, path: outputPath, },),],
        },);
        /**
         What the second build left there.
         */
        const after = observeOutput({ path: outputPath, },);

        // While the build compiles and renders, the path still holds the first build's
        // complete text: the build removes nothing before it has a file to put in place.
        expect(observations.renderStart.map(function textAtRenderStart(observation: OutputObservation,): string | undefined {
          return observation.text;
        },),).toEqual([before.text,],);
        expect(observations.generateBundle.map(function textAtGenerateBundle(observation: OutputObservation,): string | undefined {
          return observation.text;
        },),).toEqual([before.text,],);
        // The finished build put its own file at that name by replacing the name with one,
        // never by writing over the file the previous build left.
        expect(after.exists,).toBe(true,);
        expect(after.inode,).not.toBe(before.inode,);
        expect(after.text,).toContain('second build',);
      },
    },),
    it({
      name: 'renames the files into place only after every other plugin finished writing them',
      fn: async function movesAfterOtherWriters(): Promise<void> {
        using fixture = createNodeBuildFixture();
        writeEntry({
          fixture,
          marker: 'first build',
        },);
        await buildNodeConfig({ fixture, },);
        /**
         Path the `mvm` command's shim names: the entry chunk of this build.
         */
        const outputPath = join(fixture.outputDirectory, ENTRY_FILE_NAME,);
        /**
         What the first build left there.
         */
        const before = observeOutput({ path: outputPath, },);
        writeEntry({
          fixture,
          marker: 'second build',
        },);
        /**
         Observations taken inside one plain `writeBundle` hook of the second build.
         */
        const duringWriteBundle: OutputObservation[] = [];
        await buildNodeConfig({
          fixture,
          plugins: [{
            name: 'monochromatic-test:observes-the-output-path-in-a-plain-write-bundle-hook',
            writeBundle(): void {
              duringWriteBundle.push(observeOutput({ path: outputPath, },),);
            },
          },],
        },);

        // `writeBundle` hooks are one parallel batch, and the shared executable-bit plugin
        // chmods a staged file in its own: a rename that raced it would leave a command without
        // its executable bit, or stop the build on a path that is gone. The rename therefore
        // runs in the ordered `post` group, which a plain hook observes as the previous build's
        // file still in place when it looks.
        expect(duringWriteBundle.map(function inodeDuringWriteBundle(observation: OutputObservation,): number | undefined {
          return observation.inode;
        },),).toEqual([before.inode,],);
      },
    },),
    it({
      name: 'removes a file an earlier build emitted and this one does not',
      fn: async function removesStaleOutput(): Promise<void> {
        using fixture = createNodeBuildFixture();
        writeEntry({
          fixture,
          marker: 'first build',
        },);
        await buildNodeConfig({ fixture, },);
        /**
         File only the earlier build emitted, as a chunk an entry it no longer builds left behind.
         */
        const stalePath = join(fixture.outputDirectory, 'gone.mjs',);
        writeFileSync(stalePath, 'export {};\n',);
        await buildNodeConfig({ fixture, },);
        expect(existsSync(stalePath,),).toBe(false,);
        expect(existsSync(join(fixture.outputDirectory, ENTRY_FILE_NAME,),),).toBe(true,);
      },
    },),
    it({
      name: 'leaves a sibling build of the same folder alone when cleaning was turned off',
      fn: async function leavesSiblingBuildAlone(): Promise<void> {
        using fixture = createNodeBuildFixture();
        writeEntry({
          fixture,
          marker: 'first build',
        },);
        await buildNodeConfig({
          fixture,
          inputs: [
            'entry.ts',
            'sibling.ts',
          ],
        },);
        /**
         Output the sibling entry emitted, which a later build of the same folder must keep.
         */
        const siblingPath = join(fixture.outputDirectory, SIBLING_FILE_NAME,);
        expect(existsSync(siblingPath,),).toBe(true,);
        writeEntry({
          fixture,
          marker: 'second build',
        },);
        await buildNodeConfig({
          fixture,
          outputOverrides: { cleanDir: false, },
        },);
        expect(existsSync(siblingPath,),).toBe(true,);
        expect(existsSync(join(fixture.outputDirectory, ENTRY_FILE_NAME,),),).toBe(true,);
      },
    },),
  ],
},);
