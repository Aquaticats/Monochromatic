import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  gateLabelOnMainTip,
  gateLabelProblems,
  gateLabelStem,
  gateUnitName,
  gateUnitNamePrefix,
  localStamp,
} from './native-gate-unit-name.ts';

/**
 Minutes in one hour, which turns a zone offset into a sign and two components.
 */
const minutesPerHour = 60;

/**
 Tells whether text is the shape a stamp carries: eight digits, a `T`, six digits, a sign and four digits.

 @param stamp - the text the test read

 @returns whether it is a second-precise ISO 8601 basic timestamp with its offset
 */
function isStampShaped(stamp: string): boolean {
  /**
   The digits of the day, the time and the offset's own digits, all of which are numbers.
   */
  const digits = `${stamp.slice(0, 8)}${stamp.slice(9, 15)}${stamp.slice(16)}`;
  if ((stamp.length !== 20) || (stamp.charAt(8) !== 'T'))
    return false;
  if ((stamp.charAt(15) !== '+') && (stamp.charAt(15) !== '-'))
    return false;
  for (let index = 0; index < digits.length; index += 1) {
    /**
     The character at this position.
     */
    const character = digits.charAt(index);
    if ((character < '0') || (character > '9'))
      return false;
  }
  return true;
}

/**
 The offset a stamp should carry, derived from the moment rather than written down, so a test passes in any zone.

 @param moment - the moment the stamp was made for

 @returns the sign and four digits a timestamp carries
 */
function offsetOf(moment: Date): string {
  /**
   Minutes this zone is offset from UTC, as the time zone reports it: positive west of UTC.
   */
  const minutes = moment.getTimezoneOffset();
  return `${minutes > 0 ? '-' : '+'}${String(Math.floor(Math.abs(minutes) / minutesPerHour)).padStart(2, '0')}${String(Math.abs(minutes) % minutesPerHour).padStart(2, '0')}`;
}

await describe({
  name: 'the name of one gate',
  children: [
    it({
      name: 'accepts a short kebab-case label, which is what names a unit, a log and an image tag',
      fn: async () => {
        expect(gateLabelProblems({ label: 'wrapper-next-alpha-on-47eb745abb', }),).toEqual([],);
      },
    }),
    it({
      name: 'refuses an empty label, saying where the label is used',
      fn: async () => {
        expect(gateLabelProblems({ label: '', }).join('\n'),).toContain('empty',);
        expect(gateLabelProblems({ label: '', }).join('\n'),).toContain('image tag',);
      },
    }),
    it({
      name: 'refuses a label with an uppercase letter, a space or an underscore, and shows a usable shape',
      fn: async () => {
        for (const label of [
          'Wrapper',
          'a label',
          'a_label',
        ]) {
          expect(gateLabelProblems({ label, }).join('\n'),).toContain('lowercase letters, digits and hyphens',);
          expect(gateLabelProblems({ label, }).join('\n'),).toContain('parallel-rounds-rebuilt-note',);
        }
      },
    }),
    it({
      name: 'stamps a moment to the second with its offset, never a bare date',
      fn: async () => {
        /**
         A local moment, so the stamp's own digits do not depend on the zone the test runs in.
         */
        const moment = new Date(2_026, 10, 10, 18, 2, 3);
        expect(localStamp({ moment, }),).toBe(`20261110T180203${offsetOf(moment)}`,);
        expect(
          isStampShaped(localStamp({ moment, })),
        ).toBe(true,);
      },
    }),
    it({
      name: 'pads every part of a stamp, so one hour of a day sorts beside another',
      fn: async () => {
        /**
         A local moment in the first minute of a single-digit month and day.
         */
        const moment = new Date(2_026, 0, 2, 3, 4, 5);
        expect(localStamp({ moment, }),).toBe(`20260102T030405${offsetOf(moment)}`,);
      },
    }),
    it({
      name: 'names a unit from the prefix, the label and the stamp',
      fn: async () => {
        expect(gateUnitName({
          label: 'wrapper-next-alpha',
          stamp: '20261110T180203-0500',
        }),).toBe('gpc-gate-wrapper-next-alpha-20261110T180203-0500',);
        expect(gateUnitNamePrefix,).toBe('gpc-gate',);
      },
    }),
    it({
      name: 'drops the gate prefix of a worktree name, because the label already says what the unit is',
      fn: async () => {
        expect(gateLabelStem({ worktree: 'gate-wrapper-next-alpha', }),).toBe('wrapper-next-alpha',);
        expect(gateLabelStem({ worktree: 'wrapper-next-alpha', }),).toBe('wrapper-next-alpha',);
        expect(gateLabelStem({ worktree: '/repo/.claude/worktrees/gate-wrapper-next-alpha/', }),).toBe('wrapper-next-alpha',);
      },
    }),
    it({
      name: 'labels a gate at main\'s tip with the worktree it ran in and the commit it tested',
      fn: async () => {
        expect(gateLabelOnMainTip({
          worktree: 'gate-wrapper-next-alpha',
          sha: '47eb745abb',
        }),).toBe('wrapper-next-alpha-on-47eb745abb',);
        expect(gateLabelProblems({ label: gateLabelOnMainTip({
          worktree: 'gate-wrapper-next-alpha',
          sha: '47eb745abb',
        }), }),).toEqual([],);
      },
    }),
  ],
});
