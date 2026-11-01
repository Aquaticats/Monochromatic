import { REVIEWED_CORPUS_SHA, } from './fidelity-reference-manifest.ts';
import type { FidelityReferenceSpec, } from './fidelity-reference-model.ts';

//region Editor fidelity pool, drawn specifications 2
// Ids, offsets and hashes only; no corpus text. Drawn by the rule in `editor-fidelity-pool-manifest.ts` with seed `round-4`:
// entries `noname` to `Toka_ls` in the order of the draw.

/**
 Specifications 2 of 10 drawn from the pinned corpus for the editor fidelity pool, each a heading section of
 an entry with the hashes of its source, its archive text, its donor and its damaged twins.

 @example
 ```ts
 const drawn: readonly FidelityReferenceSpec[] = DRAWN_EDITOR_SPECS_2;
 ```
 */
export const DRAWN_EDITOR_SPECS_2: readonly FidelityReferenceSpec[] = [
  {
    id: 'noname-s3',
    entryId: 'noname',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 1_753,
      endOffset: 1_999,
      hash: '33572c2e5e53521226c66dde0e5b0f1c9f593c0e69144a4b889c978ec4ee7d75',
    },
    archive: {
      startOffset: 3_799,
      endOffset: 4_521,
      hash: '439b5fbe1ed173468aedd881ea8d35c4dd4d9f2bb1e5737e068e6d51c6dbd0a1',
    },
    referenceHash: '439b5fbe1ed173468aedd881ea8d35c4dd4d9f2bb1e5737e068e6d51c6dbd0a1',
    referenceChars: 722,
    edits: [],
    donor: {
      startOffset: 157,
      endOffset: 357,
      hash: '9d2ac01dfc11ade63dac14ec611843f6dda6675d05aa385d36b2e4acae14c906',
    },
    damages: [
      {
        kind: 'alteration',
        hash: '0cf2bf237f225e3c69e3458c6f020c9664995b67cbb79f304f69b1aeeea17d87',
        changedChars: 4,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'Rentable_A-s1',
    entryId: 'Rentable_A',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 174,
      endOffset: 437,
      hash: '258709f34f6a53c3041a357d94db7b3dece6d370f0c5f9fe6f1871da328bfc42',
    },
    archive: {
      startOffset: 391,
      endOffset: 1_009,
      hash: '1622dfda732ccd1a64a7c9728a6e06f0fab79378612a6ed7a971f40eb247d45d',
    },
    referenceHash: '1622dfda732ccd1a64a7c9728a6e06f0fab79378612a6ed7a971f40eb247d45d',
    referenceChars: 618,
    edits: [],
    donor: {
      startOffset: 1_011,
      endOffset: 1_565,
      hash: '6b9e3a27ef368593c95d90bb795c691e1ded19eac47340a16dd18d5dc596d3c4',
    },
    damages: [
      {
        kind: 'alteration',
        hash: 'b94224e5359b4909d364544b8d2ae5e41937da35efe6412e25e7533abb306cfe',
        changedChars: 4,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'zheermao101-s1',
    entryId: 'zheermao101',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 180,
      endOffset: 422,
      hash: '72edc306caa1de11a73025c860a7890c96df1c51897a174cffa46699d46b66b2',
    },
    archive: {
      startOffset: 391,
      endOffset: 959,
      hash: '7b784b4fc9d09a4a2e3107a5ce2caa9b7e49706f3f2e3b4ef9007b03ebfc3cf2',
    },
    referenceHash: '7b784b4fc9d09a4a2e3107a5ce2caa9b7e49706f3f2e3b4ef9007b03ebfc3cf2',
    referenceChars: 568,
    edits: [],
    donor: {
      startOffset: 961,
      endOffset: 5_650,
      hash: '927d386d0bc0c7ea1355823cb0a7143bd27e923a68e8d3a112472e8b62c98154',
    },
    damages: [
      {
        kind: 'insertion',
        hash: 'f79f0179b15899b5fbbd53cf9b3c5b0594d998aceac92040849ce6557c1bc36b',
        changedChars: 227,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'spike0qy-s1',
    entryId: 'spike0qy',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 147,
      endOffset: 586,
      hash: 'd65dea2fde78c4e5c933797999633b7a8c88ba6062c9c082002b1642bf4763c2',
    },
    archive: {
      startOffset: 320,
      endOffset: 1_752,
      hash: '42757c30f93d2e29ecf170121b67f1d9e78e7aea2cb737c4929879804c57b5f7',
    },
    referenceHash: '42757c30f93d2e29ecf170121b67f1d9e78e7aea2cb737c4929879804c57b5f7',
    referenceChars: 1_432,
    edits: [],
    donor: {
      startOffset: 2_608,
      endOffset: 3_325,
      hash: '59672c1a8b8e8749ae6398530f5c0d874cd2d1b10c1880f027e856c8050f359f',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '991279020aaa76ed6522a0154602123ab3ac3ed323776669eb7f2c820275182f',
        changedChars: 223,
      },
      {
        kind: 'alteration',
        hash: '8aa2383a21a3b9335420d2f8ccfa96505f215538ac6c8961c5925e35c00d4892',
        changedChars: 2,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'ttttsuuukikoo_-s0',
    entryId: 'ttttsuuukikoo_',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 86,
      endOffset: 296,
      hash: '199a2d7f952486b4717a4d3facdfc35c0e4155cb6cd978f613d06b07dba94c31',
    },
    archive: {
      startOffset: 120,
      endOffset: 585,
      hash: '505588f982ec44f6a0a9ba9147b90dc80ccd9a17ccd63189a171dfca4b426fd9',
    },
    referenceHash: '505588f982ec44f6a0a9ba9147b90dc80ccd9a17ccd63189a171dfca4b426fd9',
    referenceChars: 465,
    edits: [],
    donor: {
      startOffset: 589,
      endOffset: 1_588,
      hash: '143ff84d5c0be83065b0f1ca1c25e47f5066f19e8f153e9fe9c7b0680759ac29',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '5830ec829c17ece512d1f49a939d51f20ebb6a3914d40e1a132eb7f7b9427b04',
        changedChars: 63,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'Kotori-s2',
    entryId: 'Kotori',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 485,
      endOffset: 624,
      hash: 'b480ad47654867be63b256342122af1cbcc1756b78894bb5b83abc943d218adb',
    },
    archive: {
      startOffset: 1_780,
      endOffset: 2_190,
      hash: '8db19121d696b358687b4133078479bed0d5666d74f1759baa9d83c23d9a3a5e',
    },
    referenceHash: '8db19121d696b358687b4133078479bed0d5666d74f1759baa9d83c23d9a3a5e',
    referenceChars: 410,
    edits: [],
    donor: {
      startOffset: 2_192,
      endOffset: 3_151,
      hash: '193a7bc457be0490dc987caf186cb0ce2d6e52347d69ad6ffa8243d1da3cba94',
    },
    damages: [
      {
        kind: 'insertion',
        hash: 'c8370b65231e0d9bc7a30f4df937d194ffe79e2ad08cf0c004419b3b09979ccb',
        changedChars: 121,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'Kotori-s1',
    entryId: 'Kotori',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 128,
      endOffset: 483,
      hash: '3c6eaa0aac03a3cc103b4c8ac74175aa42649b3829a4da69f4b0c614295d6bb6',
    },
    archive: {
      startOffset: 268,
      endOffset: 1_778,
      hash: '0f5749da7c13ef15462f1d6fecf5d2cf81d62e6be176021b44ba7dfd477e17a3',
    },
    referenceHash: '0f5749da7c13ef15462f1d6fecf5d2cf81d62e6be176021b44ba7dfd477e17a3',
    referenceChars: 1_510,
    edits: [],
    donor: {
      startOffset: 2_192,
      endOffset: 3_151,
      hash: '193a7bc457be0490dc987caf186cb0ce2d6e52347d69ad6ffa8243d1da3cba94',
    },
    damages: [
      {
        kind: 'alteration',
        hash: 'cb67b5243b5a597ee736d5c8c6d72b55962beb186f5a7efd926378201a8b97b2',
        changedChars: 4,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'Toka_ls-s0',
    entryId: 'Toka_ls',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 57,
      endOffset: 616,
      hash: '99f84f956907844c084423efb6e1ffd979afd2102a8913100522114176ce7847',
    },
    archive: {
      startOffset: 75,
      endOffset: 2_275,
      hash: '4517074ad5f08359f0945d6cdfdaf83b637d9502a3b34d96d4e04c2e593d6115',
    },
    referenceHash: '92891efc36fdfc4fb97e92f10c188eaf4b7e0c83dcb74526e2a8865404e18004',
    referenceChars: 2_197,
    edits: [],
    donor: {
      startOffset: 3_336,
      endOffset: 4_264,
      hash: '6bf170338e8c609a07863bf8d092652a3b1c1e6e9f01238b32aa130dbacd5191',
    },
    damages: [
      {
        kind: 'insertion',
        hash: 'a729d996f7127598cc8510960b04e150b8c7686a835f71789c1861e553e5a332',
        changedChars: 102,
      },
    ],
    reviewedOn: '2026-10-24',
  },
];

//endregion Editor fidelity pool, drawn specifications 2
