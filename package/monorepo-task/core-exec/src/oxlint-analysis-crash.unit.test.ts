import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  detectNativeAnalysisCrash,
  NO_NATIVE_ANALYSIS_CRASH,
} from '../dist/final/node/testing.mjs';

/**
 Crash header of the TypeScript 7.1 API server panic this detector exists for.
 */
const SEGFAULT_HEADER = 'panic: runtime error: invalid memory address or nil pointer dereference';
/**
 Signal line the Go runtime prints below that header.
 */
const SEGFAULT_SIGNAL = '[signal SIGSEGV: segmentation violation code=0x1 addr=0x8 pc=0xbf31dc]';
/**
 Goroutine dump line that follows both crash shapes.
 */
const GOROUTINE_DUMP = 'goroutine 215 [running]:';
/**
 Go runtime fatal error, which no recover intercepts.
 */
const FATAL_HEADER = 'fatal error: concurrent map writes';

/**
 One captured crash report, spelled as the crashing child wrote it to the inherited stream.
 */
const SEGFAULT_REPORT = `${SEGFAULT_HEADER}\n${SEGFAULT_SIGNAL}\n\n${GOROUTINE_DUMP}\n`
  + 'github.com/microsoft/TypeScript/tsc/internal/project/dirty.(*SyncMapEntry[...]).ChangeIf(...)\n'
  + '\tgithub.com/microsoft/TypeScript/tsc/internal/project/dirty/syncmap.go:128 +0x3c\n';

/**
 Header the detection reported for one captured stream, or the empty string for the sentinel.

 Keeps the assertions free of the sentinel check, so each one reads as the header it expects.

 @param stderr - Standard error to scan

 @returns The crash header, or `''` when no crash was reported

 @example
 ```ts
 expect(headerOf({ stderr: report, },),).toBe(SEGFAULT_HEADER,);
 ```
 */
function headerOf({ stderr }: { readonly stderr: string },): string {
  /**
   What the detector answered for this stream.
   */
  const detection = detectNativeAnalysisCrash({ stderr, },);
  return (detection === NO_NATIVE_ANALYSIS_CRASH) ? '' : detection.headerLine;
}

await describe({
  name: detectNativeAnalysisCrash.name,
  children: [
    //region Reports that prove a crash

    it({
      name: 'finds the header of a panic report carrying a goroutine dump',
      fn: async () => {
        expect(headerOf({ stderr: SEGFAULT_REPORT, },),).toBe(SEGFAULT_HEADER,);
      },
    },),
    it({
      name: 'finds the header of a report carrying only the signal line',
      fn: async () => {
        expect(headerOf({ stderr: `${SEGFAULT_HEADER}\n${SEGFAULT_SIGNAL}\n`, },),).toBe(SEGFAULT_HEADER,);
      },
    },),
    it({
      name: 'finds a Go fatal error, which no recover intercepts',
      fn: async () => {
        expect(headerOf({ stderr: `${FATAL_HEADER}\n\n${GOROUTINE_DUMP}\n`, },),).toBe(FATAL_HEADER,);
      },
    },),
    it({
      name: 'finds a report that follows ordinary Oxlint output',
      fn: async () => {
        expect(headerOf({ stderr: `Found 0 warnings and 2 errors.\n${SEGFAULT_REPORT}`, },),)
          .toBe(SEGFAULT_HEADER,);
      },
    },),
    it({
      name: 'finds a report whose last line carries no newline',
      fn: async () => {
        expect(headerOf({ stderr: `${SEGFAULT_HEADER}\n${GOROUTINE_DUMP}`, },),).toBe(SEGFAULT_HEADER,);
      },
    },),

    //endregion Reports that prove a crash

    //region Text that proves nothing

    it({
      name: 'ignores a clean run',
      fn: async () => {
        expect(detectNativeAnalysisCrash({ stderr: 'Found 0 warnings and 0 errors.\n', },),)
          .toBe(NO_NATIVE_ANALYSIS_CRASH,);
        expect(detectNativeAnalysisCrash({ stderr: '', },),).toBe(NO_NATIVE_ANALYSIS_CRASH,);
      },
    },),
    it({
      name: 'ignores a crash header with no runtime report beside it',
      fn: async () => {
        expect(detectNativeAnalysisCrash({ stderr: `${SEGFAULT_HEADER}\n`, },),)
          .toBe(NO_NATIVE_ANALYSIS_CRASH,);
      },
    },),
    it({
      name: 'ignores a goroutine dump with no crash header above it',
      fn: async () => {
        expect(detectNativeAnalysisCrash({ stderr: `${GOROUTINE_DUMP}\n`, },),)
          .toBe(NO_NATIVE_ANALYSIS_CRASH,);
      },
    },),
    it({
      name: 'ignores a message that merely mentions a panic',
      fn: async () => {
        expect(
          detectNativeAnalysisCrash({ stderr: 'help: a panic: here is not a crash report\n', },),
        ).toBe(NO_NATIVE_ANALYSIS_CRASH,);
      },
    },),
    it({
      name: 'ignores a goroutine line whose number is not a number',
      fn: async () => {
        expect(
          detectNativeAnalysisCrash({ stderr: `${SEGFAULT_HEADER}\ngoroutine many [running]:\n`, },),
        ).toBe(NO_NATIVE_ANALYSIS_CRASH,);
      },
    },),

    //endregion Text that proves nothing
  ],
},);
