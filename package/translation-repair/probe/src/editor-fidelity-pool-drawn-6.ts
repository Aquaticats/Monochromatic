import { REVIEWED_CORPUS_SHA, } from './fidelity-reference-manifest.ts';
import type { FidelityReferenceSpec, } from './fidelity-reference-model.ts';

//region Editor fidelity pool, drawn specifications 6
// Ids, offsets and hashes only; no corpus text. Drawn by the rule in `editor-fidelity-pool-manifest.ts` with seed `round-4`:
// entries `shihai4h` to `SevenBird` in the order of the draw.

/**
 Specifications 6 of 10 drawn from the pinned corpus for the editor fidelity pool, each a heading section of
 an entry with the hashes of its source, its archive text, its donor and its damaged twins.

 @example
 ```ts
 const drawn: readonly FidelityReferenceSpec[] = DRAWN_EDITOR_SPECS_6;
 ```
 */
export const DRAWN_EDITOR_SPECS_6: readonly FidelityReferenceSpec[] = [
  {
    id: 'shihai4h-s0',
    entryId: 'shihai4h',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 99,
      endOffset: 348,
      hash: '3b0b69b9ff6e437c58f3f56674ad231d73763d73364a3290ccfb467ac16ee862',
    },
    archive: {
      startOffset: 146,
      endOffset: 756,
      hash: '1b519fcd4ff73139b161ac9a2e13e4dbea55f37dcc5f6f6947bab6f9c2886988',
    },
    referenceHash: '1b519fcd4ff73139b161ac9a2e13e4dbea55f37dcc5f6f6947bab6f9c2886988',
    referenceChars: 610,
    edits: [],
    donor: {
      startOffset: 11_090,
      endOffset: 28_941,
      hash: '9767fcd13e8a492f90b80f6e358cf8775e386ba0b8078d7c94df03cd28e724c9',
    },
    damages: [
      {
        kind: 'insertion',
        hash: 'd423d5bfe3f0cee492629919d52f04a177914713632b454e3079422249629ef4',
        changedChars: 611,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'mikaela_khara-s0',
    entryId: 'mikaela_khara',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 93,
      endOffset: 748,
      hash: '1c36dd6d720318307becaa8be6af7c62d787ecbe48588245d2881d022031e858',
    },
    archive: {
      startOffset: 122,
      endOffset: 1_726,
      hash: '710653de9e2786d8e544cf1fde86c21127228782b3f19b7d6ba052df531236fc',
    },
    referenceHash: '710653de9e2786d8e544cf1fde86c21127228782b3f19b7d6ba052df531236fc',
    referenceChars: 1_604,
    edits: [],
    donor: {
      startOffset: 8_595,
      endOffset: 10_108,
      hash: 'c014667e23a892a53f23cb60c4a4338a75036e250f2501756cb45abe4f39c2e1',
    },
    damages: [
      {
        kind: 'insertion',
        hash: 'f38536a005a71bc7f8a903aa87d6aac7c6e5bdab47a7ec413f8adf14bcb33128',
        changedChars: 147,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'mikaela_khara-s2',
    entryId: 'mikaela_khara',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 1_059,
      endOffset: 1_610,
      hash: '2983c2cc9e8027967af63e6ae7d9c36656e5bbf3a7efb7fb6b9f6124c8d951ec',
    },
    archive: {
      startOffset: 2_719,
      endOffset: 4_478,
      hash: '005f10635c1fd6e3b7a3130adadbc96a76a86ca3364f6bf07f450ffdbc9971eb',
    },
    referenceHash: '005f10635c1fd6e3b7a3130adadbc96a76a86ca3364f6bf07f450ffdbc9971eb',
    referenceChars: 1_759,
    edits: [],
    donor: {
      startOffset: 8_595,
      endOffset: 10_108,
      hash: 'c014667e23a892a53f23cb60c4a4338a75036e250f2501756cb45abe4f39c2e1',
    },
    damages: [
      {
        kind: 'alteration',
        hash: 'a84447f615b56de0a10cb8f16d0ed2e5823dc08f4b16c289fc699da809f3b8b8',
        changedChars: 2,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'qianyuanakg-s0',
    entryId: 'qianyuanakg',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 62,
      endOffset: 222,
      hash: '67ba63024059939f7202f3c5ee7ce26dad1d99a7f47a5891612922339dcbea2c',
    },
    archive: {
      startOffset: 92,
      endOffset: 543,
      hash: '969870fe6245e878f32e8b5ac891f73e2fbb465f62109899fcd5073cf2b49555',
    },
    referenceHash: '969870fe6245e878f32e8b5ac891f73e2fbb465f62109899fcd5073cf2b49555',
    referenceChars: 451,
    edits: [],
    donor: {
      startOffset: 1_874,
      endOffset: 2_921,
      hash: '12e23be03ce284b86f1f87f8576f2e2d0fb4cee142497861203c965f526d34e4',
    },
    damages: [
      {
        kind: 'insertion',
        hash: 'e543c2c25c3de73acb3e57f861d846b60e988e0c26950b71840e13660e5e3cdb',
        changedChars: 155,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'qianyuanakg-s1',
    entryId: 'qianyuanakg',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 224,
      endOffset: 607,
      hash: 'fbaec0b827b5a0a23ee1e0bb3f4fc532f3b547c2c4bba011db7e27ff0a29c215',
    },
    archive: {
      startOffset: 545,
      endOffset: 1_872,
      hash: 'd9bb59a758481043f5b5432a3b551547c53cfd0ff9a1d74f2c6b027ee2d51275',
    },
    referenceHash: 'd9bb59a758481043f5b5432a3b551547c53cfd0ff9a1d74f2c6b027ee2d51275',
    referenceChars: 1_327,
    edits: [],
    donor: {
      startOffset: 1_874,
      endOffset: 2_921,
      hash: '12e23be03ce284b86f1f87f8576f2e2d0fb4cee142497861203c965f526d34e4',
    },
    damages: [
      {
        kind: 'alteration',
        hash: '18b9c650ecad9a34dff5b51b0899f0a00e2ccd9ada61767f07c1857e84020468',
        changedChars: 4,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'republic_o85611-s1',
    entryId: 'republic_o85611',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 245,
      endOffset: 641,
      hash: 'cbdd9b0c53ce1e99e343600ce9ec82034695a4bbdbbe10da64d460d64d88432e',
    },
    archive: {
      startOffset: 408,
      endOffset: 1_557,
      hash: '68a5a12d9b7a0fedea4432559b021418674aff67094ee366f295b4b28dfa55fb',
    },
    referenceHash: '68a5a12d9b7a0fedea4432559b021418674aff67094ee366f295b4b28dfa55fb',
    referenceChars: 1_149,
    edits: [],
    donor: {
      startOffset: 2_288,
      endOffset: 3_184,
      hash: 'fda5c30272224da4ba0ae921ff383bb9550b0db44028e610f35d6957cc65a15c',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '1f5b1b1ddfe4a1d452a9fc6ba14e4f901fe8868bc7b1b8daa5f48869ec1b3e1d',
        changedChars: 125,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'republic_o85611-s3',
    entryId: 'republic_o85611',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 875,
      endOffset: 1_244,
      hash: '289caf281983b16186ea5c725ffc4a55d8b0281f9eb0ea4543e999ed258010ce',
    },
    archive: {
      startOffset: 2_288,
      endOffset: 3_184,
      hash: 'fda5c30272224da4ba0ae921ff383bb9550b0db44028e610f35d6957cc65a15c',
    },
    referenceHash: 'b096ec325850d6c150bf515777994106170920ad246cf5285e7eb8aaf8bf5b41',
    referenceChars: 896,
    edits: [],
    donor: {
      startOffset: 146,
      endOffset: 406,
      hash: 'f42606be492d701639b487c732f312e1d0a89d00fa29d3fe382e4b10bcaef7ef',
    },
    damages: [
      {
        kind: 'alteration',
        hash: '4a6430988276119a27f8d5afd02d37d5d23015145d943bab4c0f47414c6f5975',
        changedChars: 4,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'SevenBird-s1',
    entryId: 'SevenBird',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 139,
      endOffset: 462,
      hash: '4c14194b4368fac880dbc7bb261b65ea571f2427cd4f25d1abbb470183934fd9',
    },
    archive: {
      startOffset: 274,
      endOffset: 1_169,
      hash: '3883f373fcb8cf8330c7aae9652e4ced7dbd8c370873bcdfacca6b9be9665528',
    },
    referenceHash: '3883f373fcb8cf8330c7aae9652e4ced7dbd8c370873bcdfacca6b9be9665528',
    referenceChars: 895,
    edits: [],
    donor: {
      startOffset: 139,
      endOffset: 272,
      hash: '0729560e457e7f4901c07f20d32689f52018b9ae9269f2e38f0cb18964507eff',
    },
    damages: [
      {
        kind: 'alteration',
        hash: 'b2ccfb7c360ce5ab9ba8f53835b9c6a8abf8f2c554eacf723c8fc70b501048eb',
        changedChars: 4,
      },
    ],
    reviewedOn: '2026-10-24',
  },
];

//endregion Editor fidelity pool, drawn specifications 6
