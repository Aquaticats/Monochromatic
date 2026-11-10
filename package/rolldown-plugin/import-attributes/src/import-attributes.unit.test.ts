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
import { readFile, } from 'node:fs/promises';
import { join, } from 'node:path';
import {
  rolldown,
  type Plugin,
  type RolldownOutput,
} from 'rolldown';

import {
  ATTR_QUERY_KEY,
  importAttributesPlugin,
} from '../dist/final/node/index.mjs';

/** Absolute path to the fixtures directory. */
const FIXTURES_DIR = join(import.meta.dirname, 'fixture',);

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
  return joinChunkCode(output,);
}

/**
 Concatenates the code of every chunk in a rolldown output bundle,
 skipping assets, so a caller can assert on everything the browser or runtime loads.
 
 @param output - Output bundle produced by `generate`
 @returns Concatenated chunk code
 */
function joinChunkCode(output: RolldownOutput['output'],): string {
  return output
    .map(function chunkCode(chunk,): string {
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
      name: 'leaves an import without an attribute clause untouched',
      fn: async () => {
        const entryPath = join(FIXTURES_DIR, 'entry-noattr.ts',);
        /**
         Transform inputs a following plugin observed, so the case can prove
         the import attributes plugin changed nothing on its way through.
         */
        const seenByFollower: { readonly id: string; readonly code: string; }[] = [];
        /**
         Plugin placed after {@link importAttributesPlugin}, recording the code
         each module carries once the import attributes plugin has run.
         */
        const follower: Plugin = {
          name: 'record-transform-input',
          transform(code, id,): null {
            seenByFollower.push({ id, code, },);
            return null;
          },
        };

        const build = await rolldown({
          input: entryPath,
          plugins: [importAttributesPlugin(), follower,],
        },);
        const { output, } = await build.generate({ format: 'esm', },);
        const code = joinChunkCode(output,);

        /** The build really ran and inlined the plain module. */
        expect(code,).toContain('plain module text',);
        /**
         An untagged specifier never gains the attribute marker, so no handler
         loaded this module on the plugin's behalf.
         */
        expect(code,).not.toContain(ATTR_QUERY_KEY,);
        expect(code,).not.toContain('with',);
        /**
         The entry reaches the following plugin byte-identical to its file.
         */
        const entrySource = await readFile(entryPath, 'utf8',);
        expect(seenByFollower.filter(function byEntry(seen,) {
          return seen.id === entryPath;
        },)
          .map(function toCode(seen,) {
            return seen.code;
          },),).toEqual([entrySource,],);
      },
    },),
  ],
},);
