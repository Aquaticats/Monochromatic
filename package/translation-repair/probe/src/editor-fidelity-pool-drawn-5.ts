import { REVIEWED_CORPUS_SHA, } from './fidelity-reference-manifest.ts';
import type { FidelityReferenceSpec, } from './fidelity-reference-model.ts';

//region Editor fidelity pool, drawn specifications 5
// Ids, offsets and hashes only; no corpus text. Drawn by the rule in `editor-fidelity-pool-manifest.ts` with seed `round-4`:
// entries `Xu_Yushu` to `Mio` in the order of the draw.

/**
 Specifications 5 of 10 drawn from the pinned corpus for the editor fidelity pool, each a heading section of
 an entry with the hashes of its source, its archive text, its donor and its damaged twins.

 @example
 ```ts
 const drawn: readonly FidelityReferenceSpec[] = DRAWN_EDITOR_SPECS_5;
 ```
 */
export const DRAWN_EDITOR_SPECS_5: readonly FidelityReferenceSpec[] = [
  {
    id: 'Xu_Yushu-s3',
    entryId: 'Xu_Yushu',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 1_763,
      endOffset: 2_075,
      hash: 'f4fc89cf6d9e6de9da5046f329a63288c4437c122ef55fc666ebc80c06ab2653',
    },
    archive: {
      startOffset: 5_145,
      endOffset: 5_919,
      hash: 'd111fbd00316669ebecf8d266c750448895ec74ceb4453df526728376538c7c5',
    },
    referenceHash: 'd111fbd00316669ebecf8d266c750448895ec74ceb4453df526728376538c7c5',
    referenceChars: 774,
    edits: [],
    donor: {
      startOffset: 8_940,
      endOffset: 9_350,
      hash: '92d1da458e10dc5c4cdab762684751a17b10f351f091f66e917dbfb09ad75f6f',
    },
    damages: [
      {
        kind: 'alteration',
        hash: '7480827e89de94c81351c0440e4f934b2bd9908c3f73210d7d0ec0ca4bffeae3',
        changedChars: 2,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'MushroomGuuuu-s0',
    entryId: 'MushroomGuuuu',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 81,
      endOffset: 401,
      hash: 'e6702fc6485e6236e6c3f35e1a16aed624afc649f1970784ef779e84182fbf34',
    },
    archive: {
      startOffset: 105,
      endOffset: 1_105,
      hash: '9657ba4e2e7e0bf2a984f5223f587a2f489580e7113c3c6922b5b141a5f0cd5f',
    },
    referenceHash: '9657ba4e2e7e0bf2a984f5223f587a2f489580e7113c3c6922b5b141a5f0cd5f',
    referenceChars: 1_000,
    edits: [],
    donor: {
      startOffset: 1_437,
      endOffset: 2_167,
      hash: '4e6b522f74282a88fd3a34e54317be859f0ce194f63d6ef50a3164e11bdf67df',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '208443ae453423e683cde6fece78fdf70e3b8225ea4e77a60d8692371e1bd405',
        changedChars: 155,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'MushroomGuuuu-s2',
    entryId: 'MushroomGuuuu',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 496,
      endOffset: 801,
      hash: '8d736bf3905428d6d238474bd14d1bb05527eb98ac5806bc08cccd3d28c0b3da',
    },
    archive: {
      startOffset: 1_437,
      endOffset: 2_167,
      hash: '4e6b522f74282a88fd3a34e54317be859f0ce194f63d6ef50a3164e11bdf67df',
    },
    referenceHash: '4e6b522f74282a88fd3a34e54317be859f0ce194f63d6ef50a3164e11bdf67df',
    referenceChars: 730,
    edits: [],
    donor: {
      startOffset: 105,
      endOffset: 1_105,
      hash: '9657ba4e2e7e0bf2a984f5223f587a2f489580e7113c3c6922b5b141a5f0cd5f',
    },
    damages: [
      {
        kind: 'alteration',
        hash: '48e70f4e966c35f28783cedbb8f41f27bb59a76e716cbfb4859544a3dfc7b947',
        changedChars: 9,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'Huasheng-s0',
    entryId: 'Huasheng',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 87,
      endOffset: 217,
      hash: '07d804527dc8658190c09a8e851fdbf7f9b49109f43b518fecd817474bdb86bf',
    },
    archive: {
      startOffset: 147,
      endOffset: 549,
      hash: 'a103b9d25d3ea71275416ced098e011911b49c7e6aca378545125fb5d9dd5924',
    },
    referenceHash: 'a103b9d25d3ea71275416ced098e011911b49c7e6aca378545125fb5d9dd5924',
    referenceChars: 402,
    edits: [],
    donor: {
      startOffset: 5_677,
      endOffset: 7_914,
      hash: '2f4314fb840b22875d4b187c16335f7137cb3be5bafdd179515239456700a7d5',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '5589925fc03a4f8b33bb8edab05ffbb3ddae50d83010c956f5a371fbf80acce1',
        changedChars: 411,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'Susiethegamer-s1',
    entryId: 'Susiethegamer',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 1_599,
      endOffset: 2_092,
      hash: '7204476568f236a66813256da69fe524bd2019f62e7bc927d1a859c2c4ed42c7',
    },
    archive: {
      startOffset: 3_965,
      endOffset: 5_604,
      hash: '3ffd175a905cc7f292612d6e829bf6a8d3b99166caa70d9fe689f12dd369d611',
    },
    referenceHash: '3ffd175a905cc7f292612d6e829bf6a8d3b99166caa70d9fe689f12dd369d611',
    referenceChars: 1_639,
    edits: [],
    donor: {
      startOffset: 108,
      endOffset: 3_963,
      hash: 'f2e2680ef64aa1cf40f1fa62ad3faaa04a81ebc4541fb3dead541ba4083ce137',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '082a9e8e581a4119979f845663b906b196481b8c71bfbc7f6cf43086f323c0c8',
        changedChars: 150,
      },
      {
        kind: 'alteration',
        hash: 'db06c67c67a6eccdc22f98c8f64408ac04f40122c228390f20d602a8dc4a2e98',
        changedChars: 6,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'xixi_yuexi-s1',
    entryId: 'xixi_yuexi',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 566,
      endOffset: 742,
      hash: 'c470a571d8dbd75854397cf62b9e08c9e0771537298fb8d0c43ffbdca3053bdb',
    },
    archive: {
      startOffset: 969,
      endOffset: 1_745,
      hash: 'f55b46dc265009d819560a780b304ce5ab10d6cca3c259d6e0deaa4c23952a3b',
    },
    referenceHash: 'f55b46dc265009d819560a780b304ce5ab10d6cca3c259d6e0deaa4c23952a3b',
    referenceChars: 776,
    edits: [],
    donor: {
      startOffset: 90,
      endOffset: 967,
      hash: '287ddfd94c8fda8c623235d17aa880248fd1a60c37df317365cadee1bd6a5ea5',
    },
    damages: [
      {
        kind: 'insertion',
        hash: 'e2e1ef0d9b0825acf331ad33d1b305e38cea6b84a3a1cc0f02f265af0fb68e71',
        changedChars: 84,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'xixi_yuexi-s0',
    entryId: 'xixi_yuexi',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 75,
      endOffset: 564,
      hash: '7f50483a0442703e7a95e40140b5ec387352f433ad626da5c6bf6a27719af51d',
    },
    archive: {
      startOffset: 90,
      endOffset: 967,
      hash: '287ddfd94c8fda8c623235d17aa880248fd1a60c37df317365cadee1bd6a5ea5',
    },
    referenceHash: '287ddfd94c8fda8c623235d17aa880248fd1a60c37df317365cadee1bd6a5ea5',
    referenceChars: 877,
    edits: [],
    donor: {
      startOffset: 969,
      endOffset: 1_745,
      hash: 'f55b46dc265009d819560a780b304ce5ab10d6cca3c259d6e0deaa4c23952a3b',
    },
    damages: [
      {
        kind: 'alteration',
        hash: '1f52f57c72f8b44a38cfecf63a13783bafe8dc346640058a8b37c4ece8bbc9eb',
        changedChars: 4,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'Mio-s0',
    entryId: 'Mio',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 82,
      endOffset: 468,
      hash: '065758e6236cb52165520c03bb8c496d754ef8a3d76caf3e4a1dfa47436df5f1',
    },
    archive: {
      startOffset: 124,
      endOffset: 1_230,
      hash: '77054317152313ca8dd5bacda407f3e9228d7ea0514ec3d96b186683e93b313b',
    },
    referenceHash: '77054317152313ca8dd5bacda407f3e9228d7ea0514ec3d96b186683e93b313b',
    referenceChars: 1_106,
    edits: [],
    donor: {
      startOffset: 3_864,
      endOffset: 7_932,
      hash: 'aa213d0098bec6e73a6e9363a590a53ed6175d4b0b94ca0a451876b1090e34cc',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '06922ba1e0c1d2f5dbe953329510f529173d6d2b75fb6d1eadf965138918d04f',
        changedChars: 168,
      },
    ],
    reviewedOn: '2026-10-24',
  },
];

//endregion Editor fidelity pool, drawn specifications 5
