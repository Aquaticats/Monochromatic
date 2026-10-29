/**
 Unit tests for the low-level scanners: whitespace classification, string and
 number token scanning (including escapes, EOF, and invalid runs), keyword
 matching, and line and block comment scanning.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  isJsonWhitespace,
  matchKeyword,
  scanBlockComment,
  scanLineComment,
  scanNumber,
  scanString,
} from '../dist/final/neutral/index.mjs';

await describe({
  name: 'scan',
  children: [
    describe({
      name: isJsonWhitespace.name,
      children: [
        it({
          name: 'recognizes every JSON whitespace character and nothing else',
          fn: async () => {
            expect(isJsonWhitespace(' ',),).toBe(true,);
            expect(isJsonWhitespace('\t',),).toBe(true,);
            expect(isJsonWhitespace('\n',),).toBe(true,);
            expect(isJsonWhitespace('\r',),).toBe(true,);
            expect(isJsonWhitespace('x',),).toBe(false,);
            expect(isJsonWhitespace('/',),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: scanString.name,
      children: [
        it({
          name: 'scans a plain string, keeping the raw slice',
          fn: async () => {
            expect(scanString({ source: '"a" ', index: 0, },),).toEqual({ value: 'a', raw: '"a"', end: 3, },);
          },
        },),
        it({
          name: 'honors backslash escapes so an escaped quote does not end the string',
          fn: async () => {
            const scan = scanString({ source: String.raw`"a\"b"`, index: 0, },);
            expect(scan.value,).toBe('a"b',);
            expect(scan.end,).toBe(6,);
          },
        },),
        it({
          name: 'scans a string at a non-zero offset',
          fn: async () => {
            expect(scanString({ source: '[ "x"]', index: 2, },).value,).toBe('x',);
          },
        },),
        it({
          name: 'throws on an unterminated string',
          fn: async () => {
            expect(() => {
              scanString({ source: '"abc', index: 0, },);
            },).toThrow('unterminated string',);
          },
        },),
      ],
    },),
    describe({
      name: scanNumber.name,
      children: [
        it({
          name: 'scans an integer and stops at the first non-number character',
          fn: async () => {
            expect(scanNumber({ source: '12,', index: 0, },),).toEqual({ value: 12, raw: '12', end: 2, },);
          },
        },),
        it({
          name: 'scans an uppercase-exponent number',
          fn: async () => {
            expect(scanNumber({ source: '2E3]', index: 0, },),).toEqual({ value: 2_000, raw: '2E3', end: 3, },);
          },
        },),
        it({
          name: 'scans a signed fractional number with an exponent',
          fn: async () => {
            const scan = scanNumber({ source: '-1.5e+2,', index: 0, },);
            expect(scan.value,).toBe(-150,);
            expect(scan.raw,).toBe('-1.5e+2',);
            expect(scan.end,).toBe(7,);
          },
        },),
        it({
          name: 'throws on a malformed number run',
          fn: async () => {
            expect(() => {
              scanNumber({ source: '1.2.3', index: 0, },);
            },).toThrow('invalid number',);
          },
        },),
      ],
    },),
    describe({
      name: matchKeyword.name,
      children: [
        it({
          name: 'matches a keyword exactly at the offset',
          fn: async () => {
            expect(matchKeyword({ source: 'true,', index: 0, keyword: 'true', },),).toBe(true,);
            expect(matchKeyword({ source: 'xtrue', index: 1, keyword: 'true', },),).toBe(true,);
          },
        },),
        it({
          name: 'does not match when the keyword is absent at the offset',
          fn: async () => {
            expect(matchKeyword({ source: 'xtrue', index: 0, keyword: 'true', },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: scanLineComment.name,
      children: [
        it({
          name: 'scans a line comment up to but not including the newline',
          fn: async () => {
            expect(scanLineComment({ source: '// hi\nx', index: 0, },),).toEqual({ text: ' hi', end: 5, },);
          },
        },),
        it({
          name: 'scans a line comment that runs to end of input',
          fn: async () => {
            expect(scanLineComment({ source: '// hi', index: 0, },),).toEqual({ text: ' hi', end: 5, },);
          },
        },),
        it({
          name: 'ends a line comment at a bare carriage return',
          fn: async () => {
            expect(scanLineComment({ source: '// hi\r"b":2', index: 0, },),).toEqual({ text: ' hi', end: 5, },);
          },
        },),
        it({
          name: 'ends a line comment at the carriage return of a CRLF pair',
          fn: async () => {
            expect(scanLineComment({ source: '// hi\r\nx', index: 0, },),).toEqual({ text: ' hi', end: 5, },);
          },
        },),
      ],
    },),
    describe({
      name: scanBlockComment.name,
      children: [
        it({
          name: 'scans a block comment body between the delimiters',
          fn: async () => {
            expect(scanBlockComment({ source: '/* a */', index: 0, },),).toEqual({ text: ' a ', end: 7, },);
          },
        },),
        it({
          name: 'starts the close search past the opener so a slash in the body is not a close',
          fn: async () => {
            expect(scanBlockComment({ source: '/*/ */', index: 0, },),).toEqual({ text: '/ ', end: 6, },);
          },
        },),
        it({
          name: 'throws on an unterminated block comment',
          fn: async () => {
            expect(() => {
              scanBlockComment({ source: '/* open', index: 0, },);
            },).toThrow('unterminated block comment',);
          },
        },),
      ],
    },),
  ],
},);
