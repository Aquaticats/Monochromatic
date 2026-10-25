/**
 Guards against references by position (ledger D33): "the case above", "see
 below", "the paragraph before this", "the latter". A reference by position
 names nothing once a case, a paragraph or a line is added between the two, so
 the repository rule is to name what a text points at: a case by its name, a
 constant or function by its identifier, a doc by its heading or ledger entry.
 The census found 296 such phrases in the package and 79 in the living docs.

 WHAT IS READ is every text the task-number guard reads: the package's
 TypeScript under `src` (comments and strings alike, since a test's name is a
 string), its docs, README and task file, and the living repository-level
 docs. Wrapped comment and string lines are joined first, so a phrase broken
 across two lines is still one phrase.

 WHAT IS FOUND is the shapes a positional reference takes, matched over words:
 a reference verb before "above" or "below" ("see above", "described
 below"); "above" or "below" in parentheses; ANY WORD before either ("the case
 above", "the estimate below", "either early return below"), unless that word
 marks a comparison, a bound or a placement ("at or above", "unbounded below",
 "far above", "sits above"), what follows makes it a comparison ("ranks the
 house rules above its own", "a rate above one", "the threshold below which")
 or what follows names the target ("a heading above" a quoted heading, "above
 `limit`"); a structure noun before "before this" or "after it"; "earlier in
 this file" and its kin; and "the former" or "the latter".

 ANY WORD, NOT A LIST OF NOUNS, since 2026-09-29. The guard first read a
 position as a pointer only after a listed structure noun ("case", "list",
 "count"), and a scan for "above" and "below" ending a phrase then found a
 couple of hundred pointers after nouns no list held ("the estimate below",
 "the walk below", "the rethrows below", "the standing directive below"). A
 physical position ending a phrase ("slept below.") now reads as a pointer
 too; outside page text, which is exempt, the texts this guard reads hold
 none.

 A COMPARED WORD THAT IS ALSO SOMETHING ELSE IS READ WITH THE WORD AFTER IT (re-check 8, nit A). A verb of the list
 ("the lines below zero the counter", with baseline, threshold and total) and "that" opening a clause
 ("the statement above that a run-off keeps its key holds") compare unless a determiner, a quantifier or a pronoun
 follows; a quantifier that floats away from its noun ("the cases below all pass", with each) compares
 only when a determiner, "of", "else", "other" or "others" follows. Thirteen words that no text needed left the list
 (an, his, her, some, half, exact, both and four to ten), so a position before one is a pointer. What the scan still lets through, a bare verb
 after "that" ("the lines below that write it") and a quantifier before "of", is stated in the TSDoc of
 `COMPARED_OBJECTS` and in ledger B938.

 WHAT IS LEFT is a phrase inside double quotes, which quotes the words rather
 than using them, a position inside a Markdown code span, which quotes data,
 and the exemptions, each named with its reason: positions in a page's own
 text (a note above a letter), comparisons the shapes cannot tell apart,
 Unicode character names, and the text of a model-facing sheet. A sheet is rendered whole, in the
 one order its builder fixes, and read once; editing one changes the cache key
 it is stored under, so a sheet's wording is exempt as a class. "The next
 heading" and "the previous section" are not read at all: this package handles
 documents, and those phrases name a unit of the data far more often than a
 unit of the text they sit in.

 THE FIXTURES COME FIRST, so the package-wide case is read against a scan
 shown able to find each shape (ledger M21). Fixtures are cat-themed.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  PACKAGE_EXEMPTIONS,
  REPOSITORY_EXEMPTIONS,
} from './position-references-exemptions.test-fixture.ts';
import {
  positionReferences,
  staleExemptions,
  unexempted,
} from './position-references.test-fixture.ts';
import {
  readPackageTexts,
  readRepositoryTexts,
} from './prose-texts.test-fixture.ts';
import { expectNoFindings, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scan-findings.test-fixture.ts';



await describe({
  name: 'references by position',
  children: [
    it({
      name: 'FINDS a reference verb, a parenthesised position, a structure noun, a sequence noun before this, a '
        + 'position in this text and the former or the latter',
      fn: async () => {
        expect(positionReferences({
          file: {
            path: 'doc/cat.md',
            text: [
              'Naps as noted above.',
              'The bowl (see below) is full.',
              'The case above pins the bowl, and every count below is a whisker.',
              'The paragraph before this one purrs, and the note after it naps.',
              'Earlier in this file the cat napped.',
              'The kitten and the cat napped; the latter purred.',
            ].join('\n',),
          },
        },).map(function phraseOf({ phrase, },): string {
          return phrase;
        },),).toEqual([
          'noted above',
          'see below',
          'case above',
          'count below',
          'The paragraph before this',
          'the note after it',
          'Earlier in this file',
          'the latter',
        ],);
      },
    },),
    it({
      name: 'JOINS a phrase wrapped across comment lines and across a string joined by a sign, and locates it by '
        + 'the line it starts on',
      fn: async () => {
        expect(positionReferences({
          file: {
            path: 'src/cat.unit.test.ts',
            text: [
              '// The bowl case',
              '// above is a whisker.',
              'const name = \'PURRS, which the case \'',
              '  + \'above needs\';',
            ].join('\n',),
          },
        },).map(function locatedPhrase({ line, phrase, },): string {
          return `${String(line,)} ${phrase}`;
        },),).toEqual([
          '1 case above',
          '3 case \' above',
        ],);
      },
    },),
    it({
      name: 'READS QUOTES ACROSS A WRAPPED LINE, so a quotation that opens on one line and closes on the next '
        + 'is left and a phrase after it is found',
      fn: async () => {
        expect(positionReferences({
          file: {
            path: 'doc/cat.md',
            text: [
              'Never write "the note',
              'above" in a doc; the case above names nothing.',
            ].join('\n',),
          },
        },).map(function phraseOf({ phrase, },): string {
          return phrase;
        },),).toEqual(['case above',],);
      },
    },),
    it({
      name: 'FINDS A POSITION AFTER A WORD NO NOUN LIST HOLDS, ending a phrase or running on into a verb, in a '
        + 'comment and in a printed line',
      fn: async () => {
        expect(positionReferences({
          file: {
            path: 'src/cat.ts',
            text: [
              '// More than the estimate below, and the walk below is type-checked.',
              '// Read before either early return below: the bowl is full.',
              'console.log(`naps: 3, never summed with the whiskers above`,);',
              '// The kitten slept below.',
              '// The above-mentioned bowl is full.',
            ].join('\n',),
          },
        },).map(function phraseOf({ phrase, },): string {
          return phrase;
        },),).toEqual([
          'estimate below',
          'walk below',
          'return below',
          'whiskers above',
          'slept below',
          'The above',
        ],);
      },
    },),
    it({
      name: 'FINDS A POSITION BEFORE A WORD THAT IS ALSO A VERB, since "ties" after a position read as the object '
        + 'of a comparison and let "the asked count below ties the script" pass (re-check 7, nit 2)',
      fn: async () => {
        expect(positionReferences({
          file: {
            path: 'src/cat.unit.test.ts',
            text: '// and the asked count below ties the script to a window of four.',
          },
        },).map(function phraseOf({ phrase, },): string {
          return phrase;
        },),).toEqual(['count below',],);
      },
    },),
    it({
      name: 'FINDS A POSITION BEFORE A "that" THAT OPENS A CLAUSE, in the two cache-version sentences that read as '
        + 'a comparison and passed, and in a clause opened by another determiner or a pronoun',
      fn: async () => {
        expect(positionReferences({
          file: {
            path: 'src/cat.ts',
            text: [
              '// and the statement above that a run-off among the tied labels keeps its key holds for the',
              '// run-off among places alone.',
              '// The statement above that no slice-cache file had been written since 2026-09-28 holds no longer.',
              '// The claim above that the bowl was full, and the note above that both cats napped.',
              '// The claim above that it purred holds.',
            ].join('\n',),
          },
        },).map(function phraseOf({ phrase, },): string {
          return phrase;
        },),).toEqual([
          'statement above',
          'statement above',
          'claim above',
          'note above',
          'claim above',
        ],);
      },
    },),
    it({
      name: 'FINDS A POSITION BEFORE A "that" AND A BARE VERB, in the two real pointers that passed ("the lines '
        + 'below that write it", "the paragraphs above that explain") and one more verb of each kind',
      fn: async () => {
        expect(positionReferences({
          file: {
            path: 'src/cat.ts',
            text: [
              '// The lines below that write it name the bowl.',
              '// The paragraphs above that explain the nap are short.',
              '// The cases below that cover the whiskers pass.',
              '// The notes above that say the cat purred hold.',
            ].join('\n',),
          },
        },).map(function phraseOf({ phrase, },): string {
          return phrase;
        },),).toEqual([
          'lines below',
          'paragraphs above',
          'cases below',
          'notes above',
        ],);
      },
    },),
    it({
      name: 'FINDS A POSITION BEFORE A WORD OF THE COMPARED LIST THAT IS A VERB HERE, one for each such word, since '
        + 'the verb read as the object of a comparison and let "the lines below zero the counter" pass',
      fn: async () => {
        expect(positionReferences({
          file: {
            path: 'src/cat.unit.test.ts',
            text: [
              '// The lines below zero the counter.',
              '// The assertions below exact a fixed count.',
              '// The checks below baseline the run.',
              '// The cases below threshold the score.',
              '// The bowls below half the set.',
              '// The counts below total the exchanges sent.',
            ].join('\n',),
          },
        },).map(function phraseOf({ phrase, },): string {
          return phrase;
        },),).toEqual([
          'lines below',
          'assertions below',
          'checks below',
          'cases below',
          'bowls below',
          'counts below',
        ],);
      },
    },),
    it({
      name: 'FINDS A POSITION BEFORE A QUANTIFIER THAT FLOATS AWAY FROM ITS NOUN, one for each of all, both, each '
        + 'and some, since the quantifier read as the object of a comparison',
      fn: async () => {
        expect(positionReferences({
          file: {
            path: 'src/cat.unit.test.ts',
            text: [
              '// The cases below all pass.',
              '// The checks below both fail.',
              '// The asserts below each count a whisker.',
              '// The naps below some fail.',
            ].join('\n',),
          },
        },).map(function phraseOf({ phrase, },): string {
          return phrase;
        },),).toEqual([
          'cases below',
          'checks below',
          'asserts below',
          'naps below',
        ],);
      },
    },),
    it({
      name: 'FINDS A POSITION BEFORE A WORD THE COMPARED LIST NO LONGER HOLDS, the thirteen words that no text of the '
        + 'package or the living docs needed (an, his, her, some, half, exact, both and four to ten)',
      fn: async () => {
        expect(positionReferences({
          file: {
            path: 'src/cat.unit.test.ts',
            text: [
              '// The rate above an hour, and the bowl below his chair, and the cat above her bowl.',
              '// The naps above some. The cats below half. The bowls above exact weights. The mice below both.',
              '// A rate above four and a rate below six, a count above seven and a count below eight.',
              '// A tally above nine and a tally below ten.',
            ].join('\n',),
          },
        },).map(function phraseOf({ phrase, },): string {
          return phrase;
        },),).toEqual([
          'rate above',
          'bowl below',
          'cat above',
          'naps above',
          'cats below',
          'bowls above',
          'mice below',
          'rate above',
          'rate below',
          'count above',
          'count below',
          'tally above',
          'tally below',
        ],);
      },
    },),
    it({
      name: 'LEAVES a position before a word that is a noun here: that floor, zero, baseline, total resolution, '
        + 'a threshold, and a quantifier that names what is compared',
      fn: async () => {
        expect(positionReferences({
          file: {
            path: 'src/cat.unit.test.ts',
            text: [
              '// A roster below that floor declines, and a cat above that of the dog naps.',
              '// A count above zero, a nap below baseline for a day, a vote below threshold.',
              '// A rate above total resolution, a cap below that speed, and a nap not below that size.',
              '// It ranks above all else and above all others, below each of them, and above all, it naps.',
            ].join('\n',),
          },
        },),).toEqual([],);
      },
    },),
    it({
      name: 'FINDS A POSITION AFTER A BACKTICK RUN NOTHING CLOSES, which Markdown prints as a plain backtick '
        + 'rather than opening a code span',
      fn: async () => {
        expect(positionReferences({
          file: {
            path: 'doc/cat.md',
            text: 'A stray `` opens nothing, so the note above counts.',
          },
        },).map(function phraseOf({ phrase, },): string {
          return phrase;
        },),).toEqual(['note above',],);
      },
    },),
    it({
      name: 'LEAVES a comparison, a bound, a placement, a named target, a quoted phrase and a position in a '
        + 'Markdown code span',
      fn: async () => {
        expect(positionReferences({
          file: {
            path: 'doc/cat.md',
            text: [
              'The cat ranks the house rules above its own, at a rate above one.',
              'Record it under a new heading above "## Naps" and the counts above 3 and above `limit`.',
              'Never write "see above" or "the case above".',
              'The kitten leapt above the bowl at or above quorum, unbounded below and above.',
              'The threshold below which a nap counts sits above U+2E80, far above anything, kept below {@link NAPS}.',
              'The label read `Translation of the above photos:` on the page, and a below-threshold vote counts.',
              '            `the above note`',
              'The report printed `` the credits above` `` on its last line.',
              'A log line read `` total` then the note above was cut `` in full.',
            ].join('\n',),
          },
        },),).toEqual([],);
      },
    },),
    it({
      name: 'LEAVES a comparison wrapped across a string joined by a sign, since the quote and the sign between '
        + 'the position and its object join them',
      fn: async () => {
        expect(positionReferences({
          file: {
            path: 'src/cat.unit.test.ts',
            text: [
              'const name = \'NAPS at any ratio, since below \'',
              '  + \'the floor a nap is short\';',
            ].join('\n',),
          },
        },),).toEqual([],);
      },
    },),
    it({
      name: 'FINDS NO REFERENCE BY POSITION across the package\'s source, tests, docs, README and task file outside '
        + 'the named exemptions, and every exemption still names one',
      fn: async () => {
        /**
         Every file the guard reads.
         */
        const files = await readPackageTexts();
        expect(files.some(function isLedger({ path, },): boolean {
          return path.startsWith('doc/audit-ledger-process-',);
        },),).toBe(true,);
        expectNoFindings({ findings: unexempted({ files, exemptions: PACKAGE_EXEMPTIONS, },), },);
        expectNoFindings({ findings: staleExemptions({ files, exemptions: PACKAGE_EXEMPTIONS, },), },);
      },
    },),
    it({
      name: 'FINDS NO REFERENCE BY POSITION in the living repository-level docs outside the named exemptions, and '
        + 'every exemption still names one',
      fn: async () => {
        /**
         Every living repository-level doc.
         */
        const files = await readRepositoryTexts();
        expect(files.length,).toBeGreaterThan(0,);
        expectNoFindings({ findings: unexempted({ files, exemptions: REPOSITORY_EXEMPTIONS, },), },);
        expectNoFindings({ findings: staleExemptions({ files, exemptions: REPOSITORY_EXEMPTIONS, },), },);
      },
    },),
  ],
},);
