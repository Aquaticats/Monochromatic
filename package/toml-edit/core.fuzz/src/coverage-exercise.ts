/**
 Per-input reachability sweeps for the toml-edit fuzz coverage gate.
 
 Imports the package implementation from source (`../index.ts`) so a run under
 `NODE_V8_COVERAGE` attributes coverage to the `src` files the gate measures.
 Each function drives a slice of the public API or the unstable `_` seams over
 caller-supplied inputs; this is a reachability harness, not an oracle, and
 asserts nothing (the property suite owns correctness). The run-and-count
 primitives and edit sequence live in `./coverage-harness.ts` and
 `./coverage-edits.ts`.
 
 @module
 */

import {
  _emitContentNode,
  _emitStringValue,
  _encodeKey,
  _jsValueToTomlText,
  _emitDocument,
  type CanonicalOptions,
  parseTomlEdit,
  type TomlEditState,
  type TomlPath,
  tomlGet,
  tomlGetCommentAfter,
  tomlGetComments,
  tomlGetCommentsBefore,
  tomlGetNode,
  tomlGetRaw,
  tomlGetValue,
  tomlHas,
  tomlInsertCommentAfter,
  tomlInsertCommentBefore,
  tomlKeys,
  tomlSetHeaderComment,
  tomlStringify,
} from '@monochromatic-dev/toml-edit-core/ts';

import {
  attempt,
  tryParse,
} from './coverage-harness.ts';
import { exerciseEditSequence, } from './coverage-edits.ts';

//region Reads and comments

/**
 Drive every read accessor over each top-level key of a parsed splice state.
 */
function exerciseReads({ edit, }: { readonly edit: TomlEditState; },): void {
  attempt({
    thunk: function reads() {
      tomlGetComments({ edit, },);
      for (const key of tomlKeys({ edit, },)) {
        /**
         Single-segment path for the current top-level key.
         */
        const path: TomlPath = [key,];
        tomlHas({
          edit,
          path,
        },);
        tomlGet({
          edit,
          path,
        },);
        tomlGetValue({
          edit,
          path,
        },);
        tomlGetNode({
          edit,
          path,
        },);
        tomlKeys({
          edit,
          path,
        },);
        tomlGetCommentsBefore({
          edit,
          path,
        },);
        tomlGetCommentAfter({
          edit,
          path,
        },);
        tomlGetRaw({
          edit,
          path,
        },);
      }
    },
  },);
}

/**
 Drive the comment writers: a header comment, then before and after comments on
 the first top-level key, stringifying each result.
 */
function exerciseComments({ edit, }: { readonly edit: TomlEditState; },): void {
  attempt({
    thunk: function comments() {
      tomlStringify({ edit: tomlSetHeaderComment({
        edit,
        comment: 'coverage header note',
      },), },);
      /**
       First top-level key, when the document has one.
       */
      const [first,] = tomlKeys({ edit, },);
      if (first === undefined) return;
      /**
       Single-segment path for the first key.
       */
      const path: TomlPath = [first,];
      /**
       State after inserting a preceding comment, fed to the after-insert.
       */
      const before = tomlInsertCommentBefore({
        edit,
        path,
        comment: 'coverage before',
      },);
      tomlStringify({ edit: tomlInsertCommentAfter({
        edit: before,
        path,
        comment: 'coverage after',
      },), },);
    },
  },);
}

//endregion Reads and comments

//region Seams

/**
 Drive the unstable `_` seam exports directly: key encoding, value encoding,
 and parsed-node re-emission.
 
 @mutates jsonValues - Value encoding can invoke caller-owned proxy, accessor, and coercion hooks.
 
 @example
 ```ts
 exerciseSeams({ keyNames: [ 'a', ], jsonValues: [ 1, ], scalarTexts: [ '1', ], canonicalOptions, },);
 ```
 */
export function exerciseSeams(
  {
    keyNames,
    jsonValues,
    scalarTexts,
    canonicalOptions,
  }: {
    readonly keyNames: readonly string[];
    readonly jsonValues: readonly unknown[];
    readonly scalarTexts: readonly string[];
    readonly canonicalOptions: CanonicalOptions;
  },
): void {
  for (const key of keyNames) {
    attempt({
      thunk: function encodeKey() {
        _encodeKey({ key, },);
      },
    },);
  }
  for (const input of jsonValues) {
    attempt({
      thunk: function encodeValue() {
        _jsValueToTomlText({
          input,
          options: canonicalOptions,
        },);
      },
    },);
  }
  for (const text of scalarTexts) {
    attempt({
      thunk: function reemitNode() {
        /**
         Parse-time value node for a single-scalar document.
         */
        const node = tomlGetNode({
          edit: parseTomlEdit({ source: `probe = ${text}\n`, },),
          path: ['probe',],
        },);
        if ((!('type' in node)) || (node.type !== 'TOMLValue')) return;
        _emitContentNode({
          node,
          options: canonicalOptions,
        },);
        if (node.kind === 'string') _emitStringValue({ node, },);
      },
    },);
  }
}

//endregion Seams

//region Source-driven sweeps

/**
 Drive the read, splice, and canonical spread for one source the parser may or
 may not accept. A rejected parse still exercises the parser error paths.
 
 @example
 ```ts
 exerciseValidSource({ source: 'a = 1\n', },);
 ```
 */
export function exerciseValidSource({ source, }: { readonly source: string; },): void {
  /**
   Splice-mode parse result.
   */
  const splice = tryParse({
    source,
    mode: 'splice',
  },);
  if (splice.ok) {
    attempt({
      thunk: function emitAndStringify() {
        _emitDocument({ edit: splice.edit, },);
        tomlStringify({ edit: splice.edit, },);
      },
    },);
    exerciseReads({ edit: splice.edit, },);
  }
  /**
   Canonical-mode parse result, whose stringify rebuilds every node from the
   AST and so exercises the canonical emitter and value-string escaper.
   */
  const canonical = tryParse({
    source,
    mode: 'canonical',
  },);
  if (canonical.ok) {
    attempt({
      thunk: function canonicalStringify() {
        tomlStringify({ edit: canonical.edit, },);
      },
    },);
    attempt({
      thunk: function rawRejects() {
        tomlGetRaw({
          edit: canonical.edit,
          path: ['probe',],
        },);
      },
    },);
  }
}

/**
 Run an invalid or corrupted source through the parser so its rejection and
 error-wrapping paths are reached. Acceptance is fine too; accepted sources
 just fall through to a stringify.
 
 @example
 ```ts
 exerciseInvalidSource({ source: 'a = = 1\n', },);
 ```
 */
export function exerciseInvalidSource({ source, }: { readonly source: string; },): void {
  /**
   Parse result; rejection is the common, intended outcome here.
   */
  const parsed = tryParse({
    source,
    mode: 'splice',
  },);
  if (parsed.ok) {
    attempt({
      thunk: function stringify() {
        tomlStringify({ edit: parsed.edit, },);
      },
    },);
  }
}

/**
 Run the comment writers and the deterministic edit sequence from a parsed
 base. The machinery is base-independent, so the driver runs this over a
 bounded subset rather than every sampled document.
 
 @example
 ```ts
 exerciseEditsAndComments({ source: 'a = 1\n', },);
 ```
 */
export function exerciseEditsAndComments({ source, }: { readonly source: string; },): void {
  /**
   Splice-mode parse of the base; a rejected base contributes nothing.
   */
  const base = tryParse({
    source,
    mode: 'splice',
  },);
  if (!base.ok) return;
  exerciseComments({ edit: base.edit, },);
  exerciseEditSequence({ base: base.edit, },);
}

//endregion Source-driven sweeps
