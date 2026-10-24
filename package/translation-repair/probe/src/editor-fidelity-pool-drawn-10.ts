import { REVIEWED_CORPUS_SHA, } from './fidelity-reference-manifest.ts';
import type { FidelityReferenceSpec, } from './fidelity-reference-model.ts';

//region Editor fidelity pool, drawn specifications 10
// Ids, offsets and hashes only; no corpus text. Drawn by the rule in `editor-fidelity-pool-manifest.ts` with seed `round-4`:
// entries `Proselyte093` to `Acheron` in the order of the draw.

/**
 Specifications 10 of 10 drawn from the pinned corpus for the editor fidelity pool, each a heading section of
 an entry with the hashes of its source, its archive text, its donor and its damaged twins.

 @example
 ```ts
 const drawn: readonly FidelityReferenceSpec[] = DRAWN_EDITOR_SPECS_10;
 ```
 */
export const DRAWN_EDITOR_SPECS_10: readonly FidelityReferenceSpec[] = [
  {
    id: 'Proselyte093-s0',
    entryId: 'Proselyte093',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 71,
      endOffset: 306,
      hash: '060dc41bfec84a00a37c97854941ec0336a3061a40563ad9cf94b1509d3b5c46',
    },
    archive: {
      startOffset: 78,
      endOffset: 608,
      hash: 'cb90bfea49aa5047e4a62dedaa7d0a24d6dd485c4ac24fd0acec134499e03dbf',
    },
    referenceHash: 'cb90bfea49aa5047e4a62dedaa7d0a24d6dd485c4ac24fd0acec134499e03dbf',
    referenceChars: 530,
    edits: [],
    donor: {
      startOffset: 1_739,
      endOffset: 2_753,
      hash: '14a18c4279e1ff08d13fc1ac24abe9d1573832d49dfe231a066f9d31205398c2',
    },
    damages: [
      {
        kind: 'insertion',
        hash: 'bba8c28ac1fe9780a70f441697bae49e2f8d439a0fe9f08d1d4f8782e03c4b15',
        changedChars: 150,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'Proselyte093-s2',
    entryId: 'Proselyte093',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 755,
      endOffset: 1_191,
      hash: 'dfd878b9357a38b5e350c3eb0ba4813ed693da7c3297a66d802e12c00889d05a',
    },
    archive: {
      startOffset: 1_739,
      endOffset: 2_753,
      hash: '14a18c4279e1ff08d13fc1ac24abe9d1573832d49dfe231a066f9d31205398c2',
    },
    referenceHash: '14a18c4279e1ff08d13fc1ac24abe9d1573832d49dfe231a066f9d31205398c2',
    referenceChars: 1_014,
    edits: [],
    donor: {
      startOffset: 78,
      endOffset: 608,
      hash: 'cb90bfea49aa5047e4a62dedaa7d0a24d6dd485c4ac24fd0acec134499e03dbf',
    },
    damages: [
      {
        kind: 'alteration',
        hash: '711ec50b9864cf5f06708ea10f32876cdc8d7c1b68e2deafcef5986da1881b72',
        changedChars: 6,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'SS3B_0016-s0',
    entryId: 'SS3B_0016',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 88,
      endOffset: 309,
      hash: 'e334dcb8d363d3e47d29852a505577bfc81b4247a40957c29b8061e740af3ef2',
    },
    archive: {
      startOffset: 170,
      endOffset: 718,
      hash: '307abc7aff47e150917b8f04c246c2b2a061e33de8fb8874c595382f0ed96205',
    },
    referenceHash: '307abc7aff47e150917b8f04c246c2b2a061e33de8fb8874c595382f0ed96205',
    referenceChars: 548,
    edits: [],
    donor: {
      startOffset: 720,
      endOffset: 2_197,
      hash: 'c8326643fc6175449a1cd9e9b1ee90470b9c36d5074409965b5b13331b985a99',
    },
    damages: [
      {
        kind: 'insertion',
        hash: 'eaff6efb58730898df88fe9d3185dbb2227fe3b047ef3e604543d9c627e565e7',
        changedChars: 148,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'SS3B_0016-s1',
    entryId: 'SS3B_0016',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 311,
      endOffset: 911,
      hash: 'd08ef72082b44e14e67cf6a3dca71aaab7153bb34579999d2fc9dcf984a8d000',
    },
    archive: {
      startOffset: 720,
      endOffset: 2_197,
      hash: 'c8326643fc6175449a1cd9e9b1ee90470b9c36d5074409965b5b13331b985a99',
    },
    referenceHash: 'c8326643fc6175449a1cd9e9b1ee90470b9c36d5074409965b5b13331b985a99',
    referenceChars: 1_477,
    edits: [],
    donor: {
      startOffset: 170,
      endOffset: 718,
      hash: '307abc7aff47e150917b8f04c246c2b2a061e33de8fb8874c595382f0ed96205',
    },
    damages: [
      {
        kind: 'alteration',
        hash: 'a7971f80ac58cd5c970bd1d64c515c96d333f493f55537991f62f37c81d63fc6',
        changedChars: 4,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'zhangyubaka-s1',
    entryId: 'zhangyubaka',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 175,
      endOffset: 835,
      hash: '3bbe1f106bcea8369f6f755d16e215e70eb0a561ccec92a33798cf9c27eece5c',
    },
    archive: {
      startOffset: 348,
      endOffset: 2_104,
      hash: '86d3e4e44d745de36c881b5c35dcc551e6a4046170eaa8fad3e41c49674ce1b5',
    },
    referenceHash: '86d3e4e44d745de36c881b5c35dcc551e6a4046170eaa8fad3e41c49674ce1b5',
    referenceChars: 1_756,
    edits: [],
    donor: {
      startOffset: 4_909,
      endOffset: 8_637,
      hash: 'e65dcbbdf2bbb8cebd7a14f90deb8476988d3431faf1a8c086cc21d49f474a50',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '9d0756bf7c66f7fe190209b2f3ca379c2de39a7fd96ac4eb8a93f8667e0fa1b8',
        changedChars: 232,
      },
      {
        kind: 'alteration',
        hash: '035f5a2c9965560752137b70f4cb284e03a7fc6e980f8b322c525cefe91d8411',
        changedChars: 14,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'Acheron-s0',
    entryId: 'Acheron',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 56,
      endOffset: 219,
      hash: '9f7aa2f2321a826fa994553b27808fbd20b4f8ad841098f2f7b84fa3748ef81b',
    },
    archive: {
      startOffset: 56,
      endOffset: 641,
      hash: 'c05de67059d03ea4b7852302c363333aeeea316bb027dac52b3444ba82c71fe9',
    },
    referenceHash: 'c05de67059d03ea4b7852302c363333aeeea316bb027dac52b3444ba82c71fe9',
    referenceChars: 585,
    edits: [],
    donor: {
      startOffset: 643,
      endOffset: 1_268,
      hash: '184e9aa4180cfb2e13c6324d3ff943d8e00280b036f61c4c6cf1f68ba16e3b02',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '0e89c8911c8f3a8e8ad3219830f4966e58b214e60b4b6e04fe30cf43f36bcb8c',
        changedChars: 107,
      },
      {
        kind: 'alteration',
        hash: '592c58626916dde0dea47486735098282a8a77c74b9299e902ab557eaa55dd39',
        changedChars: 2,
      },
    ],
    reviewedOn: '2026-10-24',
  },
];

//endregion Editor fidelity pool, drawn specifications 10
