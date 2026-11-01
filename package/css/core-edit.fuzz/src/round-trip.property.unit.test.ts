/**
 Property tests proving the css-edit CST is byte-lossless on valid input and
 total (throws only `CssParseError`) on arbitrary input.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  asCssSource,
  CssParseError,
  parseCss,
  stringifyCss,
  transformStylesheet,
} from '@monochromatic-dev/css-core-edit/ts';
import {
  assert,
  property,
  string,
} from 'fast-check';

import { cssDocumentArb, } from './css-arbitrary.ts';
import { fuzzRuns, } from './fuzz-budget.ts';

await describe({
  name: 'round-trip property',
  children: [
    it({
      name: 'a generated document parses and stringifies byte-identically',
      fn: async () => {
        assert(
          property(cssDocumentArb, (css,) => {
            /**
             Parsed state of the generated document.
             */
            const state = parseCss({ source: asCssSource(css,), },);
            expect(stringifyCss({ state, },),).toBe(css,);
          },),
          { numRuns: fuzzRuns, },
        );
      },
    },),

    it({
      name: 'an arbitrary string either round-trips or throws CssParseError',
      fn: async () => {
        assert(
          property(string({ unit: 'binary', },), (candidate,) => {
            try {
              /**
               Parsed state when the candidate happens to be valid CSS.
               */
              const state = parseCss({ source: asCssSource(candidate,), },);
              expect(stringifyCss({ state, },),).toBe(candidate,);
            }
            catch (error) {
              expect(error,).toBeInstanceOf(CssParseError,);
            }
          },),
          { numRuns: fuzzRuns, },
        );
      },
    },),

    it({
      name: 'a keep-everything transform preserves reference identity',
      fn: async () => {
        assert(
          property(cssDocumentArb, (css,) => {
            /**
             Root of the generated document.
             */
            const { root, } = parseCss({ source: asCssSource(css,), },);
            expect(transformStylesheet({
              root,
              visit: function keepAll(node,) {
                return node;
              },
            },),).toBe(root,);
          },),
          { numRuns: fuzzRuns, },
        );
      },
    },),
  ],
},);
