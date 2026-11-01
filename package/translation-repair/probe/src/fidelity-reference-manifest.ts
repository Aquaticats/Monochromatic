import type { FidelityReferenceSpec, } from './fidelity-reference-model.ts';

//region Source-reviewed calibration manifest
// No complete corpus passage is committed. Exact ranges and hashes pin the reviewed comparison.

/**
 Immutable review pin, deliberately independent of future runtime corpus defaults, which the editor fidelity pool's
 drawn specifications are pinned to as well.
 */
export const REVIEWED_CORPUS_SHA = 'a41fc607ea5a70d8a7625cc67d5ed8c444f53379';

/**
 References and every permitted damage reviewed before model calibration.
 The local Y1Ran correction is assistant-authored calibration text, never a corpus edit.
 Evidence: doc/planning/translation-repair-reviewed-calibration-fixtures-2026-09-11.md.

 @example
 ```ts
 const references = await readReviewedFidelityReferences({ pin, specs: REVIEWED_FIDELITY_REFERENCES });
 ```
 */
export const REVIEWED_FIDELITY_REFERENCES: readonly FidelityReferenceSpec[] = [
  {
    id: 'gqt-reference',
    entryId: 'gqt',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 148,
      endOffset: 323,
      hash: '26e2530c3df46725496b99535bc639bf57c4baf57932b4a255c483a9d2967aed',
    },
    archive: {
      startOffset: 216,
      endOffset: 711,
      hash: '0d22307e7129d898d58008cd02efda7867c4d77cff65801689fe9639de3722d5',
    },
    referenceHash: '0d22307e7129d898d58008cd02efda7867c4d77cff65801689fe9639de3722d5',
    referenceChars: 495,
    edits: [],
    donor: {
      startOffset: 736,
      endOffset: 916,
      hash: 'a6db88cb60c0665296dbefbc2f6eb8ba24170dbf7229252b84048ebd8bc13948',
    },
    damages: [
      {
        kind: 'deletion',
        hash: 'a34addd8b41d5a6000241182f18c44f8e3e317ca366f535cf027aeab7b0b29f5',
        changedChars: 247,
      },
      {
        kind: 'insertion',
        hash: '663f7f4334b13e6ecf3b1fc37c4b11911ecac9648cbe2e971b1292c0303c02d5',
        changedChars: 180,
      },
    ],
    reviewedOn: '2026-09-11',
  },
  {
    id: 'MTF_0615-reference',
    entryId: 'MTF_0615',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 177,
      endOffset: 310,
      hash: '711f0cac19f66dabc0860aafcdd30dd83ecca44045f1d4665c71a3716e636c59',
    },
    archive: {
      startOffset: 258,
      endOffset: 692,
      hash: '15f2e2b7a7e6a79aae8b86b2d17d18d8d6ef57cae75b0c56945c546d8b93f7fc',
    },
    referenceHash: 'ee204288b7705fe223d123b5a84567cebafc676bf526f5ea1607ea3da17d820a',
    referenceChars: 434,
    edits: [],
    donor: {
      startOffset: 142,
      endOffset: 256,
      hash: '64d95b6d6ca4a430c57827295b552430593b3361f045e7a461bede31ca564877',
    },
    damages: [
      {
        kind: 'deletion',
        hash: '69471d0c1d3f5d0a6ac86b2eaf43de447f11ac96689359779dea3afe01117903',
        changedChars: 177,
      },
      {
        kind: 'insertion',
        hash: '8bde760c6646720aa6f5c97cbdc167d79e58c9ae60a0ad850de554fe329d486e',
        changedChars: 63,
      },
    ],
    reviewedOn: '2026-09-11',
  },
  {
    id: 'Y1Ran-reference',
    entryId: 'Y1Ran',
    corpusSha: REVIEWED_CORPUS_SHA,
    source: {
      startOffset: 400,
      endOffset: 521,
      hash: '1e056b57dd4c19c833aae60270a132a764552ed783c08b6aa7cc2a6811f28688',
    },
    archive: {
      startOffset: 613,
      endOffset: 1_020,
      hash: 'ce461b9f640ae6ee7f154f15fc2772304fe4df415a06ad3f794e540c6a334425',
    },
    referenceHash: '18a13efe9a1b5437f24342173f5a451d2e44ade76ad145eb9c6839b00dec4515',
    referenceChars: 409,
    edits: [
      {
        startOffset: 214,
        endOffset: 244,
        replacement: 'the birthday wishes she deserved',
        expectedHash: '378a255704db9c5d5786fa63bd9f9b6bf9eb41d609533b2ed270d79ddb3e1e44',
        author: 'gpt-6-astra',
        rationale: 'Restore source deservedness rather than formal status.',
      },
    ],
    donor: {
      startOffset: 288,
      endOffset: 611,
      hash: '7aa1ccfdf586f05ebbb6d11f564c2d82e4bd77f535e7306fd5b42d821fcc11e2',
    },
    damages: [
      {
        kind: 'deletion',
        hash: '33e7a092b11e8521a46d6ca34e6e7edffb6db0d31437706b529102dc05f59af0',
        changedChars: 143,
      },
      {
        kind: 'insertion',
        hash: 'da851685bf948a33f594d7a008c431e0ef997bf662c0d9ba85fc395cb24d8ad3',
        changedChars: 98,
      },
      {
        kind: 'alteration',
        hash: '48bfc175307479254ee2b82f01accb74d232d539ab8b9e165df7b5a703e610b6',
        changedChars: 4,
      },
    ],
    reviewedOn: '2026-09-11',
  },
];

//endregion Source-reviewed calibration manifest
