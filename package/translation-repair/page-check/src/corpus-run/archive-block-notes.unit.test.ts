/**
 Guards the placement rule shared by the page assembly's note carry and the
 archive block review's splice (`archive-block-notes.ts`): a note between
 translators that a replacement lacks goes on a line of its own after it, in
 the archive's order, and nothing else is added. Fixtures are cat-themed
 invention.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  appendedNotes,
  keptNotesFinding,
  withBlockNotes,
} from '../../dist/final/node/index.mjs';

/**
 Note the translators left each other.
 */
const NOTE = '<!-- 前面这句猫话懂什么意思，先这么翻译，大家看一下 -->';

/**
 A second note, in a JSX comment.
 */
const SECOND_NOTE = '{/* 猫窝：Cat Bed */}';

await describe({
  name: 'notes between translators put back after a replacement',
  children: [
    it({
      name: 'APPENDS notes on lines of their own after the text, and keeps the whitespace that ends it',
      fn: async () => {
        expect(appendedNotes({ text: 'The cat napped.', notes: [NOTE, SECOND_NOTE,], },),).toBe(
          `The cat napped.\n\n${NOTE}\n\n${SECOND_NOTE}`,
        );
        expect(appendedNotes({ text: 'The cat napped.\n', notes: [NOTE,], },),).toBe(`The cat napped.\n\n${NOTE}\n`,);
      },
    },),
    it({
      name: 'WRITES the line ending the text uses, so a CRLF text gains no bare line feed',
      fn: async () => {
        expect(appendedNotes({ text: 'The cat napped.\r\nIt purred.\r\n', notes: [NOTE, SECOND_NOTE,], },),).toBe(
          `The cat napped.\r\nIt purred.\r\n\r\n${NOTE}\r\n\r\n${SECOND_NOTE}\r\n`,
        );
        expect(appendedNotes({ text: '\r\n', notes: [NOTE, SECOND_NOTE,], },),).toBe(`${NOTE}\r\n\r\n${SECOND_NOTE}`,);
        expect(appendedNotes({ text: 'The cat napped.', notes: [NOTE, SECOND_NOTE,], },),).toBe(
          `The cat napped.\n\n${NOTE}\n\n${SECOND_NOTE}`,
        );
      },
    },),
    it({
      name: 'STANDS the notes alone where the text shows a reader nothing',
      fn: async () => {
        expect(appendedNotes({ text: '', notes: [NOTE, SECOND_NOTE,], },),).toBe(`${NOTE}\n\n${SECOND_NOTE}`,);
        expect(appendedNotes({ text: '\u200B \n', notes: [NOTE,], },),).toBe(NOTE,);
      },
    },),
    it({
      name: 'ADDS the block\'s missing notes in its order, and none when the replacement carries them re-wrapped',
      fn: async () => {
        const blockText = `The cat slept. ${NOTE}\n\nIt purred. ${SECOND_NOTE}`;
        const added = withBlockNotes({ blockText, replacement: 'The cat napped.', original: '', },);
        expect(added.text,).toBe(`The cat napped.\n\n${NOTE}\n\n${SECOND_NOTE}`,);
        expect(added.kept,).toEqual([NOTE, SECOND_NOTE,],);
        const rewrapped = withBlockNotes({
          blockText,
          replacement: `The cat napped. ${NOTE}\n\n{/* 猫窝：Cat\n   Bed */}`,
          original: '',
        },);
        expect(rewrapped.kept,).toEqual([],);
        expect(rewrapped.text,).toBe(`The cat napped. ${NOTE}\n\n{/* 猫窝：Cat\n   Bed */}`,);
      },
    },),
    it({
      name: 'LEAVES a comment in English alone and a comment the original itself carries',
      fn: async () => {
        const english = '<!-- check the cat\'s name -->';
        const originals = '<!-- 原文的注释 -->';
        const kept = withBlockNotes({
          blockText: `The cat slept. ${english} ${originals} ${NOTE}`,
          replacement: 'The cat napped.',
          original: `猫睡了。${originals}`,
        },);
        expect(kept.kept,).toEqual([NOTE,],);
        expect(kept.text,).toBe(`The cat napped.\n\n${NOTE}`,);
      },
    },),
    it({
      name: 'NAMES the block and each note, quoted, in the finding',
      fn: async () => {
        expect(keptNotesFinding({ identity: 'block/1', kept: [NOTE, SECOND_NOTE,], },),).toBe(
          `archive block replacement kept the notes between translators the block held: block/1: ${
            JSON.stringify(NOTE,)
          }, ${JSON.stringify(SECOND_NOTE,)}`,
        );
      },
    },),
  ],
},);
