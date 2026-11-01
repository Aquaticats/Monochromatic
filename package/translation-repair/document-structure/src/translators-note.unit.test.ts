/**
 Tests for the notes between translators: which comments they are, how a text
 is shown without them, and which of an archive's a page lacks. The shapes are
 the ones the pinned archives carry (a note ending a sentence, a note on lines
 of its own, a multi-line glossary, a JSX note, a note the original repeats).
 Fixtures are cat-themed invention only.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  archiveNotesMissingFrom,
  translatorsNotesIn,
  withoutTranslatorsNotes,
} from '../dist/final/node/index.mjs';

/**
 A note ending a sentence, as XingZ60 slice 6's does.
 */
const TRAILING = '<!-- 前面这句猫话懂什么意思，先这么翻译，大家看一下 -->';

/**
 A note on a line of its own, as zheermao101's archive has one inside a quote.
 */
const JSX_NOTE = '{/* 这句猫话不知道如何翻译 */}';

/**
 A third note, short, for a line that holds three.
 */
const SHORT_NOTE = '{/* 注 */}';

await describe({
  name: '',
  children: [
    describe({
      name: translatorsNotesIn.name,
      children: [
        it({
          name: 'FINDS a closed HTML comment and a closed JSX comment whose words hold Han, with their offsets',
          fn: async () => {
            const text = `The cat slept. ${TRAILING}\n\nIt purred. {/* 这句不知道如何翻译 */}\n`;
            const notes = translatorsNotesIn({ text, },);
            expect(notes.map(function textOf(note,): string {
              return note.text;
            },),).toEqual([TRAILING, '{/* 这句不知道如何翻译 */}',],);
            expect(text.slice(notes[0]?.startOffset, notes[0]?.endOffset,),).toBe(TRAILING,);
          },
        },),
        it({
          name: 'LEAVES a comment in English, a contributor credit the archive translated, and an open comment',
          fn: async () => {
            expect(translatorsNotesIn({ text: 'Mew. <!-- Contributor: Cat Friend -->', },),).toEqual([],);
            expect(translatorsNotesIn({ text: 'Mew. <!-- 猫的备注，没有结尾', },),).toEqual([],);
            expect(translatorsNotesIn({ text: 'Mew. {/* 猫的备注，没有结尾', },),).toEqual([],);
          },
        },),
        it({
          name: 'READS a multi-line glossary as one note, and a JSX note after it',
          fn: async () => {
            const glossary = '<!-- 猫窝：Cat Bed\n\n逗猫棒：Cat Teaser -->';
            const notes = translatorsNotesIn({ text: `Mew.\n\n${glossary}\n\nPurr. {/* 注 */}`, },);
            expect(notes.map(function textOf(note,): string {
              return note.text;
            },),).toEqual([glossary, '{/* 注 */}',],);
          },
        },),
      ],
    },),

    describe({
      name: withoutTranslatorsNotes.name,
      children: [
        it({
          name: 'CUTS a note ending a sentence with the space before it',
          fn: async () => {
            expect(withoutTranslatorsNotes({ text: `The cat slept. ${TRAILING}`, },),).toBe('The cat slept.',);
            expect(withoutTranslatorsNotes({ text: `The cat slept. ${TRAILING} \nIt purred.`, },),).toBe(
              'The cat slept.\nIt purred.',
            );
          },
        },),
        it({
          name: 'CUTS a note alone on its lines with the lines, so no blank line splits a paragraph',
          fn: async () => {
            expect(withoutTranslatorsNotes({ text: `The cat slept.\n${TRAILING}\nIt purred.\n`, },),).toBe(
              'The cat slept.\nIt purred.\n',
            );
            expect(withoutTranslatorsNotes({ text: `The cat slept.\n\n${TRAILING}\n\nIt purred.\n`, },),).toBe(
              'The cat slept.\n\n\nIt purred.\n',
            );
            expect(withoutTranslatorsNotes({ text: `The cat slept.\n${TRAILING}`, },),).toBe('The cat slept.\n',);
          },
        },),
        it({
          name: 'CUTS a note between words with one of the two spaces around it, and every note of a text',
          fn: async () => {
            expect(withoutTranslatorsNotes({ text: `The ${TRAILING} cat slept.`, },),).toBe('The cat slept.',);
            expect(withoutTranslatorsNotes({ text: `A ${TRAILING} cat {/* 注 */}`, },),).toBe('A cat',);
          },
        },),
        it({
          name: 'CUTS a note alone on a quoted line with its line, at one level of nesting, so no bare quote line splits the quote',
          fn: async () => {
            expect(withoutTranslatorsNotes({ text: `> The cat slept.\n> ${JSX_NOTE}\n> It purred.\n`, },),).toBe(
              '> The cat slept.\n> It purred.\n',
            );
            expect(withoutTranslatorsNotes({ text: `> The cat slept.\n> ${TRAILING}\n> It purred.\n`, },),).toBe(
              '> The cat slept.\n> It purred.\n',
            );
            expect(withoutTranslatorsNotes({ text: `>${JSX_NOTE}\n> It purred.\n`, },),).toBe('> It purred.\n',);
            expect(withoutTranslatorsNotes({ text: `> ${JSX_NOTE}   \n> It purred.\n`, },),).toBe('> It purred.\n',);
          },
        },),
        it({
          name: 'CUTS a note alone on a quoted line with its line, at two levels of nesting, as zheermao101\'s archive has one',
          fn: async () => {
            expect(
              withoutTranslatorsNotes({
                text: `>   > The cat slept.\n>   > ${JSX_NOTE}\n>   > It purred.\n`,
              },),
            ).toBe('>   > The cat slept.\n>   > It purred.\n',);
            expect(withoutTranslatorsNotes({ text: `> The cat slept.\n>   > ${JSX_NOTE}\n> It purred.\n`, },),).toBe(
              '> The cat slept.\n> It purred.\n',
            );
            expect(withoutTranslatorsNotes({ text: `>> The cat slept.\n>> ${JSX_NOTE}\n>> It purred.\n`, },),).toBe(
              '>> The cat slept.\n>> It purred.\n',
            );
          },
        },),
        it({
          name: 'CUTS a note alone on the first or the last line of a quote, and each of several notes on lines of their own',
          fn: async () => {
            expect(withoutTranslatorsNotes({ text: `> ${JSX_NOTE}\n> The cat slept.\n`, },),).toBe('> The cat slept.\n',);
            expect(withoutTranslatorsNotes({ text: `> The cat slept.\n> ${JSX_NOTE}\n\nIt purred.\n`, },),).toBe(
              '> The cat slept.\n\nIt purred.\n',
            );
            expect(withoutTranslatorsNotes({ text: `> The cat slept.\n> ${JSX_NOTE}\n> ${TRAILING}\n> It purred.\n`, },),)
              .toBe('> The cat slept.\n> It purred.\n',);
            expect(withoutTranslatorsNotes({ text: `> The cat slept.\n>\n> ${JSX_NOTE}\n>\n> It purred.\n`, },),).toBe(
              '> The cat slept.\n>\n>\n> It purred.\n',
            );
          },
        },),
        it({
          name: 'CUTS a note alone on the last line of a text without a trailing line break, quoted or not, and keeps the line break before it',
          fn: async () => {
            expect(withoutTranslatorsNotes({ text: `> The cat slept.\n> ${JSX_NOTE}`, },),).toBe('> The cat slept.\n',);
            expect(withoutTranslatorsNotes({ text: `>   > The cat slept.\n>   > ${JSX_NOTE}`, },),).toBe(
              '>   > The cat slept.\n',
            );
            expect(withoutTranslatorsNotes({ text: `The cat slept.\n${JSX_NOTE}`, },),).toBe('The cat slept.\n',);
          },
        },),
        it({
          name: 'CUTS a note on a quoted line across CRLF line ends, along with the line break that ends the line',
          fn: async () => {
            expect(withoutTranslatorsNotes({ text: `> The cat slept.\r\n> ${JSX_NOTE}\r\n> It purred.\r\n`, },),).toBe(
              '> The cat slept.\r\n> It purred.\r\n',
            );
            expect(
              withoutTranslatorsNotes({
                text: `>   > The cat slept.\r\n>   > ${JSX_NOTE}\r\n>   > It purred.`,
              },),
            ).toBe('>   > The cat slept.\r\n>   > It purred.',);
            expect(withoutTranslatorsNotes({ text: `> The cat slept.\r\n> ${JSX_NOTE}`, },),).toBe('> The cat slept.\r\n',);
            expect(
              withoutTranslatorsNotes({
                text: `> The cat slept.\r\n> <!-- 猫窝：Cat Bed\r\n> 逗猫棒：Cat Teaser -->\r\n> It purred.\r\n`,
              },),
            ).toBe('> The cat slept.\r\n> It purred.\r\n',);
          },
        },),
        it({
          name: 'CUTS a note whose own words hold a quote mark, and reads a mark before the note that is not a quote head as text',
          fn: async () => {
            expect(withoutTranslatorsNotes({ text: `> The cat slept.\n> {/* 猫 > 窝 >> 猫 */}\n> It purred.\n`, },),).toBe(
              '> The cat slept.\n> It purred.\n',
            );
            expect(withoutTranslatorsNotes({ text: `> The cat slept.\n> <!-- 猫 > 窝 -->\n> It purred.\n`, },),).toBe(
              '> The cat slept.\n> It purred.\n',
            );
            expect(withoutTranslatorsNotes({ text: `The cat -> ${JSX_NOTE}\nIt purred.\n`, },),).toBe(
              'The cat ->\nIt purred.\n',
            );
            expect(withoutTranslatorsNotes({ text: `1 > 0 ${JSX_NOTE}\nIt purred.\n`, },),).toBe('1 > 0\nIt purred.\n',);
          },
        },),
        it({
          name: 'KEEPS the line of a note that shares it with text, quoted or not',
          fn: async () => {
            expect(withoutTranslatorsNotes({ text: `> The cat slept. ${JSX_NOTE}\n> It purred.\n`, },),).toBe(
              '> The cat slept.\n> It purred.\n',
            );
            expect(withoutTranslatorsNotes({ text: `> ${JSX_NOTE} The cat slept.\n> It purred.\n`, },),).toBe(
              '> The cat slept.\n> It purred.\n',
            );
            expect(withoutTranslatorsNotes({ text: `>   > The cat ${JSX_NOTE} slept.\n>   > It purred.\n`, },),).toBe(
              '>   > The cat slept.\n>   > It purred.\n',
            );
            expect(withoutTranslatorsNotes({ text: `> ${JSX_NOTE}The cat slept.\n> It purred.\n`, },),).toBe(
              '> The cat slept.\n> It purred.\n',
            );
          },
        },),
        it({
          name: 'CUTS two or three notes that follow text on one line with the spaces before them, and keeps the line break after them',
          fn: async () => {
            expect(withoutTranslatorsNotes({ text: `The cat slept. ${TRAILING} ${JSX_NOTE}\nIt purred.\n`, },),).toBe(
              'The cat slept.\nIt purred.\n',
            );
            expect(
              withoutTranslatorsNotes({
                text: `The cat slept. ${TRAILING} ${JSX_NOTE} ${SHORT_NOTE}\nIt purred.\n`,
              },),
            ).toBe('The cat slept.\nIt purred.\n',);
            expect(withoutTranslatorsNotes({ text: `The cat slept. ${TRAILING}${JSX_NOTE}   \nIt purred.\n`, },),).toBe(
              'The cat slept.\nIt purred.\n',
            );
          },
        },),
        it({
          name: 'CUTS two or three notes that are all a line holds with the line, so no blank line splits a paragraph',
          fn: async () => {
            expect(withoutTranslatorsNotes({ text: `The cat slept.\n${TRAILING} ${JSX_NOTE}\nIt purred.\n`, },),).toBe(
              'The cat slept.\nIt purred.\n',
            );
            expect(
              withoutTranslatorsNotes({
                text: `The cat slept.\n  ${TRAILING} ${JSX_NOTE}${SHORT_NOTE}  \nIt purred.\n`,
              },),
            ).toBe('The cat slept.\nIt purred.\n',);
            expect(withoutTranslatorsNotes({ text: `${TRAILING} ${JSX_NOTE}\nIt purred.\n`, },),).toBe('It purred.\n',);
          },
        },),
        it({
          name: 'CUTS two or three notes on one quoted line, at one and two levels of nesting, with the line or with the spaces before them',
          fn: async () => {
            expect(withoutTranslatorsNotes({ text: `> The cat slept.\n> ${TRAILING} ${JSX_NOTE}\n> It purred.\n`, },),).toBe(
              '> The cat slept.\n> It purred.\n',
            );
            expect(
              withoutTranslatorsNotes({
                text: `>   > The cat slept.\n>   > ${TRAILING} ${JSX_NOTE} ${SHORT_NOTE}\n>   > It purred.\n`,
              },),
            ).toBe('>   > The cat slept.\n>   > It purred.\n',);
            expect(withoutTranslatorsNotes({ text: `> The cat slept. ${TRAILING} ${JSX_NOTE}\n> It purred.\n`, },),).toBe(
              '> The cat slept.\n> It purred.\n',
            );
            expect(
              withoutTranslatorsNotes({
                text: `>   > The cat slept. ${TRAILING} ${JSX_NOTE}\n>   > It purred.\n`,
              },),
            ).toBe('>   > The cat slept.\n>   > It purred.\n',);
          },
        },),
        it({
          name: 'CUTS two notes on one line across CRLF line ends as it cuts one, and keeps the line break after a line of notes',
          fn: async () => {
            expect(withoutTranslatorsNotes({ text: `The cat slept.\r\n${TRAILING} ${JSX_NOTE}\r\nIt purred.\r\n`, },),).toBe(
              'The cat slept.\r\nIt purred.\r\n',
            );
            expect(
              withoutTranslatorsNotes({
                text: `> The cat slept.\r\n> ${TRAILING} ${JSX_NOTE}\r\n> It purred.\r\n`,
              },),
            ).toBe('> The cat slept.\r\n> It purred.\r\n',);
            expect(withoutTranslatorsNotes({ text: `The cat slept. ${TRAILING} ${JSX_NOTE}\r\nIt purred.\r\n`, },),).toBe(
              withoutTranslatorsNotes({ text: `The cat slept. ${TRAILING}\r\nIt purred.\r\n`, },),
            );
          },
        },),
        it({
          name: 'KEEPS the carriage return of a CRLF line a note ends, so the text keeps one kind of line end',
          fn: async () => {
            expect(withoutTranslatorsNotes({ text: `The cat slept. ${JSX_NOTE}\r\nIt purred.\r\n`, },),).toBe(
              'The cat slept.\r\nIt purred.\r\n',
            );
            expect(withoutTranslatorsNotes({ text: `> The cat slept. ${JSX_NOTE}\r\n> It purred.\r\n`, },),).toBe(
              '> The cat slept.\r\n> It purred.\r\n',
            );
            expect(withoutTranslatorsNotes({ text: `The cat slept. ${JSX_NOTE}  \r\nIt purred.\r\n`, },),).toBe(
              'The cat slept.\r\nIt purred.\r\n',
            );
            expect(withoutTranslatorsNotes({ text: `The cat slept. ${TRAILING} ${JSX_NOTE}\r\nIt purred.\r\n`, },),).toBe(
              'The cat slept.\r\nIt purred.\r\n',
            );
          },
        },),
        it({
          name: 'CUTS two notes on the last line of a text without a trailing line break, quoted or not, with the spaces before them',
          fn: async () => {
            expect(withoutTranslatorsNotes({ text: `The cat slept. ${TRAILING} ${JSX_NOTE}`, },),).toBe('The cat slept.',);
            expect(withoutTranslatorsNotes({ text: `The cat slept.\n${TRAILING} ${JSX_NOTE}`, },),).toBe('The cat slept.\n',);
            expect(withoutTranslatorsNotes({ text: `> The cat slept.\n> ${TRAILING} ${JSX_NOTE}`, },),).toBe(
              '> The cat slept.\n',
            );
            expect(withoutTranslatorsNotes({ text: `${TRAILING} ${JSX_NOTE}`, },),).toBe('',);
          },
        },),
        it({
          name: 'CUTS two or three notes between words with one of the spaces around them, as one note is',
          fn: async () => {
            expect(withoutTranslatorsNotes({ text: `The ${TRAILING} ${JSX_NOTE} cat slept.`, },),).toBe('The cat slept.',);
            expect(withoutTranslatorsNotes({ text: `The ${TRAILING}${JSX_NOTE} ${SHORT_NOTE} cat slept.`, },),).toBe(
              'The cat slept.',
            );
            expect(withoutTranslatorsNotes({ text: `> The ${TRAILING} ${JSX_NOTE} cat slept.\n> It purred.\n`, },),).toBe(
              '> The cat slept.\n> It purred.\n',
            );
          },
        },),
        it({
          name: 'KEEPS the cut of notes that stand apart: one note, notes on lines of their own, and notes with words between them',
          fn: async () => {
            expect(withoutTranslatorsNotes({ text: `The cat slept. ${TRAILING}\nIt purred.\n`, },),).toBe(
              'The cat slept.\nIt purred.\n',
            );
            expect(withoutTranslatorsNotes({ text: `The cat slept. ${TRAILING}\nIt purred. ${JSX_NOTE}\nIt ran.\n`, },),)
              .toBe('The cat slept.\nIt purred.\nIt ran.\n',);
            expect(withoutTranslatorsNotes({ text: `The cat slept.\n${TRAILING}\n${JSX_NOTE}\nIt purred.\n`, },),).toBe(
              'The cat slept.\nIt purred.\n',
            );
            expect(withoutTranslatorsNotes({ text: `The ${TRAILING} cat ${JSX_NOTE}\nIt purred.\n`, },),).toBe(
              'The cat\nIt purred.\n',
            );
            expect(withoutTranslatorsNotes({ text: `${TRAILING} The cat slept. ${JSX_NOTE}\nIt purred.\n`, },),).toBe(
              ' The cat slept.\nIt purred.\n',
            );
          },
        },),
        it({
          name: 'KEEPS a comment alone on a quoted line that holds no Han, and one the original carries, and leaves the notes\' offsets as they are',
          fn: async () => {
            /**
             The text whose quoted lines hold a comment of each kind.
             */
            const text = `> The cat slept.\n> <!-- Cat Bed -->\n> ${JSX_NOTE}\n> ${TRAILING}\n> It purred.\n`;
            expect(withoutTranslatorsNotes({ text, original: `${JSX_NOTE}${TRAILING}`, },),).toBe(text,);
            expect(withoutTranslatorsNotes({ text, },),).toBe('> The cat slept.\n> <!-- Cat Bed -->\n> It purred.\n',);
            /**
             Notes of the text, with offsets in the text.
             */
            const notes = translatorsNotesIn({ text, },);
            expect(notes.map(function textOf(note,): string {
              return note.text;
            },),).toEqual([JSX_NOTE, TRAILING,],);
            expect(notes.map(function startOf(note,): number {
              return note.startOffset;
            },),).toEqual([text.indexOf(JSX_NOTE,), text.indexOf(TRAILING,),],);
            expect(notes.map(function endOf(note,): number {
              return note.endOffset;
            },),).toEqual([text.indexOf(JSX_NOTE,) + JSX_NOTE.length, text.indexOf(TRAILING,) + TRAILING.length,],);
          },
        },),
        it({
          name: 'KEEPS a comment the original carries, in Han or not, and returns a text with no note as it is',
          fn: async () => {
            const original = '猫睡了。<!-- 猫窝：Cat Bed -->';
            const text = `The cat slept. <!-- 猫窝：Cat Bed --> ${TRAILING}`;
            expect(withoutTranslatorsNotes({ text, original, },),).toBe('The cat slept. <!-- 猫窝：Cat Bed -->',);
            expect(withoutTranslatorsNotes({ text: 'The cat slept. <!-- Cat Bed -->', original, },),).toBe(
              'The cat slept. <!-- Cat Bed -->',
            );
          },
        },),
      ],
    },),

    describe({
      name: archiveNotesMissingFrom.name,
      children: [
        it({
          name: 'NAMES the archive\'s notes the page lacks, in archive order, and none the page repeats',
          fn: async () => {
            const archive = `The cat slept. ${TRAILING}\n\n<!-- 猫窝：Cat Bed -->`;
            const missing = archiveNotesMissingFrom({ archive, page: `The cat napped. ${TRAILING}`, },);
            expect(missing.map(function textOf(note,): string {
              return note.text;
            },),).toEqual(['<!-- 猫窝：Cat Bed -->',],);
            expect(archiveNotesMissingFrom({ archive, page: archive, },),).toEqual([],);
          },
        },),
        it({
          name: 'COUNTS a note the page re-wrapped across lines as carried',
          fn: async () => {
            const archive = 'Mew. <!-- 猫窝：Cat Bed 逗猫棒：Cat Teaser -->';
            expect(archiveNotesMissingFrom({ archive, page: 'Mew.\n\n<!-- 猫窝：Cat Bed\n  逗猫棒：Cat Teaser -->', },),).toEqual([],);
          },
        },),
      ],
    },),
  ],
},);
