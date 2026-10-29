/**
 * Unit tests for the pkt-line codec.
 *
 * Covers encode/decode round-trips for data lines, the flush-pkt
 * (`"0000"`) and delim-pkt (`"0001"`) sentinels, and the malformed
 * input rejection paths.
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core';

import {
  decodePktLines,
  delimPkt,
  encodePkt,
  flushPkt,
  type PktLine,
} from './pkt-line.ts';

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
  name: 'pkt-line',
  concurrency: 1,
  children: [
    it({
      name: 'encodePkt prepends 4-byte hex length',
      async fn() {
        await Promise.resolve();
        const result = encodePkt('a',);
        expect(new TextDecoder().decode(result,),).toBe('0005a',);
      },
    },),
    it({
      name: 'encodePkt handles empty string',
      async fn() {
        await Promise.resolve();
        const result = encodePkt('',);
        expect(new TextDecoder().decode(result,),).toBe('0004',);
      },
    },),
    it({
      name: 'flushPkt is "0000"',
      async fn() {
        await Promise.resolve();
        expect(
          new TextDecoder().decode(flushPkt(),),
        ).toBe('0000',);
      },
    },),
    it({
      name: 'delimPkt is "0001"',
      async fn() {
        await Promise.resolve();
        expect(
          new TextDecoder().decode(delimPkt(),),
        ).toBe('0001',);
      },
    },),
    it({
      name: 'decode round-trips a single line',
      async fn() {
        await Promise.resolve();
        const original = encodePkt('hello\n',);
        const lines: PktLine[] = decodePktLines(original,);
        expect(lines.length,).toBe(1,);
        const [first,] = lines;
        if ((first === null) || (first === 'delim') || (first === undefined))
          throw new Error('expected data line',);
        expect(new TextDecoder().decode(first,),).toBe('hello\n',);
      },
    },),
    it({
      name: 'decode handles flush + delim sentinels',
      async fn() {
        await Promise.resolve();
        const stream = concat([
          encodePkt('first',),
          flushPkt(),
          encodePkt('second',),
          delimPkt(),
        ],);
        const lines = decodePktLines(stream,);
        expect(lines.length,).toBe(4,);
        expect(lines[1],).toBe(null,);
        expect(lines[3],).toBe('delim',);
      },
    },),
    it({
      name: 'decode throws on truncated body',
      async fn() {
        await Promise.resolve();
        // "0010" claims 16 bytes total but only 4 are present.
        const truncated = new TextEncoder().encode('0010',);
        expect(function attemptDecode() {
          decodePktLines(truncated,);
        },)
          .toThrow();
      },
    },),
    it({
      name: 'decode throws on invalid length prefix',
      async fn() {
        await Promise.resolve();
        const garbage = new TextEncoder().encode('zzzz',);
        expect(function attemptDecode() {
          decodePktLines(garbage,);
        },)
          .toThrow();
      },
    },),
  ],
},);
