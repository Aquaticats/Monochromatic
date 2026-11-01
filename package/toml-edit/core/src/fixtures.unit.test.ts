/**
 Cornerstone fixture coverage for TOML parser data from `toml-eslint-parser`.
 
 Valid fixtures round-trip byte-for-byte in splice mode with the current parser
 stack. Invalid fixtures throw `TomlEditError` on parse.
 
 @module
 */

import {
  readdir,
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';
import { fileURLToPath, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  parseTomlEdit,
  TomlEditError,
  tomlStringify,
} from '@monochromatic-dev/toml-edit-core';

/** Root directory for shared TOML edit fixture data. */
const fixturesRootDir = fileURLToPath(
  new URL('../../../test-fixture/toml-edit/src/', import.meta.url,),
);

/** Directory containing TOML inputs expected to parse and round-trip with the current parser stack. */
const validFixturesDir = join(fixturesRootDir, 'valid',);

/** Directory containing TOML inputs expected to fail parsing with the current parser stack. */
const invalidFixturesDir = join(fixturesRootDir, 'invalid',);

/** Expected count of parser-accepted TOML fixture inputs. */
const EXPECTED_VALID_FIXTURE_COUNT = 91;

/** Expected count of parser-rejected TOML fixture inputs. */
const EXPECTED_INVALID_FIXTURE_COUNT = 108;

/** Valid fixture file names in stable lexical order. */
const validFixtureFiles = (await readdir(validFixturesDir,))
  .filter(function isInput(name,) {
    return name.endsWith('-input.toml',);
  },)
  .toSorted();

/** Invalid fixture file names in stable lexical order. */
const invalidFixtureFiles = (await readdir(invalidFixturesDir,))
  .filter(function isInput(name,) {
    return name.endsWith('-input.toml',);
  },)
  .toSorted();

expect(validFixtureFiles.length,).toBe(EXPECTED_VALID_FIXTURE_COUNT,);
expect(invalidFixtureFiles.length,).toBe(EXPECTED_INVALID_FIXTURE_COUNT,);

await describe({
  name: 'fixtures',
  children: [
    describe({
      name: 'valid round-trip',
      children: validFixtureFiles.map(function makeValidCase(filename,) {
        return it({
          name: filename,
          fn: async () => {
            const source = await readFile(join(validFixturesDir, filename,), 'utf8',);
            const edit = parseTomlEdit({ source, },);
            const text = tomlStringify({ edit, },);
            expect(text,).toBe(source,);
          },
        },);
      },),
    },),
    describe({
      name: 'invalid parse failures',
      children: invalidFixtureFiles.map(function makeInvalidCase(filename,) {
        return it({
          name: filename,
          fn: async () => {
            const source = await readFile(join(invalidFixturesDir, filename,), 'utf8',);
            expect(function parseInvalidFixture() {
              parseTomlEdit({ source, },);
            },)
              .toThrow(TomlEditError,);
          },
        },);
      },),
    },),
  ],
},);
