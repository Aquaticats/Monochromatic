import { REVIEWED_CORPUS_SHA, } from './fidelity-reference-manifest.ts';
import type { FidelityReferenceSpec, } from './fidelity-reference-model.ts';

//region Editor fidelity pool, drawn specifications 4
// Ids, offsets and hashes only; no corpus text. Drawn by the rule in `editor-fidelity-pool-manifest.ts` with seed `round-4`:
// entries `cheonwoomaeng` to `Xu_Yushu` in the order of the draw.

/**
 Specifications 4 of 10 drawn from the pinned corpus for the editor fidelity pool, each a heading section of
 an entry with the hashes of its source, its archive text, its donor and its damaged twins.

 @example
 ```ts
 const drawn: readonly FidelityReferenceSpec[] = DRAWN_EDITOR_SPECS_4;
 ```
 */
export const DRAWN_EDITOR_SPECS_4: readonly FidelityReferenceSpec[] = [
  {
    id: 'cheonwoomaeng-s1',
    entryId: 'cheonwoomaeng',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 131,
      endOffset: 445,
      hash: '7b22b5f9a68f8d9f9ed4c52eb0a7feb591a865848dc0bc3a345809786486cb5f',
    },
    archive: {
      startOffset: 245,
      endOffset: 940,
      hash: '4a680181bc72b1c26841024004539e06abe2ac4d104467cd76278517a1437ca9',
    },
    referenceHash: '4a680181bc72b1c26841024004539e06abe2ac4d104467cd76278517a1437ca9',
    referenceChars: 695,
    edits: [],
    donor: {
      startOffset: 2_680,
      endOffset: 3_215,
      hash: '0a6ab68f9dfe679329ec395819c6330503f2da5ff2186e9a92a8a1533347789f',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '7a489dc2213de5a529ba7fc71fec5e61fbe39ed992e00ff1bcf3b4b01f6f32ee',
        changedChars: 190,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'cheonwoomaeng-s2',
    entryId: 'cheonwoomaeng',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 447,
      endOffset: 1_219,
      hash: '281b1f11508486921cdac5a675d3f0315880d163991915709bd7f593f6f5e369',
    },
    archive: {
      startOffset: 942,
      endOffset: 2_678,
      hash: '2096003f1a124605c84b8105e5421f71c3a0bc612274208e2ffcdda7b029d7cd',
    },
    referenceHash: '2096003f1a124605c84b8105e5421f71c3a0bc612274208e2ffcdda7b029d7cd',
    referenceChars: 1_736,
    edits: [],
    donor: {
      startOffset: 121,
      endOffset: 243,
      hash: 'e622fa42a476c156d42b7c507ec8f9e59656f53797a6a60ec2ea777b17949fed',
    },
    damages: [
      {
        kind: 'alteration',
        hash: 'd5fc3df8279969a26797fa874134c7a4a06865c1b80323f9f015deca800286c3',
        changedChars: 4,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'Anilovr-s1',
    entryId: 'Anilovr',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 153,
      endOffset: 956,
      hash: '5b7e134c91413ff4cd6c7887940d51629e7ec2d1f831de425bbac07cc8cd81da',
    },
    archive: {
      startOffset: 234,
      endOffset: 2_235,
      hash: 'e417825ff5cf356cb27116c31545942bce4e1b0f68223a9ff6c8c0979a7543fc',
    },
    referenceHash: 'e417825ff5cf356cb27116c31545942bce4e1b0f68223a9ff6c8c0979a7543fc',
    referenceChars: 2_001,
    edits: [],
    donor: {
      startOffset: 116,
      endOffset: 232,
      hash: '8ec74366ff85ceb8cef6990764fca844b2a48c76c62b02559a8abd57b617436a',
    },
    damages: [
      {
        kind: 'alteration',
        hash: '527113c12da3e6d4070e19aca9d40a4c206c238b99cbb4ca9f099eb401db9d78',
        changedChars: 6,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'LCG_Akiball-s0',
    entryId: 'LCG_Akiball',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 89,
      endOffset: 517,
      hash: 'f2d55510aa558e9f786f0e5937f8ba2ff7b5c4e1e5f1dad9438b556e3e07d0f8',
    },
    archive: {
      startOffset: 117,
      endOffset: 1_353,
      hash: '03a68aa44b897bbaf10a34360ea8525ac69630a8d5573ac3fb05a96e2d6d264f',
    },
    referenceHash: '03a68aa44b897bbaf10a34360ea8525ac69630a8d5573ac3fb05a96e2d6d264f',
    referenceChars: 1_236,
    edits: [],
    donor: {
      startOffset: 1_355,
      endOffset: 4_368,
      hash: '00812413dc27a5e9288cb981aa9c84c51ecb70df5a01c0585eaef3b53d89b1de',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '4a54daefd32b5db996f31871774f2121aa3c34805aaf8f2769a0843407f7bd3f',
        changedChars: 203,
      },
      {
        kind: 'alteration',
        hash: 'ce1b1d4e3ff2b93a87697daa7297d38d313bd950effaaf48371b8751cbf2d988',
        changedChars: 4,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'Everythings99-s0',
    entryId: 'Everythings99',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 74,
      endOffset: 315,
      hash: 'ab30e9ea80ec7a4a57b30c458946cac2214a75c8a2f4ec170f00a50871fd97f0',
    },
    archive: {
      startOffset: 113,
      endOffset: 833,
      hash: '8f66f9dc3b0690ab482ed947401c5828f3f056dc2cd01b7a846ea3df9ea3c415',
    },
    referenceHash: '8f66f9dc3b0690ab482ed947401c5828f3f056dc2cd01b7a846ea3df9ea3c415',
    referenceChars: 720,
    edits: [],
    donor: {
      startOffset: 1_153,
      endOffset: 2_040,
      hash: '120fb43c39d4922361b56139d6cd3544db53bfb97df9e955520a069047d6479f',
    },
    damages: [
      {
        kind: 'insertion',
        hash: 'ee79a1307d7203ab384d03bb689bacbd7241e8ae5d4ae1cab10dce6900a62960',
        changedChars: 166,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'Everythings99-s2',
    entryId: 'Everythings99',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 510,
      endOffset: 800,
      hash: '055d3980ab9230e8f28195bc21f5062c596032e625591063deafc63b725d312b',
    },
    archive: {
      startOffset: 1_153,
      endOffset: 2_040,
      hash: '120fb43c39d4922361b56139d6cd3544db53bfb97df9e955520a069047d6479f',
    },
    referenceHash: '120fb43c39d4922361b56139d6cd3544db53bfb97df9e955520a069047d6479f',
    referenceChars: 887,
    edits: [],
    donor: {
      startOffset: 113,
      endOffset: 833,
      hash: '8f66f9dc3b0690ab482ed947401c5828f3f056dc2cd01b7a846ea3df9ea3c415',
    },
    damages: [
      {
        kind: 'alteration',
        hash: '399b122ddf8fb06fc043facbd940a727be51239d7e94fd441e0e1645af702e01',
        changedChars: 2,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'juzh50533467-s0',
    entryId: 'juzh50533467',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 75,
      endOffset: 337,
      hash: '6cc8c3ed73628d4d7111ad48c4ca946d00610c46bbdc82b1b1ac91980a3d1afe',
    },
    archive: {
      startOffset: 108,
      endOffset: 809,
      hash: '6bef45d9ce8c90b4dbcad90a0f034af776a33abce0e8a16d5a2b8c1a6bcf74f8',
    },
    referenceHash: '6bef45d9ce8c90b4dbcad90a0f034af776a33abce0e8a16d5a2b8c1a6bcf74f8',
    referenceChars: 701,
    edits: [],
    donor: {
      startOffset: 3_539,
      endOffset: 4_053,
      hash: '678763010eb705803df33a8a5e3d189d50b87c729fcb8f2d46a45ea5205be676',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '0eb2ca21c8b94468a930a1155b620ca4c664ce99ba02ac5e0320b5dc503f652c',
        changedChars: 92,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'Xu_Yushu-s0',
    entryId: 'Xu_Yushu',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 64,
      endOffset: 229,
      hash: '0fa3926a040a72a014189287c7a046e006755edf24afac51c73862bee0e8179b',
    },
    archive: {
      startOffset: 107,
      endOffset: 568,
      hash: 'a562d979a3f83b226ebaa00fb2eba0c08e29c4c9f425328220a91c8f4ba428c1',
    },
    referenceHash: 'a562d979a3f83b226ebaa00fb2eba0c08e29c4c9f425328220a91c8f4ba428c1',
    referenceChars: 461,
    edits: [],
    donor: {
      startOffset: 8_940,
      endOffset: 9_350,
      hash: '92d1da458e10dc5c4cdab762684751a17b10f351f091f66e917dbfb09ad75f6f',
    },
    damages: [
      {
        kind: 'insertion',
        hash: 'f64f4b5f7b617c75103493cbb9baaee3ce976892fa251b6d4df7b640b9c4fa63',
        changedChars: 153,
      },
    ],
    reviewedOn: '2026-10-24',
  },
];

//endregion Editor fidelity pool, drawn specifications 4
