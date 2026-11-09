/**
 Built-artifact tests for balanced JSON extraction and its diagnostics.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  extractStructuredJson,
} from '../dist/final/node/index.mjs';

/**
 Capture a synchronous throw without promise matcher indirection.
 
 @param action - action expected to fail
 
 @returns thrown value
 
 @example
 ```ts
 captureThrown(() => { throw new Error('x'); });
 ```
 */
function captureThrown(action: () => unknown,): unknown {
  try {
    action();
  }
  catch (error) {
    return error;
  }
  throw new Error('expected action to fail',);
}

await describe({
  name: extractStructuredJson.name,
  children: [
    it({
      name: 'parses whole-text JSON object',
      fn: async () => {
        expect(extractStructuredJson('{"approved":true,"feedback":"complete"}',),).toEqual({
          approved: true,
          feedback: 'complete',
        },);
      },
    },),
    it({
      name: 'extracts first balanced object from surrounding prose',
      fn: async () => {
        expect(extractStructuredJson('prefix {"approved":false} suffix',),).toEqual({
          approved: false,
        },);
      },
    },),
    it({
      name: 'retains the whole-text parse failure as the cause when text holds no object',
      fn: async () => {
        /**
         Failure thrown for reviewer text without a JSON object.
         */
        const error = captureThrown(function extractWithoutObject() {
          return extractStructuredJson('not structured',);
        },);
        expect(error,).toBeInstanceOf(Error,);
        expect((error as Error).message,).toContain('without JSON object',);
        expect((error as Error).cause,).toBeInstanceOf(SyntaxError,);
        expect(((error as Error).cause as Error).message,).toContain('not structured',);
      },
    },),
  ],
},);
