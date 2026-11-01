import { REVIEWED_CORPUS_SHA, } from './fidelity-reference-manifest.ts';
import type { FidelityReferenceSpec, } from './fidelity-reference-model.ts';

//region Editor fidelity pool, drawn specifications 3
// Ids, offsets and hashes only; no corpus text. Drawn by the rule in `editor-fidelity-pool-manifest.ts` with seed `round-4`:
// entries `Toka_ls` to `CutOceanHeyFis1` in the order of the draw.

/**
 Specifications 3 of 10 drawn from the pinned corpus for the editor fidelity pool, each a heading section of
 an entry with the hashes of its source, its archive text, its donor and its damaged twins.

 @example
 ```ts
 const drawn: readonly FidelityReferenceSpec[] = DRAWN_EDITOR_SPECS_3;
 ```
 */
export const DRAWN_EDITOR_SPECS_3: readonly FidelityReferenceSpec[] = [
  {
    id: 'Toka_ls-s1',
    entryId: 'Toka_ls',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 618,
      endOffset: 1_046,
      hash: '12fef72eddc6048228fd5ee37f5215b4401fbe20b761e79fed5673ed90d72bc4',
    },
    archive: {
      startOffset: 2_277,
      endOffset: 3_334,
      hash: 'efa83810337289101e309d7347b114e889dc371455fd6fa148d3135b544ed027',
    },
    referenceHash: 'efa83810337289101e309d7347b114e889dc371455fd6fa148d3135b544ed027',
    referenceChars: 1_057,
    edits: [],
    donor: {
      startOffset: 3_336,
      endOffset: 4_264,
      hash: '6bf170338e8c609a07863bf8d092652a3b1c1e6e9f01238b32aa130dbacd5191',
    },
    damages: [
      {
        kind: 'alteration',
        hash: '9dd43525c294574e4e86782c5a3ad4667cb298a91e1d68a9bddbcb3dcd2101d8',
        changedChars: 4,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'hakureico-s1',
    entryId: 'hakureico',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 253,
      endOffset: 703,
      hash: '4fed519b5f085e9cdfa32181c38c76bacabf74013bbc01007ff3e4b4db9829fe',
    },
    archive: {
      startOffset: 381,
      endOffset: 1_538,
      hash: '798a02cb35fd8b7e9dec8ac853bdd6460a3b13f9b2b6eaf8d9c2529a6cb1897e',
    },
    referenceHash: '798a02cb35fd8b7e9dec8ac853bdd6460a3b13f9b2b6eaf8d9c2529a6cb1897e',
    referenceChars: 1_157,
    edits: [],
    donor: {
      startOffset: 3_840,
      endOffset: 4_560,
      hash: '8a2b04919774529f1db28bbae830c8196d9cd726b16de439e76a0bb000eab7c0',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '786f2e0739f5222b906bd940bb3f86f4c1cc9eac71292e883cbf154c9cafc097',
        changedChars: 52,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'hakureico-s3',
    entryId: 'hakureico',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 1_510,
      endOffset: 1_957,
      hash: '31373db5b59c03c98fe2a885638a64adfda0d56acd2dbca619423a81eacc8004',
    },
    archive: {
      startOffset: 3_840,
      endOffset: 4_560,
      hash: '8a2b04919774529f1db28bbae830c8196d9cd726b16de439e76a0bb000eab7c0',
    },
    referenceHash: '8a2b04919774529f1db28bbae830c8196d9cd726b16de439e76a0bb000eab7c0',
    referenceChars: 720,
    edits: [],
    donor: {
      startOffset: 167,
      endOffset: 379,
      hash: 'eb648964775fc3d0f12f49c7b7cda4caf67421301cfb177345bf1c62b7cc8bb8',
    },
    damages: [
      {
        kind: 'alteration',
        hash: '2df0320a5a9134adc39a1d6504efd15332d4f0bc40cb11e9aa30da80bb99777a',
        changedChars: 2,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'interrgned-s0',
    entryId: 'interrgned',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 96,
      endOffset: 409,
      hash: '5f52137a54516c910f960a028ce94dbb037ed03c8ab55f8ee8fbbec4173360cd',
    },
    archive: {
      startOffset: 128,
      endOffset: 714,
      hash: '6523b7d168abd82baaa6359f46a64e572a67989ea9d433e6b93cb5cb8a432616',
    },
    referenceHash: '6523b7d168abd82baaa6359f46a64e572a67989ea9d433e6b93cb5cb8a432616',
    referenceChars: 586,
    edits: [],
    donor: {
      startOffset: 12_040,
      endOffset: 13_982,
      hash: '4176bf31d71d1707246905c3c2a1c7b74c33b187f66d961fcd86c613c45b9add',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '1fe9c2b4733570dabffe489c743958815ae5788b222ef1757b0451086acb7a27',
        changedChars: 113,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'interrgned-s4',
    entryId: 'interrgned',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 4_011,
      endOffset: 4_704,
      hash: '1f5cb1d50aaaa38ded85bf355801db6c53e0029144c848162cdf3d7070de8e05',
    },
    archive: {
      startOffset: 12_040,
      endOffset: 13_982,
      hash: '4176bf31d71d1707246905c3c2a1c7b74c33b187f66d961fcd86c613c45b9add',
    },
    referenceHash: '4176bf31d71d1707246905c3c2a1c7b74c33b187f66d961fcd86c613c45b9add',
    referenceChars: 1_942,
    edits: [],
    donor: {
      startOffset: 128,
      endOffset: 714,
      hash: '6523b7d168abd82baaa6359f46a64e572a67989ea9d433e6b93cb5cb8a432616',
    },
    damages: [
      {
        kind: 'alteration',
        hash: 'f70532073ac6b6a57ad20da3f5360089e26e5b0ea32bf4bbbfe435a2cdbcb056',
        changedChars: 2,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'xuewulihuameng-s1',
    entryId: 'xuewulihuameng',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 177,
      endOffset: 511,
      hash: 'dfcc2e0a762515747b2df6b6a4695ec187b451afe712c719d0cdf102de588d63',
    },
    archive: {
      startOffset: 354,
      endOffset: 1_041,
      hash: '254923faebaeac7b0cbd7ffcc4a6a2f392dd7027530a8ee897a78904ef61e92e',
    },
    referenceHash: '254923faebaeac7b0cbd7ffcc4a6a2f392dd7027530a8ee897a78904ef61e92e',
    referenceChars: 687,
    edits: [],
    donor: {
      startOffset: 2_097,
      endOffset: 2_563,
      hash: '91a9de97734694026493a0a7268f8e1b69416ef16e1809073fa9f6b47416c4f1',
    },
    damages: [
      {
        kind: 'insertion',
        hash: 'b932611e98eb7a1a8809a0948531a14daa90c6858bf1ae810e93032b1d324e7d',
        changedChars: 137,
      },
      {
        kind: 'alteration',
        hash: '6859702dbc0345a01f3a96b447f5b5f923c1f532091cc0d5a20b01b03ed39dd9',
        changedChars: 9,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'CutOceanHeyFis1-s1',
    entryId: 'CutOceanHeyFis1',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 75,
      endOffset: 293,
      hash: '63fb4008d41ed08eb7e8a6593d874937273f1bfaa8fa9f108bd7674a83cf2f17',
    },
    archive: {
      startOffset: 92,
      endOffset: 715,
      hash: '5e932c7e8481e9f019729fb761ecce9f01cfb037d67ce44287abef22367c42bf',
    },
    referenceHash: '5e932c7e8481e9f019729fb761ecce9f01cfb037d67ce44287abef22367c42bf',
    referenceChars: 623,
    edits: [],
    donor: {
      startOffset: 717,
      endOffset: 1_206,
      hash: '6a8c95bf222a546944d41694bf574ff2d46723b2fe5a51e6eae9d8f9b510b3d1',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '30def004c765612bf24c6419055c5d63f7a773db35106fce0e2c1ecf70201712',
        changedChars: 112,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'CutOceanHeyFis1-s2',
    entryId: 'CutOceanHeyFis1',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 295,
      endOffset: 442,
      hash: 'c81e64a8992debf60472c3774a9d2c5f2243db90721f7dffdd0c61328da9f304',
    },
    archive: {
      startOffset: 717,
      endOffset: 1_206,
      hash: '6a8c95bf222a546944d41694bf574ff2d46723b2fe5a51e6eae9d8f9b510b3d1',
    },
    referenceHash: '6a8c95bf222a546944d41694bf574ff2d46723b2fe5a51e6eae9d8f9b510b3d1',
    referenceChars: 489,
    edits: [],
    donor: {
      startOffset: 66,
      endOffset: 90,
      hash: '1adfb2e47efccdecbf94e7b4d3ec49ba5f794cf220f7e91f9182839b47edf4ec',
    },
    damages: [
      {
        kind: 'alteration',
        hash: 'd9c631fcd5f795bd25ff7f42a5b759f3a79e4daa9e9c77488de16f8c15ae5af3',
        changedChars: 4,
      },
    ],
    reviewedOn: '2026-10-24',
  },
];

//endregion Editor fidelity pool, drawn specifications 3
