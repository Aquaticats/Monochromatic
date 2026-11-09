import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  detectNativeAnalysisCrash,
  type NativeAnalysisCrash,
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
 One captured crash report, spelled as the crashing child wrote it to the inherited stream.
 */
const SEGFAULT_REPORT = `${SEGFAULT_HEADER}\n${SEGFAULT_SIGNAL}\n\n${GOROUTINE_DUMP}\n`
  + 'github.com/microsoft/TypeScript/tsc/internal/project/dirty.(*SyncMapEntry[...]).ChangeIf(...)\n'
  + '\tgithub.com/microsoft/TypeScript/tsc/internal/project/dirty/syncmap.go:128 +0x3c\n';

await describe({
  name: detectNativeAnalysisCrash.name,
  children: [
    //region Reports that prove a crash

    it({
      name: 'finds the header of a panic report carrying a goroutine dump',
      fn: async () => {
        const crash: NativeAnalysisCrash | undefined = detectNativeAnalysisCrash({ stderr: SEGFAULT_REPORT, },);
        expect(crash?.headerLine,).toBe(SEGFAULT_HEADER,);
      },
    },),
    it({
      name: 'finds the header of a report carrying only the signal line',
      fn: async () => {
        const crash = detectNativeAnalysisCrash({ stderr: `${SEGFAULT_HEADER}\n${SEGFAULT_SIGNAL}\n`, },);
        expect(crash?.headerLine,).toBe(SEGFAULT_HEADER,);
      },
    },),
    it({
      name: 'finds a Go fatal error, which no recover intercepts',
      fn: async () => {
        const crash = detectNativeAnalysisCrash({
          stderr: `fatal error: concurrent map writes\n\n${GOROUTINE_DUMP}\n`,
        },);
        expect(crash?.headerLine,).toBe('fatal error: concurrent map writes',);
      },
    },),
    it({
      name: 'finds a report that follows ordinary Oxlint output',
      fn: async () => {
        const crash = detectNativeAnalysisCrash({
          stderr: `Found 0 warnings and 2 errors.\n${SEGFAULT_REPORT}`,
        },);
        expect(crash?.headerLine,).toBe(SEGFAULT_HEADER,);
      },
    },),
    it({
      name: 'finds a report whose last line carries no newline',
      fn: async () => {
        const crash = detectNativeAnalysisCrash({ stderr: `${SEGFAULT_HEADER}\n${GOROUTINE_DUMP}`, },);
        expect(crash?.headerLine,).toBe(SEGFAULT_HEADER,);
      },
    },),

    //endregion Reports that prove a crash

    //region Text that proves nothing

    it({
      name: 'ignores a clean run',
      fn: async () => {
        expect(detectNativeAnalysisCrash({ stderr: 'Found 0 warnings and 0 errors.\n', },),).toBeUndefined();
        expect(detectNativeAnalysisCrash({ stderr: '', },),).toBeUndefined();
      },
    },),
    it({
      name: 'ignores a crash header with no runtime report beside it',
      fn: async () => {
        expect(detectNativeAnalysisCrash({ stderr: `${SEGFAULT_HEADER}\n`, },),).toBeUndefined();
      },
    },),
    it({
      name: 'ignores a goroutine dump with no crash header above it',
      fn: async () => {
        expect(detectNativeAnalysisCrash({ stderr: `${GOROUTINE_DUMP}\n`, },),).toBeUndefined();
      },
    },),
    it({
      name: 'ignores a message that merely mentions a panic',
      fn: async () => {
        expect(
          detectNativeAnalysisCrash({ stderr: 'help: a panic: here is not a crash report\n', },),
        ).toBeUndefined();
      },
    },),
    it({
      name: 'ignores its own wording, which names the markers without printing a report',
      fn: async () => {
        expect(
          detectNativeAnalysisCrash({
            stderr: 'panic: runtime error: x is not a crash unless a goroutine line follows\n',
          },),
        ).toBeUndefined();
      },
    },),

    //endregion Text that proves nothing
  ],
},);
