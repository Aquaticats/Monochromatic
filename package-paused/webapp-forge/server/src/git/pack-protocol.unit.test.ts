/**
 * Unit tests for the smart-HTTP wire helpers in `pack-protocol.ts`
 * and `pack-protocol-writers.ts`.
 *
 * Covers:
 *
 * - `parseUploadPackBody` round-trips wants/haves/capabilities
 * - `parseReceivePackBody` round-trips triplets/capabilities/pack
 * - `multiplexSideband` chunks to spec ceilings
 * - `writeUploadPackResponse` includes NAK + sideband framing
 * - `writeReceivePackResponse` produces the per-ref status report
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

import {
  multiplexSideband,
  parseReceivePackBody,
  parseUploadPackBody,
  SidebandChannels,
  writeReceivePackResponse,
  writeUploadPackResponse,
} from './pack-protocol.ts';
import {
  encodePkt,
  flushPkt,
} from './pkt-line.ts';

/** OID of the canonical empty git tree, used to fabricate test triplets. */
const EMPTY_TREE_OID = '4b825dc642cb6eb9a060e54bf8d69288fbee4904';

/** Another fixed OID for the new-side of a triplet. */
const ANOTHER_OID = '0123456789abcdef0123456789abcdef01234567';

/** Number of repetitions used to provoke sideband chunking in tests. */
const SIDEBAND_CHUNKS_TO_PROVOKE = 3;

/** Bytes of payload per chunk in the sideband chunking test. */
const SIDEBAND_CHUNK_SIZE = 65_518;

/**
 * Concatenates pkt-line bytes into a single `Uint8Array`.
 *
 * @param chunks - ordered chunks
 *
 * @returns flattened bytes
 *
 * @example
 * ```ts
 * const buf = concat([encodePkt('a'), flushPkt()]);
 * ```
 */
function concat(chunks: readonly Uint8Array[],): Uint8Array {
  let total = 0;
  for (const chunk of chunks)
    total += chunk.byteLength;
  const out = new Uint8Array(total,);
  let cursor = 0;
  for (const chunk of chunks) {
    out.set(
      chunk,
      cursor,
    );
    cursor += chunk.byteLength;
  }
  return out;
}

await describe({
  name: 'pack-protocol',
  concurrency: 1,
  children: [
    describe({
      name: parseUploadPackBody.name,
      concurrency: 1,
      children: [
        it({
          name: 'extracts wants and capabilities from the first line',
          async fn() {
            await Promise.resolve();
            const body = concat([
              encodePkt(`want ${EMPTY_TREE_OID} side-band-64k thin-pack\n`,),
              encodePkt(`want ${ANOTHER_OID}\n`,),
              flushPkt(),
              encodePkt('done\n',),
            ],);
            const result = parseUploadPackBody(body,);
            expect(result.wants,).toEqual([
              EMPTY_TREE_OID,
              ANOTHER_OID,
            ],);
            expect(result.capabilities,).toEqual([
              'side-band-64k',
              'thin-pack',
            ],);
            expect(result.done,).toBe(true,);
          },
        },),
        it({
          name: 'extracts haves',
          async fn() {
            await Promise.resolve();
            const body = concat([
              encodePkt(`want ${EMPTY_TREE_OID}\n`,),
              flushPkt(),
              encodePkt(`have ${ANOTHER_OID}\n`,),
              flushPkt(),
              encodePkt('done\n',),
            ],);
            const result = parseUploadPackBody(body,);
            expect(result.haves,).toEqual([ANOTHER_OID,],);
          },
        },),
      ],
    },),
    describe({
      name: parseReceivePackBody.name,
      concurrency: 1,
      children: [
        it({
          name: 'extracts triplets and capabilities; treats trailing bytes as packfile',
          async fn() {
            await Promise.resolve();
            const triplet1 =
              `${EMPTY_TREE_OID} ${ANOTHER_OID} refs/heads/main\0report-status side-band-64k\n`;
            const triplet2 = `${ANOTHER_OID} ${EMPTY_TREE_OID} refs/heads/feat\n`;
            const fakePack = new TextEncoder().encode('PACK_DATA',);
            const body = concat([
              encodePkt(triplet1,),
              encodePkt(triplet2,),
              flushPkt(),
              fakePack,
            ],);
            const result = parseReceivePackBody(body,);
            expect(result.triplets,).toEqual([
              {
                oldOid: EMPTY_TREE_OID,
                newOid: ANOTHER_OID,
                refName: 'refs/heads/main',
              },
              {
                oldOid: ANOTHER_OID,
                newOid: EMPTY_TREE_OID,
                refName: 'refs/heads/feat',
              },
            ],);
            expect(result.capabilities,).toEqual([
              'report-status',
              'side-band-64k',
            ],);
            expect(new TextDecoder().decode(result.packfile,),).toBe('PACK_DATA',);
          },
        },),
        it({
          name: 'returns empty triplets when only flush-pkt is present',
          async fn() {
            await Promise.resolve();
            const body = flushPkt();
            const result = parseReceivePackBody(body,);
            expect(result.triplets.length,).toBe(0,);
            expect(result.packfile.byteLength,).toBe(0,);
          },
        },),
        it({
          name: 'throws on missing flush-pkt',
          async fn() {
            await Promise.resolve();
            const body = encodePkt(
              `${EMPTY_TREE_OID} ${ANOTHER_OID} refs/heads/main\n`,
            );
            expect(function attemptParse() {
              parseReceivePackBody(body,);
            },)
              .toThrow();
          },
        },),
      ],
    },),
    describe({
      name: multiplexSideband.name,
      concurrency: 1,
      children: [
        it({
          name: 'wraps payload in sideband-64k chunks at spec ceiling',
          async fn() {
            await Promise.resolve();
            const payload = new Uint8Array(
              SIDEBAND_CHUNK_SIZE * SIDEBAND_CHUNKS_TO_PROVOKE,
            );
            payload.fill(0x42,);
            const chunks = multiplexSideband({
              payload,
              channel: SidebandChannels.PACK,
              useSideBand64k: true,
            },);
            expect(chunks.length,).toBe(SIDEBAND_CHUNKS_TO_PROVOKE,);
            for (const chunk of chunks) {
              // Each chunk: 4-byte header + 1-byte channel + payload
              expect(chunk[4],).toBe(SidebandChannels.PACK,);
            }
          },
        },),
        it({
          name: 'returns one chunk for tiny payloads',
          async fn() {
            await Promise.resolve();
            const payload = new TextEncoder().encode('x',);
            const chunks = multiplexSideband({
              payload,
              channel: SidebandChannels.PROGRESS,
              useSideBand64k: false,
            },);
            expect(chunks.length,).toBe(1,);
            const [single,] = chunks;
            expect(single,).toBeDefined();
            if (single === undefined)
              throw new Error('expected one chunk',);
            expect(single[4],).toBe(SidebandChannels.PROGRESS,);
          },
        },),
      ],
    },),
    describe({
      name: writeUploadPackResponse.name,
      concurrency: 1,
      children: [
        it({
          name: 'emits NAK then sideband-multiplexed pack data when sideband requested',
          async fn() {
            await Promise.resolve();
            const fakePack = new TextEncoder().encode('PACK_BYTES',);
            const chunks = writeUploadPackResponse({
              packfile: fakePack,
              useSideBand: true,
              useSideBand64k: true,
            },);
            const flat = concat(chunks,);
            const text = new TextDecoder().decode(flat,);
            expect(text.startsWith('0008NAK\n',),).toBe(true,);
            // The flat stream ends with a flush-pkt.
            expect(text.endsWith('0000',),).toBe(true,);
          },
        },),
        it({
          name: 'emits raw pack after NAK when sideband not requested',
          async fn() {
            await Promise.resolve();
            const fakePack = new TextEncoder().encode('RAW',);
            const chunks = writeUploadPackResponse({
              packfile: fakePack,
              useSideBand: false,
              useSideBand64k: false,
            },);
            const flat = concat(chunks,);
            // 0008NAK\n + RAW
            expect(new TextDecoder().decode(flat,),).toBe('0008NAK\nRAW',);
          },
        },),
      ],
    },),
    describe({
      name: writeReceivePackResponse.name,
      concurrency: 1,
      children: [
        it({
          name: 'emits unpack ok + per-ref status + flush',
          async fn() {
            await Promise.resolve();
            const chunks = writeReceivePackResponse({
              unpackOk: true,
              refResults: [
                {
                  refName: 'refs/heads/main',
                  ok: true,
                },
                {
                  refName: 'refs/heads/feat',
                  ok: false,
                  error: 'fetch-first',
                },
              ],
            },);
            const flat = concat(chunks,);
            const text = new TextDecoder().decode(flat,);
            expect(text.includes('unpack ok\n',),).toBe(true,);
            expect(text.includes('ok refs/heads/main\n',),).toBe(true,);
            expect(text.includes('ng refs/heads/feat fetch-first\n',),).toBe(true,);
            expect(text.endsWith('0000',),).toBe(true,);
          },
        },),
        it({
          name: 'emits unpack <error> when unpackOk is false',
          async fn() {
            await Promise.resolve();
            const chunks = writeReceivePackResponse({
              unpackOk: false,
              unpackError: 'corrupt pack',
              refResults: [
                {
                  refName: 'refs/heads/main',
                  ok: false,
                  error: 'unpack failed',
                },
              ],
            },);
            const flat = concat(chunks,);
            const text = new TextDecoder().decode(flat,);
            expect(text.includes('unpack corrupt pack\n',),).toBe(true,);
          },
        },),
        it({
          name:
            'wraps the report on sideband channel 1 when client negotiated side-band-64k',
          async fn() {
            await Promise.resolve();
            const chunks = writeReceivePackResponse({
              unpackOk: true,
              refResults: [
                {
                  refName: 'refs/heads/main',
                  ok: true,
                },
              ],
              useSideBand: true,
              useSideBand64k: true,
            },);
            const flat = concat(chunks,);
            // Outer flush-pkt terminates the sideband stream: last 4 bytes are "0000".
            const tail = new TextDecoder().decode(flat.subarray(flat.byteLength - 4,),);
            expect(tail,).toBe('0000',);
            // First pkt-line: 4 hex bytes of length, then 0x01 channel marker.
            const firstChannelByteIndex = 4;
            expect(flat[firstChannelByteIndex],).toBe(0x01,);
            // The wrapped payload still contains the literal report bytes;
            // sideband only inserts a 1-byte channel marker per pkt-line.
            const inner = new TextDecoder().decode(flat,);
            expect(inner.includes('unpack ok\n',),).toBe(true,);
            expect(inner.includes('ok refs/heads/main\n',),).toBe(true,);
          },
        },),
      ],
    },),
  ],
},);
