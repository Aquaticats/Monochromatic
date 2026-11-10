/**
 Tests for the shared Figma export fixture catalogue.

 The catalogue is test-only support, so this suite imports the package's own
 source module instead of the built bundle: `figma-kiwi`'s public API is the
 parser, and the maintainer's fixture paths must not enter it. The rule that
 requires tests to import built output forgives relative imports of
 `fixture.*` modules for exactly this reason.

 The fixtures themselves are not checked into the repository, so nothing here
 reads them; only the catalogue's own shape and path resolution are asserted,
 which keeps the suite runnable on a machine without the maintainer's exports.

 @module figma-kiwi-export-fixtures-test
 */

import { homedir, } from 'node:os';
import {
  isAbsolute,
  join,
} from 'node:path';

import {
  describe,
  expect,
  expectTypeOf,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import type { FigmaFileType, } from '../dist/final/node/index.mjs';
import {
  type FigmaExportFileType,
  FIGMA_EXPORT_DIR,
  FIGMA_EXPORT_FIXTURES,
  FIGMA_FILE_TYPES,
  figmaExportFixturePath,
} from './fixture.figma-export.ts';

await describe({
  name: '',
  children: [
    //region Fixture directory

    describe({
      name: 'FIGMA_EXPORT_DIR',
      children: [
        it({
          name: 'derives an absolute directory from the home directory instead of naming one',
          fn: async () => {
            const underHome = FIGMA_EXPORT_DIR.startsWith(homedir(),);
            expect(isAbsolute(FIGMA_EXPORT_DIR,),).toBe(true,);
            expect(underHome,).toBe(true,);
          },
        },),
      ],
    },),
    //endregion

    //region Export kinds

    describe({
      name: 'export kinds',
      children: [
        it({
          name: 'names exactly the export kinds the parser reports',
          fn: async () => {
            expectTypeOf<FigmaExportFileType>().toEqualTypeOf<FigmaFileType>();
          },
        },),
      ],
    },),
    //endregion

    //region Fixture catalogue

    describe({
      name: 'FIGMA_EXPORT_FIXTURES',
      children: [
        it({
          name: 'holds one entry for every Figma export kind',
          fn: async () => {
            expect(Object.keys(FIGMA_EXPORT_FIXTURES,),).toEqual([...FIGMA_FILE_TYPES,],);
          },
        },),
        ...FIGMA_FILE_TYPES.map(function mapFileExtension(fileType,) {
          return it({
            name: `names the ${fileType} fixture with the extension its kind implies`,
            fn: async () => {
              expect(
                FIGMA_EXPORT_FIXTURES[fileType].fileName.endsWith(`.${fileType}`,),
              ).toBe(true,);
            },
          },);
        },),
      ],
    },),
    //endregion

    //region Path resolution

    describe({
      name: figmaExportFixturePath.name,
      children: FIGMA_FILE_TYPES.map(function mapFixturePath(fileType,) {
        return it({
          name: `resolves the ${fileType} fixture inside the shared directory`,
          fn: async () => {
            const fixture = FIGMA_EXPORT_FIXTURES[fileType];
            expect(figmaExportFixturePath(fixture,),).toBe(
              join(
                FIGMA_EXPORT_DIR,
                fixture.fileName,
              ),
            );
          },
        },);
      },),
    },),
    //endregion
  ],
},);
