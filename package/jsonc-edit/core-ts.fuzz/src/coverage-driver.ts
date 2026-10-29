/**
 Deterministic coverage driver: exercises every exported function and its error
 paths with fixed inputs, so the V8 coverage it produces is reproducible. Run
 under `NODE_V8_COVERAGE` by the `fuzz:coverage` task, then summarized by
 `coverage-report.ts`.
 
 @module
 */

import type { StringJsonc, } from '@monochromatic-dev/jsonc-edit-core-ts/ts/brand.ts';
import {
  emitJsoncValue,
  jsoncDelete,
  jsoncGetComment,
  jsoncGetKeyComment,
  jsoncGetValue,
  jsoncHas,
  jsoncKeys,
  jsoncSet,
  jsoncSetComment,
  jsoncSetKeyComment,
  jsoncStringify,
  mergeComments,
  parseJsonc,
  parseJsoncEdit,
} from '@monochromatic-dev/jsonc-edit-core-ts/ts';

//region Helpers

/**
 Runs a thunk that is expected to throw, swallowing the error so the driver
 keeps exercising remaining paths. Re-throws anything that is not an `Error`.
 
 @param thunk - Operation expected to throw.
 
 @example
 ```ts
 swallow(function bad() { parseJsonc({ source: '42' as StringJsonc }); });
 ```
 */
function swallow(thunk: () => void,): void {
  try {
    thunk();
  }
  catch (error: unknown) {
    if (!(Error.isError(error,)))
      throw error;
  }
}

/**
 Brands a string as JSONC for the driver's fixed inputs.
 
 @param source - Raw source.
 
 @returns Branded source.
 
 @example
 ```ts
 asJsonc('{}');
 ```
 */
function asJsonc(source: string,): StringJsonc {
  // oxlint-disable-next-line typescript/no-unsafe-type-assertion -- branding a fixed driver input string
  return source as StringJsonc;
}

//endregion Helpers

//region Exercise

/**
 Exercises parsing, serialization, editing, and the comment API across the
 fast-path, the structured path, and every error class.
 
 @example
 ```ts
 exercise();
 ```
 */
function exercise(): void {
  // Value serializer over a parsed document.
  emitJsoncValue({ value: parseJsonc({ source: asJsonc('{"a":1,"b":[1,2,3]}',), },), },);
  // The `plainJson` leaf kind: parseJsonc no longer produces it (every document
  // parses structurally), but it remains in the public value union, so build one
  // directly to keep its serializer, `emitPlainJson`, reached.
  emitJsoncValue({
    value: {
      kind: 'plainJson',
      json: {
        a: 1,
        b: [
          1,
          2,
          3,
        ],
      },
    },
  },);
  // Structured path: comments on keys and values, trailing commas, blocks, region.
  /**
   Structured state exercising comments, trailing commas, and nesting.
   */
  const state = parseJsoncEdit({
    source: asJsonc('// doc\n{\n  // k\n  "a": 1, // v\n  /* b */ "list": [10, 20,],\n  "obj": { "x": true }\n}',),
  },);
  jsoncStringify({ state, },);
  jsoncGetValue({
    state,
    path: [
      'list',
      0,
    ],
  },);
  jsoncHas({
    state,
    path: ['a',],
  },);
  jsoncKeys({
    state,
    path: [],
  },);
  jsoncGetComment({
    state,
    path: ['a',],
  },);
  jsoncGetKeyComment({
    state,
    path: ['a',],
  },);

  // Edits: replace, add key, append, nested, delete key and element.
  /**
   State after a chain of set, add, append, and delete edits.
   */
  const edited = jsoncDelete({
    state: jsoncSet({
      state: jsoncSet({
        state: jsoncSet({
          state,
          path: ['a',],
          value: 99,
        },),
        path: ['fresh',],
        value: { nested: [
          1,
          2,
        ], },
      },),
      path: [
        'list',
        2,
      ],
      value: 30,
    },),
    path: [
      'list',
      0,
    ],
  },);
  jsoncStringify({ state: edited, },);

  // Comment-as-data writes.
  jsoncSetComment({
    state,
    path: ['a',],
    comment: {
      type: 'block',
      text: ' set ',
    },
  },);
  jsoncSetKeyComment({
    state,
    path: ['a',],
    comment: {
      type: 'inline',
      text: ' key',
    },
  },);
  mergeComments({
    first: {
      type: 'inline',
      text: 'a',
    },
    second: {
      type: 'block',
      text: 'b',
    },
  },);

  // Error paths.
  swallow(function topScalar() {
    parseJsonc({ source: asJsonc('42',), },);
  },);
  swallow(function unterminated() {
    parseJsonc({ source: asJsonc('{ // x',), },);
  },);
  swallow(function badBlock() {
    parseJsonc({ source: asJsonc('{ /* open\n"a":1 }',), },);
  },);
  swallow(function badNumber() {
    parseJsonc({ source: asJsonc('{ "a": 1, "b": 1.2.3 }',), },);
  },);
  swallow(function missingColon() {
    parseJsonc({ source: asJsonc('{ "a" 1, // x\n}',), },);
  },);
  swallow(function pathMissing() {
    jsoncGetValue({
      state,
      path: ['nope',],
    },);
  },);
  swallow(function typeMismatch() {
    jsoncSet({
      state,
      path: [
        'a',
        'b',
      ],
      value: 1,
    },);
  },);
  swallow(function deleteRoot() {
    jsoncDelete({
      state,
      path: [],
    },);
  },);
  swallow(function keyOnEmptyPath() {
    jsoncGetKeyComment({
      state,
      path: [],
    },);
  },);
}

//endregion Exercise

exercise();
