import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { createOpfsSink, } from '@monochromatic-dev/logger-core/browser';

/**
 Installs an in-memory origin private file system as `navigator.storage`,
 restoring the previous `navigator` descriptor when the returned guard leaves
 `using` scope.
 
 The fake publishes bytes the way the real backend does: a writable opened
 without `keepExistingData` starts from an empty file, and `close` is what the
 next `getFile()` reads.

 @returns Disposable exposing the fake's files by name.
 */
function installFakeOpfs(): Disposable & { readonly files: Map<string, string>; } {
  const original = Object.getOwnPropertyDescriptor(
    globalThis,
    'navigator',
  );
  const files = new Map<string, string>();

  /**
   Handle for one file of the fake file system.
   
   @param name - File name inside the fake root.
   
   @returns File handle exposing the writable and snapshot operations the sink uses.
   */
  function createFileHandle(name: string,) {
    return {
      createWritable: async function createWritable(options?: { readonly keepExistingData?: boolean; },) {
        if (options?.keepExistingData !== true)
          files.set(name, '',);
        return {
          close: async function close(): Promise<void> {},
          write: async function write(data: string,): Promise<void> {
            files.set(
              name,
              (files.get(name,) ?? '') + data,
            );
          },
        };
      },
      getFile: async function getFile() {
        return {
          text: async function text(): Promise<string> {
            return files.get(name,) ?? '';
          },
        };
      },
    };
  }

  /**
   Fake origin private file system root.
   */
  const root = {
    getFileHandle: async function getFileHandle(
      name: string,
      options?: { readonly create?: boolean; },
    ) {
      if ((!files.has(name,)) && (options?.create !== true))
        throw new DOMException(
          `no entry named ${name}`,
          'NotFoundError',
        );
      if (!files.has(name,))
        files.set(name, '',);
      return createFileHandle(name,);
    },
    removeEntry: async function removeEntry(name: string,): Promise<void> {
      if (!files.delete(name,))
        throw new DOMException(
          `no entry named ${name}`,
          'NotFoundError',
        );
    },
  };

  Object.defineProperty(
    globalThis,
    'navigator',
    {
      configurable: true,
      value: {
        storage: {
          getDirectory: async function getDirectory() {
            return root;
          },
        },
      },
    },
  );
  return {
    files,
    [Symbol.dispose](): void {
      if (original === undefined)
        Reflect.deleteProperty(globalThis, 'navigator',);
      else
        Object.defineProperty(
          globalThis,
          'navigator',
          original,
        );
    },
  };
}

// Node/Bun has no `navigator.storage`, so the absent-backend fallback (a caught
// `getDirectory` throw and drained batches hitting the unset-stream guard) is
// covered here against a fake file system, and the real-backend differences
// live in `opfs.browser.test.ts`. The shared buffering policy is covered in
// `record-buffer.unit.test.ts`. The factory is reached through the `./browser`
// subpath because the root entry no longer exports it.
// Serial because the fake backend replaces the process-global `navigator`.
await describe({
  name: 'OPFS sink',
  concurrency: 1,
  children: [
    it({
      name: 'verify resolves false when OPFS is absent',
      fn: async () => {
        // `navigator.storage.getDirectory()` dereferences an undefined member
        // and throws; verify catches it and reports the backend unavailable.
        const sink = createOpfsSink();
        expect(await sink.verify(),)
          .toBe(false,);
      },
    },),

    it({
      name: 'write buffers without throwing when OPFS is absent',
      fn: async () => {
        // The record buffers; nothing touches the missing backend until a
        // flush trigger fires.
        const sink = createOpfsSink();
        /**
         Resolved write result; the sink write contract is `Promise<void>`.
         */
        const result = await sink.write({
          level: 'info',
          message: 'dropped',
          timestamp: 0,
        },);
        expect(result,)
          .toBeUndefined();
      },
    },),

    it({
      name: 'flush drains the buffer into the unset-stream guard and resolves',
      fn: async () => {
        // A warn record drains synchronously on add, an info record drains on
        // the flush hook; both batches hit the guard and are dropped silently.
        const sink = createOpfsSink();
        await sink.write({
          level: 'warn',
          message: 'urgent but backendless',
          timestamp: 0,
        },);
        await sink.write({
          level: 'info',
          message: 'buffered but backendless',
          timestamp: 1,
        },);
        /**
         Resolved flush result; must settle even with no stream to write to.
         */
        const result = await sink.flush?.();
        expect(result,)
          .toBeUndefined();
      },
    },),

    it({
      name: 'verification leaves only JSONL log records in the file it opens',
      fn: async () => {
        using opfs = installFakeOpfs();
        const sink = createOpfsSink();
        expect(await sink.verify(),)
          .toBe(true,);

        await sink.write({
          level: 'warn',
          message: 'urgent',
          timestamp: 1,
        },);
        await sink.flush?.();

        /**
         Every file the sink left behind, so probe residue is visible too.
         */
        const names = [...opfs.files.keys(),];
        /**
         The one log file this run appends to.
         */
        const logNames = names.filter(function isLogFile(name: string,): boolean {
          return name.endsWith('.log.jsonl',);
        },);
        expect(logNames,)
          .toHaveLength(1,);
        const [logName,] = logNames;
        if (logName === undefined)
          throw new Error('Verified OPFS sink created no log file',);

        /**
         Lines of that log file; an appended probe record would appear here as an extra line.
         */
        const lines = (opfs.files.get(logName,) ?? '').trimEnd()
          .split('\n',);
        expect(lines,)
          .toHaveLength(1,);
        const [line,] = lines;
        if (line === undefined)
          throw new Error('Verified OPFS sink wrote no record',);
        expect(JSON.parse(line,),)
          .toEqual({
            level: 'warn',
            message: 'urgent',
            timestamp: 1,
          },);
        // Verification cleans up its own probe file rather than leaving residue.
        expect(names.filter(function isProbe(name: string,): boolean {
          return name.endsWith('.probe.tmp',);
        },),)
          .toEqual([],);
      },
    },),
  ],
},);
