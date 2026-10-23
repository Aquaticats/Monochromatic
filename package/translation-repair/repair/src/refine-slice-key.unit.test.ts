/**
 Tests for what makes two runs' refinements the same refinement.

 WHAT THESE PIN is the half of a cache that fails silently. A key that is too
 WIDE discards settled work on an unrelated change, which is expensive and
 obvious. A key that is too NARROW returns a rewrite reached under a different
 question, and nothing looks wrong: the slice text matches, so the key
 matches, and a run publishes wording it never bought.

 The INCUMBENT is the member that looks wrong. It reaches no prompt in this
 stage at all, and it is in the key because the settlement stores a `changed`
 flag computed against it and drops the confirmed set wherever a rewrite
 lands back on the archive wording. A key blind to it returns a verdict
 reached against wording this run no longer carries, and the resume then
 throws rather than correcting itself.

 The DEFINITIONS are the case this stage adds over the others. They are
 collected from the whole assembled document rather than from this slice, so a
 neighbouring slice settling differently changes what this rewriter is shown.
 A key blind to that resumes a stale rewrite after its neighbour moves, which
 is the failure already recorded once at the accuracy window.

 Content fixtures are cat-themed invention. No corpus content appears here.
 Model identifiers come from the catalog, because `RosterModelId` is a
 closed union and an invented one does not typecheck.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import type { AdjudicatedIssue, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  REFINE_CACHE_VERSION,
  refineRunShape,
  refineSliceKey,
} from '../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Roster this run asks, as the phase assembles one.
 */
const RUN_SHAPE = refineRunShape({
  refinerModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
  judgeModelIds: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
  checkerModelIds: [SEAT_SYNTHETIC_VISION_WITHHELD,],
},);

/**
 One accepted claim, whose wording reaches the checkers.
 */
const ISSUE = {
  issueId: 'adjudicated/one',
  status: 'accepted',
  severity: 'major',
  claims: [],
  tallies: {},
} as unknown as AdjudicatedIssue;

/**
 Every input the key covers, as one run supplies them.
 */
const BASE = {
  runShape: RUN_SHAPE,
  sourceText: '猫猫每天下午都在窗台上晒太阳。',
  repairedText: 'The cat sunbathes on the windowsill every afternoon.',
  incumbentText: 'The cat suns herself on the windowsill each afternoon.',
  definitions: '[nap]: https://example.invalid/nap',
  declaredNames: ['Mimi',],
  issues: [ISSUE,],
  resolvedIssueIds: ['adjudicated/one',],
  nonTranslationStanding: false,
} as const;

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: refineSliceKey.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'IS AT VERSION 16: the reader of a model\'s reply takes a value that differs from a listed enum or '
            + 'const value only in capitalization as the listed value, and the key hashes the version, the shape of '
            + 'the run and the texts, not the reader, so only this number retires a record written under '
            + 'the reader before',
          fn: async () => {
            expect(REFINE_CACHE_VERSION,).toBe(16,);
          },
        },),
        it({
          name: 'ANSWERS THE SAME KEY for the same question asked twice, which is '
            + 'the property the whole resume rests on: without it every run rebuys '
            + 'a lane it already paid for',
          fn: async () => {
            expect(refineSliceKey(BASE,),).toBe(refineSliceKey(BASE,),);
          },
        },),

        it({
          name: 'ANSWERS THE PINNED KEY for the base question, so a change to the version, the shape or the fields '
            + 'moves it on purpose (the version moved from 6 to 7 with the n159 readers: a stored refinement was '
            + 'decided by quote and word-edge readers that changed; and from 7 to 8 with n166: a stored refinement '
            + 'was written under a house tense rule that gave quoted words no exception in the sentence that puts the '
            + 'life in the past; and from 8 to 9 with the XingZ6015 read: a stored refinement was written under '
            + 'reader protection that reached past the account of how a death or an attempt happened, and under the '
            + 'old declared-name floor; and from 9 to 10 with the cut of a translators\' note alone on a quoted line: '
            + 'a stored refinement was chosen from a slate that showed a bare quote line where the note had stood; '
            + 'and from 10 to 11 with the cut of the notes from the prober, the resolution checker and the objection '
            + 'correction\'s slate: a stored refinement was judged over texts that showed every note between '
            + 'translators; and from 11 to 12 with the ballots that give their reason first (owner, 2026-10-21): a '
            + 'stored refinement was chosen on a selection ballot that asked for the candidate before its reason; and '
            + 'from 12 to 13 with the answer schemas in alphabetical order (ledger B762): a stored refinement was '
            + 'answered under the old reply keys; and from 13 to 14 with the quorum rule of 2026-10-22 (ledger B816): '
            + 'a stored refinement was gathered before a short bench could close one voice fewer than every live '
            + 'seat; and from 14 to 15 with the fence length moved out of the system text of the slates, R4 of the '
            + 'third review of the Claude route; and from 15 to 16 with the reader that takes a capitalization-only '
            + 'difference from a listed value as the listed value: a stored refinement may rest on a round that lost '
            + 'such a voice)',
          fn: async () => {
            expect(refineSliceKey(BASE,),).toBe('dc6b56d4e49a54a9744cc6a573b1cdcf864df31b53ab2fbdb612cee43ab2bb97',);
          },
        },),

        it({
          name: 'MOVES WHEN THE DEFINITIONS MOVE, which is what separates this key '
            + 'from the accuracy pass\'s own. Definitions come from the whole '
            + 'assembled document, so a neighbouring slice settling differently '
            + 'changes what this rewriter is shown, and a key blind to it resumes a '
            + 'rewrite made against a page that no longer exists',
          fn: async () => {
            expect(
              refineSliceKey({
                ...BASE,
                definitions: '[nap]: https://example.invalid/elsewhere',
              },),
            ).not.toBe(refineSliceKey(BASE,),);
          },
        },),

        it({
          name: 'MOVES WHEN THE ACCURACY TEXT MOVES, since that text is both what '
            + 'the rewriter is handed and what the damage probe measures against',
          fn: async () => {
            expect(
              refineSliceKey({
                ...BASE,
                repairedText: 'The cat sunbathes on the sill each afternoon.',
              },),
            ).not.toBe(refineSliceKey(BASE,),);
          },
        },),

        it({
          name: 'MOVES WHEN ONLY THE INCUMBENT MOVES, which reads like a mistake '
            + 'until the stored record is read: no rewriter, judge or checker is '
            + 'ever shown the archive wording, but the settlement computes '
            + '`changed` against it and drops the confirmed set wherever a rewrite '
            + 'lands back on it. A field no model reads still belongs in a key '
            + 'whose record is computed from it',
          fn: async () => {
            expect(
              refineSliceKey({
                ...BASE,
                incumbentText: 'The cat suns herself on the sill each afternoon.',
              },),
            ).not.toBe(refineSliceKey(BASE,),);
          },
        },),

        it({
          name: 'MOVES WHEN THE SOURCE MOVES, because the source is the standard '
            + 'every rewrite is judged faithful against',
          fn: async () => {
            expect(
              refineSliceKey({
                ...BASE,
                sourceText: '猫猫每天早上都在窗台上晒太阳。',
              },),
            ).not.toBe(refineSliceKey(BASE,),);
          },
        },),

        it({
          name: 'MOVES WHEN THE CONFIRMED SET MOVES, because it decides what a '
            + 'rollback is measured against: the same rewrite ships under one set '
            + 'and is rolled back under another',
          fn: async () => {
            expect(
              refineSliceKey({
                ...BASE,
                resolvedIssueIds: [],
              },),
            ).not.toBe(refineSliceKey(BASE,),);
          },
        },),

        it({
          name: 'MOVES WHEN AN ISSUE IS REWORDED under an unchanged identifier, '
            + 'because the checkers read the wording rather than the name',
          fn: async () => {
            expect(
              refineSliceKey({
                ...BASE,
                issues: [
                  {
                    ...ISSUE,
                    severity: 'minor',
                  } as unknown as AdjudicatedIssue,
                ],
              },),
            ).not.toBe(refineSliceKey(BASE,),);
          },
        },),

        it({
          name: 'MOVES WHEN THE NON-TRANSLATION VERDICT FLIPS, since that decides '
            + 'whether the lane runs at all and a resumed slice must not skip a '
            + 'rewrite this run would have bought',
          fn: async () => {
            expect(
              refineSliceKey({
                ...BASE,
                nonTranslationStanding: true,
              },),
            ).not.toBe(refineSliceKey(BASE,),);
          },
        },),

        it({
          name: 'MOVES WHEN THE DECLARED NAMES MOVE, because they are the guard a '
            + 'rewrite is held to rather than decoration on the prompt',
          fn: async () => {
            expect(
              refineSliceKey({
                ...BASE,
                declaredNames: ['Mimi', 'Momo',],
              },),
            ).not.toBe(refineSliceKey(BASE,),);
          },
        },),
      ],
    },),

    describe({
      name: refineRunShape.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'SEPARATES TWO REWRITER ROSTERS, so a resumed slice cannot return '
            + 'wording a voice this run never asked produced',
          fn: async () => {
            expect(
              refineRunShape({
                refinerModelIds: [SEAT_HYPER_OPENROUTER_UNMEASURED,],
                judgeModelIds: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
                checkerModelIds: [SEAT_SYNTHETIC_VISION_WITHHELD,],
              },),
            ).not.toBe(RUN_SHAPE,);
          },
        },),

        it({
          name: 'SEPARATES TWO CHECKER ROSTERS even though checkers never rewrite anything. They decide whether a '
            + 'rewrite is rolled back for breaking a confirmed repair, so a different checker roster ships wording '
            + 'this one refused',
          fn: async () => {
            expect(
              refineRunShape({
                refinerModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
                judgeModelIds: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
                checkerModelIds: [SEAT_HYPER_OPENROUTER_UNMEASURED,],
              },),
            ).not.toBe(RUN_SHAPE,);
          },
        },),

        it({
          name: 'SEPARATES TWO IDENTITY CONTEXTS, which is front-matter-derived prompt content that varies per pair '
            + 'and measurably changes what a rewriter produces',
          fn: async () => {
            expect(
              refineRunShape({
                refinerModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
                judgeModelIds: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
                checkerModelIds: [SEAT_SYNTHETIC_VISION_WITHHELD,],
                identityContext: 'Mimi is the cat.',
              },),
            ).not.toBe(RUN_SHAPE,);
          },
        },),

        it({
          name: 'MOVES WHEN THE WINDOW MOVES. The damage probe inside the '
            + 'settlement is shown the passages either side, and its findings ride '
            + 'in the cached record, so a slice audited against its neighbours was '
            + 'asked a different question from the same slice audited alone and '
            + 'nothing else records which it was',
          fn: async () => {
            expect(
              refineSliceKey({
                ...BASE,
                neighbouringSourceText: '邻居的橘猫在门口等鱼干。',
              },),
            ).not.toBe(refineSliceKey(BASE,),);
          },
        },),

        it({
          name: 'DISTINGUISHES THE TWO SIDES OF THE WINDOW carrying identical '
            + 'text, which is the repair slice key defect exactly: spread bare into a positional array '
            + 'a source-only window and an incumbent-only window hash the same, '
            + 'and one cached audit then serves two different questions. '
            + 'Asymmetric windows are real, since a neighbour that is an insertion '
            + 'anchor has original text and no archive text',
          fn: async () => {
            /**
             Text placed on one side of the window and then the other, so only
             the LABEL differs between the two keys.
             */
            const nearby = '邻居的橘猫在门口等鱼干。';

            expect(
              refineSliceKey({
                ...BASE,
                neighbouringSourceText: nearby,
              },),
            ).not.toBe(
              refineSliceKey({
                ...BASE,
                neighbouringIncumbentText: nearby,
              },),
            );
          },
        },),

        it({
          name: 'READS AN EMPTY WINDOW AS AN ABSENT ONE, because a slice with no neighbours is asked exactly what a '
            + 'caller without the parameter asks: `introduced-defect-wire` renders no nearby block for either. Keying '
            + 'them apart would rebuy every document-edge slice to reach the identical answer',
          fn: async () => {
            expect(
              refineSliceKey({
                ...BASE,
                neighbouringSourceText: '',
                neighbouringIncumbentText: '',
              },),
            ).toBe(refineSliceKey(BASE,),);
          },
        },),

        it({
          name: 'READS AN ABSENT IDENTITY CONTEXT AS AN EMPTY ONE rather than as a '
            + 'separate question, so a pair declaring nothing resumes across runs '
            + 'instead of rebuying its whole lane',
          fn: async () => {
            expect(
              refineRunShape({
                refinerModelIds: [SEAT_HYPER_OPENROUTER_VISION_EDITOR,],
                judgeModelIds: [SEAT_SYNTHETIC_VISION_NO_OPENROUTER,],
                checkerModelIds: [SEAT_SYNTHETIC_VISION_WITHHELD,],
                identityContext: '',
              },),
            ).toBe(RUN_SHAPE,);
          },
        },),
      ],
    },),
  ],
},);
