/**
 Tests for system error narrowing in the built bash-poke artifact.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isErrorWithCode,
  isMissingFileCode,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    //region isErrorWithCode

    describe({
      name: isErrorWithCode.name,
      children: [
        it({
          name: 'accepts an error carrying a code',
          fn: async () => {
            expect(
              isErrorWithCode(Object.assign(new Error('gone'), { code: 'ENOENT', }, ), ),
            )
              .toBe(true);
          },
        }, ),
        it({
          name: 'rejects an error without a code',
          fn: async () => {
            expect(
              isErrorWithCode(new Error('plain'), ),
            ).toBe(false);
          },
        }, ),
        it({
          name: 'rejects non-error values',
          fn: async () => {
            expect(isErrorWithCode('ENOENT', ), ).toBe(false);
            expect(isErrorWithCode(undefined, ), ).toBe(false);
            expect(isErrorWithCode({ code: 'ENOENT', }, ), ).toBe(false);
            expect(isErrorWithCode(null, ), ).toBe(false);
          },
        }, ),
      ],
    }, ),

    //endregion isErrorWithCode

    //region isMissingFileCode

    describe({
      name: isMissingFileCode.name,
      children: [
        it({
          name: 'matches only the code it was asked about',
          fn: async () => {
            expect(
              isMissingFileCode({
                error: Object.assign(new Error('gone'), { code: 'ENOENT', }, ),
                code: 'ENOENT',
              }, ),
            ).toBe(true);
            expect(
              isMissingFileCode({
                error: Object.assign(new Error('denied'), { code: 'EACCES', }, ),
                code: 'ENOENT',
              }, ),
            ).toBe(false);
          },
        }, ),
        it({
          name: 'reports false for values that are not errors',
          fn: async () => {
            expect(isMissingFileCode({ error: 'ENOENT', code: 'ENOENT', }, ), ).toBe(false);
            expect(isMissingFileCode({ error: null, code: 'ENOENT', }, ), ).toBe(false);
          },
        }, ),
      ],
    }, ),

    //endregion isMissingFileCode
  ],
}, );
