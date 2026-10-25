/**
 Guards the sentence a judge reads about what a decline does, as the translate stage renders it round by round over
 wording that cannot ship (re-check 4, F1).

 The sentence once promised, in every round, what only the last round of a pair does. The first ask of a pair
 (`initial-selection`) ships nothing and keeps nothing on a decline: `settleAbsentDecline` raises, and
 `judgeSlateWithRetry` asks the same panel again. The challenge ships the preferred candidate two or more judges named,
 or keeps the wording, only where its caller ships past a decline; the translate lane's first call does not, and
 re-produces with the judges' reasons instead (`translate-stage-repair.ts`). These cases render the real sheets through
 `judgeSlateWithRetry` with every judge declining, and read the sentence between the line that calls declining a real
 answer and the line that asks for the reply, by its clauses (`decline-consequence-clauses.test-fixture.ts`). The
 sentence for each shape a declined round can have is read in `translate-decline-sentence-shapes.unit.test.ts`.

 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import { TranslateAbsenceError, } from '@monochromatic-dev/translation-repair-translate-floor';
import { judgeSlateWithRetry, } from '../dist/final/node/index.mjs';
import {
  failuresOf,
  JUDGES,
  judgedOver,
  QUARTET,
  slateOf,
} from './translate-decline-judging.test-fixture.ts';

await describe({
  name: judgeSlateWithRetry.name,
  children: [
    it({
      name: 'TELLS THE FIRST ASK THAT A DECLINE SETTLES NOTHING and the panel is asked again, where the '
        + 'caller ships past a decline in its challenge: it promises neither the ship nor the stay',
      fn: async () => {
        /**
         Every sheet the judging sent.
         */
        const sheets: string[] = [];
        await judgedOver({
          sheets,
          candidates: slateOf({
            width: QUARTET,
            secondIsJudgesOwn: false,
          },),
        },);
        expect(sheets.length,).toBe(JUDGES.length * 2,);
        expect(failuresOf({
          sheets,
          mode: 'asked-again',
          round: 0,
        },),).toEqual([],);
      },
    },),

    it({
      name: 'TELLS THE CHALLENGE THAT SHIPS PAST A DECLINE the two outcomes, the ship of the preferred candidate two '
        + 'or more judges named and the stay of the wording, the control for the other two rounds',
      fn: async () => {
        /**
         Every sheet the judging sent.
         */
        const sheets: string[] = [];
        await judgedOver({
          sheets,
          candidates: slateOf({
            width: QUARTET,
            secondIsJudgesOwn: false,
          },),
        },);
        expect(failuresOf({
          sheets,
          mode: 'ships',
          round: 1,
        },),).toEqual([],);
      },
    },),

    it({
      name: 'TELLS THE FIRST ASK THAT A DECLINE SETTLES NOTHING where the caller does not ship past a decline either '
        + '(the translate lane\'s first call): the first ask never ships or keeps, whatever its caller does later',
      fn: async () => {
        /**
         Every sheet the judging sent.
         */
        const sheets: string[] = [];
        await judgedOver({
          sheets,
          candidates: slateOf({
            width: QUARTET,
            secondIsJudgesOwn: false,
          },),
          shipPastDecline: false,
        },);
        expect(failuresOf({
          sheets,
          mode: 'asked-again',
          round: 0,
        },),).toEqual([],);
      },
    },),

    it({
      name: 'TELLS THE CHALLENGE OF A CALLER THAT DOES NOT SHIP PAST A DECLINE that nothing ships but a level tie two '
        + 'or more judges named, and that the passage goes back for new proposals (the translate lane\'s first '
        + 'call, which raises the absence its follow-up round answers)',
      fn: async () => {
        /**
         Every sheet the judging sent.
         */
        const sheets: string[] = [];
        /**
         What the judging came to.
         */
        const outcome = await judgedOver({
          sheets,
          candidates: slateOf({
            width: QUARTET,
            secondIsJudgesOwn: false,
          },),
          shipPastDecline: false,
        },);
        expect(outcome instanceof TranslateAbsenceError,).toBe(true,);
        expect(failuresOf({
          sheets,
          mode: 'defers',
          round: 1,
        },),).toEqual([],);
      },
    },),
  ],
},);
