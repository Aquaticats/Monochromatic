/**
 Test-only outside reads for a preparation (ledger X19): nothing from the
 web and nothing from the pinned corpus, so a test that prepares an original
 naming a title or linking a page buys no search with the key the suite
 inherits from `mise`, writes no real cache, and does not depend on the
 corpus being checked out. A case that needs a reader to answer overrides
 that one reader by spreading this object.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import type { PassOutsideReads, } from '../../dist/final/node/index.mjs';

/**
 Outside reads that read nothing: no work-title evidence, no references and
 no names from other entries.
 */
export const NO_OUTSIDE_READS: PassOutsideReads = {
  workTitles: function noWorkTitles(): Promise<readonly string[]> {
    return Promise.resolve([],);
  },
  references: function noReferences(): Promise<string> {
    return Promise.resolve('',);
  },
  corpusNames: function noCorpusNames() {
    return Promise.resolve([],);
  },
  entryInfo: function noEntryInfo() {
    return Promise.resolve({ kind: 'absent', } as const,);
  },
};
