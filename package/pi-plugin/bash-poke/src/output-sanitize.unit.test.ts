/**
 Tests for output sanitization in the built bash-poke artifact.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  constants,
  FORMAT_CHARACTER_RANGES,
  isControlCode,
  isFormatCharacter,
  isKeptCode,
  isKeptWhitespace,
  isSurrogateCode,
  sanitizeOutput,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    //region sanitizeOutput

    describe({
      name: sanitizeOutput.name,
      children: [
        it({
          name: 'keeps ordinary printable text unchanged',
          fn: async () => {
            expect(sanitizeOutput({ text: 'build ok 42', }, ), ).toBe('build ok 42');
          },
        }, ),
        it({
          name: 'keeps tab, line feed, and carriage return',
          fn: async () => {
            expect(sanitizeOutput({ text: 'a\tb\nc\rd', }, ), ).toBe('a\tb\nc\rd');
          },
        }, ),
        it({
          name: 'drops other C0 control characters',
          fn: async () => {
            expect(sanitizeOutput({ text: 'a\u0007b\u0000c\u001Bd', }, ), ).toBe('abcd');
          },
        }, ),
        it({
          name: 'drops delete and C1 control characters',
          fn: async () => {
            expect(sanitizeOutput({ text: 'a\u007Fb\u009Bc', }, ), ).toBe('abc');
          },
        }, ),
        it({
          name: 'drops an unpaired high surrogate',
          fn: async () => {
            expect(sanitizeOutput({ text: 'a\uD83Db', }, ), ).toBe('ab');
          },
        }, ),
        it({
          name: 'drops an unpaired low surrogate',
          fn: async () => {
            expect(sanitizeOutput({ text: 'a\uDE00b', }, ), ).toBe('ab');
          },
        }, ),
        it({
          name: 'keeps a paired surrogate as one astral character',
          fn: async () => {
            expect(sanitizeOutput({ text: 'a\uD83D\uDE00b', }, ), ).toBe('a\uD83D\uDE00b');
          },
        }, ),
        it({
          name: 'drops invisible formatting characters',
          fn: async () => {
            expect(
              sanitizeOutput({ text: 'a\u00ADb\u200Bc\u202Ed\uFEFFe\u2060f', }, ),
            ).toBe('abcdef');
          },
        }, ),
        it({
          name: 'returns an empty string for empty input',
          fn: async () => {
            expect(sanitizeOutput({ text: '', }, ), ).toBe('');
          },
        }, ),
        it({
          name: 'drops every character of an all-control input',
          fn: async () => {
            expect(sanitizeOutput({ text: '\u0007\u001B\u0000', }, ), ).toBe('');
          },
        }, ),
      ],
    }, ),

    //endregion sanitizeOutput

    //region Predicates

    describe({
      name: isKeptWhitespace.name,
      children: [
        it({
          name: 'accepts exactly tab, line feed, and carriage return',
          fn: async () => {
            expect(isKeptWhitespace({ code: constants.TAB_CODE, }, ), ).toBe(true);
            expect(isKeptWhitespace({ code: constants.LINE_FEED_CODE, }, ), ).toBe(true);
            expect(isKeptWhitespace({ code: constants.CARRIAGE_RETURN_CODE, }, ), ).toBe(true);
            expect(isKeptWhitespace({ code: constants.DELETE_CODE, }, ), ).toBe(false);
          },
        }, ),
      ],
    }, ),

    describe({
      name: isControlCode.name,
      children: [
        it({
          name: 'covers C0, delete, and C1 while sparing printable text',
          fn: async () => {
            expect(isControlCode({ code: constants.C0_START_CODE, }, ), ).toBe(true);
            expect(isControlCode({ code: constants.C0_END_CODE, }, ), ).toBe(true);
            expect(isControlCode({ code: constants.DELETE_CODE, }, ), ).toBe(true);
            expect(isControlCode({ code: constants.C1_START_CODE, }, ), ).toBe(true);
            expect(isControlCode({ code: constants.C1_END_CODE, }, ), ).toBe(true);
            expect(isControlCode({ code: 0x41, }, ), ).toBe(false);
          },
        }, ),
      ],
    }, ),

    describe({
      name: isSurrogateCode.name,
      children: [
        it({
          name: 'bounds the surrogate range inclusively',
          fn: async () => {
            expect(isSurrogateCode({ code: constants.SURROGATE_START_CODE, }, ), ).toBe(true);
            expect(isSurrogateCode({ code: constants.SURROGATE_END_CODE, }, ), ).toBe(true);
            expect(isSurrogateCode({ code: constants.SURROGATE_START_CODE - 1, }, ), ).toBe(false);
            expect(isSurrogateCode({ code: constants.SURROGATE_END_CODE + 1, }, ), ).toBe(false);
          },
        }, ),
      ],
    }, ),

    describe({
      name: isFormatCharacter.name,
      children: [
        it({
          name: 'recognizes bidi override and zero-width space',
          fn: async () => {
            expect(isFormatCharacter({ code: 0x20_2E, }, ), ).toBe(true);
            expect(isFormatCharacter({ code: 0x20_0B, }, ), ).toBe(true);
            expect(isFormatCharacter({ code: 0x41, }, ), ).toBe(false);
          },
        }, ),
      ],
    }, ),

    describe({
      name: isKeptCode.name,
      children: [
        it({
          name: 'keeps whitespace that is also a control code',
          fn: async () => {
            expect(isKeptCode({ code: constants.LINE_FEED_CODE, }, ), ).toBe(true);
            expect(isKeptCode({ code: constants.C0_END_CODE, }, ), ).toBe(false);
            expect(isKeptCode({ code: constants.SURROGATE_START_CODE, }, ), ).toBe(false);
            expect(isKeptCode({ code: 0xFE_FF, }, ), ).toBe(false);
            expect(isKeptCode({ code: 0x41, }, ), ).toBe(true);
          },
        }, ),
      ],
    }, ),

    //endregion Predicates

    //region Range table

    describe({
      name: 'FORMAT_CHARACTER_RANGES',
      children: [
        it({
          name: 'holds non-empty inclusive ranges',
          fn: async () => {
            expect(FORMAT_CHARACTER_RANGES.length > 0, ).toBe(true);
            for (const range of FORMAT_CHARACTER_RANGES)
              expect(range.end >= range.start, ).toBe(true);
          },
        }, ),
      ],
    }, ),

    //endregion Range table
  ],
}, );
