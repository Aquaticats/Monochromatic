import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

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
        await writeFile(
          join(
            dir,
            relativePath,
          ),
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

//endregion Test helpers

const integrationChildren = integrationFixtures.map(({ label, dir, },) => {
  /** Path to the main CSS entry point for this fixture */
  const fixtureMainCss = join(dir, 'src', 'main.css',);

  /**
   Builds a distinct output path per (fixture, test). module-test runs the it-blocks within a
   describe concurrently, so a path shared across tests lets one test's cleanup rm the file
   another test is mid-read on (the ENOENT this guards against).
   
   @param testSlug - Short discriminator unique to the calling test
   @returns Absolute output path under dist/ owned solely by that test
   */
  function outputFor(testSlug: string,): string {
    return join(
      import.meta.dirname,
      '..',
      'dist',
      `test-output-${label.replaceAll(' ', '-',)}-${testSlug}.css`,
    );
  }

  return describe({
    name: `buildCss (${label})`,
    children: [
      it({
        name: 'builds fixture CSS with import resolution and mixin expansion',
        fn: async () => {
          const output = outputFor('builds',);
          const result = await buildCss({
            input: fixtureMainCss,
            output,
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
          await rm(output, { force: true, },);
        },
      },),

      it({
        name: 'writes output file to disk',
        fn: async () => {
          const output = outputFor('writes',);
          await buildCss({
            input: fixtureMainCss,
            output,
          },);

          const written = await readFile(output, 'utf8',);
          expect(written,).toContain('--primary: rebeccapurple',);
          await rm(output, { force: true, },);
        },
      },),

      it({
        name: 'expands nested mixin references in build output',
        fn: async () => {
          const output = outputFor('nested',);
          const result = await buildCss({
            input: fixtureMainCss,
            output,
          },);

          // --card uses @apply --flex-center, so .nested-card should have flex styles
          expect(result,).toContain('.nested-card',);
          // The flex-center content should appear inside .nested-card
          const nestedCardMatch = result.slice(result.indexOf('.nested-card',),);
          expect(nestedCardMatch,).toContain('display: flex',);
          await rm(output, { force: true, },);
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
          },
        },),

        //endregion Import inlining

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
