/**
 Tests for the refusal that stops a winning repair dropping a declared name.

 WHAT THIS EXISTS TO CATCH. The repair lane's editor judges were shown two
 candidates differing only in a declared alias and chose the one that dropped
 it, six judges out of six, each reasoning that the alias had no basis in the
 original. The lane records no ballots, so nothing about that decision reaches
 a settled artifact and no reader could have found it. The refusal is
 therefore deterministic, and this file proves the settlement CONSULTS it: a
 guard computed and not consulted looks exactly like a guard that passed.

 IT ALSO PINS WHICH VERDICT MOVES. `patchSelected` says the patch beat the
 archive on the measurements, and that stays true through a refusal, because
 it did. Only the text that ships changes. Collapsing the two would make a
 refusal indistinguishable from the case this file's subject already owns, a
 patch that wins selection and whose envelope operations write no byte.

 A GUARD PROVES NOTHING UNTIL SHOWN TO FAIL, so the cases come in threes: a
 winning patch accepted, the same shape of patch refused for dropping a
 declared name, and that same dropping patch accepted once nothing is
 declared.

 Fixtures are cat-themed invention mirroring corpus structure only.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { parseDocument, } from '@monochromatic-dev/translation-repair-shared-contract';
import {
  type ChunkVerdict,
  settleChunkFromChecks,
  settleChunkVerdict,
} from '../dist/final/node/index.mjs';
import { catIssue, } from '@monochromatic-dev/translation-repair-shared-contract/ts/accepted-cat-issue.test-fixture.ts';

/**
 Slice these verdicts are settled for.
 */
const CHUNK_INDEX = 4;

/**
 Name the archive's front matter declares.
 */
const DECLARED_NAME = 'Meowmeow';

/**
 Alias it declares beside that, which is the form real judges removed.
 */
const DECLARED_ALIAS = 'Dumpling';

/**
 Forms the archive declares for this person.
 */
const DECLARED_NAMES: readonly string[] = [
  DECLARED_NAME,
  DECLARED_ALIAS,
];

/**
 Archive wording of the slice, carrying both declared forms.
 */
const INCUMBENT_TEXT =
  'Meowmeow, who everyone called Dumpling, kept the windowsill warm through that whole winter.';

/**
 Repair that fixes the accuracy complaint and keeps both forms.
 */
const KEEPS_EVERY_NAME =
  'Meowmeow, who everyone called Dumpling, kept the windowsill warm all that winter.';

/**
 Repair that fixes the same complaint and quietly drops the alias.
 */
const DROPS_THE_ALIAS = 'Meowmeow kept the windowsill warm all that winter.';

/**
 Archive wording that names the person by the display name alone, where the
 original names her by her alias (XingZ60 slice 80).
 */
const NAME_ONLY_INCUMBENT_TEXT = 'Meowmeow kept the windowsill warm through that whole winter.';

/**
 Repair that names her by the alias instead, so it carries a declared form the
 archive did not and loses the one it did.
 */
const SWAPS_THE_NAME_FOR_THE_ALIAS = 'Dumpling kept the windowsill warm all that winter.';

/**
 Measurements of a patch that beat the archive.

 ONE RESOLVED ISSUE AND NOTHING WORSE is the smallest shape that wins
 selection outright, so a case using it is testing the refusal rather than the
 ranking.
 */
const WINNING_MEASUREMENTS = {
  integrityOk: true,
  resolvedHighSeverity: 1,
  resolvedTotal: 1,
  regressedKnownIssues: 0,
  touchedRegionChars: INCUMBENT_TEXT.length,
};

/**
 Measurements of a patch that lost: it resolved nothing and broke the
 document.
 */
const LOSING_MEASUREMENTS = {
  integrityOk: false,
  resolvedHighSeverity: 0,
  resolvedTotal: 0,
  regressedKnownIssues: 1,
  touchedRegionChars: INCUMBENT_TEXT.length,
};

/**
 Settles one slice with a given patch and declaration list.

 @param patchedText - wording the editor produced

 @param declaredNames - forms preparation found in the front matter

 @param measurements - what the checks made of that patch

 @param incumbentText - archive wording of the slice, the fixture's own when absent

 @param declaredSubjectForms - the declared forms that name the subject, absent when preparation named none

 @returns Verdict the lane settled on

 @example
 ```ts
 const verdict = settleWith({ patchedText: DROPS_THE_ALIAS, declaredNames: DECLARED_NAMES, },);
 ```
 */
function settleWith(
  {
    patchedText,
    declaredNames,
    measurements = WINNING_MEASUREMENTS,
    incumbentText = INCUMBENT_TEXT,
    declaredSubjectForms,
  }: {
    readonly patchedText: string;
    readonly declaredNames: readonly string[];
    readonly measurements?: typeof WINNING_MEASUREMENTS;
    readonly incumbentText?: string;
    readonly declaredSubjectForms?: readonly string[];
  },
): ChunkVerdict {
  return settleChunkVerdict({
    sliceIndex: CHUNK_INDEX,
    incumbentText,
    patchedText,
    measurements,
    declaredNames,
    ...((declaredSubjectForms === undefined) ? {} : { declaredSubjectForms, }),
  },);
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: settleChunkVerdict.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'POSITIVE CONTROL: ACCEPTS a winning patch that keeps every declared form, so a later '
            + 'assertion that some settlement kept the archive text is reading a refusal rather than a '
            + 'patch that simply lost',
          fn: async () => {
            const verdict = settleWith({
              patchedText: KEEPS_EVERY_NAME,
              declaredNames: DECLARED_NAMES,
            },);
            expect(verdict.repairedText,).toBe(KEEPS_EVERY_NAME,);
            expect(verdict.patchSelected,).toBe(true,);
            expect(verdict.changed,).toBe(true,);
            expect(verdict.droppedDeclaredNames,).toEqual([],);
          },
        },),
        it({
          name: 'REFUSES a winning patch that drops a declared form, shipping the archive text and '
            + 'naming what would have gone, which is the decision no judge on this roster made unaided',
          fn: async () => {
            const verdict = settleWith({
              patchedText: DROPS_THE_ALIAS,
              declaredNames: DECLARED_NAMES,
            },);
            expect(verdict.repairedText,).toBe(INCUMBENT_TEXT,);
            expect(verdict.changed,).toBe(false,);
            expect(verdict.droppedDeclaredNames,).toEqual([DECLARED_ALIAS,],);
          },
        },),
        it({
          name: 'KEEPS reporting that the patch beat the archive on the measurements, since it did and '
            + 'a refusal nobody can see the cost of is a refusal nobody can audit',
          fn: async () => {
            const verdict = settleWith({
              patchedText: DROPS_THE_ALIAS,
              declaredNames: DECLARED_NAMES,
            },);
            expect(verdict.patchSelected,).toBe(true,);
          },
        },),
        it({
          name: 'ACCEPTS that same dropping patch when the archive declares nothing, so the refusal is '
            + 'attributable to the declared list and not to anything else about that wording',
          fn: async () => {
            const verdict = settleWith({
              patchedText: DROPS_THE_ALIAS,
              declaredNames: [],
            },);
            expect(verdict.repairedText,).toBe(DROPS_THE_ALIAS,);
            expect(verdict.changed,).toBe(true,);
          },
        },),
        it({
          name: 'ACCEPTS a winning patch that names the person by another declared form of hers, the shape of '
            + 'XingZ60 slice 80 where the patch won and was refused for dropping the display name the '
            + 'archive wrote where the original wrote the alias',
          fn: async () => {
            const verdict = settleWith({
              patchedText: SWAPS_THE_NAME_FOR_THE_ALIAS,
              declaredNames: DECLARED_NAMES,
              incumbentText: NAME_ONLY_INCUMBENT_TEXT,
              declaredSubjectForms: DECLARED_NAMES,
            },);
            expect(verdict.repairedText,).toBe(SWAPS_THE_NAME_FOR_THE_ALIAS,);
            expect(verdict.changed,).toBe(true,);
            expect(verdict.droppedDeclaredNames,).toEqual([],);
          },
        },),
        it({
          name: 'REFUSES that same swap when preparation named no subject identity, so the acceptance '
            + 'of that swap is attributable to the identity and to nothing else about that wording',
          fn: async () => {
            const verdict = settleWith({
              patchedText: SWAPS_THE_NAME_FOR_THE_ALIAS,
              declaredNames: DECLARED_NAMES,
              incumbentText: NAME_ONLY_INCUMBENT_TEXT,
            },);
            expect(verdict.repairedText,).toBe(NAME_ONLY_INCUMBENT_TEXT,);
            expect(verdict.changed,).toBe(false,);
            expect(verdict.droppedDeclaredNames,).toEqual([DECLARED_NAME,],);
          },
        },),
        it({
          name: 'SAYS WHY THE ARCHIVE WON a patch that resolved every accepted issue: it lost on integrity, the '
            + 'first rank, and the log could not say so (XingZ6022 slice 44 closed "unchanged, the archive won, 3/3 '
            + 'served accepted issues resolved" after a clean checker round and a clean probe)',
          fn: async () => {
            const verdict = settleWith({
              patchedText: KEEPS_EVERY_NAME,
              declaredNames: DECLARED_NAMES,
              measurements: {
                integrityOk: false,
                resolvedHighSeverity: 1,
                resolvedTotal: 3,
                regressedKnownIssues: 0,
                touchedRegionChars: INCUMBENT_TEXT.length,
              },
            },);
            expect(verdict.patchSelected,).toBe(false,);
            expect(verdict.patchLostOn,).toBe('integrity',);
          },
        },),
        it({
          name: 'SAYS A PATCH LOST ON REGRESSIONS where integrity held and it resolved nothing high, and names no '
            + 'reason for a patch that merely tied the archive',
          fn: async () => {
            /**
             Measurements of a patch whose integrity held and which made an accepted issue worse.
             */
            const regressed = {
              integrityOk: true,
              resolvedHighSeverity: 0,
              resolvedTotal: 0,
              regressedKnownIssues: 1,
              touchedRegionChars: INCUMBENT_TEXT.length,
            };
            /**
             Measurements of a patch that changed nothing the ranking can see.
             */
            const tied = {
              integrityOk: true,
              resolvedHighSeverity: 0,
              resolvedTotal: 0,
              regressedKnownIssues: 0,
              touchedRegionChars: INCUMBENT_TEXT.length,
            };
            expect({
              regressed: settleWith({
                patchedText: KEEPS_EVERY_NAME,
                declaredNames: DECLARED_NAMES,
                measurements: regressed,
              },).patchLostOn,
              tied: settleWith({
                patchedText: KEEPS_EVERY_NAME,
                declaredNames: DECLARED_NAMES,
                measurements: tied,
              },).patchLostOn,
              won: settleWith({
                patchedText: KEEPS_EVERY_NAME,
                declaredNames: DECLARED_NAMES,
              },).patchLostOn,
            },).toEqual({
              regressed: 'regression',
              tied: undefined,
              won: undefined,
            },);
          },
        },),
        it({
          name: 'ASKS NOTHING of a patch the archive already beat, since a slice nobody is replacing '
            + 'needs no permission to stay as it is and a refusal there would name a protection that '
            + 'protected nothing',
          fn: async () => {
            const verdict = settleWith({
              patchedText: DROPS_THE_ALIAS,
              declaredNames: DECLARED_NAMES,
              measurements: LOSING_MEASUREMENTS,
            },);
            expect(verdict.repairedText,).toBe(INCUMBENT_TEXT,);
            expect(verdict.patchSelected,).toBe(false,);
            expect(verdict.droppedDeclaredNames,).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: settleChunkFromChecks.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'CREDITS exactly the creditable issues the checker majority confirmed fixed, in issue order, and '
            + 'none the tallies refuse or omit',
          fn: async () => {
            const settled = settleChunkFromChecks({
              sliceIndex: CHUNK_INDEX,
              incumbentText: INCUMBENT_TEXT,
              patchedText: KEEPS_EVERY_NAME,
              appliedOperations: [],
              creditableIssues: [
                catIssue({ issueId: 'issue-1', },),
                catIssue({ issueId: 'issue-2', },),
                catIssue({ issueId: 'issue-3', },),
              ],
              tallies: {
                'issue-1': {
                  fixed: 2,
                  notFixed: 0,
                  worse: 0,
                  resolved: true,
                  regressed: false,
                },
                'issue-2': {
                  fixed: 0,
                  notFixed: 2,
                  worse: 0,
                  resolved: false,
                  regressed: false,
                },
              },
              envelopes: [],
              targetDocument: parseDocument({ text: INCUMBENT_TEXT, },),
              declaredNames: DECLARED_NAMES,
            },);

            expect(settled.resolvedIssueIds,).toEqual(['issue-1',],);
          },
        },),
      ],
    },),
  ],
},);
