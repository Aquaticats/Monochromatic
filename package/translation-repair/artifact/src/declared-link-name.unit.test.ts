/**
 Guards class one hundred fourteen (2026-09-24): a linked title
 that names the person the front matter declares shipped the archive's other
 form of the name although the page-name glossary (class eighty-six) said the
 title takes the declared form. Two of three translate judges chose the
 archive's own rendering as "the archive's established rendering for this
 link text", and the repair lane's rewrite was reverted by a probe calling
 the declared form an introduced change. A rendering whose link text, under
 the href of a source link naming a declared person, lacks the declared form
 is refused before any judge reads it, wherever the rule runs: the validator
 itself, the translate slate's floor, the lane contest's winner, the
 consolidation's standing and the lane texts offered to its slate.
 Cat-themed invention throughout; no corpus content appears here.

 @module
 */

import {
  type Logger,
  tagged,
} from '@monochromatic-dev/logger-core/ts';
import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  type SliceValidation,
  validateTranslatedSlice,
} from '@monochromatic-dev/translation-repair-translate-floor';
import type { RosterModelId, } from '@monochromatic-dev/translation-repair-shared-contract';
import { readStandingVerdict, } from '@monochromatic-dev/translation-repair-consolidate';
import { laneContestChoiceVerdict, } from '@monochromatic-dev/translation-repair-lane-contest';
import { floorTranslateVoices, } from '@monochromatic-dev/translation-repair-translate-stage';
import { laneTextsForSlate, } from '../dist/final/node/index.mjs';

/**
 Original definition linking a breakfast post whose title names the cat.
 */
const SOURCE = '[^2]: [早安。今天也要乖乖吃饭哦，咪咪。](https://example.invalid/breakfast-mimi.html)';

/**
 Archive's rendering, naming the cat by another form.
 */
const ARCHIVE = '[^2]: [Good morning. Eat all your breakfast today, Whiskers.](https://example.invalid/breakfast-mimi.html)';

/**
 Rendering carrying the declared form inside the title.
 */
const DECLARED = '[^2]: [Good morning. Eat all your breakfast today, Mittens.](https://example.invalid/breakfast-mimi.html)';

/**
 Name pairs the front matter declares on this page.
 */
const PAIRS = [
  {
    source: '咪咪',
    rendering: 'Mittens',
  },
] as const;

/**
 Name pairs for the word-edge cases (ledger B23): a cat declared in
 Chinese, and one whose source form is itself Latin.
 */
const TOM_PAIRS = [
  {
    source: '\u{6C64}\u{59C6}',
    rendering: 'Tom',
  },
] as const;

/**
 Pair whose source form is Latin, so the original's link text names the cat
 in Latin letters.
 */
const LATIN_SOURCE_PAIRS = [
  {
    source: 'Tom',
    rendering: 'Thomas',
  },
] as const;

/**
 Original question title @-mentioning the declared cat.
 */
const QUESTION = '[如何评价论坛用户@咪咪？](https://example.invalid/question/7)';

/**
 Archive's rendering of the question, carrying the account handle.
 */
const QUESTION_PAGE = '[What do you think of forum user @mi-mi-42 ?](https://example.invalid/question/7)';

/**
 Verdict on a candidate no floor refuses, read against a page.
 */
const VALID_ON_PAGE: SliceValidation = {
  kind: 'valid',
  pageGrammar: 'strict',
};

/**
 Verdict on a candidate no floor refuses, with no page behind it.
 */
const VALID_WITHOUT_PAGE: SliceValidation = {
  kind: 'valid',
  pageGrammar: 'absent',
};

/**
 Logger that forwards every line.
 */
const l: Logger = tagged({ tag: 'declared-link-name-test', },);

/**
 Verdict refusing a link text that lacks the declared form.

 @param href - destination both link texts sit under

 @param source - declared source form the original's link text carries

 @param rendering - declared form

 @param quoted - rendered link text, as the finding quotes it

 @param handles - account handles the page writes under the href, which an
 @-mention may carry instead; none off a mention

 @returns Whole verdict, so a check reads every word of the finding

 @example
 ```ts
 linkNameRefusal({ href, source: '咪咪', rendering: 'Mittens', quoted: 'Diary', handles: [], },);
 ```
 */
function linkNameRefusal(
  {
    href,
    source,
    rendering,
    quoted,
    handles,
  }: {
    readonly href: string;
    readonly source: string;
    readonly rendering: string;
    readonly quoted: string;
    readonly handles: readonly string[];
  },
): SliceValidation {
  /**
   Handle alternative the finding offers, empty off a mention.
   */
  const handleClause = (handles.length === 0)
    ? ''
    : ' As an @-mention it may instead carry the account handle the existing translation writes there '
      + `(${handles.join(', ',)}).`;
  return {
    kind: 'invalid',
    findings: [
      `The link text for ${href} names ${source}, whom this page's front matter declares "${rendering}", but your `
        + `link text "${quoted}" does not carry "${rendering}". A declared name inside a title takes its declared form, `
        + `whatever the existing translation calls the person there; keep the rest of the title as you rendered it.${
        handleClause
      }`,
    ],
  };
}

await describe({
  name: 'a declared name inside a linked title (class one hundred fourteen)',
  children: [
    it({
      name: 'REFUSES a rendering whose link text lacks the declared form, and names both forms',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: SOURCE,
          candidateText: ARCHIVE,
          pageText: ARCHIVE,
          declared: PAIRS,
        },),).toEqual(linkNameRefusal({
          href: 'https://example.invalid/breakfast-mimi.html',
          source: '咪咪',
          rendering: 'Mittens',
          quoted: 'Good morning. Eat all your breakfast today, Whiskers.',
          handles: [],
        },),);
      },
    },),
    it({
      name: 'ACCEPTS the declared form in the title, and stays silent where nothing is declared or the link is elsewhere',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: SOURCE,
          candidateText: DECLARED,
          pageText: ARCHIVE,
          declared: PAIRS,
        },),).toEqual(VALID_ON_PAGE,);
        expect(validateTranslatedSlice({
          sourceText: SOURCE,
          candidateText: ARCHIVE,
          pageText: ARCHIVE,
        },),).toEqual(VALID_ON_PAGE,);
        expect(validateTranslatedSlice({
          sourceText: '咪咪在窗台上打盹。',
          candidateText: 'The kitten dozed on the windowsill.',
          declared: PAIRS,
        },),).toEqual(VALID_WITHOUT_PAGE,);
      },
    },),
    it({
      name: 'WITHHOLDS the archive-worded candidate from the translate slate',
      fn: async () => {
        /**
         Slate after the floor.
         */
        const floored = floorTranslateVoices({
          voices: [
            {
              modelId: 'hf:cat/Cat-A' as unknown as RosterModelId,
              value: { translation: ARCHIVE, },
            },
            {
              modelId: 'hf:cat/Cat-B' as unknown as RosterModelId,
              value: { translation: DECLARED, },
            },
          ],
          sourceText: SOURCE,
          incumbentText: ARCHIVE,
          lineStructured: false,
          declared: PAIRS,
          l,
        },);
        expect(floored.voices.map(function text(voice,): string {
          return voice.value.translation;
        },),).toEqual([DECLARED,],);
      },
    },),
    it({
      name: 'REFUSES the archive-worded lane as the contest winner and as the standing, and offers only the declared lane',
      fn: async () => {
        expect(laneContestChoiceVerdict({
          outcome: {
            choice: 'repair',
            ballots: [],
            usable: 2,
            findings: [],
          },
          sourceText: SOURCE,
          incumbentText: ARCHIVE,
          repairText: ARCHIVE,
          translateText: DECLARED,
          lineStructured: false,
          declared: PAIRS,
        },).mayShip,).toBe(false,);
        expect(readStandingVerdict({
          sourceText: SOURCE,
          standingText: ARCHIVE,
          incumbentText: ARCHIVE,
          lineStructured: false,
          choice: 'repair',
          contestVerdict: {
            kind: 'lane-won',
            lane: 'repair',
          },
          sliceIndex: 3,
          l,
          declared: PAIRS,
        },).standingValid,).toBe(false,);
        expect(laneTextsForSlate({
          sourceText: SOURCE,
          incumbentText: ARCHIVE,
          repairText: ARCHIVE,
          translateText: DECLARED,
          standingText: '',
          standingMayShip: false,
          standingEligible: false,
          declared: PAIRS,
        },).map(function lane(offer,): string {
          return offer.lane;
        },),).toEqual(['translate',],);
      },
    },),
    it({
      name: 'PASSES AN @-MENTION CARRYING THE ACCOUNT HANDLE THE PAGE WRITES, or the declared form, and refuses '
        + 'any other handle, offering the page\'s (owner, 2026-09-27, "Account handle")',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: QUESTION,
          candidateText: QUESTION_PAGE,
          pageText: QUESTION_PAGE,
          declared: PAIRS,
        },),).toEqual(VALID_ON_PAGE,);
        expect(validateTranslatedSlice({
          sourceText: QUESTION,
          candidateText: '[What do you think of forum user @Mittens?](https://example.invalid/question/7)',
          pageText: QUESTION_PAGE,
          declared: PAIRS,
        },),).toEqual(VALID_ON_PAGE,);
        expect(validateTranslatedSlice({
          sourceText: QUESTION,
          candidateText: '[What do you think of forum user @mimi?](https://example.invalid/question/7)',
          pageText: QUESTION_PAGE,
          declared: PAIRS,
        },),).toEqual(linkNameRefusal({
          href: 'https://example.invalid/question/7',
          source: '咪咪',
          rendering: 'Mittens',
          quoted: 'What do you think of forum user @mimi?',
          handles: ['@mi-mi-42',],
        },),);
      },
    },),
    it({
      name: 'READS A HANDLE ENDING THE PAGE\'S LINK TEXT WHOLE: the rendering carrying it passes, and another handle '
        + 'is offered it',
      fn: async () => {
        /**
         Archive's rendering of the question, the handle closing its link text.
         */
        const pageText = '[What do you think of forum user @mi-mi-42](https://example.invalid/question/7)';
        expect(validateTranslatedSlice({
          sourceText: QUESTION,
          candidateText: pageText,
          pageText,
          declared: PAIRS,
        },),).toEqual(VALID_ON_PAGE,);
        expect(validateTranslatedSlice({
          sourceText: QUESTION,
          candidateText: '[What do you think of forum user @mimi?](https://example.invalid/question/7)',
          pageText,
          declared: PAIRS,
        },),).toEqual(linkNameRefusal({
          href: 'https://example.invalid/question/7',
          source: '咪咪',
          rendering: 'Mittens',
          quoted: 'What do you think of forum user @mimi?',
          handles: ['@mi-mi-42',],
        },),);
      },
    },),
    it({
      name: 'REFUSES A LINK TEXT HOLDING THE DECLARED FORM ONLY INSIDE A LONGER WORD (ledger B23), and passes it '
        + 'standing apart',
      fn: async () => {
        /**
         Original breakfast link naming the cat.
         */
        const sourceText = '[^3]: [\u{65E9}\u{5B89}\u{FF0C}\u{6C64}\u{59C6}\u{3002}](https://example.invalid/tom)';
        expect(validateTranslatedSlice({
          sourceText,
          candidateText: '[^3]: [Good morning, tomcat.](https://example.invalid/tom)',
          declared: TOM_PAIRS,
        },),).toEqual(linkNameRefusal({
          href: 'https://example.invalid/tom',
          source: '\u{6C64}\u{59C6}',
          rendering: 'Tom',
          quoted: 'Good morning, tomcat.',
          handles: [],
        },),);
        expect(validateTranslatedSlice({
          sourceText,
          candidateText: '[^3]: [Good morning, Tom.](https://example.invalid/tom)',
          declared: TOM_PAIRS,
        },),).toEqual(VALID_WITHOUT_PAGE,);
      },
    },),
    it({
      name: 'READS A LATIN SOURCE FORM ONLY AS A WHOLE HANDLE: a link about a tomcat or about BigTom names no '
        + 'Tom, and one about Tom does',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: '[Tomcat\u{7684}\u{65E9}\u{9910}](https://example.invalid/tomcat)',
          candidateText: '[Tomcat\'s breakfast](https://example.invalid/tomcat)',
          declared: LATIN_SOURCE_PAIRS,
        },),).toEqual(VALID_WITHOUT_PAGE,);
        expect(validateTranslatedSlice({
          sourceText: '[BigTom\u{7684}\u{65E9}\u{9910}](https://example.invalid/bigtom)',
          candidateText: '[BigTom\'s breakfast](https://example.invalid/bigtom)',
          declared: LATIN_SOURCE_PAIRS,
        },),).toEqual(VALID_WITHOUT_PAGE,);
        expect(validateTranslatedSlice({
          sourceText: '[Tom\u{7684}\u{65E9}\u{9910}](https://example.invalid/tom)',
          candidateText: '[Tom\'s breakfast](https://example.invalid/tom)',
          declared: LATIN_SOURCE_PAIRS,
        },),).toEqual(linkNameRefusal({
          href: 'https://example.invalid/tom',
          source: 'Tom',
          rendering: 'Thomas',
          quoted: 'Tom\'s breakfast',
          handles: [],
        },),);
      },
    },),
    it({
      name: 'TAKES NO LONGER HANDLE FOR A MENTION OF THE DECLARED FORM, and no longer handle for the one the page '
        + 'writes',
      fn: async () => {
        expect(validateTranslatedSlice({
          sourceText: '[Tom\u{548C}\u{8BBA}\u{575B}\u{7528}\u{6237}@Tomcat](https://example.invalid/pair)',
          candidateText: '[Tomcat and forum user @Tomcat](https://example.invalid/pair)',
          pageText: '[Tom and forum user @Tomcat](https://example.invalid/pair)',
          declared: LATIN_SOURCE_PAIRS,
        },),).toEqual(linkNameRefusal({
          href: 'https://example.invalid/pair',
          source: 'Tom',
          rendering: 'Thomas',
          quoted: 'Tomcat and forum user @Tomcat',
          handles: [],
        },),);
        expect(validateTranslatedSlice({
          sourceText: QUESTION,
          candidateText: '[What do you think of forum user @mi-mi-420?](https://example.invalid/question/7)',
          pageText: QUESTION_PAGE,
          declared: PAIRS,
        },),).toEqual(linkNameRefusal({
          href: 'https://example.invalid/question/7',
          source: '咪咪',
          rendering: 'Mittens',
          quoted: 'What do you think of forum user @mi-mi-420?',
          handles: ['@mi-mi-42',],
        },),);
      },
    },),
    it({
      name: 'KEEPS THE DECLARED FORM owed where the original names the cat without an @-mention, whatever handle '
        + 'the page writes',
      fn: async () => {
        /**
         Page's rendering carrying a handle in place of the name.
         */
        const pageText = '[Diary of @mi-mi-42](https://example.invalid/diary)';
        expect(validateTranslatedSlice({
          sourceText: '[咪咪的日记](https://example.invalid/diary)',
          candidateText: pageText,
          pageText,
          declared: PAIRS,
        },),).toEqual(linkNameRefusal({
          href: 'https://example.invalid/diary',
          source: '咪咪',
          rendering: 'Mittens',
          quoted: 'Diary of @mi-mi-42',
          handles: [],
        },),);
      },
    },),
  ],
},);
