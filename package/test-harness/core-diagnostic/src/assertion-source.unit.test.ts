/**
 Tests for `./assertion-source.ts`: the total string helpers
 (`extractLocationSubstring`, `isIntegerString`), the backward
 source-window extraction, and the node-only error-tree reader that
 splices an assertion's source line into failure output. Frame parsing
 has no standalone optional-returning function (absence is a `continue`
 guard inside the reader), so its edge cases (`file://` URL, missing
 column, native frame) are exercised through `readAssertionSites`.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  extractAssertionExpression,
  extractLocationSubstring,
  isIntegerString,
  readAssertionSites,
} from '@monochromatic-dev/test-harness-core-diagnostic';

/**
 Writes `lines` to a unique temp `.ts` file and returns its absolute
 path, so a synthetic stack frame can point at a real readable file.
 The name is keyed by pid and `index` to avoid collisions across the
 suite without relying on randomness.
 
 @param lines - file contents, one element per line
 
 @param index - per-test discriminator folded into the filename
 
 @returns absolute path of the written fixture
 
 @example
 ```ts
 const path = await writeTempSource({
   lines: ['expect(x,)', '  .toBe(2,);'],
   index: 0,
 });
 ```
 */
async function writeTempSource({
  lines,
  index,
}: {
  readonly lines: readonly string[];
  readonly index: number;
},): Promise<string> {
  /**
   Node fs imported here (not at module top) so this module stays loadable where filesystem access is absent.
   */
  const { mkdir, writeFile, } = await import('node:fs/promises');
  const { resolve, } = await import('node:path');
  /**
   Gitignored scratch directory inside this package (not under node_modules, whose frames the source reader skips), the disposable home for fixture files this test reads back;
   never the system temporary directory, whose backing device is unknowable.
   */
  const directory = resolve(import.meta.dirname, '..', '.cache', 'test-scratch',);
  await mkdir(directory, { recursive: true, },);
  /**
   Absolute fixture path, unique per process and test index.
   */
  const path = `${directory}/assertion-source-${String(process.pid,)}-${String(index,)}.ts`;
  await writeFile(
    path,
    lines.join('\n',),
    'utf8',
  );
  return path;
}

/**
 Removes a temp fixture, ignoring errors so cleanup never masks a real
 assertion failure.
 
 @param path - absolute fixture path to delete
 */
async function removeTempSource(path: string,): Promise<void> {
  /**
   Node fs imported lazily, matching {@link writeTempSource}.
   */
  const { rm, } = await import('node:fs/promises');
  try {
    await rm(path,);
  }
  catch (error: unknown) {
    if (!Error.isError(error,))
      throw error;
    // best-effort cleanup; the OS reclaims tmp regardless
  }
}

await describe({
  name: 'assertion-source',
  children: [
    //region extractLocationSubstring

    it({
      name: 'extracts the parenthesised location body',
      fn: async () => {
        expect(extractLocationSubstring('at fn (/abs/file.ts:21:12)',),)
          .toBe('/abs/file.ts:21:12',);
      },
    },),

    it({
      name: 'strips a file:// URL scheme from the location body',
      fn: async () => {
        expect(extractLocationSubstring('at fn (file:///abs/file.ts:21:12)',),)
          .toBe('/abs/file.ts:21:12',);
      },
    },),

    it({
      name: 'handles the bare at <path> form without a function name',
      fn: async () => {
        expect(extractLocationSubstring('at /abs/file.ts:9:3',),)
          .toBe('/abs/file.ts:9:3',);
      },
    },),

    //endregion extractLocationSubstring

    //region isIntegerString

    it({
      name: 'accepts a run of digits and rejects empty or non-digit input',
      fn: async () => {
        expect(isIntegerString('21',),).toBe(true,);
        expect(isIntegerString('',),).toBe(false,);
        expect(isIntegerString('a1',),).toBe(false,);
        expect(isIntegerString('1.2',),).toBe(false,);
      },
    },),

    //endregion isIntegerString

    //region extractAssertionExpression

    it({
      name: 'joins a multi-line assertion back to its expect( opener',
      fn: async () => {
        expect(extractAssertionExpression({
          sourceLines: [
            'const c = 3;',
            'expect(errorSpy.callCount,)',
            '  .toBe(2,);',
          ],
          lineNumber: 3,
        },),)
          .toBe('expect(errorSpy.callCount,) .toBe(2,);',);
      },
    },),

    it({
      name: 'returns a single-line assertion unchanged but trimmed',
      fn: async () => {
        expect(extractAssertionExpression({
          sourceLines: ['        expect(x,).toBe(2,);',],
          lineNumber: 1,
        },),)
          .toBe('expect(x,).toBe(2,);',);
      },
    },),

    it({
      name: 'falls back to the target line when no expect( is in range',
      fn: async () => {
        expect(extractAssertionExpression({
          sourceLines: [
            'const a = 1;',
            'doSomething(a,);',
          ],
          lineNumber: 2,
        },),)
          .toBe('doSomething(a,);',);
      },
    },),

    it({
      name: 'does not reach an expect( beyond the lookback window',
      fn: async () => {
        /**
         14 lines: an `expect(` opener on line 1, then 12 filler lines,
         then the matcher on line 14, so the opener sits one line past
         the 12-line lookback and must not be folded in.
         */
        const sourceLines = [
          'expect(farAway,)',
          ...Array.from({ length: 12, }, function filler(_unused, i,) {
            return `const filler${String(i,)} = ${String(i,)};`;
          },),
          '  .toBe(0,);',
        ];
        expect(extractAssertionExpression({
          sourceLines,
          lineNumber: 14,
        },),)
          .toBe('.toBe(0,);',);
      },
    },),

    it({
      name: 'returns empty string when the line is out of range',
      fn: async () => {
        expect(extractAssertionExpression({
          sourceLines: ['expect(x,).toBe(1,);',],
          lineNumber: 99,
        },),)
          .toBe('',);
      },
    },),

    //endregion extractAssertionExpression

    //region readAssertionSites

    it({
      name: 'reads the assertion site from the first non-harness frame',
      fn: async () => {
        const path = await writeTempSource({
          lines: [
            'const errorSpyCallCount = 3;',
            'expect(errorSpyCallCount,)',
            '  .toBe(2,);',
          ],
          index: 0,
        },);
        const error = new Error('expected 3 to equal 2',);
        // Harness matcher frame first (must be skipped), then the user frame.
        error.stack = [
          'Error: expected 3 to equal 2',
          '    at toBe (package/test-harness/core/src/expect-matchers-core.ts:70:10)',
          `    at fn (${path}:3:12)`,
        ]
          .join('\n',);
        const sites = await readAssertionSites(error,);
        /**
         Site recovered for the root error, asserted present before reading its fields.
         */
        const site = sites.get(error,);
        expect(site,).toBeDefined();
        expect(site?.expression,).toBe('expect(errorSpyCallCount,) .toBe(2,);',);
        expect(site?.location,).toContain(':3',);
        await removeTempSource(path,);
      },
    },),

    it({
      name: 'parses a file:// URL frame and a frame without a column',
      fn: async () => {
        const path = await writeTempSource({
          lines: [
            'expect(value,)',
            '  .toBe(expected,);',
          ],
          index: 1,
        },);
        /**
         file:// frame carrying only `path:line` (no column), exercising both parse branches at once.
         */
        const error = new Error('boom',);
        error.stack = [
          'Error: boom',
          `    at fn (file://${path}:2)`,
        ]
          .join('\n',);
        const sites = await readAssertionSites(error,);
        expect(sites.get(error,)?.expression,).toBe('expect(value,) .toBe(expected,);',);
        await removeTempSource(path,);
      },
    },),

    it({
      name: 'skips a native frame and resolves the next readable frame',
      fn: async () => {
        const path = await writeTempSource({
          lines: ['expect(ok,).toBe(true,);',],
          index: 2,
        },);
        const error = new Error('boom',);
        error.stack = [
          'Error: boom',
          '    at processTicksAndRejections (node:internal/process/task_queues)',
          `    at fn (${path}:1:1)`,
        ]
          .join('\n',);
        const sites = await readAssertionSites(error,);
        expect(sites.get(error,)?.expression,).toBe('expect(ok,).toBe(true,);',);
        await removeTempSource(path,);
      },
    },),

    it({
      name: 'reads sites for both the error and its cause',
      fn: async () => {
        const path = await writeTempSource({
          lines: [
            'expect(value,)',
            '  .toBe(expected,);',
          ],
          index: 3,
        },);
        const root = new Error('root assertion',);
        root.stack = [
          'Error: root assertion',
          `    at fn (${path}:2:5)`,
        ]
          .join('\n',);
        const outer = new Error('wrapper', { cause: root, },);
        outer.stack = [
          'Error: wrapper',
          `    at fn (${path}:2:5)`,
        ]
          .join('\n',);
        const sites = await readAssertionSites(outer,);
        expect(sites.get(outer,)?.expression,).toBe('expect(value,) .toBe(expected,);',);
        expect(sites.get(root,)?.expression,).toBe('expect(value,) .toBe(expected,);',);
        await removeTempSource(path,);
      },
    },),

    it({
      name: 'yields no site when no frame resolves to a readable file',
      fn: async () => {
        const error = new Error('boom',);
        error.stack = [
          'Error: boom',
          '    at fn (/no/such/path/missing.ts:9:1)',
        ]
          .join('\n',);
        const sites = await readAssertionSites(error,);
        expect(sites.get(error,),).toBeUndefined();
      },
    },),

    //endregion readAssertionSites
  ],
},);
