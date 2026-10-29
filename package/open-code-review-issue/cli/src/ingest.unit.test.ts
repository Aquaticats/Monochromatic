import {
  mkdtempDisposable,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  InputValidationError,
  parseStructuredInput,
  readStructuredInputFile,
} from '../dist/final/node/index.mjs';

await describe({
  name: parseStructuredInput.name,
  children: [
    it({
      name: 'accepts a complete OCR JSON result',
      fn: async () => {
        /**
         Complete result fixture carrying repository-head provenance.
         */
        const text = JSON.stringify({
          status: 'complete',
          comments: [
            {
              path: 'src/example.ts',
              content: 'Boundary handling is incorrect.',
              existing_code: 'return value + 1;',
              suggestion_code: 'return value;',
              start_line: 4,
              end_line: 5,
              category: 'bug',
              severity: 'high',
              thinking: 'private reasoning',
            },
          ],
          manifest: {
            input: {
              resolved_head: '0123456789abcdef0123456789abcdef01234567',
            },
          },
        },);

        /**
         Normalized adapter input.
         */
        const result = parseStructuredInput({ text, },);

        expect(result,).toStrictEqual({
          inputKind: 'result',
          resolvedHead: '0123456789abcdef0123456789abcdef01234567',
          findings: [
            {
              position: {
                kind: 'record',
                value: 1,
              },
              path: 'src/example.ts',
              content: 'Boundary handling is incorrect.',
              existingCode: 'return value + 1;',
              suggestionCode: 'return value;',
              startLine: 4,
              endLine: 5,
              category: 'bug',
              severity: 'high',
            },
          ],
        },);
      },
    },),
    it({
      name: 'accepts a bare OCR comment array',
      fn: async () => {
        /**
         Bare comments fixture with optional metadata absent.
         */
        const text = JSON.stringify([
          {
            path: 'src/plain.ts',
            content: 'Use the guarded branch.',
            start_line: 7,
            end_line: 7,
          },
        ],);

        /**
         Normalized adapter input.
         */
        const result = parseStructuredInput({ text, },);

        expect(result,).toStrictEqual({
          inputKind: 'comments',
          findings: [
            {
              position: {
                kind: 'record',
                value: 1,
              },
              path: 'src/plain.ts',
              content: 'Use the guarded branch.',
              existingCode: '',
              suggestionCode: '',
              startLine: 7,
              endLine: 7,
            },
          ],
        },);
      },
    },),
    it({
      name: 'replays JSONL supersession and failure records',
      fn: async () => {
        /**
         Session transcript whose later records replace and remove checkpoints.
         */
        const text = [
          JSON.stringify({ type: 'session_start', sessionId: 'run-1', }),
          JSON.stringify({
            type: 'review_item_done',
            filePath: 'src/replayed.ts',
            fingerprint: 'fingerprint-a',
            comments: [
              {
                path: '',
                content: 'First version.',
                start_line: 2,
                end_line: 2,
              },
            ],
          },),
          JSON.stringify({
            type: 'review_item_reused',
            filePath: 'src/replayed.ts',
            fingerprint: 'fingerprint-a',
            comments: [
              {
                path: '',
                content: 'Replacement version.',
                start_line: 3,
                end_line: 3,
                category: 'maintainability',
              },
            ],
          },),
          JSON.stringify({
            type: 'review_item_done',
            filePath: 'src/removed.ts',
            fingerprint: 'fingerprint-b',
            comments: [
              {
                path: '',
                content: 'Removed by failure.',
                start_line: 8,
                end_line: 8,
              },
            ],
          },),
          JSON.stringify({
            type: 'review_item_failed',
            fingerprint: 'fingerprint-b',
          },),
          JSON.stringify({
            type: 'session_end',
            run_manifest: {
              input: {
                resolved_head: 'abcdefabcdefabcdefabcdefabcdefabcdefabcd',
              },
            },
          },),
        ].join('\n',);

        /**
         Replayed adapter input.
         */
        const result = parseStructuredInput({ text, },);

        expect(result,).toStrictEqual({
          inputKind: 'jsonl',
          resolvedHead: 'abcdefabcdefabcdefabcdefabcdefabcdefabcd',
          findings: [
            {
              position: {
                kind: 'line',
                value: 3,
              },
              path: 'src/replayed.ts',
              content: 'Replacement version.',
              existingCode: '',
              suggestionCode: '',
              startLine: 3,
              endLine: 3,
              category: 'maintainability',
            },
          ],
        },);
      },
    },),
    it({
      name: 'rejects findings without a meaningful title source',
      fn: async () => {
        /**
         Sparse finding forbidden by atomic input validation.
         */
        const text = JSON.stringify([
          {
            path: 'src/empty.ts',
            content: ' \n\t ',
            existing_code: '\n',
            suggestion_code: '  ',
            start_line: 1,
            end_line: 1,
          },
        ],);
        /**
         Captured validation failure.
         */
        let caught: unknown;
        try {
          parseStructuredInput({ text, },);
        }
        catch (error: unknown) {
          caught = error;
        }

        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain('record 1',);
        expect((caught as Error).message,).toContain('non-whitespace line',);
      },
    },),
    it({
      name: 'rejects a UTF-8 byte-order mark in a named file',
      fn: async () => {
        /**
         Disposable input directory.
         */
        await using directory = await mkdtempDisposable(join(tmpdir(), 'ocr-issue-input-',),);
        /**
         Named file carrying forbidden UTF-8 BOM before valid JSON.
         */
        const path = join(directory.path, 'review.json',);
        await writeFile(path, Buffer.concat([
          Buffer.from('\uFEFF', 'utf8',),
          Buffer.from('[]', 'utf8',),
        ],),);

        /**
         Captured transport validation failure.
         */
        let caught: unknown;
        try {
          await readStructuredInputFile({ path, },);
        }
        catch (error: unknown) {
          caught = error;
        }

        expect(caught,).toBeInstanceOf(Error,);
        expect((caught as Error).message,).toContain('byte-order mark',);
      },
    },),
    it({
      name: 'reads a strict UTF-8 named file',
      fn: async () => {
        /**
         Disposable input directory.
         */
        await using directory = await mkdtempDisposable(join(tmpdir(), 'ocr-issue-input-',),);
        /**
         Valid named comment-array file.
         */
        const path = join(directory.path, 'review.json',);
        await writeFile(path, '[]', 'utf8',);

        /**
         Parsed named-file input.
         */
        const result = await readStructuredInputFile({ path, },);

        expect(result,).toStrictEqual({
          inputKind: 'comments',
          findings: [],
        },);
      },
    },),
    it({
      name: 'rejects malformed UTF-8 bytes in a named file',
      fn: async () => {
        /**
         Disposable input directory.
         */
        await using directory = await mkdtempDisposable(join(tmpdir(), 'ocr-issue-input-',),);
        /**
         Invalid two-byte UTF-8 sequence fixture.
         */
        const path = join(directory.path, 'review.json',);
        await writeFile(path, Buffer.from('c328', 'hex',),);

        /**
         Captured strict-decoding failure.
         */
        let caught: unknown;
        try {
          await readStructuredInputFile({ path, },);
        }
        catch (error: unknown) {
          caught = error;
        }

        expect(caught,).toBeInstanceOf(InputValidationError,);
        expect((caught as Error).message,).toContain('not strict UTF-8',);
      },
    },),
    it({
      name: 'rejects malformed records atomically',
      fn: async () => {
        /**
         Inputs whose recognized envelopes contain one malformed record.
         */
        const cases: readonly {
          readonly text: string;
          readonly expected: string;
        }[] = [
          {
            text: JSON.stringify({ status: 'complete', comments: 'wrong', },),
            expected: 'complete OCR result or comment array',
          },
          {
            text: JSON.stringify([
              {
                path: 'src/valid.ts',
                content: 'Valid.',
                start_line: 1,
                end_line: 1,
              },
              {
                path: 'src/invalid.ts',
                content: 'Invalid category.',
                start_line: 1,
                end_line: 1,
                category: 'mystery',
              },
            ],),
            expected: 'record 2 has unsupported category',
          },
          {
            text: [
              JSON.stringify({ type: 'session_start', }),
              '',
              JSON.stringify({ type: 'session_end', }),
            ].join('\n',),
            expected: 'line 2 must not be blank',
          },
        ];

        for (const invalidCase of cases) {
          /**
           Captured validation failure for current malformed fixture.
           */
          let caught: unknown;
          try {
            parseStructuredInput({ text: invalidCase.text, },);
          }
          catch (error: unknown) {
            caught = error;
          }
          expect(caught,).toBeInstanceOf(InputValidationError,);
          expect((caught as Error).message,).toContain(invalidCase.expected,);
        }
      },
    },),
    it({
      name: 'accepts completed JSONL checkpoints without comments field',
      fn: async () => {
        /**
         OCR omits comments property when completed item has zero findings.
         */
        const text = JSON.stringify({
          type: 'review_item_done',
          filePath: 'src/clean.ts',
          fingerprint: 'clean-fingerprint',
        },);

        expect(parseStructuredInput({ text, },),).toStrictEqual({
          inputKind: 'jsonl',
          findings: [],
        },);
      },
    },),
  ],
},);
