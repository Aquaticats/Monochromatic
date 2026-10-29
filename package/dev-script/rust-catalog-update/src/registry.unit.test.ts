/**
 Unit tests for sparse-index and crates.io parsing and the cached, throttled client.

 @module
 */

import {
  mkdtemp,
  readFile,
  rm,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  createRegistry,
  HISTORY_CACHE_DIR,
  parseIndexPubtimes,
  parseVersionHistory,
  sparseIndexPath,
} from './registry.ts';

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
  const root = await mkdtemp(join(tmpdir(), 'rust-catalog-update-registry-',),);
  return {
    root,
    [Symbol.asyncDispose]: async function removeRoot(): Promise<void> {
      await rm(root, { recursive: true, force: true, },);
    },
  };
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
          root: tmpdir(),
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
