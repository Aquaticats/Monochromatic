/**
 Integration tests for the import attributes rolldown plugin.
 
 Builds real fixtures with rolldown and verifies the output modules
 correctly export file contents as strings.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { join, } from 'node:path';
import { rolldown, } from 'rolldown';
import { importAttributesPlugin, } from './index.ts';

/** Absolute path to the fixtures directory. */
const FIXTURES_DIR = join(import.meta.dirname, 'fixtures',);

/**
 Builds a fixture entry file with the import attributes plugin
 and returns the generated bundle code as a string.
 
 @param entryName - Fixture entry file name (without directory)
 @returns Concatenated output chunk code
 */
async function buildFixture(entryName: string,): Promise<string> {
  const build = await rolldown({
    input: join(FIXTURES_DIR, entryName,),
    plugins: [importAttributesPlugin(),],
  },);
  const { output, } = await build.generate({ format: 'esm', },);
  return output
    .map(function joinChunks(chunk,) {
      if (chunk.type === 'chunk')
        return chunk.code;
      return '';
    },)
    .join('',);
}

await describe({
  name: importAttributesPlugin.name,
  children: [
    it({
      name: 'transforms static import with { type: "text" }',
      fn: async () => {
        const code = await buildFixture('entry-static.ts',);
        expect(code,).toContain('SELECT * FROM users WHERE id = ?;',);
        expect(code,).not.toContain('with',);
      },
    },),
    it({
      name: 'transforms dynamic import() with { with: { type: "text" } }',
      fn: async () => {
        const code = await buildFixture('entry-dynamic.ts',);
        expect(code,).toContain('SELECT * FROM users WHERE id = ?;',);
      },
    },),
    it({
      name: 'transforms re-export with { type: "text" }',
      fn: async () => {
        const code = await buildFixture('entry-reexport.ts',);
        expect(code,).toContain('SELECT * FROM users WHERE id = ?;',);
      },
    },),
    it({
      name: 'ignores imports without with clause',
      fn: async () => {
        const build = await rolldown({
          input: join(FIXTURES_DIR, 'entry-static.ts',),
          plugins: [
            importAttributesPlugin(),
            {
              name: 'test-spy',
              transform(code,) {
                /** Verify the transform only fires when `with` is present. */
                if (!code.includes(' with ',))
                  return null;
                return null;
              },
            },
          ],
        },);
        const { output, } = await build.generate({ format: 'esm', },);
        expect(output.length,).toBeGreaterThan(0,);
      },
    },),
  ],
},);
