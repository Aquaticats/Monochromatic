/**
 Tests for what a judge is told a decline does.

 WHY THIS FILE EXISTS. The sentence a judge reads about a declined round over wording that cannot ship said both that
 declining "does not keep it as the shipped text" and, in its last clause, that the archive's wording stays (re-check 2,
 finding F2, 2026-10-25). A judge told two things about the cost of declining is asked for caution by a promise that
 contradicts itself. The sentence now says what happens in order: a decline is no quiet keep, a candidate two or more
 judges named ships, and the archive's wording stays only when none was named twice.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  declineConsequenceFor,
  KEEPS_TRUSTED_TEXT,
  LEAVES_PASSAGE_UNTRANSLATED,
  SHIPS_WHAT_TWO_JUDGES_NAMED,
} from '../dist/final/node/index.mjs';

await describe({
  name: declineConsequenceFor.name,
  children: [
    it({
      name: 'SAYS WHAT HAPPENS IN ORDER over wording that cannot ship: declining is no quiet keep, a candidate two or '
        + 'more judges named ships, and the archive\'s wording stays only when none was named twice',
      fn: async () => {
        expect({
          noQuietKeep: SHIPS_WHAT_TWO_JUDGES_NAMED.includes('declining is not a quiet keep',),
          twoJudges: SHIPS_WHAT_TWO_JUDGES_NAMED.includes('ships a candidate only when two or more judges named it',),
          staysWhenNone: SHIPS_WHAT_TWO_JUDGES_NAMED.includes(
            'when no candidate was named by two judges that wording stays',
          ),
        },).toEqual({
          noQuietKeep: true,
          twoJudges: true,
          staysWhenNone: true,
        },);
      },
    },),

    it({
      name: 'NEVER SAYS BOTH that declining does not keep the wording and that it stays',
      fn: async () => {
        expect(SHIPS_WHAT_TWO_JUDGES_NAMED.includes('does not keep it as the shipped text',),).toBe(false,);
      },
    },),

    it({
      name: 'GIVES EACH CALLER ITS OWN CONSEQUENCE: trusted text kept where there is an incumbent, the two-judge rule '
        + 'over withheld wording, and an untranslated passage where nothing ever existed',
      fn: async () => {
        expect([
          declineConsequenceFor({ incumbentKind: 'present', withheldStanding: false, },),
          declineConsequenceFor({ incumbentKind: 'absent', withheldStanding: true, },),
          declineConsequenceFor({ incumbentKind: 'absent', withheldStanding: false, },),
        ],).toEqual([
          KEEPS_TRUSTED_TEXT,
          SHIPS_WHAT_TWO_JUDGES_NAMED,
          LEAVES_PASSAGE_UNTRANSLATED,
        ],);
      },
    },),
  ],
},);
