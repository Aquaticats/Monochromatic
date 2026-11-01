import { REVIEWED_CORPUS_SHA, } from './fidelity-reference-manifest.ts';
import type { FidelityReferenceSpec, } from './fidelity-reference-model.ts';

//region Editor fidelity pool, drawn specifications 7
// Ids, offsets and hashes only; no corpus text. Drawn by the rule in `editor-fidelity-pool-manifest.ts` with seed `round-4`:
// entries `Jennife80677612` to `yulianNyanner` in the order of the draw.

/**
 Specifications 7 of 10 drawn from the pinned corpus for the editor fidelity pool, each a heading section of
 an entry with the hashes of its source, its archive text, its donor and its damaged twins.

 @example
 ```ts
 const drawn: readonly FidelityReferenceSpec[] = DRAWN_EDITOR_SPECS_7;
 ```
 */
export const DRAWN_EDITOR_SPECS_7: readonly FidelityReferenceSpec[] = [
  {
    id: 'Jennife80677612-s2',
    entryId: 'Jennife80677612',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 1_160,
      endOffset: 1_426,
      hash: '98fb54710a0faa6438367d7d528374e50b862595971696ae7f52728655040615',
    },
    archive: {
      startOffset: 3_449,
      endOffset: 4_275,
      hash: '252f477527a295909068d35c3ce6fb3fc47caf15ce75098400e2301a4a12a7db',
    },
    referenceHash: '252f477527a295909068d35c3ce6fb3fc47caf15ce75098400e2301a4a12a7db',
    referenceChars: 826,
    edits: [],
    donor: {
      startOffset: 393,
      endOffset: 3_447,
      hash: '927479d2ef6e7da124f5bcc3f0ec8249ca203d16aa5d4e4e19c900e635cab119',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '0219c2cb9aa280b46215909f8709fafe8d4710b71191c52a60bb0a3df7d19957',
        changedChars: 156,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'MioCardMeow-s0',
    entryId: 'MioCardMeow',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 80,
      endOffset: 546,
      hash: '950129b278a58305dbc32e96ce6178ab53700ee4a0bd011cb8e430ff986eb947',
    },
    archive: {
      startOffset: 113,
      endOffset: 1_062,
      hash: '053578451bd528502703d2c39658b4c90c2ec2cfe2df81d4437e6a40480f9f4f',
    },
    referenceHash: '053578451bd528502703d2c39658b4c90c2ec2cfe2df81d4437e6a40480f9f4f',
    referenceChars: 949,
    edits: [],
    donor: {
      startOffset: 1_386,
      endOffset: 1_827,
      hash: '3e80a484cb32e95a39b651599e0303649b8e1a5040927e1cfcd4f787f2524ec9',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '6f40f9a741aa31613e88ac1eb8ff9ef5ccaa7b4f22c752c05f6f0162baf05752',
        changedChars: 120,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'MioCardMeow-s2',
    entryId: 'MioCardMeow',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 738,
      endOffset: 984,
      hash: '0628df2c31580a1957009e49e03cad1b0296f5922e144653b19b1620510e2916',
    },
    archive: {
      startOffset: 1_386,
      endOffset: 1_827,
      hash: '3e80a484cb32e95a39b651599e0303649b8e1a5040927e1cfcd4f787f2524ec9',
    },
    referenceHash: '3e80a484cb32e95a39b651599e0303649b8e1a5040927e1cfcd4f787f2524ec9',
    referenceChars: 441,
    edits: [],
    donor: {
      startOffset: 113,
      endOffset: 1_062,
      hash: '053578451bd528502703d2c39658b4c90c2ec2cfe2df81d4437e6a40480f9f4f',
    },
    damages: [
      {
        kind: 'alteration',
        hash: '3431ee539616e99af1e8a89edf5bfc0d3f4f9563ea00622ce80397e613a99c1c',
        changedChars: 9,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'Futajuhuacha-s0',
    entryId: 'Futajuhuacha',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 82,
      endOffset: 582,
      hash: '59b59f43daf0220944951b7519980b517e0e587d6c4cec005251d3ddf119f094',
    },
    archive: {
      startOffset: 130,
      endOffset: 1_504,
      hash: '82cf3baa55b9535381e1590ab8eb817675ca4b89d193b94b20065159c43e55f3',
    },
    referenceHash: '82cf3baa55b9535381e1590ab8eb817675ca4b89d193b94b20065159c43e55f3',
    referenceChars: 1_374,
    edits: [],
    donor: {
      startOffset: 4_195,
      endOffset: 6_368,
      hash: '606b7d5ab86af3e2a51e9590565382cbb050ba3848b27b215200d374321f220e',
    },
    damages: [
      {
        kind: 'insertion',
        hash: 'f0dce473f33f821c3b8a03e892527523bd169a411219cfeb9dd7cb9a317d637c',
        changedChars: 228,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'Futajuhuacha-s2',
    entryId: 'Futajuhuacha',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 1_455,
      endOffset: 2_281,
      hash: 'fbaeb841ac856e4383178080fe769aa8f5e818ad85ad537a6fe10b01ef2ab3ce',
    },
    archive: {
      startOffset: 4_195,
      endOffset: 6_368,
      hash: '606b7d5ab86af3e2a51e9590565382cbb050ba3848b27b215200d374321f220e',
    },
    referenceHash: '606b7d5ab86af3e2a51e9590565382cbb050ba3848b27b215200d374321f220e',
    referenceChars: 2_173,
    edits: [],
    donor: {
      startOffset: 130,
      endOffset: 1_504,
      hash: '82cf3baa55b9535381e1590ab8eb817675ca4b89d193b94b20065159c43e55f3',
    },
    damages: [
      {
        kind: 'alteration',
        hash: '844ab03dfbdff3fe6df28fa5221dfd616e7460aaeef40469d0e97390f759c4ff',
        changedChars: 2,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'Chinatsu_Suzuki-s0',
    entryId: 'Chinatsu_Suzuki',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 85,
      endOffset: 298,
      hash: 'a47acf50d10543c3cc885f3bf1457bb3ba0647a628814146e774790d868e7e56',
    },
    archive: {
      startOffset: 89,
      endOffset: 724,
      hash: '1e6d247646346c8b308d1dd554a5ebe68e2fa264e5682ff791080940a21caa93',
    },
    referenceHash: '1e6d247646346c8b308d1dd554a5ebe68e2fa264e5682ff791080940a21caa93',
    referenceChars: 635,
    edits: [],
    donor: {
      startOffset: 5_058,
      endOffset: 6_054,
      hash: '4f7974e5c8fd9285f7255c6c949112de0b21e8dbb63b2f199a3e55870ee50338',
    },
    damages: [
      {
        kind: 'insertion',
        hash: 'e1dc8526845db9d9c27f1350cbe75cc8b22819ab103c6a81f44217c129cd43f3',
        changedChars: 101,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'Chinatsu_Suzuki-s6',
    entryId: 'Chinatsu_Suzuki',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 1_537,
      endOffset: 2_228,
      hash: '0606b801239b56ac3f33a89655f204762595e831ebaef6f04f3ead1ac4e9b087',
    },
    archive: {
      startOffset: 5_058,
      endOffset: 6_054,
      hash: '4f7974e5c8fd9285f7255c6c949112de0b21e8dbb63b2f199a3e55870ee50338',
    },
    referenceHash: '4f7974e5c8fd9285f7255c6c949112de0b21e8dbb63b2f199a3e55870ee50338',
    referenceChars: 996,
    edits: [],
    donor: {
      startOffset: 89,
      endOffset: 724,
      hash: '1e6d247646346c8b308d1dd554a5ebe68e2fa264e5682ff791080940a21caa93',
    },
    damages: [
      {
        kind: 'alteration',
        hash: '7dd005571a09947ab02b0b33989a271a0828a99c9c61dd3e4ad9e26a83c4bc5d',
        changedChars: 2,
      },
    ],
    reviewedOn: '2026-10-24',
  },
  {
    id: 'yulianNyanner-s0',
    entryId: 'yulianNyanner',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 78,
      endOffset: 488,
      hash: '652de0df38e0321fe34f389be824adca1251f95c400214d80e917549a455c8bc',
    },
    archive: {
      startOffset: 122,
      endOffset: 1_252,
      hash: 'a8fafc651fb18b36ddfb492196807695c841ec677d45384bd2972af949d227fb',
    },
    referenceHash: 'a8fafc651fb18b36ddfb492196807695c841ec677d45384bd2972af949d227fb',
    referenceChars: 1_130,
    edits: [],
    donor: {
      startOffset: 4_228,
      endOffset: 5_814,
      hash: '15f82b6f990d81aa3fc4ef20a4b1005451784cdccb814629d7e9874f22ff7e20',
    },
    damages: [
      {
        kind: 'insertion',
        hash: '0d92b25fbb4c29bd9dbca33505c110eaf4ab85f26ce9f3e680206a4e4f092d33',
        changedChars: 87,
      },
    ],
    reviewedOn: '2026-10-24',
  },
];

//endregion Editor fidelity pool, drawn specifications 7
