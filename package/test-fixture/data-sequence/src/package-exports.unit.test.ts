/**
 The exports map is the only surface the documented import forms go through.

 Every TSDoc example in this package imports the bare package name, so the manifest
 must declare a package entry for that name. Node answers a bare specifier against
 the `"."` key and throws `ERR_PACKAGE_PATH_NOT_EXPORTED` when the map has none.

 Node resolves these specifiers by self-reference, using this package's own manifest,
 so the check needs no consumer install and runs without starting the fixture's timers.

 @module
 */

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import {
  fileURLToPath,
  pathToFileURL,
} from 'node:url';

import { describe, expect, it, } from '@monochromatic-dev/test-harness-core/ts';

//region Fixture paths

/** Package name the manifest declares and every documented example imports. */
const PACKAGE_NAME = '@monochromatic-dev/test-fixture-data-sequence';

/** Package directory, the scope holding the manifest Node resolves these specifiers against. */
const PACKAGE_ROOT = fileURLToPath(new URL('..', import.meta.url,),);

/** Source directory holding the modules whose TSDoc shows the documented import forms. */
const SOURCE_DIRECTORY = join(PACKAGE_ROOT, 'src',);

/** Barrel module both the package entry and the `/ts` entry name. */
const BARREL_FILE = join(SOURCE_DIRECTORY, 'index.ts',);

/** Suffix that marks a file as a test, which documents no import form of its own. */
const TEST_FILE_MARKER = '.test.';

/** Source extension the documented examples live in. */
const SOURCE_EXTENSION = '.ts';

//endregion Fixture paths

//region Documented import forms

/**
 Collects the import specifiers the TSDoc examples in the source modules name.

 @returns Sorted unique specifiers naming this package, in either bare or subpath form.
 */
async function documentedImportSpecifiers(): Promise<string[]> {
  const specifiers = new Set<string>();
  for (const entry of await readdir(SOURCE_DIRECTORY, { withFileTypes: true, },)) {
    if (!entry.isFile()) continue;
    if (!entry.name.endsWith(SOURCE_EXTENSION,)) continue;
    if (entry.name.includes(TEST_FILE_MARKER,)) continue;

    const moduleText = await readFile(join(SOURCE_DIRECTORY, entry.name,), 'utf8',);
    for (const match of moduleText.matchAll(/from '(?<specifier>[^']+)'/gu,)) {
      const specifier = match.groups?.['specifier'];
      if (specifier === undefined) continue;
      if (specifier.startsWith(`${PACKAGE_NAME}/`,)) {
        specifiers.add(specifier,);
        continue;
      }
      if (specifier === PACKAGE_NAME) specifiers.add(specifier,);
    }
  }
  return [...specifiers,].toSorted();
}

/** Every import form the TSDoc examples show, resolved before any case runs. */
const DOCUMENTED_SPECIFIERS = await documentedImportSpecifiers();

//endregion Documented import forms

await describe({
  name: 'package exports',
  children: [
    //region Declared entries

    it({
      name: 'resolves the bare package name to the barrel module',
      fn: async () => {
        expect(import.meta.resolve(PACKAGE_NAME,),).toBe(
          pathToFileURL(BARREL_FILE,).href,
        );
      },
    },),
    it({
      name: 'resolves the /ts entry to the barrel module',
      fn: async () => {
        expect(import.meta.resolve(`${PACKAGE_NAME}/ts`,),).toBe(
          pathToFileURL(BARREL_FILE,).href,
        );
      },
    },),
    it({
      name: 'resolves a /ts wildcard subpath to the module it names',
      fn: async () => {
        const moduleFile = join(SOURCE_DIRECTORY, 'array.0to999.ts',);
        expect(import.meta.resolve(`${PACKAGE_NAME}/ts/array.0to999.ts`,),).toBe(
          pathToFileURL(moduleFile,).href,
        );
      },
    },),
    it({
      name: 'resolves a subpath below the package root to the module it names',
      fn: async () => {
        const moduleFile = join(SOURCE_DIRECTORY, 'generator.0to999.ts',);
        expect(import.meta.resolve(`${PACKAGE_NAME}/generator.0to999.ts`,),).toBe(
          pathToFileURL(moduleFile,).href,
        );
      },
    },),

    //endregion Declared entries

    //region Documented forms

    it({
      name: 'documents the bare package name, not only subpaths',
      fn: async () => {
        expect(DOCUMENTED_SPECIFIERS,).toContain(PACKAGE_NAME,);
      },
    },),
    it({
      name: 'resolves every documented import form',
      fn: async () => {
        for (const specifier of DOCUMENTED_SPECIFIERS)
          expect(() => import.meta.resolve(specifier,),).not.toThrow();
      },
    },),

    //endregion Documented forms
  ],
},);
