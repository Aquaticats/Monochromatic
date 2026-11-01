/**
 Tests target-authoritative contributor identity extraction, and the
 declaring lines it reads (ledger B81).

 Fixtures mirror archive attribution grammar only. Cat-themed invention
 throughout; no corpus content appears here.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  archiveContributorNameForms,
  contributorDeclarationLines,
} from '../dist/final/node/index.mjs';

await describe({
  name: 'contributor name authority',
  children: [
    describe({
      name: archiveContributorNameForms.name,
      children: [
        it({
          name: 'READS PLAIN AND LINKED TARGET IDENTITIES without URL',
          fn: async () => {
            expect(archiveContributorNameForms({
              text: 'Contributors for this entry: The Cat Archive, Whisker, [Pebble](https://example.test/pebble)',
            },),).toEqual([
              'The Cat Archive',
              'Whisker',
              'Pebble',
            ],);
          },
        },),

        it({
          name: 'READS SINGULAR ATTRIBUTION CONTINUATION after label-only line',
          fn: async () => {
            expect(archiveContributorNameForms({
              text: 'Contributor for this entry:\n[Snow Cat](https://example.test/snow-cat)\n\nNext paragraph.',
            },),).toEqual(['Snow Cat',],);
          },
        },),

        it({
          name: 'KEEPS COMMA INSIDE PARENTHETICAL ROLE NOTE with identity',
          fn: async () => {
            expect(archiveContributorNameForms({
              text: 'Contributor for this entry: Pumpkin (translation, review)',
            },),).toEqual(['Pumpkin (translation, review)',],);
          },
        },),

        it({
          name: 'READS FULLWIDTH LABEL DELIMITER used by archive',
          fn: async () => {
            expect(archiveContributorNameForms({
              text: 'Contributor for this entry：Memorial Editor',
            },),).toEqual(['Memorial Editor',],);
          },
        },),

        it({
          name: 'READS a name behind any of the three list markers',
          fn: async () => {
            expect(archiveContributorNameForms({
              text: 'Contributors for this entry: - Whisker, * Pebble, + Mittens',
            },),).toEqual([
              'Whisker',
              'Mittens',
              'Pebble',
            ],);
          },
        },),

        it({
          name: 'KEEPS an opening bracket no link follows as the name it is written as',
          fn: async () => {
            expect(archiveContributorNameForms({
              text: 'Contributor for this entry: [Whisker',
            },),).toEqual(['[Whisker',],);
          },
        },),

        it({
          name: 'READS NO NAME from a link whose label is empty, which shows a reader nothing, rather than its '
            + 'markup (T8)',
          fn: async () => {
            expect(archiveContributorNameForms({
              text: 'Contributors for this entry: [](https://example.test/whisker), Pebble',
            },),).toEqual(['Pebble',],);
          },
        },),

        it({
          name: 'READS NO NAME from a link whose label is only spaces, which shows a reader nothing',
          fn: async () => {
            expect(archiveContributorNameForms({
              text: 'Contributors for this entry: [ ](https://example.test/whisker), Pebble',
            },),).toEqual(['Pebble',],);
          },
        },),

        it({
          name: 'READS a link label padded with spaces as the name it shows, without the padding',
          fn: async () => {
            expect(archiveContributorNameForms({
              text: 'Contributors for this entry: [ Whisker ](https://example.test/whisker), Pebble',
            },),).toEqual([
              'Whisker',
              'Pebble',
            ],);
          },
        },),

        it({
          name: 'READS a name and a link behind list markers followed by more than one space as the names they '
            + 'show, without the spaces or the link markup',
          fn: async () => {
            expect(archiveContributorNameForms({
              text: 'Contributors for this entry: -  Whisker, *  [Pebble](https://example.test/pebble)',
            },),).toEqual([
              'Whisker',
              'Pebble',
            ],);
          },
        },),

        it({
          name: 'READS the label of a reference link, full, collapsed or shortcut, as the name it shows when the text '
            + 'defines its reference, and the whole markup as the text it shows when it does not',
          fn: async () => {
            /**
             Definitions the page carries for the three spellings.
             */
            const definitions = '[w]: https://example.test/whisker\n[Pebble]: https://example.test/pebble\n'
              + '[Tab by]: https://example.test/tabby';
            expect([
              archiveContributorNameForms({
                text: `Contributors for this entry: [Whisker][w], [Tab by][], [Pebble]\n\n${definitions}`,
              },),
              archiveContributorNameForms({
                text: 'Contributors for this entry: [Whisker][w], [Pebble]',
              },),
            ],).toEqual([
              [
                'Whisker',
                'Tab by',
                'Pebble',
              ],
              [
                '[Whisker][w]',
                '[Pebble]',
              ],
            ],);
          },
        },),

        it({
          name: 'READS the label of a full reference link off the parse where its reference label holds an escaped '
            + 'bracket, an escaped mark or an entity, whose written length is not the length the parser reports '
            + 'for it',
          fn: async () => {
            expect([
              archiveContributorNameForms({
                text: 'Contributors for this entry: [Whisker][a\\]b]\n\n[a\\]b]: https://example.test/whisker',
              },),
              archiveContributorNameForms({
                text: 'Contributors for this entry: [Mitten][m\\*n]\n\n[m\\*n]: https://example.test/mitten',
              },),
              archiveContributorNameForms({
                text: 'Contributors for this entry: [Pebble][p&amp;q]\n\n[p&amp;q]: https://example.test/pebble',
              },),
            ],).toEqual([
              ['Whisker',],
              ['Mitten',],
              ['Pebble',],
            ],);
          },
        },),

        it({
          name: 'READS NO NAME from a reference link whose link text is empty, which shows a reader nothing, '
            + 'beside the plain name before it',
          fn: async () => {
            expect(archiveContributorNameForms({
              text: 'Contributors for this entry: Mika, [][n]\n\n[n]: https://example.test/n',
            },),).toEqual(['Mika',],);
          },
        },),

        it({
          name: 'READS a reference link as the markup it is written in when the text nests too deep for the parser '
            + 'to define its reference, and every plain name beside it as before',
          fn: async () => {
            expect(archiveContributorNameForms({
              text: `Contributors for this entry: Mika, [Neko][n]\n\n[n]: https://example.test/n\n\n${
                '>'.repeat(16_000,)
              } cat`,
            },),).toEqual([
              '[Neko][n]',
              'Mika',
            ],);
          },
        },),

        it({
          name: 'IGNORES ORDINARY PROSE carrying same words away from line start',
          fn: async () => {
            expect(archiveContributorNameForms({
              text: 'The contributor for this entry shared a memory.',
            },),).toEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: contributorDeclarationLines.name,
      children: [
        it({
          name: 'NAMES EACH DECLARING LINE: a label line with its names, a label standing alone with nothing, and '
            + 'each line continuing it up to the first blank one',
          fn: async () => {
            expect(contributorDeclarationLines({
              text: 'Contributor for this entry: Mittens\n'
                + 'Contributors for this entry:\n'
                + '[Snow Cat](https://example.test/snow-cat)\n'
                + 'Biscuit\n'
                + '\n'
                + 'The cats napped.',
            },),).toEqual([
              {
                line: 0,
                names: 'Mittens',
              },
              {
                line: 1,
                names: '',
              },
              {
                line: 2,
                names: '[Snow Cat](https://example.test/snow-cat)',
              },
              {
                line: 3,
                names: 'Biscuit',
              },
            ],);
          },
        },),

        it({
          name: 'DECLARES NOTHING for a line that does not open with a label, nor for the line after a label '
            + 'line that carries its own names',
          fn: async () => {
            expect(contributorDeclarationLines({
              text: 'The contributor for this entry shared a memory.\n'
                + 'Contributor for this entry：Pebble\n'
                + 'The cat won an award in spring.',
            },),).toEqual([{
              line: 1,
              names: 'Pebble',
            },],);
          },
        },),
      ],
    },),
  ],
},);
