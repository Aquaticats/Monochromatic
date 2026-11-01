/**
 Tests for the guard that separates a rendering of nothing from nothing to
 render, and for the rule a settled record states beside it: a passage the
 archive never translated never settles on text that shows nothing.

 THE RULE HAS NO ROUTE THROUGH THE LANE, since every way of producing
 nothing throws before a record is built; its cases call it directly, as
 the census asked of the statement it replaced (ledger T8, sixth batch).

 WHAT THIS ADDS BESIDE THE JUDGE'S OWN CASE, which was measured rather than
 assumed: `translate-judge.unit.test.ts` hand-builds a slate whose sole
 candidate is whitespace and proves the judge throws, and removing this guard
 fails that case too. So the yes arm and the trim on the winner side are
 already driven, through the one caller there is.

 THE SECOND CONJUNCT IS WHAT NOTHING ELSE ASKS. That case's original always
 says something, because a slate is only judged where there was a passage to
 render, so nothing varies the source side. A slice whose original is empty
 must settle as the absence it is rather than as a passage this run emptied,
 and a guard reading the winner alone would call it a deletion.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  caught,
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  assertAbsentSliceFilled,
  blankAgainst,
  BlankSelectionError,
} from '../dist/final/node/index.mjs';

/**
 Original that says something, for every case that needs one.
 */
const SOURCE_SAYS_SOMETHING = '猫猫在窗台上睡觉。';

/**
 Findings the refusal carries, distinctive so its identity is checked.
 */
const STAGE_FINDINGS: readonly string[] = ['translate-no-voice-heard (every translator napped)',];

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: blankAgainst.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REPORTS a deletion where the winner says nothing and the original said something, whether '
            + 'the winner is empty or only whitespace. Trimming is the difference between a guard that '
            + 'fires and one a single newline walks past',
          fn: async () => {
            expect(blankAgainst({
              winner: '',
              sourceText: SOURCE_SAYS_SOMETHING,
            },),).toBe(true,);
            expect(blankAgainst({
              winner: ' \n\t \n ',
              sourceText: SOURCE_SAYS_SOMETHING,
            },),).toBe(true,);
            // Invisible characters trim() keeps are nothing too (ledger B40): a
            // zero-width space, and a Hangul filler no fold removes.
            expect(blankAgainst({
              winner: '\u{200B}',
              sourceText: SOURCE_SAYS_SOMETHING,
            },),).toBe(true,);
            expect(blankAgainst({
              winner: ' \u{3164} ',
              sourceText: SOURCE_SAYS_SOMETHING,
            },),).toBe(true,);
          },
        },),
        it({
          name: 'REPORTS no deletion where the original said nothing either, so an anchor the archive never '
            + 'translated settles as the absence it is rather than as a passage this run emptied',
          fn: async () => {
            expect(blankAgainst({
              winner: '',
              sourceText: '',
            },),).toBe(false,);
            // A source of nothing but whitespace is a source that says nothing, on
            // the same reading the winner side takes.
            expect(blankAgainst({
              winner: '',
              sourceText: '   \n  ',
            },),).toBe(false,);
            // So is a source of invisible characters alone (ledger B40).
            expect(blankAgainst({
              winner: '',
              sourceText: '\u{2060}\u{3000}',
            },),).toBe(false,);
          },
        },),
        it({
          name: 'REPORTS no deletion where the winner says something, which is every slice that ships, and '
            + 'says so even where the original is silent',
          fn: async () => {
            expect(blankAgainst({
              winner: 'The cat sleeps on the windowsill.',
              sourceText: SOURCE_SAYS_SOMETHING,
            },),).toBe(false,);
            expect(blankAgainst({
              winner: 'The cat sleeps on the windowsill.',
              sourceText: '',
            },),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: assertAbsentSliceFilled.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'REFUSES a passage the archive never translated settling on text that shows nothing, empty, '
            + 'whitespace or invisible alike (ledger B40), and carries the stage findings',
          fn: async () => {
            for (const text of ['', ' \n\t ', '\u{200B}', ' \u{3164} ',]) {
              /**
               What settling on this text raised.
               */
              const refusal = caught(function settleOnNothing() {
                assertAbsentSliceFilled({
                  incumbentKind: 'absent',
                  text,
                  findings: STAGE_FINDINGS,
                },);
              },);
              expect(refusal instanceof BlankSelectionError,).toBe(true,);
              expect((refusal instanceof BlankSelectionError) ? refusal.findings : undefined,).toBe(STAGE_FINDINGS,);
            }
          },
        },),
        it({
          name: 'LETS a passage the archive never translated settle on text that shows something, and a passage it '
            + 'did translate settle on anything, since keeping the archive\'s own wording is not this rule\'s concern',
          fn: async () => {
            expect(assertAbsentSliceFilled({
              incumbentKind: 'absent',
              text: 'The cat sleeps on the windowsill.',
              findings: STAGE_FINDINGS,
            },),).toBeUndefined();
            expect(assertAbsentSliceFilled({
              incumbentKind: 'present',
              text: '',
              findings: STAGE_FINDINGS,
            },),).toBeUndefined();
          },
        },),
      ],
    },),
  ],
},);
