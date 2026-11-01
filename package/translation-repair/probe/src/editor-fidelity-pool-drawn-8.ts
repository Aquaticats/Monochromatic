import { REVIEWED_CORPUS_SHA, } from './fidelity-reference-manifest.ts';
import type { FidelityReferenceSpec, } from './fidelity-reference-model.ts';

//region Editor fidelity pool, drawn specifications 8
// Ids, offsets and hashes only; no corpus text. Drawn by the rule in `editor-fidelity-pool-manifest.ts` with seed `round-4`:
// entries `yulianNyanner` to `a2581911655` in the order of the draw.

/**
 Specifications 8 of 10 drawn from the pinned corpus for the editor fidelity pool, each a heading section of
 an entry with the hashes of its source, its archive text, its donor and its damaged twins.

 @example
 ```ts
 const drawn: readonly FidelityReferenceSpec[] = DRAWN_EDITOR_SPECS_8;
 ```
 */
export const DRAWN_EDITOR_SPECS_8: readonly FidelityReferenceSpec[] = [
  {
    id: 'yulianNyanner-s2',
    entryId: 'yulianNyanner',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 881,
      endOffset: 1_167,
      hash: '46be5339d176172a7712a938c905ecd47691b7189b2437aaae2e0bfd58096049',
    },
    archive: {
      startOffset: 2_569,
      endOffset: 3_447,
      hash: '3287771a483fbb240c0111145895a6492ff0946f25fcb22c082114dadbb76ef8',
    },
    referenceHash: '3287771a483fbb240c0111145895a6492ff0946f25fcb22c082114dadbb76ef8',
    referenceChars: 878,
    edits: [],
    donor: {
      startOffset: 4_228,
      endOffset: 5_814,
      hash: '15f82b6f990d81aa3fc4ef20a4b1005451784cdccb814629d7e9874f22ff7e20',
    },
    damages: [
      {
        kind: 'alteration',
        hash: '5f6ea34218c6c31dd05bc3fee89464d6a2738e95b85c7e1261bd40c3c422b7cf',
        changedChars: 3,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'BI4PBV-s1',
    entryId: 'BI4PBV',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 124,
      endOffset: 435,
      hash: '1cfd28dcd857f25c18fa85b9677f15c259589740cb8c5cc38a52cf229516c264',
    },
    archive: {
      startOffset: 153,
      endOffset: 652,
      hash: '73657e7fa43457eee461a8f1b4a318db6e00db724e504ac1909a00faca9a7224',
    },
    referenceHash: '73657e7fa43457eee461a8f1b4a318db6e00db724e504ac1909a00faca9a7224',
    referenceChars: 499,
    edits: [],
    donor: {
      startOffset: 654,
      endOffset: 1_041,
      hash: '27375ce6d7a87626be098ae2949d7365a37b45772670318192c5f46aa6987ecf',
    },
    damages: [
      {
        kind: 'insertion',
        hash: 'cd6290b7ea3d982bc1f1a977d6025eae9a0eb9431805b1ea4ee99c6da364ae60',
        changedChars: 129,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'windward0032-s1',
    entryId: 'windward0032',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 1_467,
      endOffset: 2_067,
      hash: 'bf0bb3f8de5ea64c09ef54335e9cf634957d774db54ae5cc8b4112117c6bbc2e',
    },
    archive: {
      startOffset: 9_415,
      endOffset: 11_099,
      hash: '5f6415254588bacdc955bd1dd2e1e2d577e362a97c14e357773f1e67497529cf',
    },
    referenceHash: '5f6415254588bacdc955bd1dd2e1e2d577e362a97c14e357773f1e67497529cf',
    referenceChars: 1_684,
    edits: [],
    donor: {
      startOffset: 11_101,
      endOffset: 13_840,
      hash: '08da73e2d6fc0ebb4ef33984d9eac50ad4c3484ffca318294b7ebe8ba42d0030',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '6236a1654aa4077c128f5e0a72f67e36e7058ca7732fa71bfd03179c081027b7',
        changedChars: 319,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'MeowBot233-s1',
    entryId: 'MeowBot233',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 189,
      endOffset: 625,
      hash: 'b6dbc0703c06ce49cbef583fd8c3b6f1a031cf573290a99fd39c4085700c0c50',
    },
    archive: {
      startOffset: 386,
      endOffset: 1_866,
      hash: '898a6cd68cae232af9cdc449f7af3bc3a263c3d12912ab6cd9f8bb8e3af7b5f4',
    },
    referenceHash: '898a6cd68cae232af9cdc449f7af3bc3a263c3d12912ab6cd9f8bb8e3af7b5f4',
    referenceChars: 1_480,
    edits: [],
    donor: {
      startOffset: 2_114,
      endOffset: 3_053,
      hash: '46e2151d5333a0512f17129ba65ebefaaad55570d96f3353f881087cd99a14cf',
    },
    damages: [
      {
        kind: 'insertion',
        hash: 'cd07c2293a44b9b9347564ff292f13c0c810aa41bd43865ce628dc601497e916',
        changedChars: 94,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'MeowBot233-s3',
    entryId: 'MeowBot233',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 737,
      endOffset: 1_116,
      hash: '4f6e462b6230d1a86bb2f225fcff7b7ec8be67a9fb87fd703671b4a84aad2cc0',
    },
    archive: {
      startOffset: 2_114,
      endOffset: 3_053,
      hash: '46e2151d5333a0512f17129ba65ebefaaad55570d96f3353f881087cd99a14cf',
    },
    referenceHash: '46e2151d5333a0512f17129ba65ebefaaad55570d96f3353f881087cd99a14cf',
    referenceChars: 939,
    edits: [],
    donor: {
      startOffset: 109,
      endOffset: 384,
      hash: '6062a8bd348de7b49138f3d4d17609a0d2926ccf6f960467872dcd6bc5f7026c',
    },
    damages: [
      {
        kind: 'alteration',
        hash: '00f4096f6826dab1abb91ec3d39e69f86dd40af43b0af043503d16a895b8529d',
        changedChars: 4,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'hulicaijia-s1',
    entryId: 'hulicaijia',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 205,
      endOffset: 412,
      hash: 'c433ea84c7ab770be1983662eede51e5f58e48d4251138ccc68fe1ddc0081c8d',
    },
    archive: {
      startOffset: 641,
      endOffset: 1_503,
      hash: '27b0252f649634615bf670486f12d91bdbce7112ed816be4efd92f30685e071c',
    },
    referenceHash: '27b0252f649634615bf670486f12d91bdbce7112ed816be4efd92f30685e071c',
    referenceChars: 862,
    edits: [],
    donor: {
      startOffset: 18_924,
      endOffset: 20_844,
      hash: '85cc51c6683f908a06c4494359e0a16f40293020562a94b49fb619ae53d3138b',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '043f8063d6095fc1ec49b9b047ad8b4b8b2a20da9a9905a8b880974fad94fe17',
        changedChars: 129,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'hulicaijia-s3',
    entryId: 'hulicaijia',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 838,
      endOffset: 1_222,
      hash: 'f9d9831cb02fd58977fec687de2a8503764e37b0159db546fced4a0754f0027a',
    },
    archive: {
      startOffset: 2_864,
      endOffset: 3_988,
      hash: '9c801b17608e3713f4ca95ad2095175866efed6c69e32cb9af24291a00e7a8ef',
    },
    referenceHash: '9c801b17608e3713f4ca95ad2095175866efed6c69e32cb9af24291a00e7a8ef',
    referenceChars: 1_124,
    edits: [],
    donor: {
      startOffset: 18_924,
      endOffset: 20_844,
      hash: '85cc51c6683f908a06c4494359e0a16f40293020562a94b49fb619ae53d3138b',
    },
    damages: [
      {
        kind: 'alteration',
        hash: '0c0979d435bc1c2c955aee2495d554ca5058a5653bd03e83a0b1ade646221bfc',
        changedChars: 2,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'a2581911655-s0',
    entryId: 'a2581911655',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 64,
      endOffset: 432,
      hash: 'c5f060dd2e449d541a77d0b052acf1fda537373b9a0a753638b62bf486f30fb3',
    },
    archive: {
      startOffset: 104,
      endOffset: 862,
      hash: '8a3897bd9f15143dc6659fb0811c838346b295807e51c963e984700ee19f5c82',
    },
    referenceHash: '8a3897bd9f15143dc6659fb0811c838346b295807e51c963e984700ee19f5c82',
    referenceChars: 758,
    edits: [],
    donor: {
      startOffset: 864,
      endOffset: 1_497,
      hash: '578c46f49991a818cdf48ad94bdca687afd624c72e5738e0c1d8a476e60cae7e',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '6aec1c2f95e4a38ddd60ef923e21840d691e8590ca4b5cf03c793ff3e94d043d',
        changedChars: 130,
      },
    ],
    reviewedOn: '2026-10-24',
  },
];

//endregion Editor fidelity pool, drawn specifications 8
