/**
 Tests for the catalog parser.
 
 Each test writes a fixture `pnpm-workspace.yaml` under a fresh
 temp directory and calls `readCatalog({ startDir })` so the
 upward search is constrained to that temp tree.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import dedent from 'string-dedent';

import {
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  decodeAlias,
  readCatalog,
  type CatalogEntry,
} from '../dist/final/node/index.mjs';

/**
 Allocates a fresh temp directory and returns it with an
 async-disposable that recursively removes it on scope exit.
 
 @returns Tuple of root path and an async-disposable cleanup.
 */
async function tempWorkspace(): Promise<{
  dir: string;
  [Symbol.asyncDispose]: () => Promise<void>;
}> {
  const dir = await mkdtemp(join(scratchRoot(), 'deps-cube-catalog-',),);
  return {
    dir,
    [Symbol.asyncDispose]: async function dispose() {
      await rm(dir, { recursive: true, force: true, },);
    },
  };
}

await describe({
  name: 'catalog',
  children: [
    //region decodeAlias
    it({
      name: 'decodeAlias passes through plain ranges',
      fn: async () => {
        expect(decodeAlias({ key: 'preact', value: '^10.26.0', },),).toEqual({
          npmName: 'preact',
          range: '^10.26.0',
        },);
      },
    },),

    it({
      name: 'decodeAlias resolves npm: alias with explicit @range',
      fn: async () => {
        expect(decodeAlias({ key: 'local-alias', value: 'npm:aliased-target@0.8.0', },),)
          .toEqual({
            npmName: 'aliased-target',
            range: '0.8.0',
          },);
      },
    },),

    it({
      name: 'decodeAlias resolves scoped npm: alias keeping the @scope prefix',
      fn: async () => {
        expect(decodeAlias({ key: 'alias', value: 'npm:@scope/name@1.0.0', },),).toEqual({
          npmName: '@scope/name',
          range: '1.0.0',
        },);
      },
    },),

    it({
      name: 'decodeAlias falls back to range "*" when no @range trails',
      fn: async () => {
        expect(decodeAlias({ key: 'alias', value: 'npm:somepkg', },),).toEqual({
          npmName: 'somepkg',
          range: '*',
        },);
      },
    },),

    it({
      name: 'decodeAlias rejects traversal-shaped alias targets',
      fn: async () => {
        /**
         Captured failure from an alias target that could escape node_modules.
         */
        let caught: unknown;
        try {
          decodeAlias({ key: 'alias', value: 'npm:../../outside@1.0.0', },);
        }
        catch (error) {
          caught = error;
        }
        expect(caught,).toBeInstanceOf(Error,);
        expect(String(caught,),).toContain('Invalid npm alias target',);
      },
    },),
    //endregion decodeAlias

    //region readCatalog
    it({
      name: 'readCatalog reads the default catalog: block',
      fn: async () => {
        await using temp = await tempWorkspace();
        await writeFile(
          join(temp.dir, 'pnpm-workspace.yaml',),
          dedent`
            packages:
              - 'package/*'
            catalog:
              preact: ^10.26.0
              react: ^19.0.0
          `,
          'utf8',
        );
        const entries = await readCatalog({ startDir: temp.dir, },);
        expect(entries.length,).toBe(2,);
        expect(entries[0],).toMatchObject({
          catalogKey: 'preact',
          npmName: 'preact',
          range: '^10.26.0',
        },);
        expect(entries[0]?.catalogName,).toBeUndefined();
        expect(entries[1],).toMatchObject({
          catalogKey: 'react',
          npmName: 'react',
          range: '^19.0.0',
        },);
        expect(entries[1]?.catalogName,).toBeUndefined();
      },
    },),

    it({
      name: 'readCatalog reads named catalogs: blocks alongside the default',
      fn: async () => {
        await using temp = await tempWorkspace();
        await writeFile(
          join(temp.dir, 'pnpm-workspace.yaml',),
          dedent`
            catalog:
              preact: ^10.26.0
            catalogs:
              react18:
                react: ^18.0.0
              react19:
                react: ^19.0.0
          `,
          'utf8',
        );
        const entries = await readCatalog({ startDir: temp.dir, },);
        expect(entries.length,).toBe(3,);
        const namedCatalogs: readonly string[] = entries
          .flatMap(function pluckName(e: CatalogEntry,) {
            return e.catalogName === undefined ? [] : [e.catalogName,];
          },)
          .toSorted(function alphabetical(
            a: string,
            b: string,
          ) {
            return a.localeCompare(b,);
          },);
        expect(namedCatalogs,).toEqual([
          'react18',
          'react19',
        ],);
      },
    },),

    it({
      name: 'readCatalog decodes npm: aliases inside catalog entries',
      fn: async () => {
        await using temp = await tempWorkspace();
        await writeFile(
          join(temp.dir, 'pnpm-workspace.yaml',),
          dedent`
            catalog:
              'local-alias': 'npm:aliased-target@0.8.0'
          `,
          'utf8',
        );
        const entries = await readCatalog({ startDir: temp.dir, },);
        expect(entries.length,).toBe(1,);
        expect(entries[0],).toMatchObject({
          catalogKey: 'local-alias',
          npmName: 'aliased-target',
          range: '0.8.0',
        },);
      },
    },),

    it({
      name: 'readCatalog throws when pnpm-workspace.yaml is absent up the tree',
      fn: async () => {
        await using temp = await tempWorkspace();
        const caught = await (async function captureError() {
          try {
            await readCatalog({ startDir: temp.dir, },);
            return null;
          }
          catch (err) {
            return err;
          }
        })();
        expect(caught,).toBeInstanceOf(Error,);
        /** Error message lower-cased for case-insensitive substring checks below. */
        const msgLower = String(caught,).toLowerCase();
        expect(
          msgLower.includes('pnpm-workspace.yaml',) || msgLower.includes('locate',),
        )
          .toBe(true,);
      },
    },),

    it({
      name: 'readCatalog throws when both catalog blocks are empty',
      fn: async () => {
        await using temp = await tempWorkspace();
        await writeFile(
          join(temp.dir, 'pnpm-workspace.yaml',),
          dedent`
            packages:
              - 'package/*'
          `,
          'utf8',
        );
        const caught = await (async function captureError() {
          try {
            await readCatalog({ startDir: temp.dir, },);
            return null;
          }
          catch (err) {
            return err;
          }
        })();
        expect(caught,).toBeInstanceOf(Error,);
        expect(String(caught,).toLowerCase(),).toContain('catalog',);
      },
    },),
    //endregion readCatalog
  ],
},);
