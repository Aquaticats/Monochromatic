import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  _createLocalStorageStore as createLocalStorageStore,
  _parseLogKey as parseLogKey,
} from '@monochromatic-dev/logger-core';

/**
 One mebibyte of UTF-16 code units; the node cap under test is half the
 measured 5 MiB quota, so multi-mebibyte batches drive eviction.
 */
const MIB = 1_048_576;

/**
 Installs `fake` as `globalThis.localStorage` via the property descriptor
 (plain assignment would call Node's phantom setter-less path on some hosts),
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
 Builds an in-memory `Storage` stand-in with full enumeration support (the
 engine's adoption scan walks `length`/`key`), rejecting a `setItem` once
 stored value lengths would exceed `quotaChars`, throwing the same
 `QuotaExceededError` a real backend raises. Records every `removeItem`
 under `removed` so a test can assert exactly which keys were evicted.
 
 @param quotaChars - Total value length the store accepts before
 overflowing; omitted means unlimited.
 
 @returns Storage stand-in exposing `removed` and the raw `backing` map.
 */
function createFakeStorage(
  { quotaChars, }: { readonly quotaChars?: number; } = {},
): Storage & {
  readonly backing: Map<string, string>;
  readonly removed: string[];
} {
  const backing = new Map<string, string>();
  const removed: string[] = [];
  const used = { chars: 0, };
  return {
    backing,
    removed,
    get length() {
      return backing.size;
    },
    key(slot: number,) {
      return [...backing.keys(),][slot] ?? null;
    },
    clear(): void {
      backing.clear();
      used.chars = 0;
    },
    getItem(key: string,) {
      return backing.get(key,) ?? null;
    },
    setItem(key: string, value: string,): void {
      const priorLength = backing.get(key,)?.length ?? 0;
      const nextChars = (used.chars - priorLength) + value.length;
      if ((quotaChars !== undefined) && (nextChars > quotaChars))
        throw new DOMException('exceeded the quota', 'QuotaExceededError',);
      backing.set(key, value,);
      used.chars = nextChars;
    },
    removeItem(key: string,): void {
      removed.push(key,);
      const priorLength = backing.get(key,)?.length ?? 0;
      if (backing.delete(key,))
        used.chars -= priorLength;
    },
  } as unknown as Storage & {
    readonly backing: Map<string, string>;
    readonly removed: string[];
  };
}

/**
 Builds an in-memory `Storage` stand-in whose first `setItem` succeeds and
 every later one throws a non-quota error, so a test can prove the engine
 does not evict for failures other than a quota overflow. Enumeration is
 supported for the adoption scan; `removeItem` calls land in `removed`.
 
 @returns Storage stand-in exposing the evicted-key log as `removed`.
 */
function createFlakyStorage(): Storage & { readonly removed: string[]; } {
  const backing = new Map<string, string>();
  const removed: string[] = [];
  const calls = { setItem: 0, };
  return {
    removed,
    get length() {
      return backing.size;
    },
    key(slot: number,) {
      return [...backing.keys(),][slot] ?? null;
    },
    getItem(key: string,) {
      return backing.get(key,) ?? null;
    },
    setItem(key: string, value: string,): void {
      calls.setItem += 1;
      if (calls.setItem > 1)
        throw new Error('localStorage disabled mid-session',);
      backing.set(key, value,);
    },
    removeItem(key: string,): void {
      removed.push(key,);
      backing.delete(key,);
    },
  } as unknown as Storage & { readonly removed: string[]; };
}

/**
 Captures `console.warn` output, restoring the real method when the returned
 guard leaves `using` scope, so a test can count the engine's give-up
 reports.
 
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

// Serial because every test swaps the process-global `localStorage`.
await describe({
  name: createLocalStorageStore.name,
  concurrency: 1,
  children: [
    it({
      name: 'persists batches under run-scoped counter keys',
      fn: async () => {
        const fake = createFakeStorage();
        using _storage = installFakeLocalStorage(fake,);
        const store = createLocalStorageStore();

        store.persist('alpha',);
        store.persist('beta',);

        /**
         Parsed identities of every landed key; both must carry one shared run identity.
         */
        const parsed = [...fake.backing.keys(),]
          .flatMap((key,) => {
            const { parsed: identity, } = parseLogKey(key,);
            return (identity === undefined) ? [] : [identity,];
          },);
        expect(parsed,)
          .toHaveLength(2,);
        expect(parsed[1]?.stamp,)
          .toBe(parsed[0]?.stamp,);
        expect(parsed[1]?.nonce,)
          .toBe(parsed[0]?.nonce,);
        expect(parsed.map((identity,) => identity.index,),)
          .toEqual([0, 1,],);
        expect(parsed.flatMap((identity,) => fake.backing.get(identity.key,) ?? [],),)
          .toEqual(['alpha', 'beta',],);
      },
    },),

    it({
      name: 'adopts prior-run entries and evicts them oldest-first under the cap, skipping foreign keys',
      fn: async () => {
        const fake = createFakeStorage();
        // Two dead-run entries totaling 2 MiB, plus two keys the strict parse
        // must protect: a host-application key under the prefix and a
        // malformed run identity.
        fake.setItem('monochromatic.log.1000.aaaa.0', 'x'.repeat(MIB,),);
        fake.setItem('monochromatic.log.2000.bbbb.0', 'x'.repeat(MIB,),);
        fake.setItem('monochromatic.log.legacy-note', 'host data',);
        fake.setItem('monochromatic.log.3000.cccc.nan', 'malformed',);
        using _storage = installFakeLocalStorage(fake,);
        const store = createLocalStorageStore();

        // Adopted 2 MiB plus this 1 MiB batch exceeds the 2.5 MiB node cap,
        // so exactly the oldest dead-run entry must fall.
        store.persist('y'.repeat(MIB,),);

        expect(fake.removed,)
          .toEqual(['monochromatic.log.1000.aaaa.0',],);
        expect(fake.backing.has('monochromatic.log.2000.bbbb.0',),)
          .toBe(true,);
        expect(fake.backing.get('monochromatic.log.legacy-note',),)
          .toBe('host data',);
        expect(fake.backing.get('monochromatic.log.3000.cccc.nan',),)
          .toBe('malformed',);
      },
    },),

    it({
      name: 'evicts its own oldest batch after prior entries are exhausted',
      fn: async () => {
        const fake = createFakeStorage();
        using _storage = installFakeLocalStorage(fake,);
        const store = createLocalStorageStore();

        store.persist('a'.repeat(MIB,),);
        store.persist('b'.repeat(MIB,),);
        // The third mebibyte breaches the 2.5 MiB cap; with no prior-run
        // entries, this run's own index 0 is the oldest thing owned.
        store.persist('c'.repeat(MIB,),);

        expect(fake.removed,)
          .toHaveLength(1,);
        /**
         Identity of the evicted key; it must be this run's slot zero.
         */
        const { parsed: evicted, } = parseLogKey(fake.removed[0] ?? '',);
        expect(evicted?.index,)
          .toBe(0,);
        /**
         Indices still present after eviction, in insertion order.
         */
        const remaining = [...fake.backing.keys(),]
          .flatMap((key,) => {
            const { parsed: identity, } = parseLogKey(key,);
            return (identity === undefined) ? [] : [identity.index,];
          },);
        expect(remaining,)
          .toEqual([1, 2,],);
      },
    },),

    it({
      name: 'reactively evicts on a quota overflow the tally did not predict',
      fn: async () => {
        // Real capacity far below the runtime cap heuristic, as when another
        // tab's writes fill space this engine's tally never saw.
        const fake = createFakeStorage({ quotaChars: 1_500_000, },);
        fake.setItem('monochromatic.log.1000.aaaa.0', 'x'.repeat(500_000,),);
        using _storage = installFakeLocalStorage(fake,);
        const store = createLocalStorageStore();

        // 500k adopted + 1.2M batch fits the 2.5 MiB cap, so no proactive
        // eviction; the fake's 1.5M quota rejects the write, and the retry
        // loop must reclaim the dead-run entry.
        store.persist('z'.repeat(1_200_000,),);

        expect(fake.removed,)
          .toEqual(['monochromatic.log.1000.aaaa.0',],);
        /**
         Landed batch values after the retry; only the new batch remains.
         */
        const values = [...fake.backing.values(),];
        expect(values,)
          .toHaveLength(1,);
        expect(values[0]?.length,)
          .toBe(1_200_000,);
      },
    },),

    it({
      name: 'reports giving up once per episode and re-arms after a landed write',
      fn: async () => {
        const fake = createFakeStorage({ quotaChars: 100, },);
        using _storage = installFakeLocalStorage(fake,);
        using warnSpy = spyConsoleWarn();
        const store = createLocalStorageStore();

        store.persist('a'.repeat(50,),);
        // Evicts the landed batch, still cannot fit, reports once.
        store.persist('b'.repeat(200,),);
        // Same episode: no second report.
        store.persist('c'.repeat(200,),);
        expect(warnSpy.calls,)
          .toHaveLength(1,);
        expect(warnSpy.calls[0],)
          .toContain('localStorage sink record write failed',);

        // A landed write re-arms the report for the next episode.
        store.persist('d'.repeat(30,),);
        store.persist('e'.repeat(200,),);
        expect(warnSpy.calls,)
          .toHaveLength(2,);
      },
    },),

    it({
      name: 'a non-quota failure reports without evicting anything',
      fn: async () => {
        const fake = createFlakyStorage();
        using _storage = installFakeLocalStorage(fake,);
        using warnSpy = spyConsoleWarn();
        const store = createLocalStorageStore();

        store.persist('one',);
        store.persist('two',);

        expect(fake.removed,)
          .toHaveLength(0,);
        expect(warnSpy.calls,)
          .toHaveLength(1,);
      },
    },),
  ],
},);
