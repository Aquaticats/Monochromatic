import { REVIEWED_CORPUS_SHA, } from './fidelity-reference-manifest.ts';
import type { FidelityReferenceSpec, } from './fidelity-reference-model.ts';

//region Editor fidelity pool, drawn specifications 1
// Ids, offsets and hashes only; no corpus text. Drawn by the rule in `editor-fidelity-pool-manifest.ts` with seed `round-4`:
// entries `yuki418330012` to `noname` in the order of the draw.

/**
 Specifications 1 of 10 drawn from the pinned corpus for the editor fidelity pool, each a heading section of
 an entry with the hashes of its source, its archive text, its donor and its damaged twins.

 @example
 ```ts
 const drawn: readonly FidelityReferenceSpec[] = DRAWN_EDITOR_SPECS_1;
 ```
 */
export const DRAWN_EDITOR_SPECS_1: readonly FidelityReferenceSpec[] = [
  {
    id: 'yuki418330012-s0',
    entryId: 'yuki418330012',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 76,
      endOffset: 273,
      hash: '44c95adff1e906546f32c694d206df30402d43217252324a92a8340e8ce0d793',
    },
    archive: {
      startOffset: 110,
      endOffset: 830,
      hash: '62e196a2fb4ef90e64dac0f700f5c568a1cbcd2c59f2379d2093cd04a8e706ab',
    },
    referenceHash: '62e196a2fb4ef90e64dac0f700f5c568a1cbcd2c59f2379d2093cd04a8e706ab',
    referenceChars: 720,
    edits: [],
    donor: {
      startOffset: 832,
      endOffset: 5_117,
      hash: 'fc6010df1b0f397fc2411fd6783ecc7a80a4b2af1c3b7032133c956339eb06ae',
    },
    damages: [
      {
        kind: 'insertion',
        hash: 'f535d619b1ba64d48f89518e8ce8b1d4f7b1dd998c3fcc4b95b9e543029997b5',
        changedChars: 189,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'aiyysk-s0',
    entryId: 'aiyysk',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 84,
      endOffset: 333,
      hash: '35c8ec44ea8a779b9cc541366e8345d39588b57c995aa79564d7b7d5e52ef9d9',
    },
    archive: {
      startOffset: 153,
      endOffset: 917,
      hash: 'a49638d39a304ab1942f3c6081f6ada22a96e9dcf40411a31a5227edd755cbbe',
    },
    referenceHash: 'a49638d39a304ab1942f3c6081f6ada22a96e9dcf40411a31a5227edd755cbbe',
    referenceChars: 764,
    edits: [],
    donor: {
      startOffset: 25_084,
      endOffset: 25_700,
      hash: 'dcedfdf5c57b227643b165c880556d0379f5578bb50762e82ba8ebf7732fae9c',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '3fab28bde4754d1705754f29b39ae46154975e48b90ce7feda9fc0085ce3e078',
        changedChars: 208,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'aiyysk-s3',
    entryId: 'aiyysk',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 981,
      endOffset: 1_430,
      hash: '303832d722139d90fd02efa9e4e878e6e28eac509ffa0942cea88b8026169fb1',
    },
    archive: {
      startOffset: 3_442,
      endOffset: 5_310,
      hash: '7f8e3d47a841b87fce192c92f0a24d5697c870919982f9be3f5baaa59d14853b',
    },
    referenceHash: '7f8e3d47a841b87fce192c92f0a24d5697c870919982f9be3f5baaa59d14853b',
    referenceChars: 1_868,
    edits: [],
    donor: {
      startOffset: 25_084,
      endOffset: 25_700,
      hash: 'dcedfdf5c57b227643b165c880556d0379f5578bb50762e82ba8ebf7732fae9c',
    },
    damages: [
      {
        kind: 'alteration',
        hash: '71c66b8f8b72b89b56eb1db8699a1dabe39cf818c45041591da9950faf75bbad',
        changedChars: 4,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'Considerate_cat-s3',
    entryId: 'Considerate_cat',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 352,
      endOffset: 625,
      hash: '7c1576ec0b2593a7d43d88a92b94b44ef5a8ac3038203ad578cb97a7d4beec92',
    },
    archive: {
      startOffset: 870,
      endOffset: 1_583,
      hash: 'f53929f9e9127dfe5986bed7a5e5b67254318d1fb9919cd5053b1fb309be2da1',
    },
    referenceHash: 'f53929f9e9127dfe5986bed7a5e5b67254318d1fb9919cd5053b1fb309be2da1',
    referenceChars: 713,
    edits: [],
    donor: {
      startOffset: 2_952,
      endOffset: 3_652,
      hash: 'ae8ccaaa6d594e13a057b2e18091d9788b32ea1235843f655f6d83d04dca1aae',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '73957a83f64d64bcd73eefe3b4de0322acf2c7712aa7e4fd9a4c98e606307c54',
        changedChars: 102,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'Considerate_cat-s4',
    entryId: 'Considerate_cat',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 627,
      endOffset: 1_086,
      hash: '735452459f9e25c44e195c04aa020b01693f13e308a1aa483b134af4b75738c6',
    },
    archive: {
      startOffset: 1_585,
      endOffset: 2_754,
      hash: '10b5807b25c9d64f7196991cef727e1e98781f22b588923f0ceaaf5aa4da19e0',
    },
    referenceHash: '10b5807b25c9d64f7196991cef727e1e98781f22b588923f0ceaaf5aa4da19e0',
    referenceChars: 1_169,
    edits: [],
    donor: {
      startOffset: 100,
      endOffset: 252,
      hash: '73a3e1bff7b68bd156c173281e1216116fc5a389625e4e1b7216627f2d78bd76',
    },
    damages: [
      {
        kind: 'alteration',
        hash: '3fda4ad99a8df97c8777c0e2be76fe9523c20158558b6fdea2a15cb963da0e1b',
        changedChars: 2,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'MocaKawai-s0',
    entryId: 'MocaKawai',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 82,
      endOffset: 774,
      hash: 'f857132d584edb7cef5b1403e5182f499f4d9c49a5d715a458fb1fdeeb02116e',
    },
    archive: {
      startOffset: 112,
      endOffset: 2_071,
      hash: 'c6d09ef500e5d6e86cea32d8e2ceb2c5ce6b3d958143e135f3a655b7efe21126',
    },
    referenceHash: 'c6d09ef500e5d6e86cea32d8e2ceb2c5ce6b3d958143e135f3a655b7efe21126',
    referenceChars: 1_959,
    edits: [],
    donor: {
      startOffset: 2_073,
      endOffset: 3_832,
      hash: '96128aa9be6a8338262965b2bb986dba914785a86fa630d6fb2aaeb3cd7f7821',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '2c8978f7f3747c3f8f281a6bdc781c37a36f46e65ab48d6cc631d7d28ee05367',
        changedChars: 143,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'MocaKawai-s1',
    entryId: 'MocaKawai',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 776,
      endOffset: 1_311,
      hash: '21dd5becb4922b4628b55cb7e2c13b510328adfa4a9a048f52871136fc4b2e3a',
    },
    archive: {
      startOffset: 2_073,
      endOffset: 3_832,
      hash: '96128aa9be6a8338262965b2bb986dba914785a86fa630d6fb2aaeb3cd7f7821',
    },
    referenceHash: '96128aa9be6a8338262965b2bb986dba914785a86fa630d6fb2aaeb3cd7f7821',
    referenceChars: 1_759,
    edits: [],
    donor: {
      startOffset: 112,
      endOffset: 2_071,
      hash: 'c6d09ef500e5d6e86cea32d8e2ceb2c5ce6b3d958143e135f3a655b7efe21126',
    },
    damages: [
      {
        kind: 'alteration',
        hash: 'bbdeff691fcb15cf7d3877ed2283cbd28d3f48dbde4b456f80ca4db437b99567',
        changedChars: 4,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'noname-s2',
    entryId: 'noname',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 1_415,
      endOffset: 1_751,
      hash: '01102bb3d5466e48339adad47fc285254bff5221c2bcb5af886d7c0a38c5a303',
    },
    archive: {
      startOffset: 2_980,
      endOffset: 3_797,
      hash: '7462a6e809411dff8d6ecb9169f53dc824644e257f003d2f4c182528f1c84bd2',
    },
    referenceHash: '7462a6e809411dff8d6ecb9169f53dc824644e257f003d2f4c182528f1c84bd2',
    referenceChars: 817,
    edits: [],
    donor: {
      startOffset: 157,
      endOffset: 357,
      hash: '9d2ac01dfc11ade63dac14ec611843f6dda6675d05aa385d36b2e4acae14c906',
    },
    damages: [
      {
        kind: 'insertion',
        hash: 'e5c02aabf9b6153cbc981b72382c4122be3950705bb56dd6282b936debdf44ab',
        changedChars: 120,
      },
    ],
    reviewedOn: '2026-10-24',
  },
];

//endregion Editor fidelity pool, drawn specifications 1
