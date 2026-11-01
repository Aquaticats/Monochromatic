/**
 Unit tests for sparse-index and crates.io parsing and the cached, throttled client.

 @module
 */

import {
  mkdtemp,
  readFile,
  rm,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  createRegistry,
  HISTORY_CACHE_DIR,
  parseIndexPubtimes,
  parseOwnerIds,
  parseVersionHistory,
  sparseIndexPath,
} from './registry.ts';
import { RegistryError, } from './http.ts';

/**
 Creates a disposable repository root.

 @returns root that `await using` removes

 @example
 ```ts
 await using repo = await tempRoot();
 ```
 */
async function tempRoot(): Promise<AsyncDisposable & { readonly root: string; }> {
  /**
   Root directory.
   */
  const root = await mkdtemp(join(scratchRoot(), 'rust-catalog-update-registry-',),);
  return {
    root,
    [Symbol.asyncDispose]: async function removeRoot(): Promise<void> {
      await rm(root, { recursive: true, force: true, },);
    },
  };
}

/**
 Dump fetcher for tests that must not read the dump.

 @returns never

 @throws Error always

 @example
 ```ts
 createRegistry({ root, fetchText, fetchDump: unusedDump, });
 ```
 */
async function unusedDump(): Promise<AsyncIterable<Uint8Array>> {
  throw new Error('this test must not read the database dump');
}

/**
 Yields an empty tar archive: the two zero blocks that end every archive.

 @returns archive bytes

 @example
 ```ts
 endOfArchive();
 ```
 */
async function* endOfArchive(): AsyncGenerator<Uint8Array> {
  yield new Uint8Array(1_024,);
}

/**
 crates.io versions body with a token publish and a trusted publish.
 */
const VERSIONS_BODY = JSON.stringify({
  versions: [
    { num: '1.0.1', created_at: '2026-02-01T00:00:00Z', published_by: null, trustpub_data: { provider: 'github', repository: 'o/r', }, repository: 'https://github.com/o/r', },
    { num: '1.0.0', created_at: '2026-01-01T00:00:00Z', published_by: { id: 3_618, login: 'x', }, trustpub_data: null, repository: null, },
  ],
  meta: { total: 2, next_page: null, },
},);

await describe({
  name: sparseIndexPath.name,
  children: [
    it({
      name: 'follows the index layout for one-, two-, three-, and longer names, lowercased',
      fn: async () => {
        expect(sparseIndexPath('a',),).toBe('1/a',);
        expect(sparseIndexPath('cc',),).toBe('2/cc',);
        expect(sparseIndexPath('syn',),).toBe('3/s/syn',);
        expect(sparseIndexPath('Anyhow',),).toBe('an/yh/anyhow',);
      },
    },),
  ],
},);

await describe({
  name: parseIndexPubtimes.name,
  children: [
    it({
      name: 'maps each version to its pubtime, empty when the line has none, skipping blank lines',
      fn: async () => {
        /**
         Parsed times.
         */
        const times = parseIndexPubtimes('{"vers":"1.0.0"}\n\n{"vers":"1.0.18","pubtime":"2026-03-20T00:12:37Z"}\n',);
        expect([...times,],).toEqual([['1.0.0', '',], ['1.0.18', '2026-03-20T00:12:37Z',],],);
      },
    },),
  ],
},);

await describe({
  name: parseVersionHistory.name,
  children: [
    it({
      name: 'keeps the publisher id, trusted-publishing repository, and declared repository',
      fn: async () => {
        expect(parseVersionHistory(VERSIONS_BODY,),).toEqual([
          { num: '1.0.1', createdAt: '2026-02-01T00:00:00Z', publisherId: '', trustpubRepository: 'o/r', repository: 'https://github.com/o/r', },
          { num: '1.0.0', createdAt: '2026-01-01T00:00:00Z', publisherId: '3618', trustpubRepository: '', repository: '', },
        ],);
      },
    },),
  ],
},);

await describe({
  name: createRegistry.name,
  children: [
    it({
      name: 'caches histories, refetches only when a needed version is missing, and spaces API requests',
      fn: async () => {
        /**
         Disposable repository root.
         */
        await using repo = await tempRoot();
        /**
         Repository root, removed when the test ends.
         */
        const { root, } = repo;
        /**
         Requested URLs with their times.
         */
        const requests: { readonly url: string; readonly at: number; }[] = [];
        /**
         Registry over a fake fetcher.
         */
        const registry = createRegistry({
          fetchDump: unusedDump,
          root,
          minIntervalMs: 50,
          fetchText: async function fakeFetch(url: string,): Promise<string> {
            requests.push({ url, at: Date.now(), },);
            return VERSIONS_BODY;
          },
        },);
        await registry.history({ name: 'a', versions: ['1.0.0',], },);
        await registry.history({ name: 'a', versions: ['1.0.1',], },);
        expect(requests.length,).toBe(1,);
        expect(await readFile(join(root, HISTORY_CACHE_DIR, 'a.json',), 'utf8',),).toContain('"trustpubRepository":"o/r"',);
        await registry.history({ name: 'a', versions: ['2.0.0',], },);
        expect(requests.length,).toBe(2,);
        expect((requests[1]?.at ?? 0) - (requests[0]?.at ?? 0),).toBeGreaterThanOrEqual(45,);
        expect(requests[0]?.url,).toBe('https://crates.io/api/v1/crates/a/versions',);
      },
    },),
    it({
      name: 'reads publish times from the sparse index path',
      fn: async () => {
        /**
         Requested URLs.
         */
        const urls: string[] = [];
        /**
         Registry over a fake fetcher.
         */
        const registry = createRegistry({
          fetchDump: unusedDump,
          root: scratchRoot(),
          fetchText: async function fakeFetch(url: string,): Promise<string> {
            urls.push(url,);
            return '{"vers":"1.0.18","pubtime":"2026-03-20T00:12:37Z"}\n';
          },
        },);
        expect((await registry.pubtimes('itoa',)).get('1.0.18',),).toBe('2026-03-20T00:12:37Z',);
        expect(urls,).toEqual(['https://index.crates.io/it/oa/itoa',],);
      },
    },),
  ],
},);

await describe({
  name: parseOwnerIds.name,
  children: [
    it({
      name: 'keeps individual owner ids and leaves out teams, whose members the API hides',
      fn: async () => {
        expect([...parseOwnerIds('{"users":[{"id":3618,"login":"a","kind":"user"},{"id":9,"login":"github:o:t","kind":"team"},{"login":"no-id","kind":"user"}]}',),],).toEqual(['3618',],);
        expect([...parseOwnerIds('{}',),],).toEqual([],);
      },
    },),
  ],
},);

await describe({
  name: `${createRegistry.name} rate limit and dump prefetch`,
  children: [
    it({
      name: 'retries an HTTP 429 after Retry-After or the backoff, and gives up after five attempts',
      fn: async () => {
        await using repo = await tempRoot();
        /**
         Repository root.
         */
        const { root, } = repo;
        /**
         Answers per call: two 429s, then the body.
         */
        const answers = { calls: 0, };
        /**
         Registry over a flaky fetcher.
         */
        const flaky = createRegistry({
          root,
          fetchDump: unusedDump,
          minIntervalMs: 1,
          rateLimitBackoffMs: 1,
          fetchText: async function fakeFetch(url: string,): Promise<string> {
            answers.calls += 1;
            if (answers.calls <= 2)
              throw new RegistryError({ url, status: 429, },);
            return VERSIONS_BODY;
          },
        },);
        expect((await flaky.history({ name: 'a', versions: ['1.0.0',], },)).length,).toBe(2,);
        expect(answers.calls,).toBe(3,);
        /**
         Registry over a fetcher that always answers 429.
         */
        const limited = createRegistry({
          root,
          fetchDump: unusedDump,
          minIntervalMs: 1,
          rateLimitBackoffMs: 1,
          fetchText: async function fakeFetch(url: string,): Promise<string> {
            throw new RegistryError({ url, status: 429, },);
          },
        },);
        await expect(limited.history({ name: 'b', versions: ['1.0.0',], },),).rejects.toThrow(RegistryError,);
      },
    },),
    it({
      name: 'does not retry other HTTP errors',
      fn: async () => {
        await using repo = await tempRoot();
        /**
         Calls made.
         */
        const calls = { count: 0, };
        /**
         Registry over a fetcher that answers 503.
         */
        const failing = createRegistry({
          root: repo.root,
          fetchDump: unusedDump,
          minIntervalMs: 1,
          fetchText: async function fakeFetch(url: string,): Promise<string> {
            calls.count += 1;
            throw new RegistryError({ url, status: 503, },);
          },
        },);
        await expect(failing.history({ name: 'a', versions: ['1.0.0',], },),).rejects.toThrow(RegistryError,);
        expect(calls.count,).toBe(1,);
      },
    },),
    it({
      name: 'reads the dump only when at least the breakeven number of crates lack a cache file',
      fn: async () => {
        await using repo = await tempRoot();
        /**
         Repository root.
         */
        const { root, } = repo;
        /**
         Dump reads made.
         */
        const reads = { count: 0, };
        /**
         Registry whose dump is an empty archive (two zero blocks).
         */
        const registry = createRegistry({
          root,
          fetchText: async function unused(): Promise<string> {
            throw new Error('no API call expected',);
          },
          dumpBreakevenCrates: 2,
          fetchDump: async function emptyDump(): Promise<AsyncIterable<Uint8Array>> {
            reads.count += 1;
            return endOfArchive();
          },
        },);
        await registry.prefetch(new Set(['only-one',],),);
        expect(reads.count,).toBe(0,);
        await expect(
          registry.prefetch(new Set(['a', 'b',],),),
        ).rejects.toThrow('ended before versions.csv',);
        expect(reads.count,).toBe(1,);
      },
    },),
  ],
},);
