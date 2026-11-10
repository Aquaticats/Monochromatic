/**
 Catalogue of the maintainer's personal Figma export fixtures.

 The three files are not checked into the repository, so anything reading them
 can only run on a machine that keeps them under the home directory. This
 module is the single owner of that directory and of the file names, because
 the parser's verification script and the sibling converter's integration tests
 read the same three exports; two independent copies drifted apart once already.

 @module figma-kiwi-export-fixtures
 */

import { homedir, } from 'node:os';
import { join, } from 'node:path';

/**
 Directory holding the hand-curated Figma exports used as integration fixtures.
 
 Derived from `homedir()` so the path stays independent of the `/home` versus
 `/var/home` mount, and never hardcodes a user name or a home layout.
 */
export const FIGMA_EXPORT_DIR: string = join(
  homedir(),
  'Seafile',
  'Plain',
  'Text',
  'Reference',
  'Figma export',
);

/**
 Figma export kinds the catalogue covers.
 
 Mirrors the parser's `FigmaFileType` instead of importing it: this module is
 test-only support, and the rule that governs test-only modules forbids reaching
 package source from them. `fixture.figma-export.unit.test.ts` asserts the two
 unions stay identical, so a new export kind fails that test rather than
 silently leaving the catalogue short.
 */
export type FigmaExportFileType = 'deck' | 'fig' | 'jam';

/**
 One fixture file of the {@link FIGMA_EXPORT_FIXTURES} catalogue.
 */
export type FigmaExportFixture = {
  readonly fileName: string;
};

/**
 Every Figma export kind the parser decodes, in catalogue order.
 */
export const FIGMA_FILE_TYPES: readonly FigmaExportFileType[] = [
  'fig',
  'deck',
  'jam',
];

/**
 Fixture catalogue, keyed by the export kind whose decoding it exercises.
 */
export const FIGMA_EXPORT_FIXTURES: Readonly<Record<FigmaExportFileType, FigmaExportFixture>> = {
  fig: { fileName: 'Color palette - base.fig', },
  deck: { fileName: 'MTM6162-040 participation 2 cover.deck', },
  jam: { fileName: 'Todo app - Brainstorming.jam', },
};

/**
 Resolve one catalogue entry to its absolute path under {@link FIGMA_EXPORT_DIR}.

 Going through the catalogue, rather than joining a file name written at the
 call site, is what keeps a consumer from naming a fixture the catalogue does
 not list.

 @param fixture - catalogue entry to locate

 @returns absolute path of that fixture file

 @example
 ```ts
 const path = figmaExportFixturePath(FIGMA_EXPORT_FIXTURES.fig);
 ```
 */
export function figmaExportFixturePath(fixture: FigmaExportFixture,): string {
  return join(
    FIGMA_EXPORT_DIR,
    fixture.fileName,
  );
}
