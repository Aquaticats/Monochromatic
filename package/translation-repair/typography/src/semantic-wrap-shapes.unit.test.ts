/**
 Tests for the shapes of text a semantic wrap meets that a line of prose
 never shows: text a command line would mangle, text with nothing in it, and
 text with characters outside the Basic Multilingual Plane.

 WHY THESE EXIST: the wrap used to run in this process and now runs in a
 program that reads the passage on standard input and writes the wrapped
 passage on standard output. A passage that survives that trip byte for byte
 is a property of the trip, so each case here names a way a passage could be
 changed on it: shell quoting, a leading dash read as an option, a stream
 ending early, a surrogate pair split in two, or a reply too long for a
 buffer. Every expected string here was first produced by the in-process
 wrap this one replaced, so a case pinning agreement is a parity case, and
 the cases that do not agree say so in their names.

 Fixtures are cat-themed invention. No corpus content appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { wrapReplacementText, } from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: wrapReplacementText.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'KEEPS QUOTES AND BACKSLASHES exactly as the text carries them, since the passage travels '
            + 'as bytes on a stream and no shell or escape reading stands between',
          fn: async () => {
            expect({
              quoted: wrapReplacementText({
                text: 'She said "the cat naps." Then the dog, barks. \'Single\' too, here.',
              },),
              backslashed: wrapReplacementText({
                text: String.raw`The path is C:\cats\nap. It wakes, then \ stretches.`,
              },),
            },).toEqual({
              quoted: 'She said "the cat naps."\nThen the dog,\nbarks.\n\'Single\' too,\nhere.',
              backslashed: 'The path is C:\\cats\\nap.\nIt wakes,\nthen \\ stretches.',
            },);
          },
        },),

        it({
          name: 'KEEPS A LEADING DASH IN THE TEXT, not read as an option of the program: the text is never '
            + 'an argument',
          fn: async () => {
            expect({
              oneDash: wrapReplacementText({ text: '-the cat naps. It wakes, and eats.', },),
              threeDashes: wrapReplacementText({ text: '--- not a rule. Still prose, here.', },),
            },).toEqual({
              oneDash: '-the cat naps.\nIt wakes,\nand eats.',
              threeDashes: '--- not a rule.\nStill prose,\nhere.',
            },);
          },
        },),

        it({
          name: 'RETURNS A TEXT OF ONLY NEWLINES, A SINGLE ONE, AND AN EMPTY ONE as they came: a stream that '
            + 'ends early or a trailing newline trimmed would show here',
          fn: async () => {
            expect({
              several: wrapReplacementText({ text: '\n\n\n', },),
              single: wrapReplacementText({ text: '\n', },),
              empty: wrapReplacementText({ text: '', },),
            },).toEqual({
              several: '\n\n\n',
              single: '\n',
              empty: '',
            },);
          },
        },),

        it({
          name: 'BREAKS AFTER A CHARACTER OUTSIDE THE BASIC MULTILINGUAL PLANE exactly as it does after one '
            + 'inside it, which the in-process wrap did not do in three shapes: a comma after an emphasis '
            + 'span, a comma in a quotation after a paragraph, and the quotation marker on the new line',
          fn: async () => {
            expect({
              plain: wrapReplacementText({ text: 'Astral \u{1F408} naps, wakes. Done.', },),
              afterEmphasis: wrapReplacementText({ text: '\u{1F344} The *cat*, the dog.', },),
              quotationMarker: wrapReplacementText({ text: '\u{1F344}\n> The cat, the dog.\n', },),
            },).toEqual({
              plain: 'Astral \u{1F408} naps,\nwakes.\nDone.',
              afterEmphasis: '\u{1F344} The *cat*,\nthe dog.',
              quotationMarker: '\u{1F344}\n> The cat,\n> the dog.\n',
            },);
          },
        },),
      ],
    },),
  ],
},);
