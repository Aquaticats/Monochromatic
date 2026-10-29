import {
  mkdtemp,
  readFile,
  rm,
  utimes,
  writeFile,
} from 'node:fs/promises';
import { tmpdir, } from 'node:os';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/module-test/ts';

import {
  AsnDatabaseError,
  createAsnLookup,
  lookupAsnNetworks,
} from '../dist/final/node/asn-networks.mjs';
import { captureError, } from './test-fixtures.ts';

/**
 Disposable temporary directory fixture.
 */
type TempDir = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Successful HTTP status used by compressed database fixtures.
 */
const HTTP_OK = 200;

/**
 Server-failure HTTP status used by fallback fixture.
 */
const HTTP_SERVER_ERROR = 500;

/**
 Creates disposable ASN cache directory.
 
 @returns Directory removed recursively on disposal.
 
 @example
 ```ts
 await using directory = await makeTempDir();
 ```
 */
async function makeTempDir(): Promise<TempDir> {
  /**
   Fresh operating-system temporary path.
   */
  const path = await mkdtemp(join(
    tmpdir(),
    'wg-allowedips-asn-',
  ),);
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(
        path,
        {
          force: true,
          recursive: true,
        },
      );
    },
  };
}

/**
 Builds gzip-compressed IPinfo response fixture.
 
 @param text - Complete NDJSON text before compression.
 
 @param status - HTTP status.
 
 @returns Fetch-compatible response with compressed stream body.
 
 @example
 ```ts
 compressedResponse({ text: '', status: HTTP_OK });
 ```
 */
function compressedResponse(
  {
    text,
    status,
  }: {
    readonly text: string;
    readonly status: number;
  },
): Response {
  /**
   Gzip stream matching IPinfo Lite transport.
   */
  const body = new Blob([new TextEncoder().encode(text),],)
    .stream()
    .pipeThrough(new CompressionStream('gzip',),);
  return new Response(
    body,
    { status, },
  );
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: lookupAsnNetworks.name,
      children: [
        it({
          name: 'uses fresh validated cache without network access',
          fn: async ({ sinon, }) => {
            await using directory = await makeTempDir();
            await writeFile(
              join(directory.path, 'cache_AS64500.txt',),
              '192.0.2.0/24,2001:db8::1',
            );
            /**
             Fetch spy that must remain unused for fresh cache.
             */
            const fetchSpy = sinon.stub(globalThis, 'fetch',);
            /**
             Networks read from current cache.
             */
            const networks = await lookupAsnNetworks({
              asn: 'as64500',
              cacheDirectory: directory.path,
              token: '',
            },);
            expect(networks,).toEqual([
              '192.0.2.0/24',
              '2001:db8::1',
            ],);
            expect(fetchSpy,).not.toHaveBeenCalled();
          },
        },),
        it({
          name: 'streams matching records and atomically writes cache',
          fn: async ({ sinon, }) => {
            await using directory = await makeTempDir();
            /**
             Database records with target,
             unrelated,
             and unterminated final target lines.
             */
            const databaseText = [
              JSON.stringify({ asn: 'AS64500', network: '192.0.2.0/24', },),
              JSON.stringify({ asn: 'AS64501', network: '198.51.100.0/24', },),
              JSON.stringify({ asn: 'AS64500', network: '2001:db8::1', },),
            ].join('\n',);
            sinon
              .stub(globalThis, 'fetch',)
              .resolves(compressedResponse({
                text: databaseText,
                status: HTTP_OK,
              },),);
            /**
             Networks filtered from streamed fixture.
             */
            const networks = await lookupAsnNetworks({
              asn: 'AS64500',
              cacheDirectory: directory.path,
              token: 'fixture-token',
            },);
            expect(networks,).toEqual([
              '192.0.2.0/24',
              '2001:db8::1',
            ],);
            expect(await readFile(
              join(directory.path, 'cache_AS64500.txt',),
              'utf8',
            ),).toBe('192.0.2.0/24,2001:db8::1',);
          },
        },),
        it({
          name: 'falls back to valid stale cache after refresh failure',
          fn: async ({ sinon, }) => {
            await using directory = await makeTempDir();
            /**
             Existing cache path made old enough to require refresh.
             */
            const cachePath = join(directory.path, 'cache_AS64500.txt',);
            await writeFile(
              cachePath,
              '192.0.2.0/24',
            );
            await utimes(
              cachePath,
              new Date(0,),
              new Date(0,),
            );
            sinon
              .stub(globalThis, 'fetch',)
              .resolves(compressedResponse({
                text: '',
                status: HTTP_SERVER_ERROR,
              },),);
            /**
             Networks recovered from stale cache.
             */
            const networks = await lookupAsnNetworks({
              asn: 'AS64500',
              cacheDirectory: directory.path,
              token: 'fixture-token',
            },);
            expect(networks,).toEqual(['192.0.2.0/24',],);
          },
        },),
        it({
          name: 'wraps refresh failure when no stale cache exists',
          fn: async () => {
            await using directory = await makeTempDir();
            /**
             Missing-token failure captured after absent-cache lookup.
             */
            const error = await captureError({
              operation: async function lookupWithoutData(): Promise<readonly string[]> {
                return await lookupAsnNetworks({
                  asn: 'AS64500',
                  cacheDirectory: directory.path,
                  token: '',
                },);
              },
            },);
            expect(error,).toBeInstanceOf(AsnDatabaseError,);
            expect(String(error,),).toContain('no cached fallback is available',);
          },
        },),
        it({
          name: 'rejects malformed target record without poisoning cache',
          fn: async ({ sinon, }) => {
            await using directory = await makeTempDir();
            sinon
              .stub(globalThis, 'fetch',)
              .resolves(compressedResponse({
                text: `${JSON.stringify({ asn: 'AS64500', },)}\n`,
                status: HTTP_OK,
              },),);
            /**
             Wrapped malformed-record failure.
             */
            const error = await captureError({
              operation: async function lookupMalformedRecord(): Promise<readonly string[]> {
                return await lookupAsnNetworks({
                  asn: 'AS64500',
                  cacheDirectory: directory.path,
                  token: 'fixture-token',
                },);
              },
            },);
            expect(error,).toBeInstanceOf(AsnDatabaseError,);
            expect((error as Error).cause,).toBeInstanceOf(AsnDatabaseError,);
          },
        },),
      ],
    },),
    describe({
      name: createAsnLookup.name,
      children: [
        it({
          name: 'coalesces concurrent duplicate ASN lookups',
          fn: async ({ sinon, }) => {
            await using directory = await makeTempDir();
            /**
             Fetch stub resolving one reusable database response.
             */
            const fetchSpy = sinon
              .stub(globalThis, 'fetch',)
              .resolves(compressedResponse({
                text: `${JSON.stringify({ asn: 'AS64500', network: '192.0.2.0/24', },)}\n`,
                status: HTTP_OK,
              },),);
            /**
             Invocation-scoped resolver sharing pending work.
             */
            const lookup = createAsnLookup({
              cacheDirectory: directory.path,
              token: 'fixture-token',
            },);
            /**
             Duplicate lookups started before either settles.
             */
            const [first, second,] = await Promise.all([
              lookup({ asn: 'AS64500', },),
              lookup({ asn: 'AS64500', },),
            ],);
            expect(first,).toEqual(['192.0.2.0/24',],);
            expect(second,).toEqual(first,);
            expect(fetchSpy,).toHaveBeenCalledTimes(1,);
          },
        },),
      ],
    },),
  ],
},);
