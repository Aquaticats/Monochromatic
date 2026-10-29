/**
 Unit tests for located workspace catalog reading.
 
 @module
 */

import {
  mkdtemp,
  rm,
  mkdir,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  readCatalogFile,
} from '../dist/final/node/index.mjs';

/**
 Temporary workspace fixture with automatic recursive cleanup.
 
 @returns disposable temporary directory
 
 @example
 ```ts
 await using workspace = await createWorkspace();
 ```
 */
async function createWorkspace(): Promise<{
  dir: string;
  [Symbol.asyncDispose]: () => Promise<void>;
}> {
  /**
   Temporary root whose name cannot collide with another test fixture.
   */
  const dir = await mkdtemp(join(scratchRoot(), 'pnpm-workspace-catalog-',),);
  return {
    dir,
    [Symbol.asyncDispose]: async function dispose(): Promise<void> {
      await rm(dir, { recursive: true, force: true, },);
    },
  };
}

await describe({
  name: readCatalogFile.name,
  children: [
    it({
      name: 'finds the workspace file from a nested starting directory',
      fn: async () => {
        await using workspace = await createWorkspace();
        /**
         Nested directory that starts the upward search.
         */
        const nested = join(workspace.dir, 'package', 'app',);
        await mkdir(nested, { recursive: true, },);
        /**
         Exact source text used to verify raw-content preservation.
         */
        const content = [
          'packages:',
          "  - 'package/*'",
          'catalog:',
          "  'oxlint': '>=1.71.0'",
          '',
        ].join('\n',);
        await writeFile(
          join(workspace.dir, 'pnpm-workspace.yaml',),
          content,
          'utf8',
        );
        /**
         Located result returned from the nested search.
         */
        const result = await readCatalogFile({ startDir: nested, },);
        expect(result.path,).toBe(join(workspace.dir, 'pnpm-workspace.yaml',),);
        expect(result.content,).toBe(content,);
        expect(result.catalogs.defaultCatalog.oxlint,).toBe('>=1.71.0',);
      },
    },),

    it({
      name: 'throws when no workspace file exists in the search chain',
      fn: async () => {
        await using workspace = await createWorkspace();
        /**
         Captured failure from a search that cannot find the workspace file.
         */
        let caught: unknown;
        try {
          await readCatalogFile({ startDir: workspace.dir, },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
        expect(String(caught,),).toContain('pnpm-workspace.yaml',);
      },
    },),
  ],
},);
