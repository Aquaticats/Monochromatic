/**
 Guards on this package's module shape, which no lint rule enforces.

 `AGENTS.md` rule ST2 asks each logical section of a substantial module to sit
 inside a `//region` marker pair naming what the section does. Nothing checks it,
 so every product module of this package is read here and each one longer than
 `SUBSTANTIAL_MODULE_LINE_COUNT` must carry a marker.

 A module that grows past the threshold without its sections named fails this
 test, which is the state the 2026-10-17 review found this package in.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  readProductModules,
  SUBSTANTIAL_MODULE_LINE_COUNT,
  unmarkedSubstantialModules,
} from '@monochromatic-dev/electron-infra-core/ts/region-markers';

/**
 Folder holding this package's product modules, which is this test's own folder.
 */
const SOURCE_DIRECTORY = import.meta.dirname;

await describe({
  name: 'module shape',
  children: [
    it({
      name: 'names every substantial product module with a region marker',
      fn: async () => {
        expect(
          unmarkedSubstantialModules({
            lineCountThreshold: SUBSTANTIAL_MODULE_LINE_COUNT,
            modules: await readProductModules({ directory: SOURCE_DIRECTORY, },),
          },),
        ).toEqual([],);
      },
    },),
  ],
},);
