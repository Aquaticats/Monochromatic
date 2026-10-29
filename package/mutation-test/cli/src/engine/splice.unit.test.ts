import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  lineStarts,
  mutantId,
  positionAt,
  spliceReplacement,
} from '../../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: spliceReplacement.name,
      children: [
        it({
          name: 'replaces an operator token in place',
          fn: async () => {
            expect(spliceReplacement({
              source: 'a + b',
              start: 2,
              end: 3,
              text: '-',
            },),).toBe('a - b',);
          },
        },),
        it({
          name: 'splices exactly at UTF-16 offsets past astral characters',
          fn: async () => {
            /**
             Source with a surrogate-pair character before the operator.
             */
            const source = 'const s = "\u{10348}"; const n = 1 + 2;';
            /**
             Offset of the plus token, computed in UTF-16 units.
             */
            const plusAt = source.indexOf('+',);
            expect(spliceReplacement({
              source,
              start: plusAt,
              end: plusAt + 1,
              text: '-',
            },),).toBe(source.split('+',)
              .join('-',),);
          },
        },),
        it({
          name: 'rejects out-of-bounds spans',
          fails: true,
          fn: async () => {
            spliceReplacement({
              source: 'ab',
              start: 1,
              end: 5,
              text: 'x',
            },);
          },
        },),
        it({
          name: 'rejects inverted spans',
          fails: true,
          fn: async () => {
            spliceReplacement({
              source: 'abc',
              start: 2,
              end: 1,
              text: 'x',
            },);
          },
        },),
      ],
    },),
    describe({
      name: positionAt.name,
      children: [
        it({
          name: 'maps offsets to one-based lines and zero-based columns',
          fn: async () => {
            /**
             Line-start table for two-line source.
             */
            const table = lineStarts('ab\ncd',);
            expect(positionAt({
              table,
              offset: 0,
            },),).toEqual({
              line: 1,
              column: 0,
            },);
            expect(positionAt({
              table,
              offset: 4,
            },),).toEqual({
              line: 2,
              column: 1,
            },);
          },
        },),
        it({
          name: 'keeps offsets aligned after astral characters',
          fn: async () => {
            /**
             Two-line source whose first line holds an astral character.
             */
            const source = '"\u{10348}"\nconst x = 1;';
            expect(positionAt({
              table: lineStarts(source,),
              offset: source.indexOf('const',),
            },),).toEqual({
              line: 2,
              column: 0,
            },);
          },
        },),
      ],
    },),
    describe({
      name: mutantId.name,
      children: [
        it({
          name: 'is deterministic and coordinate-sensitive',
          fn: async () => {
            /**
             Baseline coordinates for identity comparison.
             */
            const base = {
              file: 'src/a.ts',
              start: 2,
              end: 3,
              operator: 'arithmetic',
              replacement: '-',
            };
            expect(mutantId(base,),).toBe(mutantId({ ...base, },),);
            expect(mutantId(base,),).not.toBe(mutantId({
              ...base,
              replacement: '*',
            },),);
          },
        },),
      ],
    },),
  ],
},);
