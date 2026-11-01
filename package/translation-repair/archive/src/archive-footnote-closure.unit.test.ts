/**
 Tests for closing a footnote relabel over the archive's labels.

 THE FOURTH YUKI LAUNCH of 2026-09-08 read one definition pair off the
 roster, mapped `[^2]` to `[^1]` alone, and the archive came out with two
 `[^1]` notes.

 @module
 */

import {
  DEFAULT_CONCURRENCY,
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  closeFootnoteRelabel,
  documentLabels,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: documentLabels.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'lists every distinct label a text carries, references and definition openers alike, in order of '
            + 'first appearance',
          fn: async () => {
            expect(documentLabels({ text: 'A[^2] and B[^1], again[^2].\n\n[^1]: one\n\n[^3]: three\n', },),).toStrictEqual([
              '2',
              '1',
              '3',
            ],);
            expect(documentLabels({ text: 'No notes.\n', },),).toStrictEqual([],);
          },
        },),
      ],
    },),

    describe({
      name: closeFootnoteRelabel.name,
      concurrency: DEFAULT_CONCURRENCY,
      children: [
        it({
          name: 'completes the one pair elimination forces, where one label is left on each side (the fourth yuki '
            + 'launch read [^2]->[^1] alone)',
          fn: async () => {
            expect(closeFootnoteRelabel({
              map: [ {
                from: '2',
                to: '1',
              }, ],
              archiveLabels: [
                '1',
                '2',
              ],
              originalLabels: [
                '2',
                '1',
              ],
            },),).toStrictEqual({
              kind: 'closed',
              correspondences: [{ from: '2', to: '1', },],
              eliminated: [{ from: '1', to: '2', },],
              retained: [],
              map: [
                {
                  from: '2',
                  to: '1',
                },
                {
                  from: '1',
                  to: '2',
                },
              ],
            },);
          },
        },),

        it({
          name: 'keeps closed rewrites unchanged, an empty map empty, and invents no identity correspondence',
          fn: async () => {
            /**
             The whole swap, read off both definitions.
             */
            const swap = [
              {
                from: '1',
                to: '2',
              },
              {
                from: '2',
                to: '1',
              },
            ];
            expect(closeFootnoteRelabel({
              map: swap,
              archiveLabels: [
                '1',
                '2',
              ],
              originalLabels: [
                '2',
                '1',
              ],
            },),).toStrictEqual({
              kind: 'closed',
              correspondences: swap,
              eliminated: [],
              retained: [],
              map: swap,
            },);
            expect(closeFootnoteRelabel({
              map: [],
              archiveLabels: [ '1', ],
              originalLabels: [ '1', ],
            },),).toStrictEqual({
              kind: 'closed',
              correspondences: [],
              eliminated: [],
              retained: [],
              map: [],
            },);
            expect(closeFootnoteRelabel({
              map: [ {
                from: '3',
                to: '2',
              }, ],
              archiveLabels: [
                '1',
                '3',
              ],
              originalLabels: [
                '1',
                '2',
              ],
            },),).toStrictEqual({
              kind: 'closed',
              correspondences: [{ from: '3', to: '2', },],
              eliminated: [],
              retained: [],
              map: [ {
                from: '3',
                to: '2',
              }, ],
            },);
          },
        },),

        it({
          name: 'leaves the archive standing where the map lands on a label the archive carries and does not move '
            + 'and elimination forces nothing',
          fn: async () => {
            /**
             One pair read, two labels left on each side.
             */
            const closure = closeFootnoteRelabel({
              map: [ {
                from: '3',
                to: '1',
              }, ],
              archiveLabels: [
                '1',
                '2',
                '3',
              ],
              originalLabels: [
                '1',
                '2',
                '3',
              ],
            },);
            expect(closure.kind,).toBe('open',);
            if (closure.kind === 'open')
              expect(closure.detail,).toContain('lands on [^1]',);
          },
        },),
      ],
    },),
  ],
},);
