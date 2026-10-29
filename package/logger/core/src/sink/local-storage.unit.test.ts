import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  _parseLogKey as parseLogKey,
  sinks,
} from '@monochromatic-dev/logger-core';

/**
 Sink factories under test, read from the built artifact's `sinks` namespace.
 */
const {
  createLocalStorageSink,
} = sinks;

/**
 Installs `fake` as `globalThis.localStorage` via the property descriptor,
 restoring the original descriptor, or removing the property when none
 existed, when the returned guard leaves `using` scope.
 
 @param fake - Storage stand-in to install for the duration of the scope.
 
 @returns Disposable that restores the original `localStorage` on exit.
 */
function installFakeLocalStorage(fake: Storage,): Disposable {
  const original = Object.getOwnPropertyDescriptor(globalThis, 'localStorage',);
  Object.defineProperty(globalThis, 'localStorage', {
    configurable: true,
    value: fake,
  },);
  return {
    [Symbol.dispose](): void {
      if (original === undefined)
        Reflect.deleteProperty(globalThis, 'localStorage',);
      else
        Object.defineProperty(globalThis, 'localStorage', original,);
    },
  };
}

/**
 Installs a minimal fake `globalThis.document` (absent under Node) so the
 sink under test takes the DOM-host verify path instead of the flagless-Node
 short-circuit, restoring or removing the global when the returned guard
 leaves `using` scope.
 
 @returns Disposable that restores the original `document` on exit.
 */
function installFakeDocument(): Disposable {
  const had = 'document' in globalThis;
  const original = globalThis.document;
  globalThis.document = {
    addEventListener(): void {},
    visibilityState: 'visible',
  } as unknown as Document;
  return {
    [Symbol.dispose](): void {
      if (had)
        globalThis.document = original;
      else
        Reflect.deleteProperty(globalThis, 'document',);
    },
  };
}

/**
 Builds an in-memory `Storage` stand-in with enumeration support, exposing
 the raw `backing` map so a test can assert exactly which keys and values
 landed.
 
 @returns Storage stand-in exposing the raw `backing` map.
 */
function createFakeStorage(): Storage & { readonly backing: Map<string, string>; } {
  const backing = new Map<string, string>();
  return {
    backing,
    get length() {
      return backing.size;
    },
    key(slot: number,) {
      return [...backing.keys(),][slot] ?? null;
    },
    clear(): void {
      backing.clear();
    },
    getItem(key: string,) {
      return backing.get(key,) ?? null;
    },
    setItem(key: string, value: string,): void {
      backing.set(key, value,);
    },
    removeItem(key: string,): void {
      backing.delete(key,);
    },
  } as unknown as Storage & { readonly backing: Map<string, string>; };
}

/**
 Captures `console.warn` output, restoring the real method when the returned
 guard leaves `using` scope, so a test can prove the flagless-Node
 short-circuit stays silent.
 
 @returns Disposable exposing captured warn lines as `calls`.
 */
function spyConsoleWarn(): Disposable & { readonly calls: string[]; } {
  const original = console.warn;
  const calls: string[] = [];
  console.warn = (...args: unknown[]): void => {
    calls.push(args.map(String,)
      .join(' ',),);
  };
  return {
    calls,
    [Symbol.dispose](): void {
      console.warn = original;
    },
  };
}

// Serial because tests swap the process-global `localStorage` and `document`,
// and the flagless-Node test depends on both staying absent.
await describe({
  name: createLocalStorageSink.name,
  concurrency: 1,
  children: [
    it({
      name: 'verify resolves false on flagless Node without touching the getter or reporting',
      fn: async () => {
        using warnSpy = spyConsoleWarn();
        const sink = createLocalStorageSink();
        expect(await sink.verify(),)
          .toBe(false,);
        // A silent skip: no internal-error report and, because the getter was
        // never touched, no ExperimentalWarning from Node either.
        expect(warnSpy.calls,)
          .toHaveLength(0,);
      },
    },),

    it({
      name: 'verify round-trips against an installed storage on a DOM host and cleans its probe',
      fn: async () => {
        using _document = installFakeDocument();
        const fake = createFakeStorage();
        using _storage = installFakeLocalStorage(fake,);
        const sink = createLocalStorageSink();

        expect(await sink.verify(),)
          .toBe(true,);
        expect(fake.backing.size,)
          .toBe(0,);
      },
    },),

    it({
      name: 'a warn record lands buffered records as one run-scoped JSONL batch',
      fn: async () => {
        using _document = installFakeDocument();
        const fake = createFakeStorage();
        using _storage = installFakeLocalStorage(fake,);
        const sink = createLocalStorageSink();

        /**
         Routine record that must stay buffered on its own.
         */
        const first = {
          level: 'info' as const,
          message: 'first',
          timestamp: 1,
        };
        /**
         Urgent record whose severity flushes itself and `first` together.
         */
        const second = {
          level: 'warn' as const,
          message: 'second',
          timestamp: 2,
        };

        await sink.write(first,);
        expect(fake.backing.size,)
          .toBe(0,);

        await sink.write(second,);
        expect([...fake.backing.values(),],)
          .toEqual([`${JSON.stringify(first,)}\n${JSON.stringify(second,)}`,],);
        /**
         Identity of the landed key; the first batch of a run takes index zero.
         */
        const { parsed: landed, } = parseLogKey([...fake.backing.keys(),][0] ?? '',);
        expect(landed?.index,)
          .toBe(0,);
      },
    },),

    it({
      name: 'the flush hook drains a buffered routine record synchronously',
      fn: async () => {
        using _document = installFakeDocument();
        const fake = createFakeStorage();
        using _storage = installFakeLocalStorage(fake,);
        const sink = createLocalStorageSink();

        await sink.write({
          level: 'info',
          message: 'buffered until flush',
          timestamp: 3,
        },);
        await sink.flush?.();

        expect([...fake.backing.values(),].join('|',),)
          .toContain('buffered until flush',);
      },
    },),
  ],
},);
