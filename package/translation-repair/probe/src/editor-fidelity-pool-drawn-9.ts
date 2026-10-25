import { REVIEWED_CORPUS_SHA, } from './fidelity-reference-manifest.ts';
import type { FidelityReferenceSpec, } from './fidelity-reference-model.ts';

//region Editor fidelity pool, drawn specifications 9
// Ids, offsets and hashes only; no corpus text. Drawn by the rule in `editor-fidelity-pool-manifest.ts` with seed `round-4`:
// entries `a2581911655` to `NIGHT81473140` in the order of the draw.

/**
 Specifications 9 of 10 drawn from the pinned corpus for the editor fidelity pool, each a heading section of
 an entry with the hashes of its source, its archive text, its donor and its damaged twins.

 @example
 ```ts
 const drawn: readonly FidelityReferenceSpec[] = DRAWN_EDITOR_SPECS_9;
 ```
 */
export const DRAWN_EDITOR_SPECS_9: readonly FidelityReferenceSpec[] = [
  {
    id: 'a2581911655-s1',
    entryId: 'a2581911655',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 434,
      endOffset: 698,
      hash: '6dc4776e472d4adc504b1ce9def2baca665c1d318f55649c3ccb54c1113597e9',
    },
    archive: {
      startOffset: 864,
      endOffset: 1_497,
      hash: '578c46f49991a818cdf48ad94bdca687afd624c72e5738e0c1d8a476e60cae7e',
    },
    referenceHash: '578c46f49991a818cdf48ad94bdca687afd624c72e5738e0c1d8a476e60cae7e',
    referenceChars: 633,
    edits: [],
    donor: {
      startOffset: 104,
      endOffset: 862,
      hash: '8a3897bd9f15143dc6659fb0811c838346b295807e51c963e984700ee19f5c82',
    },
    damages: [
      {
        kind: 'alteration',
        hash: '52b9ca2d07ceec4e8d97ea44035533adb83170f1028ed7746e557a569edc9d38',
        changedChars: 4,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'shi_Yumiaoya-s0',
    entryId: 'shi_Yumiaoya',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 57,
      endOffset: 386,
      hash: '04cfcf033e6068b6fdffb3d872e97255edb28ceab9b22a295d6d63e2de41a460',
    },
    archive: {
      startOffset: 79,
      endOffset: 817,
      hash: '44ee3d27a13d333d8cebaf30e092225f33265e3549224a6ca796052a93067b3c',
    },
    referenceHash: '44ee3d27a13d333d8cebaf30e092225f33265e3549224a6ca796052a93067b3c',
    referenceChars: 738,
    edits: [],
    donor: {
      startOffset: 1_048,
      endOffset: 1_457,
      hash: '267b07f80bd43c730badebccba097e7097cf71461f9519c15b50b77d7cc590d1',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '4ed2b007c1d763ba51792aa75a345e94f102415e9aac14bd406f816b71791d03',
        changedChars: 96,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'TianqiChen666-s2',
    entryId: 'TianqiChen666',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 1_761,
      endOffset: 2_421,
      hash: '579ce33891090b54113fe54fb33b01f13dcefea045d5b9cb6b9b33eeee9ba266',
    },
    archive: {
      startOffset: 5_467,
      endOffset: 7_529,
      hash: '6dbb2a6996c8ada032bdd1267d77557c99a55943449bd7213570ed634ed495a8',
    },
    referenceHash: '6dbb2a6996c8ada032bdd1267d77557c99a55943449bd7213570ed634ed495a8',
    referenceChars: 2_062,
    edits: [],
    donor: {
      startOffset: 124,
      endOffset: 447,
      hash: '8e397918af782bc4c73216be1b71fc691e51bbacfa0ed12c68bc53874cae979e',
    },
    damages: [
      {
        kind: 'insertion',
        hash: 'e6600341d54cb88f174af0439fa9d4af454d4ea00436e44d6b30d69935f58866',
        changedChars: 144,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'lxyddice-s3',
    entryId: 'lxyddice',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 677,
      endOffset: 1_608,
      hash: 'b8ea63b6d73048644710f01f27acacda8807ad83cdfd40a98b7f747eb34d8723',
    },
    archive: {
      startOffset: 1_903,
      endOffset: 3_538,
      hash: 'b62b3daa4a6950fafc647eceb5cfc96448054613f589c56e07fe8c360d8679cc',
    },
    referenceHash: 'b62b3daa4a6950fafc647eceb5cfc96448054613f589c56e07fe8c360d8679cc',
    referenceChars: 1_635,
    edits: [],
    donor: {
      startOffset: 195,
      endOffset: 1_211,
      hash: '3bd39632bc2c315505e2db4232f7a2faf84c3e06c1d67ca0b8f7473d1ba5be21',
    },
    damages: [
      {
        kind: 'insertion',
        hash: 'ed08ef7f664f047f5cefbb93a888ddd3f3df465583d58fd4ee6db93f08cdc465',
        changedChars: 206,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'Dethelly-s0',
    entryId: 'Dethelly',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 144,
      endOffset: 385,
      hash: '9cebe0ac13dcb428d8d4388f3027997ae168f4ed8956f1b5bbe436c154bb9a12',
    },
    archive: {
      startOffset: 261,
      endOffset: 1_247,
      hash: '9a1bc5b5dcd8951f2be42828f51788899a2aa5c61d4705c660236ac4322aee8d',
    },
    referenceHash: '9a1bc5b5dcd8951f2be42828f51788899a2aa5c61d4705c660236ac4322aee8d',
    referenceChars: 986,
    edits: [],
    donor: {
      startOffset: 6_374,
      endOffset: 7_528,
      hash: '4d25ff8b5938439535c773449a5fd4ed6d17952e6292d562915d648faeccda40',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '91bc717476c596a97584b4da764fa63283b52c61d3730fb1e4f3586232583fed',
        changedChars: 84,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'Dethelly-s1',
    entryId: 'Dethelly',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 387,
      endOffset: 850,
      hash: '3d96c4b0526667fdd14ec57aa6409066080fe96e3b43688561e9d404e4446ceb',
    },
    archive: {
      startOffset: 1_249,
      endOffset: 2_783,
      hash: '2c345114e0357812a23899190ddb66104337c52b1c58e77e9116e66168c2a825',
    },
    referenceHash: '2c345114e0357812a23899190ddb66104337c52b1c58e77e9116e66168c2a825',
    referenceChars: 1_534,
    edits: [],
    donor: {
      startOffset: 6_374,
      endOffset: 7_528,
      hash: '4d25ff8b5938439535c773449a5fd4ed6d17952e6292d562915d648faeccda40',
    },
    damages: [
      {
        kind: 'alteration',
        hash: '4252046f60c1d5b35826d7e1567a3583542abff869582cd33c0dcb1646775280',
        changedChars: 4,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'NIGHT81473140-s0',
    entryId: 'NIGHT81473140',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 85,
      endOffset: 579,
      hash: '1335c8b8e093c177a0ea2d3d4599c4909175bd32c510aaa6e9938865b23fea0d',
    },
    archive: {
      startOffset: 112,
      endOffset: 1_853,
      hash: 'df31482a2307f5a7900c042919213a28a9d5509417a8633022d1628fb0d7dfa6',
    },
    referenceHash: 'df31482a2307f5a7900c042919213a28a9d5509417a8633022d1628fb0d7dfa6',
    referenceChars: 1_741,
    edits: [],
    donor: {
      startOffset: 13_354,
      endOffset: 13_916,
      hash: '0706d0824fb9b658ad9ddf2316494b5cc3a74f387acb7bee7949354255333d89',
    },
    damages: [
      {
        kind: 'insertion',
        hash: 'b529df70379ce2ac560aca309f35f15f9eb8be2d98b1eaf1c925aa202c461af4',
        changedChars: 93,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'NIGHT81473140-s1',
    entryId: 'NIGHT81473140',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 581,
      endOffset: 975,
      hash: '6a6cdb65507ab69d8a18a16b4daf9669fb020c1effacfe3c0f0f6387a828d535',
    },
    archive: {
      startOffset: 1_855,
      endOffset: 3_011,
      hash: '105aa47ad7436c05cb5e739c12254129d12e3b2ac90fc3faeb302f39b903daf7',
    },
    referenceHash: '105aa47ad7436c05cb5e739c12254129d12e3b2ac90fc3faeb302f39b903daf7',
    referenceChars: 1_156,
    edits: [],
    donor: {
      startOffset: 13_354,
      endOffset: 13_916,
      hash: '0706d0824fb9b658ad9ddf2316494b5cc3a74f387acb7bee7949354255333d89',
    },
    damages: [
      {
        kind: 'alteration',
        hash: '15ae0f3c7f2b8a2d69227fe73a0d3ae72d7555e86b78ecc3fb31f0364e4ea3e2',
        changedChars: 3,
      },
    ],
    reviewedOn: '2026-10-24',
  },
];

//endregion Editor fidelity pool, drawn specifications 9
