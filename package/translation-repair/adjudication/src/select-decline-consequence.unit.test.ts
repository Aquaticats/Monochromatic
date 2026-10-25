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
 first ask of a pair settles nothing and asks the same judges again, and the challenge of a caller that defers ships
 only a level tie and sends the passage back for new proposals, so each is told its own sentence and the function
 takes the round.

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
          'declining does not by itself keep the wording in place: when no candidate is named by two or more judges, '
            + 'the wording in place stays and the page reports it; when one or more candidates are named by two or '
            + 'more judges, the preferred of those ships',
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
          'declining does not by itself keep the wording in place, and your own decline names no candidate: '
            + 'when a candidate is named by two or more judges, the wording in place stays and the page reports it, '
            + 'with your reason recorded; when no candidate is named by two or more judges, the preferred of those '
            + 'ships, by a fixed order (the repair lane\'s text, then the translate lane\'s, then the first on the '
            + 'slate) and not by your judgment',
        ),).toEqual([
          'no candidate named by two or more judges: the wording in place stays and the page reports it',
          'one or more candidates named by two or more judges: the preferred of those ships',
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
            + 'candidate: when no candidate is named by two or more judges, the wording in place stays and the page '
            + 'reports it, with your reason recorded; when one or more candidates are named by two or more judges, '
            + 'the preferred of those ships, by a fixed order (the repair lane\'s text, then the translate lane\'s, '
            + 'then the first on the slate) and not by your judgment',
        ),).toEqual(['declining does not by itself keep the wording in place',],);
      },
    },),

    it({
      name: 'REFUSES A SENTENCE WITH THE LANES REVERSED in the fixed order: the translate lane\'s text before the '
        + 'repair lane\'s ships a different candidate (re-check 4, F2)',
      fn: async () => {
        expect(unmetDeclineClauses(
          'declining does not by itself keep the wording in place, and your own decline names no candidate: '
            + 'when no candidate is named by two or more judges, the wording in place stays and the page reports it, '
            + 'with your reason recorded; when one or more candidates are named by two or more judges, the preferred '
            + 'of those ships, by a fixed order (the translate lane\'s text, then the repair lane\'s, then the first '
            + 'on the slate) and not by your judgment',
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
            + 'if no candidate is named by two or more judges, the wording in place remains and the page reports it; '
            + 'if one or more candidates are named by two or more judges, the preferred of them ships, by a fixed '
            + 'order (the repair lane\'s text, then the translate lane\'s, then the first on the slate)',
        ),).toEqual([],);
      },
    },),

    it({
      name: 'NEVER SAYS BOTH that declining does not keep the wording and that it stays',
      fn: async () => {
        expect(SHIPS_WHAT_TWO_JUDGES_NAMED.includes('does not keep it as the shipped text',),).toBe(false,);
      },
    },),

    it({
      name: 'TELLS THE FIRST ASK OF A PAIR THAT IT SETTLES NOTHING and the same judges are asked again, and promises '
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
