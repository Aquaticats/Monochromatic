/**
 Guards ledger B698 and B702 for every sheet the rendered-sheets fixtures
 render: text a reader never sees is cut from the sheets of the models that
 judge, audit, pair or choose, and code carries it (`archive-note-carry.ts`).

 WHY THIS EXISTS. The rule of B698 was stated for every judging sheet and
 applied to the sheets one read met: the lane contest, the two consolidation
 gates and the candidate selection slates. B702 found the archive block review
 outside it, and the absolute naturalness review followed. The fixtures now end
 every English text they render with a note between translators
 (`TRANSLATORS_NOTE`, a closed comment in Han the invented original lacks), so
 a sheet that shows one fails here until it cuts it or joins the writers with a
 reason, and a sheet that no planted text reaches is named with its reason
 instead of passing for cut.

 Fixtures are cat-themed invention; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  translatorsNotesIn,
  withoutTranslatorsNotes,
} from '@monochromatic-dev/translation-repair-document-structure';
import { renderedSheets, } from './rendered-sheets.test-fixture.ts';
import {
  ARCHIVE,
  SOURCE,
  TRANSLATORS_NOTE,
} from '@monochromatic-dev/translation-repair-repair/ts/rendered-sheets-texts.test-fixture.ts';

/**
 Words of the note itself, as a sheet that shows it prints them.
 */
const NOTE_WORDS = '大家看一下';

/**
 Words that only the planted English texts hold, never the rules, the original
 or the declared names: a sheet holding one was reached by a text the note ends.
 */
const PLANTED_WITNESSES: readonly string[] = [
  'falls asleep on the windowsill',
  'fell asleep on the windowsill',
  'kitten to sleep',
  'little treasure',
  'dreamed of fish',
  'favourite of the whole street',
  'favourite of the neighbours',
  'asleep in the sunny window',
  'lulled Mittens to sleep',
  'kitten next door fell asleep too',
];

/**
 Sheets of the models that WRITE wording, which may show a note, each with why
 the carry handles what the writer does with it.

 A writer returns wording that replaces the text it was shown, so a note it
 shows comes back kept as it was, left out, or reworded. `archiveNotesMissingFrom`
 counts a note as carried when the page holds a comment of the same words
 (`translators-note.ts`), so a note kept, even re-wrapped, stays where the writer
 put it, and a note left out is put back after the slice's last block
 (`archive-note-carry.ts`). A note reworded is the one outcome the carry does not
 recognise, and the critic is what would lead a model there: it names Han left in
 an English text as `accuracy/untranslated` (`critic-prompt.ts`), an accepted issue
 becomes an envelope, and the editor renders it. The critic and the panel cut the
 note, so no envelope covers one.
 */
const WRITERS: Readonly<Record<string, string>> = {
  'editor': 'rewrites the envelopes of the critic\'s quotes and builds each region\'s context from the full '
    + 'text by offset, so the text it shows is the stored text; the note is outside every envelope',
  'refiner': 'rewrites whole paragraphs, spliced back by the envelope\'s offsets over the exact base text; a '
    + 'note it keeps stands carried, one it leaves out is put back after the slice',
  'polish refiner': 'the refiner of the final polish, which rewrites whole paragraphs of the base as the refiner does '
    + 'and is also shown the claims the repair panel accepted (ledger B805); a note it keeps stands carried, one it '
    + 'leaves out is put back after the slice',
  'translate writer': 'shows the archive as the existing translation only, and its wording is judged without '
    + 'the note; a note it keeps stands carried, one it leaves out is put back after the slice',
  'consolidation writer': 'shows the archive and both candidates as evidence for the wording it writes; a note '
    + 'it keeps stands carried, one it leaves out is put back after the slice',
  'translate repair': 'continues the translate writer\'s own exchange, so it shows what that writer was shown',
  'page consistency carry writer, repeated sentence': 'rewrites the words of one place inside the paragraph it '
    + 'is shown, and the paragraph around them stays as it was',
  'page consistency carry writer, repeated sentence, death recorded': 'rewrites the words of one place inside '
    + 'the paragraph it is shown, and the paragraph around them stays as it was',
  'page consistency carry writer, refrain, two places in one paragraph, words to keep': 'rewrites the words of '
    + 'one place inside the paragraph it is shown, and the paragraph around them stays as it was',
};

/**
 Sheets that no planted English text reaches, each with why it cannot show a
 note.
 */
const UNREACHED: Readonly<Record<string, string>> = {
  'house policy': 'the house rules rendered alone, which hold no document text',
  'page title lexicon': 'shows the original and its Chinese titles, and no archive text',
  'derivability': 'shows the original and the sentences a probe deleted from a repair, which are prose sentences',
  'picture reading 1': 'sends the picture and a fixed question',
  'picture reading 2': 'sends the picture and a fixed question',
  'picture reading 3': 'sends the picture and a fixed question',
  'picture reading 4': 'sends the picture and a fixed question',
};

/**
 Whether a sheet prints the note's words.

 @param text - rendered sheet

 @returns Whether the words of the note are in it

 @example
 ```ts
 const shows = showsNote({ text: sheet.text, },);
 ```
 */
function showsNote({ text, }: { readonly text: string; },): boolean {
  return text.includes(NOTE_WORDS,);
}

/**
 Whether a planted English text reached a sheet.

 @param text - rendered sheet

 @returns Whether a word only the planted texts hold is in it

 @example
 ```ts
 const reached = reachedByPlanted({ text: sheet.text, },);
 ```
 */
function reachedByPlanted({ text, }: { readonly text: string; },): boolean {
  return PLANTED_WITNESSES.some(function holds(witness,): boolean {
    return text.includes(witness,);
  },);
}

await describe({
  name: 'rendered sheets show no note between translators outside the writers (ledger B698, B702)',
  children: [
    it({
      name: 'PLANTS A NOTE THE CUT FINDS, so a sheet that shows its words shows a note and not a comment of the '
        + 'original',
      fn: async () => {
        expect(TRANSLATORS_NOTE,).toContain(NOTE_WORDS,);
        expect(translatorsNotesIn({ text: ARCHIVE, },).map(function textOf(note,): string {
          return note.text;
        },),).toEqual([TRANSLATORS_NOTE,],);
        expect(withoutTranslatorsNotes({ text: ARCHIVE, original: SOURCE, },),).not.toContain(NOTE_WORDS,);
        expect(withoutTranslatorsNotes({ text: ARCHIVE, original: `${SOURCE}${TRANSLATORS_NOTE}`, },),)
          .toContain(NOTE_WORDS,);
      },
    },),
    it({
      name: 'SHOWS NO NOTE on any sheet of a model that judges, audits, pairs or chooses, the writers aside',
      fn: async () => {
        expect(
          renderedSheets()
            .filter(function showsOutsideWriters(sheet,): boolean {
              return showsNote({ text: sheet.text, },) && (!Object.hasOwn(WRITERS, sheet.name,));
            },)
            .map(function nameOf(sheet,): string {
              return sheet.name;
            },),
        ).toEqual([],);
      },
    },),
    it({
      name: 'LISTS AS WRITERS ONLY SHEETS THAT EXIST AND SHOW THE NOTE, so a sheet that cuts it keeps no exemption',
      fn: async () => {
        /**
         Every rendered sheet, by name.
         */
        const byName = new Map(renderedSheets().map(function entry(sheet,): readonly [string, string] {
          return [sheet.name, sheet.text,];
        },),);
        expect(
          Object.keys(WRITERS,).filter(function stale(name,): boolean {
            return !showsNote({ text: byName.get(name,) ?? '', },);
          },),
        ).toEqual([],);
      },
    },),
    it({
      name: 'READS EVERY SHEET IT CLAIMS TO: each sheet outside the unreached list holds a planted text, and each '
        + 'unreached one holds none',
      fn: async () => {
        /**
         Every rendered sheet.
         */
        const sheets = renderedSheets();
        expect(
          sheets
            .filter(function unreadAndUnlisted(sheet,): boolean {
              return (!reachedByPlanted({ text: sheet.text, },)) && (!Object.hasOwn(UNREACHED, sheet.name,));
            },)
            .map(function nameOf(sheet,): string {
              return sheet.name;
            },),
        ).toEqual([],);
        expect(
          sheets
            .filter(function listedButReached(sheet,): boolean {
              return Object.hasOwn(UNREACHED, sheet.name,) && reachedByPlanted({ text: sheet.text, },);
            },)
            .map(function nameOf(sheet,): string {
              return sheet.name;
            },),
        ).toEqual([],);
        expect(
          Object.keys(UNREACHED,).filter(function missing(name,): boolean {
            return !sheets.some(function named(sheet,): boolean {
              return sheet.name === name;
            },);
          },),
        ).toEqual([],);
      },
    },),
  ],
},);
