/**
 Tests for the filter every reader applies between a directory listing and
 the artifacts it opens.

 WHAT IT DECIDES. A reader lists the artifacts directory itself, asks the pool
 which entries a draw may use, and then opens only the files those entries
 own. `keepEligible` is that last step, and nothing tested it directly: four
 readers call it, and every one of them reached the suite only through a real
 pool over a throwaway directory, where an eligible entry and a malformed one
 happened never to sit beside an excluded one.

 MALFORMED FILES ARE KEPT, which is the half worth pinning. The reader
 downstream is the one that reports a corrupt artifact, so dropping it here
 would make the file vanish from the failure list instead of appearing on it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type EligibleEntries,
  keepEligible,
} from '../../dist/final/node/index.mjs';

/**
 Characters in a SHA-1 object id.
 */
const OBJECT_ID_LENGTH = 40;

/**
 Hex characters in the digest scheme this build writes.
 */
const DIGEST_HEX_LENGTH = 64;

/**
 Commit every pooled fixture records.
 */
const TIP = 'a'.repeat(OBJECT_ID_LENGTH,);

/**
 Built pipeline every pooled fixture records.
 */
const DIGEST = `sha256-tree-v1:${'c'.repeat(DIGEST_HEX_LENGTH,)}`;

/**
 Pool admitting two entries, excluding one, and carrying one malformed file.
 */
const POOL: EligibleEntries = {
  entryIds: [
    'mittens',
    'tabby',
  ],
  excludedIds: ['whiskers',],
  malformedIds: ['smudge',],
  tipByEntry: new Map([
    [
      'mittens',
      TIP,
    ],
    [
      'tabby',
      TIP,
    ],
  ],),
  digestByEntry: new Map([
    [
      'mittens',
      DIGEST,
    ],
    [
      'tabby',
      DIGEST,
    ],
  ],),
  selection: {
    kind: 'single-generation',
    digest: DIGEST,
  },
  report: [],
};

await describe({
  name: keepEligible.name,
  children: [
    it({
      name: 'KEEPS the artifacts of pooled entries in the order they were '
        + 'listed, which is the control the dropping cases depart from',
      fn: async () => {
        expect(keepEligible({
          names: [
            'tabby.json',
            'mittens.json',
          ],
          eligible: POOL,
        },),)
          .toEqual([
            'tabby.json',
            'mittens.json',
          ],);
      },
    },),

    it({
      name: 'DROPS the artifact of an entry the pool excluded, since an '
        + 'excluded entry is exactly one a rate must not divide by',
      fn: async () => {
        expect(keepEligible({
          names: ['whiskers.json',],
          eligible: POOL,
        },),)
          .toEqual([],);
      },
    },),

    it({
      name: 'CARRIES A MALFORMED ARTIFACT THROUGH, so the reader that reports '
        + 'malformed files still sees it instead of the file vanishing from '
        + 'the failure list',
      fn: async () => {
        expect(keepEligible({
          names: ['smudge.json',],
          eligible: POOL,
        },),)
          .toEqual(['smudge.json',],);
      },
    },),

    it({
      name: 'DROPS a file no list names at all, since the pool never placed it',
      fn: async () => {
        expect(keepEligible({
          names: ['patches.json',],
          eligible: POOL,
        },),)
          .toEqual([],);
      },
    },),

    it({
      name: 'KEEPS AND DROPS IN ONE LISTING, preserving the order of what it '
        + 'keeps',
      fn: async () => {
        expect(keepEligible({
          names: [
            'tabby.json',
            'whiskers.json',
            'smudge.json',
            'patches.json',
            'mittens.json',
          ],
          eligible: POOL,
        },),)
          .toEqual([
            'tabby.json',
            'smudge.json',
            'mittens.json',
          ],);
      },
    },),
  ],
},);
