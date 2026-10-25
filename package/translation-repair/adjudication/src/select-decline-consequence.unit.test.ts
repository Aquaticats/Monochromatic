/**
 Tests for what a judge is told a decline does.

 WHY THIS FILE EXISTS. The sentence a judge reads about a declined round over wording that cannot ship said both that
 declining "does not keep it as the shipped text" and, in its last clause, that the archive's wording stays (re-check 2,
 finding F2, 2026-10-25). A judge told two things about the cost of declining is asked for caution by a promise that
 contradicts itself. The sentence says what happens in order: declining does not by itself keep the wording, a judge's
 own decline names no candidate, a candidate two or more judges named ships, and the archive's wording stays only when
 none was named twice (re-check 3, M3). The cases read the clauses by their meaning, through
 `decline-consequence-clauses.test-fixture.ts`, so a rewording that keeps every clause passes and a lost clause, a
 swapped outcome, a decline said to keep the wording and a reversed lane order fail (re-check 4, F2).

 THAT SENTENCE IS TOLD IN ONE ROUND OF THREE (re-check 4, F1): the challenge of a caller that ships past a decline. The
 first ask of a pair settles nothing and asks the panel again, and the challenge of a caller that defers ships only a
 level tie and sends the passage back for new proposals, so each is told its own sentence and the function takes the
 round.

 EVERY ONE OF THE THREE SAYS WHAT A TIE THAT BACKS ONLY SOME CANDIDATES DOES (re-check 5, M-B): the panel is asked again
 first over the leaders, for as long as that narrows the question, and a challenge ships or keeps only once none does.
 NONE OF THE THREE PROMISES THE SAME SEATS (re-check 5, M-D): a round asks a window of the bench that an FNV hash of the
 sheet rotates, and a challenge sheet differs from the first ask's.

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
  SETTLES_NOTHING_AND_ASKS_AGAIN,
  SHIPS_A_LEVEL_TIE_ELSE_PRODUCES_AGAIN,
  SHIPS_WHAT_TWO_JUDGES_NAMED,
} from '../dist/final/node/index.mjs';
import {
  unmetClausesOf,
  unmetDeclineClauses,
} from './decline-consequence-clauses.test-fixture.ts';

/**
 The lead of the sentence a judge reads, with the judge's own decline named.
 */
const LEAD = 'declining does not by itself keep the wording in place, and your own decline names no candidate: ';

/**
 The part of a challenge's sentence about a tie that backs only some candidates, in the words of the constant.
 */
const RUNS_OFF = 'when the round leaves only some candidates tied at the top, or a leader only one judge named, the '
  + 'panel is asked again first, though a round may seat other members of it, over those candidates alone, or over '
  + 'that leader and its runner-up, and again for as long as that narrows the question; ';

/**
 The same part in other words, for the rewording control.
 */
const RUNS_OFF_REWORDED = 'if the round leaves just some candidates tied for the lead, or a leader that one judge alone '
  + 'named, the panel is asked once more first, though a round may seat different members of it, over those '
  + 'candidates or over that leader and its runner-up, and once more for as long as that narrows the question; ';

/**
 The two outcomes in the words of the constant, the stay and then the ship, each waiting on the run-off.
 */
const OUTCOMES = 'when no such question is left and no candidate is named by two or more judges, the wording in place '
  + 'stays and the page reports it, with your reason recorded; when no such question is left and one or more '
  + 'candidates are named by two or more judges, the preferred of those ships, by a fixed order (the repair lane\'s '
  + 'text, then the translate lane\'s, then the first on the slate) and not by your judgment';

await describe({
  name: declineConsequenceFor.name,
  children: [
    it({
      name: 'SAYS EVERY CLAUSE A JUDGE MUST READ over wording that cannot ship, whatever the words (re-check 3, N1): '
        + 'declining does not by itself keep the wording, a judge\'s own decline names no candidate, with none named '
        + 'by two or more judges the wording in place stays and the page reports it, and with one or more named by '
        + 'two or more judges the preferred of those ships by a fixed order',
      fn: async () => {
        expect(unmetDeclineClauses(SHIPS_WHAT_TWO_JUDGES_NAMED,),).toEqual([],);
      },
    },),

    it({
      name: 'IS THE CONTROL FOR THE CLAUSE READER: it names every clause a sentence lacks, so a sentence that leaves '
        + 'out the judge\'s own decline and the fixed order is told apart from the sentence the sheet carries',
      fn: async () => {
        expect(unmetDeclineClauses(
          'declining does not by itself keep the wording in place: ' + RUNS_OFF
            + 'when no such question is left and no candidate is named by two or more judges, the wording in place '
            + 'stays and the page reports it; when no such question is left and one or more candidates are named by '
            + 'two or more judges, the preferred of those ships',
        ),).toEqual([
          'a judge\'s own decline names no candidate',
          'the preferred ships by a fixed order, the repair lane first, then the translate lane, then the slate',
        ],);
      },
    },),

    it({
      name: 'REFUSES A SENTENCE WITH THE TWO OUTCOMES SWAPPED, which every clause as a phrase would pass: the '
        + 'wording in place stays where a candidate was named by two or more judges, and the preferred ships '
        + 'where none was (re-check 4, F2)',
      fn: async () => {
        expect(unmetDeclineClauses(
          LEAD + RUNS_OFF
            + 'when no such question is left and a candidate is named by two or more judges, the wording in place '
            + 'stays and the page reports it, with your reason recorded; when no such question is left and no '
            + 'candidate is named by two or more judges, the preferred of those ships, by a fixed order (the repair '
            + 'lane\'s text, then the translate lane\'s, then the first on the slate) and not by your judgment',
        ),).toEqual([
          'once no narrower question is left, with no candidate named by two or more judges the wording in place stays',
          'once no narrower question is left, with one or more candidates named by two or more judges the preferred '
          + 'ships',
        ],);
      },
    },),

    it({
      name: 'REFUSES A SENTENCE THAT SAYS DECLINING KEEPS THE WORDING, though it carries every other clause: '
        + '"not a quiet keep, but it keeps the wording in place" tells a judge the decline is the way to the stay '
        + '(re-check 4, F2)',
      fn: async () => {
        expect(unmetDeclineClauses(
          'declining is not a quiet keep, but it keeps the wording in place, and your own decline names no '
            + 'candidate: ' + RUNS_OFF + OUTCOMES,
        ),).toEqual(['declining does not by itself keep the wording in place',],);
      },
    },),

    it({
      name: 'REFUSES A SENTENCE WITH THE LANES REVERSED in the fixed order: the translate lane\'s text before the '
        + 'repair lane\'s ships a different candidate (re-check 4, F2)',
      fn: async () => {
        expect(unmetDeclineClauses(
          LEAD + RUNS_OFF
            + 'when no such question is left and no candidate is named by two or more judges, the wording in place '
            + 'stays and the page reports it, with your reason recorded; when no such question is left and one or '
            + 'more candidates are named by two or more judges, the preferred of those ships, by a fixed order (the '
            + 'translate lane\'s text, then the repair lane\'s, then the first on the slate) and not by your judgment',
        ),).toEqual([
          'the preferred ships by a fixed order, the repair lane first, then the translate lane, then the slate',
        ],);
      },
    },),

    it({
      name: 'ACCEPTS A REWORDING THAT KEEPS EVERY CLAUSE, the positive control for the three refusals: the same '
        + 'meaning in other words passes, so the refusals read the meaning and not a copy of the constant',
      fn: async () => {
        expect(unmetDeclineClauses(
          'Declining will not on its own keep the wording in place, and your own decline names no candidate: '
            + RUNS_OFF_REWORDED
            + 'if nothing is left to narrow and no candidate is named by two or more judges, the wording in place '
            + 'remains and the page reports it; if nothing is left to narrow and one or more candidates are named by '
            + 'two or more judges, the preferred of them ships, by a fixed order (the repair lane\'s text, then the '
            + 'translate lane\'s, then the first on the slate)',
        ),).toEqual([],);
      },
    },),

    it({
      name: 'REFUSES A SENTENCE THAT LOST ITS RUN-OFF: a tie that backs only some candidates sends them to the panel '
        + 'first, and a sentence that does not say so tells a judge the round ends where it does not (re-check 5, M-B)',
      fn: async () => {
        expect(unmetDeclineClauses(LEAD + OUTCOMES,),).toEqual([
          'the panel is asked again, though a round may seat other members of it',
          'a tie at the top backing only some candidates, or a leader one judge named, narrows the question to the '
          + 'leaders, for as long as it narrows',
        ],);
      },
    },),

    it({
      name: 'REFUSES OUTCOMES THAT DO NOT WAIT ON THE RUN-OFF: the stay and the ship apply once no narrower question '
        + 'is left, so an outcome said without that wait promises a ship or a stay the code gives only after the '
        + 'run-off (re-check 5, M-B)',
      fn: async () => {
        expect(unmetDeclineClauses(
          LEAD + RUNS_OFF
            + 'when no candidate is named by two or more judges, the wording in place stays and the page reports it, '
            + 'with your reason recorded; when one or more candidates are named by two or more judges, the preferred '
            + 'of those ships, by a fixed order (the repair lane\'s text, then the translate lane\'s, then the first '
            + 'on the slate) and not by your judgment',
        ),).toEqual([
          'once no narrower question is left, with no candidate named by two or more judges the wording in place stays',
          'once no narrower question is left, with one or more candidates named by two or more judges the preferred '
          + 'ships',
        ],);
      },
    },),

    it({
      name: 'REFUSES AN OUTCOME PUT NEGATIVELY, which the slots alone would read as the outcome: "never stays" and '
        + '"does not ship" are the opposite of the stay and the ship (re-check 5, N-a)',
      fn: async () => {
        expect(unmetDeclineClauses(
          LEAD + RUNS_OFF
            + 'when no such question is left and no candidate is named by two or more judges, the wording in place '
            + 'never stays and the page reports it; when no such question is left and one or more candidates are '
            + 'named by two or more judges, the preferred of those does not ship, by a fixed order (the repair '
            + 'lane\'s text, then the translate lane\'s, then the first on the slate) and not by your judgment',
        ),).toEqual([
          'once no narrower question is left, with no candidate named by two or more judges the wording in place stays',
          'once no narrower question is left, with one or more candidates named by two or more judges the preferred '
          + 'ships',
        ],);
      },
    },),

    it({
      name: 'PROMISES NO ROUND THE SAME SEATS: no sentence of the three says "the same judges", since a round asks a '
        + 'window of the bench that the sheet rotates and a challenge sheet differs from the first ask\'s '
        + '(re-check 5, M-D)',
      fn: async () => {
        expect([
          SETTLES_NOTHING_AND_ASKS_AGAIN,
          SHIPS_WHAT_TWO_JUDGES_NAMED,
          SHIPS_A_LEVEL_TIE_ELSE_PRODUCES_AGAIN,
        ].map(function promisesTheSameSeats(sentence,): boolean {
          return sentence.includes('same judges',);
        },),).toEqual([
          false,
          false,
          false,
        ],);
      },
    },),

    it({
      name: 'NEVER SAYS BOTH that declining does not keep the wording and that it stays',
      fn: async () => {
        expect(SHIPS_WHAT_TWO_JUDGES_NAMED.includes('does not keep it as the shipped text',),).toBe(false,);
      },
    },),

    it({
      name: 'TELLS THE FIRST ASK OF A PAIR THAT IT SETTLES NOTHING and the panel is asked again, and promises '
        + 'it neither a ship nor a stay: a decline in that round does not end in either (re-check 4, F1)',
      fn: async () => {
        /**
         What a judge of the first ask over withheld wording reads.
         */
        const sentence = declineConsequenceFor({
          incumbentKind: 'absent',
          withheldStanding: true,
          responsibility: 'initial-selection',
          shipPastDecline: true,
        },);
        expect(unmetClausesOf({ text: sentence, mode: 'asked-again', },),).toEqual([],);
        expect(unmetClausesOf({ text: sentence, mode: 'ships', },).length,).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'TELLS A CHALLENGE WHOSE CALLER DEFERS THAT NOTHING SHIPS BUT A LEVEL TIE two or more judges named, and '
        + 'that the passage goes back for new proposals: the translate lane\'s first call re-produces instead of '
        + 'shipping or keeping the wording (re-check 4, F1)',
      fn: async () => {
        /**
         What a judge of the challenge of the translate lane's first call reads.
         */
        const sentence = declineConsequenceFor({
          incumbentKind: 'absent',
          withheldStanding: true,
          responsibility: 'decline-challenge',
          shipPastDecline: false,
        },);
        expect(unmetClausesOf({ text: sentence, mode: 'defers', },),).toEqual([],);
        expect(unmetClausesOf({ text: sentence, mode: 'ships', },).length,).toBeGreaterThan(0,);
        expect(unmetClausesOf({ text: sentence, mode: 'asked-again', },).length,).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'TELLS A CHALLENGE WHOSE CALLER SHIPS PAST A DECLINE the two outcomes, the control for the other two '
        + 'rounds: the sentence of the challenge round is the one that ships or stays (re-check 4, F1)',
      fn: async () => {
        /**
         What a judge of the challenge that ships past a decline reads.
         */
        const sentence = declineConsequenceFor({
          incumbentKind: 'absent',
          withheldStanding: true,
          responsibility: 'decline-challenge',
          shipPastDecline: true,
        },);
        expect(unmetClausesOf({ text: sentence, mode: 'ships', },),).toEqual([],);
        expect(unmetClausesOf({ text: sentence, mode: 'asked-again', },).length,).toBeGreaterThan(0,);
        expect(unmetClausesOf({ text: sentence, mode: 'defers', },).length,).toBeGreaterThan(0,);
      },
    },),

    it({
      name: 'GIVES EACH ROUND OF A PAIR ITS OWN CONSTANT over withheld wording, and the first ask the same one '
        + 'whether or not its caller ships past a decline later',
      fn: async () => {
        expect([
          declineConsequenceFor({
            incumbentKind: 'absent',
            withheldStanding: true,
            responsibility: 'initial-selection',
            shipPastDecline: true,
          },),
          declineConsequenceFor({
            incumbentKind: 'absent',
            withheldStanding: true,
            responsibility: 'initial-selection',
            shipPastDecline: false,
          },),
          declineConsequenceFor({
            incumbentKind: 'absent',
            withheldStanding: true,
            responsibility: 'decline-challenge',
            shipPastDecline: false,
          },),
        ],).toEqual([
          SETTLES_NOTHING_AND_ASKS_AGAIN,
          SETTLES_NOTHING_AND_ASKS_AGAIN,
          SHIPS_A_LEVEL_TIE_ELSE_PRODUCES_AGAIN,
        ],);
      },
    },),

    it({
      name: 'GIVES EACH CALLER ITS OWN CONSEQUENCE: trusted text kept where there is an incumbent, the two-judge rule '
        + 'over withheld wording, and an untranslated passage where nothing ever existed',
      fn: async () => {
        expect([
          declineConsequenceFor({
            incumbentKind: 'present',
            withheldStanding: false,
            responsibility: 'initial-selection',
            shipPastDecline: false,
          },),
          declineConsequenceFor({
            incumbentKind: 'absent',
            withheldStanding: true,
            responsibility: 'decline-challenge',
            shipPastDecline: true,
          },),
          declineConsequenceFor({
            incumbentKind: 'absent',
            withheldStanding: false,
            responsibility: 'decline-challenge',
            shipPastDecline: true,
          },),
        ],).toEqual([
          KEEPS_TRUSTED_TEXT,
          SHIPS_WHAT_TWO_JUDGES_NAMED,
          LEAVES_PASSAGE_UNTRANSLATED,
        ],);
      },
    },),
  ],
},);
