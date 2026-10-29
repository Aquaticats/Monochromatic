/**
 * Equivalence tests for the LSP Content-Length header parser.
 *
 * Capture the pre-refactor behavior of `parseContentLength` so the
 * linear-pass rewrite stays behavior-identical: case-insensitive label
 * match, inline-whitespace tolerance (space and tab only), digit-run
 * accumulation stopping at the first non-digit, the empty-digit guard
 * returning null before `Number()` coercion, and stack safety on an
 * adversarial unbounded-whitespace header. Also drives the public
 * `createLspParser` consumer end to end so the framing boundary is
 * exercised the way the LSP client uses it.
 *
 * @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

import {
  createLspParser,
  type JsonRpcMessage,
  parseContentLength,
} from './json-rpc.ts';

/** Whitespace/digit run length for the stack-safety cases; large enough to overflow a recursive cursor under V8, fast on a linear pass. */
const LONG_RUN = 50_000;

await describe({
  name: '',
  children: [
    describe({
      name: parseContentLength.name,
      children: [
        //region No label / no value -> null

        it({
          name: 'returns null for an empty header',
          fn: async () => {
            expect(parseContentLength('',),).toBeNull();
          },
        },),
        it({
          name: 'returns null when no Content-Length label is present',
          fn: async () => {
            expect(parseContentLength('Content-Type: application/json',),).toBeNull();
          },
        },),
        it({
          name: 'returns null for all-whitespace input (no label)',
          fn: async () => {
            expect(parseContentLength('   \t  ',),).toBeNull();
          },
        },),
        it({
          name: 'returns null when the label is followed by no digits',
          fn: async () => {
            expect(parseContentLength('Content-Length:',),).toBeNull();
          },
        },),
        it({
          name: 'returns null when only whitespace follows the label',
          fn: async () => {
            expect(parseContentLength('Content-Length:   \t',),).toBeNull();
          },
        },),
        it({
          name: 'returns null when a non-digit immediately follows the whitespace',
          fn: async () => {
            expect(parseContentLength('Content-Length: abc',),).toBeNull();
          },
        },),

        //endregion No label / no value -> null

        //region Digit parsing

        it({
          name: 'parses digits separated from the label by a single space',
          fn: async () => {
            expect(parseContentLength('Content-Length: 42',),).toBe(42,);
          },
        },),
        it({
          name: 'parses digits directly adjacent to the label colon',
          fn: async () => {
            expect(parseContentLength('Content-Length:42',),).toBe(42,);
          },
        },),
        it({
          name: 'skips a tab between the label and the digits',
          fn: async () => {
            expect(parseContentLength('Content-Length:\t42',),).toBe(42,);
          },
        },),
        it({
          name: 'skips a mixed run of spaces and tabs before the digits',
          fn: async () => {
            expect(parseContentLength('Content-Length:  \t \t42',),).toBe(42,);
          },
        },),
        it({
          name: 'stops the digit run at the first non-digit character',
          fn: async () => {
            expect(parseContentLength('Content-Length: 123abc',),).toBe(123,);
          },
        },),
        it({
          name: 'coerces a leading-zero digit run via Number',
          fn: async () => {
            expect(parseContentLength('Content-Length: 007',),).toBe(7,);
          },
        },),

        //endregion Digit parsing

        //region Case-insensitive label

        it({
          name: 'matches an all-uppercase label',
          fn: async () => {
            expect(parseContentLength('CONTENT-LENGTH: 5',),).toBe(5,);
          },
        },),
        it({
          name: 'matches an all-lowercase label',
          fn: async () => {
            expect(parseContentLength('content-length: 5',),).toBe(5,);
          },
        },),
        it({
          name: 'finds the label after a preceding header line',
          fn: async () => {
            expect(
              parseContentLength(
                'Content-Type: application/vscode-jsonrpc; charset=utf-8\r\nContent-Length: 99',
              ),
            ).toBe(99,);
          },
        },),

        //endregion Case-insensitive label

        //region Stack safety on long runs

        it({
          name: 'parses through an adversarial unbounded whitespace run without overflow',
          fn: async () => {
            expect(
              parseContentLength(`Content-Length:${' '.repeat(LONG_RUN,)}5`,),
            ).toBe(5,);
          },
        },),
        it({
          name: 'accumulates a long digit run with the same Number coercion as the spec',
          fn: async () => {
            /** Long all-nines digit run; coerces to Infinity, but the linear scan must match the spec exactly. */
            const digits = '9'.repeat(LONG_RUN,);
            expect(parseContentLength(`Content-Length: ${digits}`,),).toBe(Number(digits,),);
          },
        },),

        //endregion Stack safety on long runs
      ],
    },),

    describe({
      name: createLspParser.name,
      children: [
        it({
          name: 'extracts a complete message framed by a long-whitespace Content-Length header',
          fn: async () => {
            /** Messages captured from the parser's onMessage callback. */
            const messages: JsonRpcMessage[] = [];
            /** Parser under test; collects decoded messages and rethrows any parse error. */
            const parser = createLspParser({
              onMessage: function captureMessage(message: JsonRpcMessage,): void {
                messages.push(message,);
              },
              onError: function rethrowError(error: unknown,): void {
                throw error;
              },
            },);
            /** Minimal JSON-RPC body; its UTF-8 byte length feeds the Content-Length value. */
            const body = '{"jsonrpc":"2.0"}';
            parser.feed(
              Buffer.from(
                `Content-Length:${' '.repeat(LONG_RUN,)}${
                  String(Buffer.byteLength(body, 'utf8',),)
                }\r\n\r\n${body}`,
                'utf8',
              ),
            );
            expect(messages,).toHaveLength(1,);
            expect(messages[0],).toEqual({ jsonrpc: '2.0', },);
          },
        },),
      ],
    },),
  ],
},);
