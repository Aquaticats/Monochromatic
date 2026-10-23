/**
 Tests for the lane contest`s cache key.

 THE KEY HAS NO OTHER WITNESS. Persist and resume both call the same function,
 so a change to how it is derived produces no failure anywhere: every run
 simply misses the cache and buys every contested slice again, and the only
 symptom is quota. The golden hash in this file is the witness, and it exists to fail
 when the derivation moves without the version moving with it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  LANE_CONTEST_CACHE_VERSION,
  laneContestRunShape,
  laneContestSliceKey,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Roster every case here asks.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
] as const;

/**
 Original of the slice every case describes.
 */
const SOURCE_NAP = '猫猫在书店的阁楼里睡觉。';

/**
 Archive`s own English for it.
 */
const ARCHIVE_NAP = 'The cat sleeps in the bookshop attic.';

/**
 Wording the repair lane would ship.
 */
const REPAIR_NAP = 'The cat naps in the bookshop attic.';

/**
 Wording the translate lane would ship.
 */
const TRANSLATE_NAP = 'The cat dozes in the attic of the bookshop.';

/**
 Every input to the key, in one place, so a case changes exactly one of them.

 @returns Key inputs for the `SOURCE_NAP` slice

 @example
 ```ts
 const key = laneContestSliceKey(catInputs(),);
 ```
 */
function catInputs(): Parameters<typeof laneContestSliceKey>[0] {
  return {
    runShape: laneContestRunShape({ modelIds: ROSTER, },),
    sourceText: SOURCE_NAP,
    incumbentText: ARCHIVE_NAP,
    incumbentKind: 'present',
    repairText: REPAIR_NAP,
    translateText: TRANSLATE_NAP,
    lineStructured: false,
  };
}

await describe({
  name: laneContestSliceKey.name,
  children: [
    it({
      name: 'IS AT VERSION 18: the reader of a model\'s reply takes a value that differs from a listed enum or const '
        + 'value only in capitalization as the listed value, and the key hashes the version, the shape of the run and '
        + 'the texts, not the reader, so only this number retires a record written under the reader before',
      fn: async () => {
        expect(LANE_CONTEST_CACHE_VERSION,).toBe(18,);
      },
    },),
    it({
      name:
        'DERIVES THE PINNED KEY for a fixed set of inputs, which is the only thing that fails when the '
          + 'derivation moves and the version does not: a moved key costs quota and nothing else, so '
          + 'nothing else would notice',
      fn: async () => {
        /**
         Key these fixed inputs derive today.
         */
        const key = laneContestSliceKey(catInputs(),);
        // MOVED ON 2026-08-29 when the fixture roster replaced GLM-5.2 with
        // GLM-5.3-Flash, then when excluded-lane participation stopped waiting
        // for every roster seat. Version 5 floors complete-form contributor
        // authority. Roster identity is explicit key material. Version 6
        // (2026-09-28, ledger M28) re-keys every ballot cast under version 5.
        // Version 7 (2026-10-15) re-keys every ballot cast before the
        // eligibility validator read prose guillemets from the page parse.
        // Version 8 (2026-10-15, n159) re-keys every ballot cast before the
        // validator refused a refused candidate that carries a guillemet.
        // Version 9 (2026-10-15, n166) re-keys every ballot cast before the
        // house tense rule said, in the sentence that puts the life in the past,
        // that quoted words keep the speaker's own tense.
        // Version 10 (2026-10-18) re-keys every ballot cast before the tense
        // candidates listed the narration between marks of different
        // single-quotation pairs.
        // Version 11 (2026-10-20) re-keys every ballot cast before reader
        // protection was scoped to the account of how a death or an attempt
        // happened and translators' notes were cut from the judges.
        // Version 12 (2026-10-20) re-keys every ballot cast before a note
        // between translators alone on a quoted line was cut with its line.
        // Version 13 (2026-10-20) re-keys every ballot cast before a candidate
        // that respells a Han name the archive spells inside a sentence was
        // refused.
        // Version 14 (2026-10-21) re-keys every ballot cast before the contest
        // ballot gave its reason before its choice.
        // Version 15 (2026-10-21) re-keys every ballot cast before the answer
        // schemas declared their properties in alphabetical order (ledger
        // B762).
        // Version 16 (2026-10-22) re-keys every ballot cast before a short
        // bench closed one voice fewer than every live seat once the straggler
        // grace ended (ledger B816).
        // Version 17 (2026-10-22) re-keys every ballot cast before the rendered-break
        // counts moved out of the system text into the user turn (R4 of the third
        // review of the Claude route).
        expect(key,).toBe('45a984ed36bfb4ffff0999650beb1dbfddd09061a4ede943b1a9999c74df7bbd',);
      },
    },),
    it({
      name: 'SEPARATES FRONT MATTER POLICY from identical Markdown candidate texts',
      fn: async () => {
        expect(laneContestSliceKey({
          ...catInputs(),
          syntax: 'front-matter',
        },),).not.toBe(laneContestSliceKey(catInputs(),),);
      },
    },),
    it({
      name:
        'SEPARATES two contests over identical candidates and different ARCHIVE wording, since the judge '
          + 'is shown the archive rendering as evidence and is therefore not being asked the same question',
      fn: async () => {
        expect(laneContestSliceKey({
          ...catInputs(),
          incumbentText: 'The cat sleeps in the attic.',
        },),).not.toBe(laneContestSliceKey(catInputs(),),);
      },
    },),
    it({
      name:
        'SEPARATES a slice the archive never rendered from one it rendered as nothing, which carry the '
          + 'same empty wording and are opposite facts about the passage',
      fn: async () => {
        expect(laneContestSliceKey({
          ...catInputs(),
          incumbentText: '',
          incumbentKind: 'absent',
        },),).not.toBe(laneContestSliceKey({
          ...catInputs(),
          incumbentText: '',
        },),);
      },
    },),
    it({
      name: 'SEPARATES two rosters asked the same question, since ballots one roster cast are not ballots another cast',
      fn: async () => {
        expect(laneContestSliceKey({
          ...catInputs(),
          runShape: laneContestRunShape({ modelIds: [ROSTER[0],], },),
        },),).not.toBe(laneContestSliceKey(catInputs(),),);
      },
    },),
    it({
      name:
        'SEPARATES a run that declared names from one that declared none, since a judge shown an attested '
          + 'name reads a candidate carrying it differently',
      fn: async () => {
        expect(laneContestSliceKey({
          ...catInputs(),
          runShape: laneContestRunShape({
            modelIds: ROSTER,
            identityContext: 'The narrator is called Whiskers.',
          },),
        },),).not.toBe(laneContestSliceKey(catInputs(),),);
      },
    },),
    it({
      name: 'SEPARATES a run shown the pages the original cites from one shown none, and keys an entry '
        + 'that cites nothing exactly as before (class thirty-six, 2026-09-16)',
      fn: async () => {
        expect(laneContestSliceKey({
          ...catInputs(),
          runShape: laneContestRunShape({
            modelIds: ROSTER,
            referenceContext: '- reference 1 https://blog.example/mittens ("In memory of Mittens"): Mittens had an older sister who was also a tabby.',
          },),
        },),).not.toBe(laneContestSliceKey(catInputs(),),);
        expect(laneContestSliceKey({
          ...catInputs(),
          runShape: laneContestRunShape({
            modelIds: ROSTER,
            referenceContext: '',
          },),
        },),).toBe(laneContestSliceKey(catInputs(),),);
      },
    },),
    it({
      name: 'SEPARATES a run whose entry records a death, and the subject it names, from one that records none, and keys an entry that records none exactly as before',
      fn: async () => {
        /**
         The shape of an entry that records a death and names its subject.
         */
        const named = laneContestRunShape({
          modelIds: ROSTER,
          lifeEnded: true,
          subjectNames: ['Mittens',],
        },);
        expect(laneContestSliceKey({
          ...catInputs(),
          runShape: named,
        },),).not.toBe(laneContestSliceKey(catInputs(),),);
        expect(laneContestSliceKey({
          ...catInputs(),
          runShape: laneContestRunShape({
            modelIds: ROSTER,
            lifeEnded: true,
            subjectNames: ['Whiskers',],
          },),
        },),).not.toBe(laneContestSliceKey({
          ...catInputs(),
          runShape: named,
        },),);
        expect(laneContestSliceKey({
          ...catInputs(),
          runShape: laneContestRunShape({
            modelIds: ROSTER,
            lifeEnded: false,
          },),
        },),).toBe(laneContestSliceKey(catInputs(),),);
      },
    },),
    it({
      name: 'SEPARATES a run whose subject carries a declared alias from one that names the subject alone, since the tense candidates read every name',
      fn: async () => {
        expect(laneContestSliceKey({
          ...catInputs(),
          runShape: laneContestRunShape({
            modelIds: ROSTER,
            lifeEnded: true,
            subjectNames: [
              'Mittens',
              'Whiskers',
            ],
          },),
        },),).not.toBe(laneContestSliceKey({
          ...catInputs(),
          runShape: laneContestRunShape({
            modelIds: ROSTER,
            lifeEnded: true,
            subjectNames: ['Mittens',],
          },),
        },),);
      },
    },),
    it({
      name: 'SEPARATES the two candidates by SIDE, so swapping which lane produced which is a different question',
      fn: async () => {
        expect(laneContestSliceKey({
          ...catInputs(),
          repairText: TRANSLATE_NAP,
          translateText: REPAIR_NAP,
        },),).not.toBe(laneContestSliceKey(catInputs(),),);
      },
    },),
    it({
      name: 'SEPARATES a question asked with probe claims against the repair candidate from one asked '
        + 'without, and KEYS an empty claim list exactly as no claims, so ballots bought before the '
        + 'claims existed still resume',
      fn: async () => {
        expect(laneContestSliceKey({
          ...catInputs(),
          repairDamageClaims: ['- minimax-m3 [tense] quotes "naps": the page holds past tense',],
        },),).not.toBe(laneContestSliceKey(catInputs(),),);
        expect(laneContestSliceKey({
          ...catInputs(),
          repairDamageClaims: [],
        },),).toBe(laneContestSliceKey(catInputs(),),);
      },
    },),
    it({
      name: 'SEPARATES a question asked with a dispute note against the archive from one asked without, and two '
        + 'different notes from each other, since the judges read the note as evidence (class one hundred eight)',
      fn: async () => {
        /**
         Key for the slice asked with one note against its archive rendering.
         */
        const disputed = laneContestSliceKey({
          ...catInputs(),
          archiveDisputeNote: 'The archive adds that the cat sleeps on a cushion.',
        },);

        /**
         Key for the same slice asked with no note.
         */
        const undisputed = laneContestSliceKey(catInputs(),);
        expect(disputed,).not.toBe(undisputed,);
        expect(disputed,).not.toBe(laneContestSliceKey({
          ...catInputs(),
          archiveDisputeNote: 'The archive adds that the attic is dusty.',
        },),);
      },
    },),
  ],
},);
