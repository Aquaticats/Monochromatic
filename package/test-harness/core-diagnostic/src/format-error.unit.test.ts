/**
 Tests for `formatErrorDeep` and `formatFailure` from
 `./format-error.ts`. Covers the dispatch-machinery stack-frame
 filter (harness bundle plus the vendored chai / sinon assertion
 stack) and the chain-walk branches (cause, cycle, non-Error throws).
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  formatErrorDeep,
  formatFailure,
} from '@monochromatic-dev/test-harness-core-diagnostic';

/**
 Builds an Error whose `.stack` is exactly the lines passed in.
 A synthetic header line (`Error: <message>`) is prepended so the
 formatter's first-line-duplicate-skip branch fires as it does in
 real V8 / JavaScriptCore stacks.
 
 @param message - human-readable message used for both the Error
   constructor and the synthetic header line
 
 @param frames - already-formatted frame lines such as
   `at fn (path/file.ts:9:19)`; rendered with the four-space
   indent V8 emits
 
 @returns Error with deterministic `.stack`
 
 @example
 ```ts
 const e = makeError({
   message: 'boom',
   frames: ['at userFn (package/foo/src/bar.ts:9:19)',],
 });
 ```
 */
function makeError({
  message,
  frames,
}: {
  readonly message: string;
  readonly frames: readonly string[];
},): Error {
  const error = new Error(message,);
  error.stack = [
    `Error: ${message}`,
    ...frames.map(function indentFrame(frame,) {
      return `    ${frame}`;
    },),
  ]
    .join('\n',);
  return error;
}

/**
 Marker frame embedded in every fixture so each test can confirm
 user code survives the filter while assertion-stack frames are
 dropped.
 */
const USER_FRAME = 'at userFn (package/foo/src/bar.ts:9:19)';

await describe({
  name: 'format-error',
  children: [
    //region Vendored chai stack culling

    it({
      name: 'drops chai frames in pnpm virtual store layout',
      fn: async () => {
        const error = makeError({
          message: 'expected x to equal y',
          frames: [
            USER_FRAME,
            'at assertEqual (node_modules/.pnpm/chai@6.2.2/node_modules/chai/index.js:2250:10)',
            'at methodWrapper (node_modules/.pnpm/chai@6.2.2/node_modules/chai/index.js:1700:25)',
          ],
        },);
        const joined = (await formatErrorDeep(error,)).join('\n',);
        expect(joined,).not.toContain('chai/index.js',);
        expect(joined,).toContain('package/foo/src/bar.ts',);
      },
    },),

    it({
      name: 'drops chai frames in flat node_modules layout',
      fn: async () => {
        const error = makeError({
          message: 'boom',
          frames: [
            USER_FRAME,
            'at assertEqual (node_modules/chai/index.js:2250:10)',
          ],
        },);
        const joined = (await formatErrorDeep(error,)).join('\n',);
        expect(joined,).not.toContain('chai/index.js',);
        expect(joined,).toContain('package/foo/src/bar.ts',);
      },
    },),

    it({
      name: 'drops chai-as-promised frames in both layouts',
      fn: async () => {
        const error = makeError({
          message: 'boom',
          frames: [
            USER_FRAME,
            'at thenWrapper (node_modules/.pnpm/chai-as-promised@8.0.2_chai@6.2.2/node_modules/chai-as-promised/lib/chai-as-promised.js:340:16)',
            'at thenWrapper (node_modules/chai-as-promised/lib/chai-as-promised.js:340:16)',
          ],
        },);
        const joined = (await formatErrorDeep(error,)).join('\n',);
        expect(joined,).not.toContain('chai-as-promised',);
        expect(joined,).toContain('package/foo/src/bar.ts',);
      },
    },),

    it({
      name: 'bare chai/ fragment does not match chai-as-promised paths',
      fn: async () => {
        /**
         Sanity check that the four fragments do not cross-match.
         A chai-as-promised frame that is not also matched by the
         `node_modules/chai-as-promised/` fragment would survive.
         Here both fragments would match; the assertion proves the
         specific substring `node_modules/chai/` (with literal slash
         after `chai`) is absent from chai-as-promised paths.
         */
        const path =
          'node_modules/.pnpm/chai-as-promised@8.0.2_chai@6.2.2/node_modules/chai-as-promised/lib/chai-as-promised.js:340:16';
        expect(path.includes('node_modules/chai/',),).toBe(false,);
        expect(path.includes('node_modules/chai-as-promised/',),).toBe(true,);
      },
    },),

    it({
      name: 'drops sinon-chai frames in both layouts',
      fn: async () => {
        const error = makeError({
          message: 'boom',
          frames: [
            USER_FRAME,
            'at sinonChaiCalled (node_modules/.pnpm/sinon-chai@4.0.1_chai@6.2.2_sinon@21.0.3/node_modules/sinon-chai/lib/sinon-chai.js:120:10)',
            'at sinonChaiCalled (node_modules/sinon-chai/lib/sinon-chai.js:120:10)',
          ],
        },);
        const joined = (await formatErrorDeep(error,)).join('\n',);
        expect(joined,).not.toContain('sinon-chai',);
        expect(joined,).toContain('package/foo/src/bar.ts',);
      },
    },),

    it({
      name: 'drops sinon frames in both layouts',
      fn: async () => {
        const error = makeError({
          message: 'boom',
          frames: [
            USER_FRAME,
            'at fakeCall (node_modules/.pnpm/sinon@21.0.3/node_modules/sinon/lib/sinon/proxy.js:42:10)',
            'at fakeCall (node_modules/sinon/lib/sinon/proxy.js:42:10)',
          ],
        },);
        const joined = (await formatErrorDeep(error,)).join('\n',);
        expect(joined,).not.toContain('node_modules/sinon/',);
        expect(joined,).toContain('package/foo/src/bar.ts',);
      },
    },),

    it({
      name: 'bare sinon/ fragment does not match sinon-chai paths',
      fn: async () => {
        const path = 'node_modules/sinon-chai/lib/sinon-chai.js:120:10';
        expect(path.includes('node_modules/sinon/',),).toBe(false,);
        expect(path.includes('node_modules/sinon-chai/',),).toBe(true,);
      },
    },),

    //endregion Vendored chai stack culling

    //region Harness bundle filter still applies

    it({
      name: 'drops harness bundle frames in workspace and node_modules forms',
      fn: async () => {
        const error = makeError({
          message: 'boom',
          frames: [
            USER_FRAME,
            'at runFnOnce (package/test-harness/core/dist/final/neutral/index.mjs:120:10)',
            'at expectImpl (test-harness-core/dist/final/neutral/index.mjs:200:5)',
          ],
        },);
        const joined = (await formatErrorDeep(error,)).join('\n',);
        expect(joined,).not.toContain('test-harness/core/dist',);
        expect(joined,).not.toContain('test-harness-core/dist',);
        expect(joined,).toContain('package/foo/src/bar.ts',);
      },
    },),

    //endregion Harness bundle filter still applies

    //region Harness source-export filter

    it({
      name: 'drops harness source dispatch frames in workspace and node_modules forms',
      fn: async () => {
        const error = makeError({
          message: 'expected 3 to equal 2',
          frames: [
            'at toBe (package/test-harness/core/src/expect-matchers-core.ts:70:10)',
            USER_FRAME,
            'at runFnOnce (package/test-harness/core/src/it.ts:113:19)',
            'at expectImpl (test-harness-core/src/expect.ts:339:5)',
          ],
        },);
        const joined = (await formatErrorDeep(error,)).join('\n',);
        // The matcher / runner frames are now the FIRST things after the
        // message in real runs; dropping them makes the user frame lead.
        expect(joined,).not.toContain('expect-matchers-core.ts',);
        expect(joined,).not.toContain('test-harness/core/src/it.ts',);
        expect(joined,).not.toContain('test-harness-core/src/expect.ts',);
        expect(joined,).toContain('package/foo/src/bar.ts',);
      },
    },),

    it({
      name: 'preserves the harness own test frames under test-harness/core/src',
      fn: async () => {
        const error = makeError({
          message: 'boom',
          frames: [
            'at fn (package/test-harness/core/src/format-error.unit.test.ts:42:7)',
            'at runFnOnce (package/test-harness/core/src/it.ts:113:19)',
          ],
        },);
        const joined = (await formatErrorDeep(error,)).join('\n',);
        // The harness tests itself; their *.test.ts frames are user code
        // and must survive even though they share the src/ prefix.
        expect(joined,).toContain('format-error.unit.test.ts',);
        expect(joined,).not.toContain('test-harness/core/src/it.ts',);
      },
    },),

    it({
      name: 'drops p-limit dispatch frames',
      fn: async () => {
        const error = makeError({
          message: 'boom',
          frames: [
            USER_FRAME,
            'at run (node_modules/.pnpm/p-limit@7.3.0/node_modules/p-limit/index.js:34:31)',
            'at generator (node_modules/p-limit/index.js:34:54)',
          ],
        },);
        const joined = (await formatErrorDeep(error,)).join('\n',);
        expect(joined,).not.toContain('p-limit',);
        expect(joined,).toContain('package/foo/src/bar.ts',);
      },
    },),

    //endregion Harness source-export filter

    //region User frames preserved

    it({
      name: 'preserves frames that do not match any fragment',
      fn: async () => {
        const error = makeError({
          message: 'boom',
          frames: [
            'at firstUserFn (package/foo/src/a.ts:9:19)',
            'at secondUserFn (package/bar/src/b.ts:33:7)',
          ],
        },);
        const joined = (await formatErrorDeep(error,)).join('\n',);
        expect(joined,).toContain('package/foo/src/a.ts',);
        expect(joined,).toContain('package/bar/src/b.ts',);
      },
    },),

    //endregion User frames preserved

    //region Cause chain

    it({
      name: 'walks the cause chain on continuation lines',
      fn: async () => {
        const root = makeError({
          message: 'root',
          frames: ['at rootFn (package/foo/src/root.ts:1:1)',],
        },);
        const outer = makeError({
          message: 'outer',
          frames: ['at outerFn (package/foo/src/outer.ts:2:2)',],
        },);
        (outer as Error & { cause?: unknown; }).cause = root;
        const lines = await formatErrorDeep(outer,);
        expect(lines,).toHaveLength(2,);
        expect(lines[0],).toContain('Error: outer',);
        expect(lines[1],).toContain('Caused by: Error: root',);
      },
    },),

    //endregion Cause chain

    //region Cycle detection

    it({
      name: 'emits cycle marker for self-referential cause',
      fn: async () => {
        const error = makeError({
          message: 'self-loop',
          frames: [USER_FRAME,],
        },);
        (error as Error & { cause?: unknown; }).cause = error;
        const joined = (await formatErrorDeep(error,)).join('\n',);
        expect(joined,).toContain('... (cycle)',);
      },
    },),

    //endregion Cycle detection

    //region Non-Error throws

    it({
      name: 'renders string throw as Threw non-Error value',
      fn: async () => {
        const lines = await formatErrorDeep('oops',);
        expect(lines,).toHaveLength(1,);
        expect(lines[0],).toContain('Threw non-Error value: oops',);
      },
    },),

    it({
      name: 'renders number throw as Threw non-Error value',
      fn: async () => {
        const lines = await formatErrorDeep(42,);
        expect(lines,).toHaveLength(1,);
        expect(lines[0],).toContain('Threw non-Error value: 42',);
      },
    },),

    //endregion Non-Error throws

    //region formatFailure summary fusion

    it({
      name: 'fuses the failure summary with first error line',
      fn: async () => {
        const error = makeError({
          message: 'boom',
          frames: [USER_FRAME,],
        },);
        const result = await formatFailure({
          summary: 'FAIL (5ms)',
          value: error,
        },);
        const [firstLine,] = result.split('\n',);
        expect(firstLine,).toContain('FAIL (5ms)',);
        expect(firstLine,).toContain('Error: boom',);
        expect(firstLine,).toContain('package/foo/src/bar.ts',);
      },
    },),
    //endregion formatFailure summary fusion
  ],
},);
