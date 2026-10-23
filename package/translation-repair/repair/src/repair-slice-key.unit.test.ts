/**
 Tests for the repair lane's cache key.

 THE KEY HAS NO OTHER WITNESS. Persist and resume both call the same function,
 so a change to how it is derived produces no failure anywhere: every run
 simply misses the cache and buys every slice again, and the only symptom is
 quota. The golden hash in this file is the witness, and it exists to fail when the
 derivation moves without the version moving with it.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { RepairModels, } from '@monochromatic-dev/translation-repair-assembly';
import { prepareDocumentPair, } from '@monochromatic-dev/translation-repair-adjudication';
import {
  repairRunShape,
  repairSliceKey,
  SLICE_CACHE_VERSION,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Rosters every case keys against.
 */
const MODELS: RepairModels = {
  criticModelIds: [
    SEAT_SYNTHETIC_VISION_WITHHELD,
    SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  ],
  panelModelIds: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
  editorModelIds: [SEAT_SYNTHETIC_TEXT_EVERYWHERE,],
  judgeModelIds: [SEAT_HYPER_VISION,],
  refinerModelIds: [SEAT_HYPER_OPENROUTER_UNMEASURED,],
  checkerModelIds: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
};

/**
 Slice these cases key.
 */
const SLICE = {
  sourceText: '猫猫在窗台上打盹。',
  targetText: 'The cat naps on the sill.',
  lineStructured: false,
};

/**
 Keys the fixture slice under a run shape.

 @param runShape - what a run asks

 @returns Key for the fixture slice

 @example
 ```ts
 const key = keyed({ runShape, },);
 ```
 */
function keyed({ runShape, }: { readonly runShape: string; },): string {
  return repairSliceKey({
    runShape,
    ...SLICE,
  },);
}

await describe({
  name: repairSliceKey.name,
  children: [
    it({
      name:
        'produces a KNOWN hash for known inputs, which is the only thing standing between a change '
          + 'to this derivation and every settled slice in the corpus silently missing the cache. '
          + 'A DERIVATION change moves this hash and needs SLICE_CACHE_VERSION moved with it; a '
          + 'change to the FIXTURE INPUTS of this case moves it too, and must not, because those inputs are '
          + 'already part of the key and invalidate their own entries',
      fn: async () => {
        expect(SLICE_CACHE_VERSION,).toBe(46,);

        // MOVED THREE TIMES ON 2026-08-20, FOR THREE DIFFERENT REASONS, which
        // is the whole point of keeping the note.
        //
        // First the fixture roster was renamed when the provider replaced one
        // model: the INPUT moved and the derivation did not, so no version moved
        // with it. Real entries on disk carry the old roster inside their own
        // key and stop matching on their own, while a bump would additionally
        // discard every entry whose roster never changed, which is a corpus of
        // work thrown away for nothing.
        //
        // Then the declared-name guard reached this lane's acceptance, and that
        // one DID need version 28: the declarations already ride in the run
        // shape, so a slice settled before the guard existed keys identically to
        // one settled after and would resume with its refusal never asked.
        //
        // Then version 29, for a change to the RECORD rather than to the
        // question: the outcome grew `rounds` and an unconditional
        // `droppedDeclaredNames`. A version-28 record fails the loader's own
        // shape check and is recomputed regardless, so the bump buys nothing at
        // runtime; it is here so one version number never names two record
        // shapes, which is what this file's versioning exists to prevent.
        //
        // Then version 30, on 2026-08-23, the same kind again: the outcome grew
        // `authorship`, naming who wrote the repaired text so the naturalness
        // recheck can discount a checker judging its own work after a resume.
        //
        // THE HASH MOVED ON 2026-08-24 WITHOUT THE DERIVATION MOVING, which is
        // the case the name of this case says must not happen, so it is written down
        // rather than quietly re-pinned. The owner blocklisted
        // `zai-org/GLM-4.7-Flash`, this case's fixture roster named it, and a
        // fixture that names a model no longer on the roster does not compile.
        // The roster IS part of the key by design, so a fixture roster change
        // moves this hash and invalidates only this fixture's own entries.
        // `SLICE_CACHE_VERSION` is deliberately NOT bumped: nothing about the
        // question changed, and bumping it would discard every settled slice in
        // the corpus to record a test fixture edit.
        //
        // THE SAME CASE MOVED THIS LITERAL TWICE ON 2026-08-29. GLM-5.3-Flash
        // replaced GLM-5.2, then the owner removed Nemotron from every stage
        // and this fixture replaced it with an active roster id. Both ids are
        // already key input. IT MOVED A THIRD TIME ON 2026-09-16, when the owner
        // removed deepseek-v4-pro-0813 and this fixture's refiner became
        // deepseek-v4.1-flash, again an id already in key input.
        //
        // VERSION 31 ON 2026-09-24 moved it the ordinary way: the version is
        // key input and the record grew `filedBy` on every adjudicated issue,
        // so the derivation moved and the version moved with it.
        //
        // VERSION 32 ON 2026-09-27 the same way: one cast checker ballot no
        // longer resolves an issue, which changes what a settled slice shipped.
        //
        // VERSION 33 THE SAME DAY: the panel sheet asks each verdict's reason
        // before its vote, and every stored ballot carries it (owner, "Reason
        // before vote"), so the question and the record both moved.
        //
        // VERSION 34 ON 2026-09-28 the same way (ledger L5): the tally holds
        // an acceptance settled at neutral for a human, and the critic and
        // panel sheets define the severity scale.
        //
        // VERSION 35 IN n156 (typography read from the page parse): the version is key input, so the literal moved with it
        // and was filled in by running this case after the bump, which is the only way to get a hash.
        //
        // VERSION 36 IN n159 (the context of a mark read as the text a reader sees): moved the same way.
        //
        // VERSION 37 IN n166 (the house tense rule says, in the sentence that puts the life in the past, that quoted words keep
        // the speaker's own tense): moved the same way.
        //
        // VERSION 38 FOR THE XingZ6015 READ (reader protection scoped to the account of how a death or an attempt happened,
        // translators' notes cut from the judges, the declared-name floor meeting a swap of declared forms): moved the same way.
        //
        // VERSION 39 ON 2026-10-20 (a translators' note alone on a quoted line is cut with its line): moved the same way.
        //
        // VERSION 40 ON 2026-10-20 (the critic, the panel, the prober, the resolution checker and the coverage sheet cut the
        // translators' notes they showed): moved the same way.
        //
        // VERSION 41 ON 2026-10-21 (the selection ballot and the coverage judge give their reason before their choice): moved
        // the same way.
        //
        // VERSION 42 ON 2026-10-21 (the answer schemas of the repair lane declare their properties in alphabetical order,
        // ledger B762): moved the same way.
        //
        // VERSION 43 ON 2026-10-22 (a short bench closes one voice below every live seat once the grace ends, ledger B816):
        // moved the same way.
        //
        // VERSION 44 ON 2026-10-22 (the outcome carries `silentStages`, and the cache refuses a settlement that names a
        // silent stage; XingZ6018 slice 85, a panel that heard nobody cached as examined): moved the same way, for the record
        // and the rule that stores it.
        //
        // VERSION 44 ON 2026-10-22 ON THE CLAUDE ROUTE (the fence length left the system text of the candidate
        // selection, R4 of the third review of the Claude route): moved the same way.
        //
        // VERSION 45 ON 2026-10-23 ON THE CLAUDE ROUTE (the reader takes a value that differs from a listed enum or
        // const value only in capitalization as the listed value): moved the same way.
        //
        // VERSION 46 ON 2026-10-23 (the merge of the silent-stage move of main with the fence-length and case-fold
        // moves of the Claude route, which no line held as one number): moved the same way.
        expect(keyed({ runShape: repairRunShape({ models: MODELS, },), },),)
          .toBe('cd4085ca1b4c02ea7b32420cf1a97c487718385a9dc6b0f44c765a59872a112f',);
      },
    },),
    it({
      name: 'KEEPS THE KEY WHATEVER CHECKER SELF-CERTIFICATION SAYS (ledger X11, M37): the flag only lets '
        + 'a roster run, and a roster it admits is asked and weighed the same either way, so splitting the '
        + 'cache on it would rebuy settled slices for no different answer',
      fn: async () => {
        /**
         Key under the fixture, which states no flag.
         */
        const settled = keyed({ runShape: repairRunShape({ models: MODELS, },), },);
        expect([
          keyed({ runShape: repairRunShape({ models: { ...MODELS, checkerSelfCertificationPermitted: true, }, },), },)
            === settled,
          keyed({ runShape: repairRunShape({ models: { ...MODELS, checkerSelfCertificationPermitted: false, }, },), },)
            === settled,
        ],).toEqual([true, true,],);
      },
    },),
    it({
      name:
        'moves the key for every input a resumed slice would be wrong about: the roster, the '
          + 'declared names, either text, and the line-structure verdict the enclosing chunk carries',
      fn: async () => {
        /**
         Key under the unchanged fixture.
         */
        const settled = keyed({ runShape: repairRunShape({ models: MODELS, },), },);
        expect(keyed({
          runShape: repairRunShape({
            models: {
              ...MODELS,
              criticModelIds: [SEAT_SYNTHETIC_VISION_WITHHELD,],
            },
          },),
        },),).not
          .toBe(settled,);
        expect(keyed({
          runShape: repairRunShape({
            models: MODELS,
            identityContext: 'Name: Whiskers',
          },),
        },),).not
          .toBe(settled,);
        // THE CITED REFERENCES (class thirty-five) are evidence the critic and
        // panel sheets read, so they move the key; an original that links
        // nowhere keys exactly as before, so the 33 entries that cite nothing
        // keep their settled slices.
        expect(keyed({
          runShape: repairRunShape({
            models: MODELS,
            referenceContext: '- reference 1 https://cats.example/post: the kitten has an older sister.',
          },),
        },),).not
          .toBe(settled,);
        expect(keyed({
          runShape: repairRunShape({
            models: MODELS,
            referenceContext: '',
          },),
        },),).toBe(settled,);

        /**
         Same run, each slice field moved in turn.
         */
        const runShape = repairRunShape({ models: MODELS, },);
        for (const moved of [
          { sourceText: '猫猫在窗台上睡觉。', },
          { targetText: 'The cat sleeps on the sill.', },
          { lineStructured: true, },
        ]) {
          expect(repairSliceKey({
            runShape,
            ...SLICE,
            ...moved,
          },),).not
            .toBe(settled,);
        }
      },
    },),
    it({
      name:
        'is the same key for the same text WHEREVER the slice sits, which is the point of version 26. '
          + 'The index used to be in here, so inserting one slice at the top of a document discarded '
          + 'every slice below it however untouched its text, and one-sided slicing inserts a slice for '
          + 'every untranslated section',
      fn: async () => {
        /**
         Document whose two sections are byte-identical on both sides, which is
         the only shape where two slices of one document can share a key. The
         pinned corpus contains none, so it is invented here rather than
         assumed away.
         */
        const prepared = prepareDocumentPair({
          sourceText: '## 甲\n\n猫猫喜欢晒太阳。\n\n## 甲\n\n猫猫喜欢晒太阳。\n',
          targetText: '## A\n\nThe cat likes the sun.\n\n## A\n\nThe cat likes the sun.\n',
        },);
        expect(prepared.slices,).toHaveLength(2,);

        /**
         Run shape both keys are taken under.
         */
        const runShape = repairRunShape({ models: MODELS, },);

        /**
         Both slices keyed exactly as the driver keys them.
         */
        const keys = prepared.slices
          .map(function toKey(slice,): string {
            return repairSliceKey({
              runShape,
              sourceText: slice.source
                .text,
              targetText: slice.target
                .text,
              lineStructured: prepared.lineStructuredSliceIndices
                .has(slice.target
                  .sliceIndex,),
            },);
          },);
        expect(prepared.slices
          .map(function toIndex(slice,): number {
            return slice.target
              .sliceIndex;
          },),).toEqual([
            0,
            1,
          ],);
        expect(keys[0],).toBe(keys[1],);
      },
    },),
  ],
},);
