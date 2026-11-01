/**
 Tests for seating judges that did not propose the claim they judge.

 The case that matters is the one that would produce a confident wrong
 number: a claim every seated model proposed, which must be reported rather
 than dropped. The rate renderer tested beside it had no caller after its
 runner changed and went on 2026-09-29 (ledger B30); the crosscheck runner
 reads `MIN_JUDGED_CLAIMS` itself.

 Model ids are the real roster, since the rule under test is about the
 relationship between authorship and the seats available. No corpus text is
 involved.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  seatJudges,
} from '../../dist/final/node/index.mjs';
import {
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/roster-seats.test-fixture.ts';

/**
 Roster the seatings run against, which is the shipped one.
 */
const ROSTER = [
  SEAT_HYPER_OPENROUTER_VISION_EDITOR,
  SEAT_HYPER_VISION,
  SEAT_SYNTHETIC_VISION_NO_OPENROUTER,
  SEAT_SYNTHETIC_VISION_WITHHELD,
  SEAT_HYPER_OPENROUTER_UNMEASURED,
  SEAT_SYNTHETIC_TEXT_EVERYWHERE,
] as const;

await describe({
  name: seatJudges.name,
  children: [
    it({
      name: 'bars the single proposer and seats the rest, which is the common '
        + 'case: attribution measured sole=10 against multi=7, so most claims '
        + 'leave five of six models free to re-examine them',
      fn: async () => {
        /**
         Claim raised by one critic.
         */
        const seating = seatJudges({
          proposers: [SEAT_SYNTHETIC_VISION_WITHHELD,],
          roster: ROSTER,
        },);

        expect(seating.judges,).toHaveLength(5,);
        expect(seating.judges,).not.toContain(SEAT_SYNTHETIC_VISION_WITHHELD,);
        expect(seating.barred,).toStrictEqual([SEAT_SYNTHETIC_VISION_WITHHELD,],);
        expect(seating.judgeable,).toBe(true,);
      },
    },),

    it({
      name: 'bars EVERY proposer when several critics agreed, since agreement '
        + 'makes a claim better supported without making any of its authors '
        + 'disinterested about it',
      fn: async () => {
        /**
         Claim three critics raised.
         */
        const seating = seatJudges({
          proposers: [
            SEAT_SYNTHETIC_VISION_WITHHELD,
            SEAT_HYPER_OPENROUTER_VISION_EDITOR,
            SEAT_SYNTHETIC_TEXT_EVERYWHERE,
          ],
          roster: ROSTER,
        },);

        expect(seating.judges,).toHaveLength(3,);
        expect(seating.barred,).toHaveLength(3,);
      },
    },),

    it({
      name: 'reports a claim NOBODY may judge as unjudgeable rather than '
        + 'returning an empty roster a caller might read as "no objections". '
        + 'Dropping it would shrink the denominator and lift every rate above '
        + 'it while looking entirely ordinary, which is a defect already found '
        + 'and fixed once in the attribution reader',
      fn: async () => {
        /**
         Claim every seated model proposed.
         */
        const seating = seatJudges({
          proposers: [...ROSTER,],
          roster: ROSTER,
        },);

        expect(seating.judges,).toHaveLength(0,);
        expect(seating.judgeable,).toBe(false,);
        expect(seating.barred,).toHaveLength(ROSTER.length,);
      },
    },),

    it({
      name: 'ignores a proposer that is not on the judging roster rather than '
        + 'throwing, since a roster may legitimately shrink between the run '
        + 'that recorded the claim and the crosscheck that re-examines it',
      fn: async () => {
        expect(seatJudges({
          proposers: ['hf:someone/Retired-1',],
          roster: ROSTER,
        },).judges,).toHaveLength(ROSTER.length,);
      },
    },),

    it({
      name: 'keeps ROSTER ORDER in both lists, so two readings of the same run '
        + 'render identically and a diff between them means something changed',
      fn: async () => {
        expect(seatJudges({
          proposers: [
            SEAT_SYNTHETIC_TEXT_EVERYWHERE,
            SEAT_HYPER_OPENROUTER_VISION_EDITOR,
          ],
          roster: ROSTER,
        },).barred,).toStrictEqual([
          SEAT_HYPER_OPENROUTER_VISION_EDITOR,
          SEAT_SYNTHETIC_TEXT_EVERYWHERE,
        ],);
      },
    },),
  ],
},);
