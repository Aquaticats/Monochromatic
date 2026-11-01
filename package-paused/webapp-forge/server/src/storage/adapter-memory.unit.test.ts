/**
 * Unit tests for the in-memory storage adapter.
 *
 * Round-trip every method, plus list-prefix filtering and the
 * concurrent-put atomicity claim.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

import { createMemoryStorage, } from './adapter-memory.ts';

const encoder = new TextEncoder();

await describe({
  name: '',
  children: [
    describe({
      name: createMemoryStorage.name,
      children: [
        it({
          name: 'put then get round-trips a value',
          async fn() {
            const storage = createMemoryStorage();
            await storage.put(
              'a',
              encoder.encode('hello',),
            );
            const value = await storage.get('a',);
            expect(value,).toBeDefined();
            expect(new TextDecoder().decode(value,),).toBe('hello',);
          },
        },),
        it({
          name: 'get returns undefined for unknown keys',
          async fn() {
            const storage = createMemoryStorage();
            const value = await storage.get('missing',);
            expect(value,).toBeUndefined();
          },
        },),
        it({
          name: 'putBatch writes every item',
          async fn() {
            const storage = createMemoryStorage();
            await storage.putBatch([
              {
                key: 'a',
                body: encoder.encode('1',),
              },
              {
                key: 'b',
                body: encoder.encode('2',),
              },
            ],);
            const a = await storage.get('a',);
            const b = await storage.get('b',);
            expect(new TextDecoder().decode(a,),).toBe('1',);
            expect(new TextDecoder().decode(b,),).toBe('2',);
          },
        },),
        it({
          name: 'delete is idempotent',
          async fn() {
            const storage = createMemoryStorage();
            await storage.put(
              'a',
              encoder.encode('x',),
            );
            await storage.delete('a',);
            await storage.delete('a',);
            const value = await storage.get('a',);
            expect(value,).toBeUndefined();
          },
        },),
        it({
          name: 'list returns keys in sorted order, prefix-filtered',
          async fn() {
            const storage = createMemoryStorage();
            await storage.put(
              'b',
              encoder.encode('',),
            );
            await storage.put(
              'a/x',
              encoder.encode('',),
            );
            await storage.put(
              'a/y',
              encoder.encode('',),
            );
            await storage.put(
              'c',
              encoder.encode('',),
            );
            expect(await storage.list('a/',),).toEqual([
              'a/x',
              'a/y',
            ],);
            expect(await storage.list('',),).toEqual([
              'a/x',
              'a/y',
              'b',
              'c',
            ],);
          },
        },),
        it({
          name: 'concurrent put leaves last-writer-wins, no torn read',
          async fn() {
            const storage = createMemoryStorage();
            const writers = Array.from(
              { length: 100, },
              function makeWriter(_unused, index,) {
                return storage.put(
                  'k',
                  encoder.encode(String(index,),),
                );
              },
            );
            await Promise.all(writers,);
            const value = await storage.get('k',);
            expect(value,).toBeDefined();
            const decoded = new TextDecoder().decode(value,);
            const parsed = Number.parseInt(
              decoded,
              10,
            );
            expect((parsed >= 0) && (parsed < 100),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
