/**
 Tests for `applyCapitalization` and `joinTokens`.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  applyCapitalization,
  joinTokens,
} from './render-helpers.ts';

await describe({
  name: '',
  children: [
    describe({
      name: applyCapitalization.name,
      children: [
        it({
          name: 'preserves text when mode is preserve',
          fn: async () => {
            expect(
              applyCapitalization({ text: 'hello world', mode: 'preserve',
                caseInvariants: new Set(), },),
            )
              .toBe('hello world',);
          },
        },),

        it({
          name: 'uppercases first character when mode is firstLetter',
          fn: async () => {
            expect(
              applyCapitalization({ text: 'hello world', mode: 'firstLetter',
                caseInvariants: new Set(), },),
            )
              .toBe('Hello world',);
          },
        },),

        it({
          name: 'leaves invariant first token untouched',
          fn: async () => {
            expect(
              applyCapitalization({ text: 'I run', mode: 'firstLetter',
                caseInvariants: new Set(['I',],), },),
            )
              .toBe('I run',);
          },
        },),

        it({
          name: 'returns empty string unchanged',
          fn: async () => {
            expect(
              applyCapitalization({ text: '', mode: 'firstLetter',
                caseInvariants: new Set(), },),
            )
              .toBe('',);
          },
        },),
      ],
    },),

    describe({
      name: joinTokens.name,
      children: [
        it({
          name: 'joins truthy tokens with single spaces',
          fn: async () => {
            expect(joinTokens(['Do', 'I', 'have', '1 cat',],),).toBe('Do I have 1 cat',);
          },
        },),

        it({
          name: 'drops empty-string tokens without doubled spaces',
          fn: async () => {
            expect(joinTokens(['Save', '', 'now',],),).toBe('Save now',);
          },
        },),

        it({
          name: 'omits spaces at adjacent CJK token boundaries',
          fn: async () => {
            expect(joinTokens(['我', '有', '1 只猫',],),).toBe('我有 1 只猫',);
          },
        },),

        it({
          name: 'returns empty string for an empty list',
          fn: async () => {
            expect(joinTokens([],),).toBe('',);
          },
        },),
      ],
    },),
  ],
},);
