import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import {
  createServer,
  type Server,
} from 'node:net';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';

import {
  addressText,
  callVarlink,
  createTunnelHostLookup,
  ERROR_NO_NAME_SERVERS,
  ERROR_NO_SUCH_RECORD,
  parseRecordReply,
  readInterfaceIndex,
  RESOLVE_FLAGS,
  RESOLVE_RECORD_METHOD,
  resolveRecordRequest,
  ttlOfRawRecord,
} from '../dist/final/node/tunnel-dns-lookup.mjs';

/**
 Builds a wire-format resource record for an address.

 @param bytes - Address bytes.

 @param ttl - TTL in seconds.

 @param compressed - Whether the owner name is a compression pointer.

 @returns Base64 of the record.

 @example
 ```ts
 wireRecord({ bytes: [1, 2, 3, 4], ttl: 300, compressed: false });
 ```
 */
function wireRecord(
  {
    bytes,
    ttl,
    compressed,
  }: {
    readonly bytes: readonly number[];
    readonly ttl: number;
    readonly compressed: boolean;
  },
): string {
  /**
   Owner name:
   two labels or a pointer.
   */
  const name = compressed
    ? Buffer.from([0xC0, 0x0C,],)
    : Buffer.from([3, ...Buffer.from('foo',), 3, ...Buffer.from('com',), 0,],);
  /**
   Type, class, TTL and data length.
   */
  const fixed = Buffer.alloc(10,);
  fixed.writeUInt16BE(bytes.length === 4 ? 1 : 28, 0,);
  fixed.writeUInt16BE(1, 2,);
  fixed.writeUInt32BE(ttl, 4,);
  fixed.writeUInt16BE(bytes.length, 8,);
  return Buffer.concat([name, fixed, Buffer.from(bytes,),],).toString('base64',);
}

/**
 Builds a successful reply for the given addresses.

 @param records - Address bytes and TTL per record.

 @returns Reply JSON text.

 @example
 ```ts
 reply({ records: [{ bytes: [1, 2, 3, 4], ttl: 60 }] });
 ```
 */
function reply(
  { records, }: { readonly records: readonly { readonly bytes: readonly number[]; readonly ttl: number; }[]; },
): string {
  return JSON.stringify({
    parameters: {
      rrs: records.map(function toRecord(record,) {
        return {
          rr: { key: { type: record.bytes.length === 4 ? 1 : 28, }, address: record.bytes, },
          raw: wireRecord({ bytes: record.bytes, ttl: record.ttl, compressed: false, },),
        };
      },),
      flags: 8_650_753,
    },
  },);
}

/**
 Starts a fake varlink server that answers every request with `answer`.

 @param path - Socket path.

 @param answer - Function from request text to reply text;
 absent makes the resolver never answer.

 @returns The server and the requests it saw.

 @example
 ```ts
 await startFakeResolver({ path, answer: () => '{}' });
 ```
 */
async function startFakeResolver(
  {
    path,
    answer,
  }: {
    readonly path: string;
    readonly answer?: (request: string,) => string;
  },
): Promise<{ readonly server: Server; readonly requests: string[]; }> {
  /**
   Requests seen.
   */
  const requests: string[] = [];
  /**
   Fake resolver.
   */
  const server = createServer((socket,) => {
    /**
     Request bytes so far.
     */
    let received = '';
    socket.on('data', (chunk,) => {
      received += chunk.toString('utf8',);
      if (received.endsWith('\0',)) {
        requests.push(received.slice(0, -1,),);
        /**
         Reply for this request.
         */
        if (answer !== undefined)
          socket.write(`${answer(received.slice(0, -1,),)}\0`,);
      }
    },);
    socket.on('error', () => undefined,);
  },);
  await new Promise<void>((resolve,) => {
    server.listen(path, resolve,);
  },);
  return { server, requests, };
}

await describe({
  name: 'resolver flags',
  children: [
    it({
      name: 'ask unicast DNS on one link only, without search domains, synthesis, the local zone or trust anchors',
      fn: async () => {
        expect(RESOLVE_FLAGS,).toBe(26_881,);
        /**
         Request for one record on link 11.
         */
        const parameters = (JSON.parse(resolveRecordRequest({ host: 'crates.io', type: 1, interfaceIndex: 11, },),) as {
          readonly method: string;
          readonly parameters: Readonly<Record<string, unknown>>;
        });
        expect(parameters.method,).toBe(RESOLVE_RECORD_METHOD,);
        expect(parameters.parameters,).toEqual({ name: 'crates.io', type: 1, class: 1, ifindex: 11, flags: 26_881, },);
      },
    },),
  ],
},);

await describe({
  name: 'wire record parsing',
  children: [
    it({
      name: 'reads the TTL after a labelled name and after a compression pointer',
      fn: async () => {
        expect(ttlOfRawRecord({ raw: wireRecord({ bytes: [1, 2, 3, 4,], ttl: 300, compressed: false, },), },),).toBe(300,);
        expect(ttlOfRawRecord({ raw: wireRecord({ bytes: [1, 2, 3, 4,], ttl: 5_018, compressed: true, },), },),).toBe(5_018,);
        expect(ttlOfRawRecord({ raw: Buffer.from([1, 2,],).toString('base64',), },),).toBe(0,);
        expect(ttlOfRawRecord({ raw: '', },),).toBe(0,);
      },
    },),
    it({
      name: 'renders IPv4 and IPv6 bytes and refuses other lengths',
      fn: async () => {
        expect(addressText({ bytes: [140, 82, 112, 3,], },),).toBe('140.82.112.3',);
        expect(addressText({ bytes: [0x26, 0x06, 0x47, 0x00, 0, 0, 0, 0, 0, 0, 0, 0, 0x68, 0x10, 0x84, 0xE5,], },),).toBe('2606:4700:0:0:0:0:6810:84e5',);
        expect(addressText({ bytes: [1, 2, 3,], },),).toBe('',);
      },
    },),
    it({
      name: 'turns a reply into addresses with TTLs, skips records without an address and names an error reply',
      fn: async () => {
        /**
         Two addresses and a record with no address,
         such as a CNAME.
         */
        const mixed = JSON.stringify({
          parameters: {
            rrs: [
              { rr: { address: [140, 82, 112, 3,], }, raw: wireRecord({ bytes: [140, 82, 112, 3,], ttl: 60, compressed: false, },), },
              { rr: { name: 'other.example', }, raw: '', },
              { rr: { address: [1, 2, 3,], }, raw: '', },
            ],
            flags: 8_650_753,
          },
        },);
        expect(parseRecordReply({ text: mixed, family: '-4', },),).toEqual({
          addresses: [{ family: '-4', address: '140.82.112.3', ttlSeconds: 60, },],
        },);
        expect(parseRecordReply({ text: mixed, family: '-6', },).addresses,).toEqual([],);
        expect(parseRecordReply({ text: JSON.stringify({ error: ERROR_NO_SUCH_RECORD, },), family: '-6', },),).toEqual({
          addresses: [],
          error: ERROR_NO_SUCH_RECORD,
        },);
        expect(parseRecordReply({ text: '[]', family: '-4', },).error,).toBe('the reply was not a JSON object',);
        expect(parseRecordReply({ text: '{}', family: '-4', },).error,).toBe('the reply held no record list',);
      },
    },),
  ],
},);

await describe({
  name: readInterfaceIndex.name,
  children: [
    it({
      name: 'reads the index from sysfs and refuses names that could reach another path',
      fn: async () => {
        /**
         Fake sysfs tree.
         */
        const root = await mkdtemp(join(scratchRoot(), 'wgv-sysfs-',),);
        await using cleanup = {
          async [Symbol.asyncDispose](): Promise<void> {
            await rm(root, { force: true, recursive: true, },);
          },
        };
        await mkdir(join(root, 'wg0',),);
        await writeFile(join(root, 'wg0', 'ifindex',), '11\n',);
        await mkdir(join(root, 'bad',),);
        await writeFile(join(root, 'bad', 'ifindex',), 'x\n',);
        expect(await readInterfaceIndex({ interfaceName: 'wg0', sysfsRoot: root, },),).toBe(11,);
        await expect(readInterfaceIndex({ interfaceName: '../wg0', sysfsRoot: root, },),).rejects.toThrow('is not an interface name',);
        await expect(readInterfaceIndex({ interfaceName: '..', sysfsRoot: root, },),).rejects.toThrow('is not an interface name',);
        await expect(readInterfaceIndex({ interfaceName: '', sysfsRoot: root, },),).rejects.toThrow('is not an interface name',);
        await expect(readInterfaceIndex({ interfaceName: 'bad', sysfsRoot: root, },),).rejects.toThrow('no usable interface index',);
        await expect(readInterfaceIndex({ interfaceName: 'absent', sysfsRoot: root, },),).rejects.toThrow();
      },
    },),
  ],
},);

await describe({
  name: callVarlink.name,
  children: [
    it({
      name: 'sends a NUL-terminated request over the socket and returns the NUL-terminated reply',
      fn: async () => {
        /**
         Directory holding the fake socket.
         */
        const directory = await mkdtemp(join(scratchRoot(), 'wgv-',),);
        /**
         Fake resolver answering with a fixed text.
         */
        const { server, requests, } = await startFakeResolver({ path: join(directory, 's',), answer: () => '{"parameters":{"ok":true}}', },);
        await using cleanup = {
          async [Symbol.asyncDispose](): Promise<void> {
            server.close();
            await rm(directory, { force: true, recursive: true, },);
          },
        };
        expect(await callVarlink({ payload: '{"method":"x"}', socketPath: join(directory, 's',), },),).toBe('{"parameters":{"ok":true}}',);
        expect(requests,).toEqual(['{"method":"x"}',],);
      },
    },),
    it({
      name: 'gives up on a resolver that never answers, and on a socket that does not exist',
      fn: async () => {
        /**
         Directory holding the fake socket.
         */
        const directory = await mkdtemp(join(scratchRoot(), 'wgv-',),);
        /**
         Fake resolver that stays silent.
         */
        const { server, } = await startFakeResolver({ path: join(directory, 's',), },);
        await using cleanup = {
          async [Symbol.asyncDispose](): Promise<void> {
            server.close();
            await rm(directory, { force: true, recursive: true, },);
          },
        };
        await expect(callVarlink({ payload: '{}', socketPath: join(directory, 's',), timeoutMs: 150, },),).rejects.toThrow('did not answer',);
        await expect(callVarlink({ payload: '{}', socketPath: join(directory, 'absent',), },),).rejects.toThrow();
      },
    },),
  ],
},);

await describe({
  name: createTunnelHostLookup.name,
  children: [
    it({
      name: 'asks both families on the tunnel link index and returns addresses with TTLs',
      fn: async () => {
        /**
         Requests the fake transport saw.
         */
        const payloads: string[] = [];
        /**
         Lookup over a fake transport.
         */
        const lookup = createTunnelHostLookup({
          interfaceName: 'wg0',
          readIndex: () => Promise.resolve(11,),
          transport: ({ payload, },) => {
            payloads.push(payload,);
            return Promise.resolve(
              (JSON.parse(payload,) as { readonly parameters: { readonly type: number; }; }).parameters.type === 1
                ? reply({ records: [{ bytes: [140, 82, 112, 3,], ttl: 60, }, { bytes: [140, 82, 112, 4,], ttl: 30, },], },)
                : JSON.stringify({ error: ERROR_NO_SUCH_RECORD, },),
            );
          },
        },);
        /**
         Answer for one host.
         */
        const result = await lookup({ host: 'github.com', },);
        expect(result.failure,).toBeUndefined();
        expect(result.addresses,).toEqual([
          { family: '-4', address: '140.82.112.3', ttlSeconds: 60, },
          { family: '-4', address: '140.82.112.4', ttlSeconds: 30, },
        ],);
        expect(payloads,).toHaveLength(2,);
        for (const payload of payloads) {
          /**
           Request parameters.
           */
          const { method, parameters, } = JSON.parse(payload,) as { readonly method: string; readonly parameters: Readonly<Record<string, unknown>>; };
          expect(method,).toBe(RESOLVE_RECORD_METHOD,);
          expect(parameters.name,).toBe('github.com',);
          expect(parameters.ifindex,).toBe(11,);
          expect(parameters.flags,).toBe(RESOLVE_FLAGS,);
        }
      },
    },),
    it({
      name: 'names a link with no DNS server, and a host with no address, as different failures',
      fn: async () => {
        /**
         Lookup whose resolver reports no name servers.
         */
        const noServers = createTunnelHostLookup({
          interfaceName: 'wg0',
          readIndex: () => Promise.resolve(11,),
          transport: () => Promise.resolve(JSON.stringify({ error: ERROR_NO_NAME_SERVERS, },),),
        },);
        expect((await noServers({ host: 'crates.io', },)).failure,).toContain('has no DNS server',);
        /**
         Lookup whose resolver knows no record.
         */
        const noRecords = createTunnelHostLookup({
          interfaceName: 'wg0',
          readIndex: () => Promise.resolve(11,),
          transport: () => Promise.resolve(JSON.stringify({ error: ERROR_NO_SUCH_RECORD, },),),
        },);
        expect((await noRecords({ host: 'crates.io', },)).failure,).toContain('no address for crates.io through wg0',);
      },
    },),
    it({
      name: 'reads the interface index again on every lookup',
      fn: async () => {
        /**
         Indexes handed out in order.
         */
        const indexes = [11, 12,];
        /**
         Indexes the transport saw.
         */
        const seen: number[] = [];
        /**
         Lookup with a changing interface index.
         */
        const lookup = createTunnelHostLookup({
          interfaceName: 'wg0',
          readIndex: () => Promise.resolve(indexes.shift() ?? 0,),
          transport: ({ payload, },) => {
            seen.push((JSON.parse(payload,) as { readonly parameters: { readonly ifindex: number; }; }).parameters.ifindex,);
            return Promise.resolve(JSON.stringify({ error: ERROR_NO_SUCH_RECORD, },),);
          },
        },);
        await lookup({ host: 'a.example', },);
        await lookup({ host: 'b.example', },);
        expect(new Set(seen,),).toEqual(new Set([11, 12,],),);
      },
    },),
  ],
},);
