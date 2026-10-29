import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

import {
  dedupDiagnostics,
  uniqueDiagnostics,
} from './dedup.ts';

import type { Diagnostic, } from './nvim-client.ts';

//region helpers: factory for test diagnostics

/**
 * Creates a diagnostic with sensible defaults, overridable by partial input.
 *
 * @param overrides - Fields to override.
 *
 * @returns Complete Diagnostic object.
 */
function makeDiag(overrides: Partial<Diagnostic> = {},): Diagnostic {
  return {
    severity: 'ERROR',
    lnum: 1,
    col: 1,
    end_lnum: 1,
    end_col: 1,
    message: 'default message',
    source: null,
    code: null,
    ...overrides,
  };
}

//endregion helpers

//region dedupDiagnostics: merges editor and lint diagnostics with deduplication

await describe({
  name: '',
  children: [
    describe({
      name: dedupDiagnostics.name,
      children: [
        it({
          name: 'returns editor diagnostics unchanged when lint is empty',
          fn: async () => {
            const editor = [
              makeDiag({ lnum: 10, col: 5, source: 'typescript', code: 2_345, },),
            ];
            const result = dedupDiagnostics({ editor, lint: [], },);
            expect(result,).toEqual(editor,);
          },
        },),
        it({
          name: 'returns lint diagnostics when editor is empty',
          fn: async () => {
            const lint = [
              makeDiag({ lnum: 3, col: 1, source: 'oxlint', code: 'no-unused-vars', },),
            ];
            const result = dedupDiagnostics({ editor: [], lint, },);
            expect(result,).toEqual(lint,);
          },
        },),
        it({
          name: 'deduplicates by lnum:col:code when both have code',
          fn: async () => {
            const editor = [makeDiag({
              lnum: 10,
              col: 5,
              source: 'typescript',
              code: 2_345,
              message: "Type 'string' is not assignable to type 'number'.",
              end_lnum: 10,
              end_col: 15,
            },),];
            const lint = [makeDiag({
              lnum: 10,
              col: 5,
              source: 'oxlint',
              code: 2_345,
              message: 'Type mismatch',
              end_lnum: 10,
              end_col: 5,
            },),];

            const result = dedupDiagnostics({ editor, lint, },);
            expect(result,).toHaveLength(1,);
            // Editor diagnostic is kept (richer end position info)
            expect(result[0]?.source,).toBe('typescript',);
          },
        },),
        it({
          name: 'deduplicates by lnum:col:message when code is absent',
          fn: async () => {
            const editor = [makeDiag({
              lnum: 5,
              col: 1,
              source: 'lsp',
              code: null,
              message: 'Unexpected token',
            },),];
            const lint = [makeDiag({
              lnum: 5,
              col: 1,
              source: 'oxlint',
              code: null,
              message: 'Unexpected token',
            },),];

            const result = dedupDiagnostics({ editor, lint, },);
            expect(result,).toHaveLength(1,);
            expect(result[0]?.source,).toBe('lsp',);
          },
        },),
        it({
          name: 'keeps both when same location but different code',
          fn: async () => {
            const editor = [
              makeDiag({ lnum: 10, col: 5, source: 'typescript', code: 2_345, },),
            ];
            const lint = [
              makeDiag({ lnum: 10, col: 5, source: 'oxlint', code: 'no-any', },),
            ];

            const result = dedupDiagnostics({ editor, lint, },);
            expect(result,).toHaveLength(2,);
            expect(result[1]?.source,).toBe('oxlint',);
          },
        },),
        it({
          name: 'keeps both when same code but different location',
          fn: async () => {
            const editor = [makeDiag({ lnum: 10, col: 5, code: 'no-unused-vars', },),];
            const lint = [makeDiag({ lnum: 20, col: 3, code: 'no-unused-vars', },),];

            const result = dedupDiagnostics({ editor, lint, },);
            expect(result,).toHaveLength(2,);
          },
        },),
        it({
          name: 'appends lint-only diagnostics after editor diagnostics',
          fn: async () => {
            const editor = [
              makeDiag({ lnum: 1, col: 1, source: 'typescript', code: 1_000, },),
            ];
            const lint = [
              makeDiag({ lnum: 50, col: 10, source: 'oxlint',
                code: 'tsdoc(require-tsdoc)', },),
            ];

            const result = dedupDiagnostics({ editor, lint, },);
            expect(result,).toHaveLength(2,);
            expect(result[0]?.source,).toBe('typescript',);
            expect(result[1]?.source,).toBe('oxlint',);
          },
        },),
        it({
          name: 'handles multiple duplicates and unique diagnostics',
          fn: async () => {
            const editor = [
              makeDiag({ lnum: 1, col: 1, code: 'rule-a', },),
              makeDiag({ lnum: 5, col: 3, code: 'rule-b', },),
            ];
            const lint = [
              makeDiag({ lnum: 1, col: 1, code: 'rule-a', },),
              makeDiag({ lnum: 5, col: 3, code: 'rule-b', },),
              makeDiag({ lnum: 10, col: 1, code: 'rule-c', },),
            ];

            const result = dedupDiagnostics({ editor, lint, },);
            // 2 from editor + 1 unique from lint
            expect(result,).toHaveLength(3,);
          },
        },),
        it({
          name: 'returns empty array when both inputs are empty',
          fn: async () => {
            const result = dedupDiagnostics({ editor: [], lint: [], },);
            expect(result,).toEqual([],);
          },
        },),
        it({
          name: 'uses code for dedup key even when messages differ',
          fn: async () => {
            const editor = [
              makeDiag({ lnum: 10, col: 5, code: 'same-rule',
                message: 'Editor phrasing', },),
            ];
            const lint = [
              makeDiag({ lnum: 10, col: 5, code: 'same-rule',
                message: 'CLI phrasing', },),
            ];

            const result = dedupDiagnostics({ editor, lint, },);
            expect(result,).toHaveLength(1,);
          },
        },),
      ],
    },),

    //endregion dedupDiagnostics

    //region uniqueDiagnostics: removes duplicates within a single array

    describe({
      name: uniqueDiagnostics.name,
      children: [
        it({
          name: 'removes exact duplicates keeping first occurrence',
          fn: async () => {
            const diag = makeDiag({ lnum: 10, col: 5, source: 'oxc',
              code: 'no-unused-vars', },);
            const result = uniqueDiagnostics([diag, { ...diag, }, { ...diag, },],);
            expect(result,).toHaveLength(1,);
            expect(result[0],).toEqual(diag,);
          },
        },),
        it({
          name: 'keeps diagnostics with different locations',
          fn: async () => {
            const diagA = makeDiag({ lnum: 10, col: 5, code: 'rule-a', },);
            const diagB = makeDiag({ lnum: 20, col: 3, code: 'rule-a', },);
            const result = uniqueDiagnostics([diagA, diagB,],);
            expect(result,).toHaveLength(2,);
          },
        },),
        it({
          name: 'keeps diagnostics with different codes at same location',
          fn: async () => {
            const diagA = makeDiag({ lnum: 10, col: 5, code: 'rule-a', },);
            const diagB = makeDiag({ lnum: 10, col: 5, code: 'rule-b', },);
            const result = uniqueDiagnostics([diagA, diagB,],);
            expect(result,).toHaveLength(2,);
          },
        },),
        it({
          name: 'deduplicates across sources at same location and code',
          fn: async () => {
            const editorDiag = makeDiag({ lnum: 10, col: 5, source: 'oxc',
              code: 'no-unused-vars', message: 'from editor', },);
            const instanceDiag = makeDiag({ lnum: 10, col: 5, source: 'oxc',
              code: 'no-unused-vars', message: 'from other instance', },);
            const result = uniqueDiagnostics([editorDiag, instanceDiag,],);
            expect(result,).toHaveLength(1,);
            expect(result[0]?.message,).toBe('from editor',);
          },
        },),
        it({
          name: 'returns empty array for empty input',
          fn: async () => {
            expect(uniqueDiagnostics([],),).toEqual([],);
          },
        },),
        it({
          name: 'passes through single diagnostic unchanged',
          fn: async () => {
            const diag = makeDiag({ lnum: 1, col: 1, },);
            expect(uniqueDiagnostics([diag,],),).toEqual([diag,],);
          },
        },),
      ],
    },),
    //endregion uniqueDiagnostics
  ],
},);
