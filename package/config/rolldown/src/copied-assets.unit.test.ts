import {
  mkdtempSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { rolldown, } from 'rolldown';

import {
  type CopiedAsset,
  copiedAssetsPlugin,
} from './copied-assets.ts';

/**
 Disposable entry-source and asset-source fixture directory.
 */
type CopyBuildFixture = {
  readonly path: string;
  [Symbol.dispose]: () => void;
};

/**
 Bytes that are not valid UTF-8, so a copy that decoded and re-encoded the
 file as text would change them.
 */
const NON_TEXT_BYTES = Uint8Array.of(0xFF, 0xFE, 0x00, 0x7B, 0x0D, 0x0A, 0xC3,);

/**
 Creates a disposable directory holding one entry source.

 @returns Disposable fixture outside repository state.

 @example
 ```ts
 using fixture = createCopyBuildFixture();
 ```
 */
function createCopyBuildFixture(): CopyBuildFixture {
  /**
   Fresh fixture directory under the system temp root.
   */
  const path = mkdtempSync(join(scratchRoot(), 'copied-assets-',),);
  writeFileSync(join(path, 'entry.ts',), 'export const cat = 1;\n',);
  return {
    path,
    [Symbol.dispose]: function removeFixture(): void {
      rmSync(path, {
        force: true,
        recursive: true,
      },);
    },
  };
}

/**
 Build the fixture entry through the plugin under test, writing to
 `out` inside the fixture.

 @param packageDir - Fixture directory holding entry.ts.

 @param files - Files the plugin copies.

 @returns Output directory the build wrote.

 @example
 ```ts
 const outputDir = await buildWithCopies({ packageDir: fixture.path, files: [] });
 ```
 */
async function buildWithCopies({ packageDir, files, }: {
  readonly packageDir: string;
  readonly files: readonly CopiedAsset[];
},): Promise<string> {
  /**
   Real rolldown build of the fixture entry.
   */
  const build = await rolldown({
    input: [join(packageDir, 'entry.ts',),],
    plugins: [copiedAssetsPlugin({ files, },),],
  },);
  /**
   Directory the build writes into.
   */
  const outputDir = join(packageDir, 'out',);
  await build.write({
    dir: outputDir,
    format: 'es',
    entryFileNames: '[name].mjs',
  },);
  return outputDir;
}

await describe({
  name: copiedAssetsPlugin.name,
  children: [
    it({
      name: 'emits each file under its output-relative name with the source bytes unchanged',
      fn: async function emitsNamedBytes(): Promise<void> {
        using fixture = createCopyBuildFixture();
        /**
         Source file holding bytes that are not text.
         */
        const source = join(fixture.path, 'data.bin',);
        writeFileSync(source, NON_TEXT_BYTES,);
        /**
         Second source file holding text, emitted under a nested name.
         */
        const textSource = join(fixture.path, 'rules.jsonc',);
        writeFileSync(textSource, '{ "rule": "cat" }\n',);
        /**
         Directory the build wrote.
         */
        const outputDir = await buildWithCopies({
          packageDir: fixture.path,
          files: [
            { source, fileName: 'copy.bin', },
            { source: textSource, fileName: 'nested/rules.jsonc', },
          ],
        },);
        /**
         Bytes of the emitted binary copy.
         */
        const copied = readFileSync(join(outputDir, 'copy.bin',),);
        expect(new Uint8Array(copied,),).toEqual(NON_TEXT_BYTES,);
        expect(readFileSync(join(outputDir, 'nested', 'rules.jsonc',), 'utf8',),).toBe('{ "rule": "cat" }\n',);
        expect(readFileSync(join(outputDir, 'entry.mjs',), 'utf8',),).toContain('cat',);
      },
    },),
    it({
      name: 'emits nothing extra when no files are listed',
      fn: async function emitsNothingForEmptyList(): Promise<void> {
        using fixture = createCopyBuildFixture();
        /**
         Directory the build wrote.
         */
        const outputDir = await buildWithCopies({ packageDir: fixture.path, files: [], },);
        expect(readFileSync(join(outputDir, 'entry.mjs',), 'utf8',),).toContain('cat',);
      },
    },),
    it({
      name: 'fails the build naming the path when a source file is missing',
      fn: async function failsOnMissingSource(): Promise<void> {
        using fixture = createCopyBuildFixture();
        /**
         Path no file occupies.
         */
        const source = join(fixture.path, 'missing.jsonc',);
        await expect(buildWithCopies({
          packageDir: fixture.path,
          files: [{ source, fileName: 'missing.jsonc', },],
        },),)
          .rejects
          .toThrow(source,);
      },
    },),
  ],
},);
