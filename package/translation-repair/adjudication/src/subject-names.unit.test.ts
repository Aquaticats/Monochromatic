/**
 Tests for the names a translation's front matter declares for its subject.

 Every case asserts the whole name list, because the defect this guards against
 is a name read twice, dropped, or split where its author did not split it, and
 each of those changes the list rather than one entry. Fixtures are cat-themed
 invention; no corpus text appears here.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { declaredSubjectNames, } from '../dist/final/node/index.mjs';

//region Subject names tests

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: declaredSubjectNames.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'READS the name first and then each comma-joined alias, in declaration order',
          fn: async () => {
            expect(declaredSubjectNames({
              data: {
                name: 'Mittens',
                info: { alias: 'Whiskers, Tom', },
              },
            },),).toEqual([
              'Mittens',
              'Whiskers',
              'Tom',
            ],);
          },
        },),

        it({
          name: 'READS a full-width comma as an alias separator, the way the corpus joins aliases',
          fn: async () => {
            expect(declaredSubjectNames({
              data: {
                name: 'Mittens',
                info: { alias: 'Whiskers，Tom', },
              },
            },),).toEqual([
              'Mittens',
              'Whiskers',
              'Tom',
            ],);
          },
        },),

        it({
          name: 'READS an alias written as a list, each item one name, a comma inside an item kept',
          fn: async () => {
            expect(declaredSubjectNames({
              data: {
                name: 'Mittens',
                info: {
                  alias: [
                    'Whiskers, Jr',
                    ' Tom ',
                  ],
                },
              },
            },),).toEqual([
              'Mittens',
              'Whiskers, Jr',
              'Tom',
            ],);
          },
        },),

        it({
          name: 'READS the aliases of a page that declares no name',
          fn: async () => {
            expect(declaredSubjectNames({ data: { info: { alias: 'Whiskers', }, }, },),).toEqual(['Whiskers',],);
          },
        },),

        it({
          name: 'LEAVES OUT a blank name, a blank or non-text list item and a number, and lists a name once',
          fn: async () => {
            expect(declaredSubjectNames({
              data: {
                name: '  ',
                info: {
                  alias: [
                    'Whiskers',
                    '',
                    7,
                    null,
                    'Whiskers',
                  ],
                },
              },
            },),).toEqual(['Whiskers',],);
            expect(declaredSubjectNames({
              data: {
                name: 'Mittens',
                info: { alias: 'Mittens, Tom', },
              },
            },),).toEqual([
              'Mittens',
              'Tom',
            ],);
            expect(declaredSubjectNames({ data: { name: 'Mittens', info: { alias: 7, }, }, },),).toEqual(['Mittens',],);
          },
        },),

        it({
          name: 'READS nothing from front matter that is no record or declares neither a name nor an alias',
          fn: async () => {
            expect(declaredSubjectNames({ data: undefined, },),).toEqual([],);
            expect(declaredSubjectNames({ data: 'Mittens', },),).toEqual([],);
            expect(declaredSubjectNames({ data: { info: 'Whiskers', }, },),).toEqual([],);
            expect(declaredSubjectNames({ data: { desc: 'a tabby', }, },),).toEqual([],);
          },
        },),

        it({
          name: 'KEEPS a name that holds a comma as one name, since only an alias is a comma-joined list, and an alias that '
            + 'holds a slash as one alias, since the corpus joins aliases with commas alone',
          fn: async () => {
            expect(declaredSubjectNames({
              data: {
                name: 'Mittens, Jr',
                info: { alias: 'Whiskers/Tom, Tabby', },
              },
            },),).toEqual([
              'Mittens, Jr',
              'Whiskers/Tom',
              'Tabby',
            ],);
          },
        },),

        it({
          name: 'LISTS an alias that repeats the name in another letter case as its own name, since the detector folds case and a repeat is harmless',
          fn: async () => {
            expect(declaredSubjectNames({
              data: {
                name: 'Mittens',
                info: { alias: 'mittens', },
              },
            },),).toEqual([
              'Mittens',
              'mittens',
            ],);
          },
        },),

        it({
          name: 'READS a name written in a script without letter case and an alias beyond the first plane like any other',
          fn: async () => {
            /**
             An alias in Deseret letters, built from code points so the source holds no such character.
             */
            const deseretAlias = String.fromCodePoint(
              0x1_04_28,
              0x1_04_29,
              0x1_04_2A,
            );
            expect(declaredSubjectNames({
              data: {
                name: '猫猫',
                info: { alias: `${deseretAlias}，Tom`, },
              },
            },),).toEqual([
              '猫猫',
              deseretAlias,
              'Tom',
            ],);
          },
        },),
      ],
    },),
  ],
},);

//endregion Subject names tests
