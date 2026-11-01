/**
 Tests final publication contributor identity boundary.

 Fixtures are invented and contain no corpus wording.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  assertContributorNamesComplete,
  ContributorCompletenessError,
} from '../../dist/final/node/index.mjs';

/**
 Archive page establishing target contributor public handles.
 */
const ARCHIVE = 'A cat slept.\n\nContributors for this entry: One Body, [Snow](https://example.test/snow)\n';

await describe({
  name: assertContributorNamesComplete.name,
  children: [
    it({
      name: 'ACCEPTS TARGET CONTRIBUTORS across line reflow and punctuation',
      fn: async () => {
        expect(() => assertContributorNamesComplete({
          entryId: 'CatEntry',
          archiveText: ARCHIVE,
          pageText: 'A cat slept.\n\nContributors for this entry:\nOne Body,\n[Snow](https://example.test/snow)\n',
        },),).not.toThrow();
      },
    },),

    it({
      name: 'REFUSES LITERAL SOURCE-SIDE RENAMING of target contributor handles',
      fn: async () => {
        expect(() => assertContributorNamesComplete({
          entryId: 'CatEntry',
          archiveText: ARCHIVE,
          pageText: 'A cat slept.\n\nContributors for this entry: One Body, Snowflake\n',
        },),).toThrow(ContributorCompletenessError,);
      },
    },),

    it({
      name: 'IGNORES PAGE WITHOUT ARCHIVE CONTRIBUTOR DECLARATION',
      fn: async () => {
        expect(() => assertContributorNamesComplete({
          entryId: 'CatEntry',
          archiveText: 'A cat slept.\n',
          pageText: 'The cat slept.\n',
        },),).not.toThrow();
      },
    },),
  ],
},);
