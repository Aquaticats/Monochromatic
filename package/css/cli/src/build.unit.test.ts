import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { existsSync, } from 'node:fs';
import {
  mkdir,
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { dirname, join, } from 'node:path';

import { buildCss, } from '@monochromatic-dev/css-cli';


//region Test helpers

// import.meta.dirname is a Bun-specific API (equivalent to __dirname in CJS)
/** Root fixture directory for all CSS integration tests */
const fixtureRoot = join(import.meta.dirname, '..', '..', '..', 'test-fixture',);

/**
 Each entry exercises a different CSS import resolution strategy.
 - exports field: resolved via package.json `exports` mappings
 - direct file path: resolved by reaching into the package's file tree (no `exports` field)
 */
const integrationFixtures = [
  { label: 'exports field', dir: join(fixtureRoot, 'css-importing',), },
  { label: 'direct file path', dir: join(fixtureRoot, 'css-importing-filepath',), },
] as const;

/**
 Runs one buildCss invocation inside a disposable temp directory seeded with
 the given files, so error-path and dedup tests never touch shared state.
 
 @param files - Relative path to file content map seeded into the temp dir.
 @returns Built CSS text.
 */
async function buildInTempDir({
  files,
}: {
  readonly files: Readonly<Record<string, string>>;
},): Promise<string> {
  /**
   Disposable directory owning all inputs and the output.
   */
  const dir = await mkdtemp(join(
    scratchRoot(),
    'build-css-test-',
  ),);
  /**
   Removes the temp directory when the scope exits, error paths included.
   */
  await using tempDirGuard = {
    [Symbol.asyncDispose]: async function removeTempDir(): Promise<void> {
      await rm(
        dir,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
  // The guard binding exists solely for its dispose hook.
  void tempDirGuard;

  await Promise.all(
    Object.entries(files,).map(
      async function writeFixture([relativePath, content,],) {
        /**
         Absolute path of the seeded file.
         */
        const absolutePath = join(
          dir,
          relativePath,
        );
        await mkdir(
          dirname(absolutePath,),
          { recursive: true, },
        );
        await writeFile(
          absolutePath,
          content,
        );
      },
    ),
  );
  return await buildCss({
    input: join(
      dir,
      'main.css',
    ),
    output: join(
      dir,
      'out.css',
    ),
  },);
}

/**
 Directory the package owns, holding its sources and its built `dist/`; no
 test may leave a build output inside it.
 */
const packageDirectory = join(import.meta.dirname, '..',);

/**
 One disposable directory an integration build writes into, removed when the
 `await using` scope it is bound to ends, throw included.
 Each test gets its own directory because module-test runs the cases of a
 suite concurrently, and no directory lives inside the package's `dist/`, so
 a failing assertion cannot leave output for a later build to pick up.
 */
type DisposableBuildOutput = {
  /**
   Output path owned solely by the calling test.
   */
  readonly outputPath: string;
  /**
   Removes the directory and whatever the build wrote into it.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Creates the disposable directory one integration build writes into.
 
 @returns Output path plus the disposal hook that removes its directory.
 */
async function disposableBuildOutput(): Promise<DisposableBuildOutput> {
  /**
   Directory owning this test's output file alone.
   */
  const outputDirectory = await mkdtemp(join(
    scratchRoot(),
    'build-css-output-',
  ),);
  return {
    outputPath: join(outputDirectory, 'out.css',),
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        outputDirectory,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

//endregion Test helpers

const integrationChildren = integrationFixtures.map(({ label, dir, },) => {
  /** Path to the main CSS entry point for this fixture */
  const fixtureMainCss = join(dir, 'src', 'main.css',);

  return describe({
    name: `buildCss (${label})`,
    children: [
      it({
        name: 'builds fixture CSS with import resolution and mixin expansion',
        fn: async () => {
          await using output = await disposableBuildOutput();
          const result = await buildCss({
            input: fixtureMainCss,
            output: output.outputPath,
          },);

          // Imports should be resolved and inlined
          expect(result,).toContain('--primary: rebeccapurple',);

          // @mixin definitions should be removed
          expect(result,).not.toContain('@mixin',);

          // @apply should be expanded
          expect(result,).not.toContain('@apply',);

          // Mixin content should be inlined
          expect(result,).toContain('display: flex',);
          expect(result,).toContain('font-weight: bold',);
        },
      },),

      it({
        name: 'writes output file to disk',
        fn: async () => {
          await using output = await disposableBuildOutput();
          await buildCss({
            input: fixtureMainCss,
            output: output.outputPath,
          },);

          const written = await readFile(output.outputPath, 'utf8',);
          expect(written,).toContain('--primary: rebeccapurple',);
        },
      },),

      it({
        name: 'writes integration build outputs outside the package directory',
        fn: async () => {
          await using output = await disposableBuildOutput();
          expect(output.outputPath.startsWith(packageDirectory,),).toBe(false,);
        },
      },),

      it({
        name: 'removes its build output even when a later assertion fails',
        fn: async () => {
          /**
           Failure the deliberate assertion raised inside the disposed scope.
           */
          let caught: unknown;
          /**
           Directory whose disposal the assertions after the scope check.
           */
          let outputDirectory = '';
          {
            await using output = await disposableBuildOutput();
            outputDirectory = dirname(output.outputPath,);
            try {
              await buildCss({
                input: fixtureMainCss,
                output: output.outputPath,
              },);
              // The build just wrote this file, so the assertion fails exactly
              // the way a real one would after the output reached disk.
              expect(existsSync(output.outputPath,),).toBe(false,);
            }
            catch (error) {
              caught = error;
            }
          }

          expect(String(caught,),).toContain('to equal false',);
          expect(existsSync(outputDirectory,),).toBe(false,);
        },
      },),

      it({
        name: 'expands nested mixin references in build output',
        fn: async () => {
          await using output = await disposableBuildOutput();
          const result = await buildCss({
            input: fixtureMainCss,
            output: output.outputPath,
          },);

          // --card uses @apply --flex-center, so .nested-card should have flex styles
          expect(result,).toContain('.nested-card',);
          // The flex-center content should appear inside .nested-card
          const nestedCardMatch = result.slice(result.indexOf('.nested-card',),);
          expect(nestedCardMatch,).toContain('display: flex',);
        },
      },),
    ],
  },);
},);

await describe({
  name: '',
  children: [
    describe({
      name: buildCss.name,
      children: [
        //region Import inlining

        it({
          name: 'resolves a slash-containing bare specifier against the importer directory',
          fn: async () => {
            const result = await buildInTempDir({
              files: {
                'main.css': "@import 'sub/theme.css';",
                'sub/theme.css': '.theme { color: red; }',
              },
            },);

            expect(result,).toContain('.theme { color: red; }',);
          },
        },),

        it({
          name: 'inlines a duplicate import only once',
          fn: async () => {
            const result = await buildInTempDir({
              files: {
                'main.css': "@import './shared.css';\n@import './shared.css';\n.a { top: 0; }",
                'shared.css': '.shared { color: red; }',
              },
            },);

            expect(result.indexOf('.shared',),).toBe(result.lastIndexOf('.shared',),);
            expect(result,).toContain('.a { top: 0; }',);
          },
        },),

        it({
          name: 'inlines through nested imports and keeps a circular pair stable',
          fn: async () => {
            const result = await buildInTempDir({
              files: {
                'main.css': "@import './one.css';\n.main { top: 0; }",
                'one.css': "@import './two.css';\n.one { color: red; }",
                'two.css': "@import './one.css';\n.two { color: blue; }",
              },
            },);

            expect(result,).toContain('.one',);
            expect(result,).toContain('.two',);
            expect(result,).toContain('.main',);
            expect(result,).not.toContain('@import',);
          },
        },),

        it({
          name: 'reads specifiers from url() with trailing conditions intact',
          fn: async () => {
            const result = await buildInTempDir({
              files: {
                'main.css': "@import url('shared.css') layer(base);\n.a { top: 0; }",
                'shared.css': '.shared { color: red; }',
              },
            },);

            expect(result,).toContain('.shared',);
            expect(result,).not.toContain('@import',);
            expect(result,).toContain('@layer base',);
          },
        },),

        it({
          name: 'wraps a media-conditioned import in its media query',
          fn: async () => {
            const result = await buildInTempDir({
              files: {
                'main.css': "@import './print.css' print;\n.a { top: 0; }",
                'print.css': '.p { color: black; }',
              },
            },);

            expect(result,).toContain('@media print',);
            expect(result,).toContain('.p { color: black; }',);
          },
        },),

        it({
          name: 'wraps a supports-conditioned import in its supports block',
          fn: async () => {
            const result = await buildInTempDir({
              files: {
                'main.css': "@import './grid.css' supports(display: grid);",
                'grid.css': '.g { display: grid; }',
              },
            },);

            expect(result,).toContain('@supports (display: grid)',);
            expect(result,).toContain('.g { display: grid; }',);
          },
        },),

        it({
          name: 'nests a layer and a media condition around one import',
          fn: async () => {
            const result = await buildInTempDir({
              files: {
                'main.css': "@import './base.css' layer(base) screen;",
                'base.css': '.b { color: red; }',
              },
            },);

            expect(result,).toContain('@layer base',);
            expect(result,).toContain('@media screen',);
          },
        },),

        //endregion Import inlining

        //region Package manifests

        it({
          name: 'throws naming the manifest when package.json is not parseable',
          fn: async () => {
            let caught: unknown;
            try {
              await buildInTempDir({
                files: {
                  'main.css': "@import 'manifest-pkg/test.css';",
                  'node_modules/manifest-pkg/package.json': '{ "exports": ',
                  'node_modules/manifest-pkg/test.css': '.m { color: red; }',
                },
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(String(caught,),).toContain('node_modules/manifest-pkg/package.json',);
          },
        },),

        it({
          name: 'falls through to the direct file path when package.json is absent',
          fn: async () => {
            const result = await buildInTempDir({
              files: {
                'main.css': "@import 'manifest-less-pkg/test.css';",
                'node_modules/manifest-less-pkg/test.css': '.m { color: red; }',
              },
            },);

            expect(result,).toContain('.m { color: red; }',);
          },
        },),

        it({
          name: 'resolves a package subpath through an exports wildcard pattern',
          fn: async () => {
            const result = await buildInTempDir({
              files: {
                'main.css': "@import 'pattern-pkg/theme.css';",
                'node_modules/pattern-pkg/package.json': '{ "exports": { "./*": "./src/*" } }',
                'node_modules/pattern-pkg/src/theme.css': '.theme { color: red; }',
              },
            },);

            expect(result,).toContain('.theme { color: red; }',);
          },
        },),

        it({
          name: 'resolves a package whose exports field is a single string',
          fn: async () => {
            const result = await buildInTempDir({
              files: {
                'main.css': "@import 'string-pkg';",
                'node_modules/string-pkg/package.json': '{ "exports": "./src/index.css" }',
                'node_modules/string-pkg/src/index.css': '.entry { color: blue; }',
              },
            },);

            expect(result,).toContain('.entry { color: blue; }',);
          },
        },),

        it({
          name: 'throws instead of reaching past an exports map that matches nothing',
          fn: async () => {
            let caught: unknown;
            try {
              await buildInTempDir({
                files: {
                  'main.css': "@import 'sealed-pkg/src/secret.css';",
                  'node_modules/sealed-pkg/package.json': '{ "exports": { "./index.css": "./index.css" } }',
                  'node_modules/sealed-pkg/index.css': '.exported { color: green; }',
                  'node_modules/sealed-pkg/src/secret.css': '.secret { color: red; }',
                },
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(String(caught,),).toContain("does not export subpath './src/secret.css'",);
          },
        },),

        //endregion Package manifests

        //region Errors

        it({
          name: 'throws when a relative import target is missing',
          fn: async () => {
            let caught: unknown;
            try {
              await buildInTempDir({
                files: {
                  'main.css': "@import './nope.css';",
                },
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(String(caught,),).toContain('relative path not found',);
          },
        },),

        it({
          name: 'throws when an import has no string or url() target',
          fn: async () => {
            let caught: unknown;
            try {
              await buildInTempDir({
                files: {
                  'main.css': '@import layer(base);',
                },
              },);
            }
            catch (error) {
              caught = error;
            }
            expect(String(caught,),).toContain('needs a string or url() target',);
          },
        },),

        //endregion Errors
      ],
    },),

    //region build (integration)

    ...integrationChildren,
    //endregion build (integration)
  ],
},);
