import { join, } from 'node:path';

//region Fixed page path
// WHERE ONE ENTRY'S FIXED ENGLISH PAGE GOES IN THE MIRRORED TREE, split from `publish-fixed.ts` at its line budget.
// `publish-fixed.ts` re-exports these names, so every importer of them keeps its import.

/**
 Directory under a runs dir holding the published corpus tree.

 BESIDE `artifacts/` RATHER THAN ANYWHERE ELSE, which is the property that
 matters rather than the name. A runs dir lives outside this repository, so a
 tree written inside one cannot be committed by accident, and the corpus is
 unlicensed: its wording must never reach git. Rooting the tree here also
 means a throwaway run produces a throwaway tree, so verification never writes
 over anything real.
 */
export const FIXED_TREE_DIR = 'fixed';

/**
 Corpus-relative directory every person entry lives under.

 EXPORTED SO `verify-published.ts` CAN LIST WHAT A RUN PUBLISHED rather than
 only compose one path at a time. A verifier that spelled this name itself
 would report an empty tree as a clean one.

 @internal
 */
export const PEOPLE_DIR = 'people';

/**
 Page file name the corpus gives an entry's English rendering.

 EXPORTED BESIDE {@link PEOPLE_DIR} and for the same reason.

 @internal
 */
export const ENGLISH_PAGE_FILE = 'page.en.md';

/**
 Names where one entry's fixed English page is written.

 MIRRORS THE CORPUS LAYOUT EXACTLY, `people/<id>/page.en.md`, because the
 owner asked for the corpus's directory structure replicated rather than for
 a flat pile named by id. A tree shaped this way is a drop-in set: it can be
 diffed against the corpus directory it mirrors, entry by entry, with nothing
 to translate between the two layouts first.

 @param publishDir - root of the tree being written, holding no corpus text itself

 @param entryId - person entry this page belongs to

 @returns Absolute path this entry's page is written to

 @example
 ```ts
 const path = fixedPagePath({ publishDir, entryId: 'lintong', },);
 ```
 */
export function fixedPagePath(
  {
    publishDir,
    entryId,
  }: {
    readonly publishDir: string;
    readonly entryId: string;
  },
): string {
  return join(
    publishDir,
    PEOPLE_DIR,
    entryId,
    ENGLISH_PAGE_FILE,
  );
}

//endregion Fixed page path
