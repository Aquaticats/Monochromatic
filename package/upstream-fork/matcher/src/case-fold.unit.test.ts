/**
 Tests for Unicode-aware case folding.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  foldCase,
  foldCharacter,
  isFastPathEligible,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: foldCharacter.name,
      children: [
        it({
          name: 'uppercases ordinary non-ASCII letters',
          fn: async () => {
            expect(foldCharacter('é',),).toBe('É',);
            /**
             Composed e-acute, normalized before folding.
             */
            const composed = 'é'.normalize('NFC',);
            expect(foldCharacter(composed,),).toBe('É',);
          },
        },),

        it({
          name: 'keeps characters whose folding changes length',
          fn: async () => {
            expect(foldCharacter('ﬁ',),).toBe('ﬁ',);
            expect(foldCharacter('\uFB00',),).toBe('\uFB00',);
          },
        },),

        it({
          name: 'keeps non-ASCII characters folding into ASCII',
          fn: async () => {
            expect(foldCharacter('ß',),).toBe('ß',);
            expect(foldCharacter('ı',),).toBe('ı',);
            expect(foldCharacter('ſ',),).toBe('ſ',);
          },
        },),

        it({
          name: 'uppercases astral halves without merging them',
          fn: async () => {
            expect(foldCharacter('\uD83D',),).toBe('\uD83D',);
            expect(foldCharacter('😀',),).toBe('😀',);
          },
        },),
      ],
    },),

    describe({
      name: isFastPathEligible.name,
      children: [
        it({
          name: 'accepts plain ASCII strings',
          fn: async () => {
            expect(isFastPathEligible('abc',),).toBe(true,);
          },
        },),

        it({
          name: 'rejects ASCII-folding exceptions and surrogates',
          fn: async () => {
            expect(isFastPathEligible('aıb',),).toBe(false,);
            expect(isFastPathEligible('aſb',),).toBe(false,);
            expect(isFastPathEligible('😀',),).toBe(false,);
            expect(isFastPathEligible('\uD800',),).toBe(false,);
            expect(isFastPathEligible('\uDC00',),).toBe(false,);
          },
        },),
      ],
    },),

    describe({
      name: foldCase.name,
      children: [
        it({
          name: 'passes the value through when case-sensitive',
          fn: async () => {
            expect(foldCase({
              value: 'Unicorn',
              caseSensitive: true,
            },),).toBe('Unicorn',);
          },
        },),

        it({
          name: 'uppercases ASCII case-insensitively',
          fn: async () => {
            expect(foldCase({
              value: 'Unicorn',
              caseSensitive: false,
            },),).toBe('UNICORN',);
          },
        },),

        it({
          name: 'folds non-ASCII through the guarded path',
          fn: async () => {
            expect(foldCase({
              value: 'école',
              caseSensitive: false,
            },),).toBe('ÉCOLE',);
            expect(foldCase({
              value: 'ß',
              caseSensitive: false,
            },),).toBe('ß',);
            expect(foldCase({
              value: 'ßa',
              caseSensitive: false,
            },),).toBe('ßA',);
          },
        },),

        it({
          name: 'keeps astral characters unchanged off the fast path',
          fn: async () => {
            expect(foldCase({
              value: '😀',
              caseSensitive: false,
            },),).toBe('😀',);
            expect(foldCase({
              value: 'a😀b',
              caseSensitive: false,
            },),).toBe('A😀B',);
          },
        },),
      ],
    },),
  ],
},);
