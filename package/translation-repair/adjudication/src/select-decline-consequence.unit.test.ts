/**
 Tests for what a judge is told a decline does.

 WHY THIS FILE EXISTS. The sentence a judge reads about a declined round over wording that cannot ship said both that
 declining "does not keep it as the shipped text" and, in its last clause, that the archive's wording stays (re-check 2,
 finding F2, 2026-10-25). A judge told two things about the cost of declining is asked for caution by a promise that
 contradicts itself. The sentence says what happens in order: declining does not by itself keep the wording, a judge's
 own decline names no candidate, a candidate two or more judges named ships, and the archive's wording stays only when
 none was named twice (re-check 3, M3). The cases read the clauses by their meaning, through
 `decline-consequence-clauses.test-fixture.ts`, so a rewording that keeps every clause passes and a lost clause fails.

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
import { unmetDeclineClauses, } from './decline-consequence-clauses.test-fixture.ts';

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
