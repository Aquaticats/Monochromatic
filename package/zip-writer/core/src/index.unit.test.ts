import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  mkdtemp,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

import spawn from 'nano-spawn';

import {
  crc32,
  dosDateTime,
  ZipWriter,
} from './index.ts';

/** Async-disposable temp directory; cleaned up on scope exit. */
type DisposableTempDir = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Create a fresh temp directory with an async-dispose cleanup attached.
 Use with `await using` so the directory is removed when the enclosing
 scope exits, regardless of how it exits.
 
 @returns Async-disposable temp directory handle
 */
async function makeTempDir(): Promise<DisposableTempDir> {
  const path = await mkdtemp(join(scratchRoot(), 'zip-writer-test-',),);
  return {
    path,
    async [Symbol.asyncDispose](): Promise<void> {
      await rm(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Verify a ZIP buffer with system `unzip -tq` (integrity test).
 
 @param bytes - Archive bytes to verify
 
 @returns Stdout/stderr from the `unzip` invocation
 */
async function verifyZipIntegrity(
  bytes: Uint8Array,
): Promise<{ stdout: string; stderr: string; }> {
  await using tempDir = await makeTempDir();
  const file = join(tempDir.path, 'test.zip',);
  await writeFile(file, bytes,);
  const { stdout, stderr, } = await spawn('unzip', ['-tq', file,],);
  return { stdout, stderr, };
}

/**
 List entries inside a ZIP buffer using `unzip -Z1`.
 
 @param bytes - Archive bytes to list
 
 @returns One entry path per element
 */
async function listZipEntries(bytes: Uint8Array,): Promise<string[]> {
  await using tempDir = await makeTempDir();
  const file = join(tempDir.path, 'test.zip',);
  await writeFile(file, bytes,);
  const { stdout, } = await spawn('unzip', ['-Z1', file,],);
  return stdout.split('\n',).filter(line => line.length > 0);
}

/**
 Extract a single file from a ZIP buffer.
 
 Extracts the archive to a temp directory and reads the entry as bytes,
 since `nano-spawn` returns string-only stdout (no `encoding: 'buffer'`).
 
 @param bytes - Archive bytes to extract from
 
 @param path - Entry path inside the archive
 
 @returns Raw bytes of the extracted entry
 */
async function extractFromZip(
  {
    bytes,
    path,
  }: {
    bytes: Uint8Array;
    path: string;
  },
): Promise<Uint8Array> {
  await using tempDir = await makeTempDir();
  const file = join(tempDir.path, 'test.zip',);
  await writeFile(file, bytes,);
  await spawn('unzip', ['-o', '-d', tempDir.path, file,],);
  return new Uint8Array(
    await readFile(join(tempDir.path, path,),),
  );
}

await describe({
  name: 'zip-writer',
  children: [
    describe({
      name: crc32.name,
      children: [
        it({
          name: 'returns 0 for empty input',
          fn: async () => {
            expect(
              crc32(new Uint8Array(),),
            ).toBe(0,);
          },
        },),
        it({
          name: 'matches the canonical "123456789" reference vector',
          fn: async () => {
            const data = new TextEncoder().encode('123456789',);
            // Standard CRC-32/ISO-HDLC test vector.
            expect(crc32(data,),).toBe(0xCB_F4_39_26,);
          },
        },),
        it({
          name:
            'matches the "The quick brown fox jumps over the lazy dog" reference vector',
          fn: async () => {
            const data = new TextEncoder().encode(
              'The quick brown fox jumps over the lazy dog',
            );
            expect(crc32(data,),).toBe(0x41_4F_A3_39,);
          },
        },),
      ],
    },),
    describe({
      name: dosDateTime.name,
      children: [
        it({
          name: 'encodes a normal date correctly',
          fn: async () => {
            const date = new Date(Date.UTC(2_024, 5, 15, 12, 30, 30,),);
            const { date: d, time: t, } = dosDateTime(date,);
            expect(d,).toBe(((2_024 - 1_980) << 9) | (6 << 5) | 15,);
            expect(t,).toBe((12 << 11) | (30 << 5) | 15,);
          },
        },),
        it({
          name: 'clamps pre-1980 dates to 1980',
          fn: async () => {
            const date = new Date(0,);
            const { date: d, } = dosDateTime(date,);
            // Year bits should encode 0 (1980 - 1980).
            expect(d >> 9,).toBe(0,);
          },
        },),
      ],
    },),
    describe({
      name: 'ZipWriter.add',
      children: [
        it({
          name: 'rejects empty path',
          fn: async () => {
            const zip = new ZipWriter();
            expect(() => zip.add('', 'x',)).toThrow('non-empty',);
          },
        },),
        it({
          name: 'rejects path with NUL byte',
          fn: async () => {
            const zip = new ZipWriter();
            expect(() => zip.add('foo\0bar', 'x',)).toThrow('NUL',);
          },
        },),
        it({
          name: 'rejects path with backslash',
          fn: async () => {
            const zip = new ZipWriter();
            expect(() => zip.add(String.raw`foo\bar`, 'x',)).toThrow('backslash',);
          },
        },),
        it({
          name: 'rejects leading slash',
          fn: async () => {
            const zip = new ZipWriter();
            expect(() => zip.add('/foo', 'x',)).toThrow('leading slash',);
          },
        },),
        it({
          name: 'rejects parent-directory segment',
          fn: async () => {
            const zip = new ZipWriter();
            expect(() => zip.add('foo/../bar', 'x',)).toThrow('parent-directory',);
          },
        },),
        it({
          name: 'rejects duplicate entry',
          fn: async () => {
            const zip = new ZipWriter();
            zip.add('foo', 'x',);
            expect(() => zip.add('foo', 'y',)).toThrow('duplicate',);
          },
        },),
      ],
    },),
    describe({
      name: 'ZipWriter.build',
      children: [
        it({
          name: 'produces a valid empty ZIP',
          fn: async () => {
            const zip = new ZipWriter();
            const bytes = zip.build();
            // Empty ZIP = just an EOCD record (22 bytes).
            expect(bytes.length,).toBe(22,);
            // PK\x05\x06 signature.
            expect(bytes[0],).toBe(0x50,);
            expect(bytes[1],).toBe(0x4B,);
            expect(bytes[2],).toBe(0x05,);
            expect(bytes[3],).toBe(0x06,);
          },
        },),
        it({
          name: 'starts with the local file header signature when entries exist',
          fn: async () => {
            const zip = new ZipWriter();
            zip.add('hello.txt', 'world',);
            const bytes = zip.build();
            // PK\x03\x04 LFH signature.
            expect(bytes[0],).toBe(0x50,);
            expect(bytes[1],).toBe(0x4B,);
            expect(bytes[2],).toBe(0x03,);
            expect(bytes[3],).toBe(0x04,);
          },
        },),
        it({
          name: 'produces a ZIP that `unzip -tq` accepts',
          fn: async () => {
            const zip = new ZipWriter();
            zip.add('a.txt', 'aaa',);
            zip.add('nested/b.json', JSON.stringify({ k: 'v', },),);
            zip.add('binary.bin', new Uint8Array([0, 1, 2, 255,],),);
            const bytes = zip.build();
            const { stdout, } = await verifyZipIntegrity(bytes,);
            expect(stdout,).toContain('No errors detected',);
          },
        },),
        it({
          name: 'preserves entry order in the central directory',
          fn: async () => {
            const zip = new ZipWriter();
            zip.add('zebra.txt', 'z',);
            zip.add('alpha.txt', 'a',);
            zip.add('mango.txt', 'm',);
            const bytes = zip.build();
            const entries = await listZipEntries(bytes,);
            expect(entries,).toEqual(['zebra.txt', 'alpha.txt', 'mango.txt',],);
          },
        },),
        it({
          name: 'round-trips string content as UTF-8',
          fn: async () => {
            const zip = new ZipWriter();
            const original = 'hello 世界 🌍';
            zip.add('greeting.txt', original,);
            const bytes = zip.build();
            const extracted = await extractFromZip({ bytes, path: 'greeting.txt', },);
            expect(new TextDecoder().decode(extracted,),).toBe(original,);
          },
        },),
        it({
          name: 'round-trips binary content byte-for-byte',
          fn: async () => {
            const zip = new ZipWriter();
            const data = new Uint8Array(1_024,);
            for (let loopIndex = 0; loopIndex < data.length; loopIndex += 1)
              data[loopIndex] = (loopIndex * 7) & 0xFF;
            zip.add('blob.bin', data,);
            const bytes = zip.build();
            const extracted = await extractFromZip({ bytes, path: 'blob.bin', },);
            expect(extracted.length,).toBe(data.length,);
            for (let loopIndex = 0; loopIndex < data.length; loopIndex += 1)
              expect(extracted[loopIndex],).toBe(data[loopIndex],);
          },
        },),
        it({
          name: 'retains supplied binary view until archive build',
          fn: async () => {
            const zip = new ZipWriter();
            const data = new Uint8Array([1,],);
            zip.add('retained.bin', data,);
            data[0] = 9;
            /**
             Fixed local-file-header byte length before encoded entry name.
             */
            const localFileHeaderLength = 30;
            /**
             Archive content offset after fixed header and encoded path.
             */
            const contentOffset = localFileHeaderLength
              + new TextEncoder()
                .encode('retained.bin',)
                .length;
            expect(zip.build()[contentOffset],).toBe(9,);
          },
        },),
        it({
          name: 'round-trips UTF-8 file names',
          fn: async () => {
            const zip = new ZipWriter();
            zip.add('日本語/ファイル.txt', 'こんにちは',);
            const bytes = zip.build();
            const entries = await listZipEntries(bytes,);
            expect(entries,).toEqual(['日本語/ファイル.txt',],);
            const extracted = await extractFromZip({ bytes,
              path: '日本語/ファイル.txt', },);
            expect(new TextDecoder().decode(extracted,),).toBe('こんにちは',);
          },
        },),
        it({
          name: 'reports correct entry count via .size',
          fn: async () => {
            const zip = new ZipWriter();
            expect(zip.size,).toBe(0,);
            zip.add('a', 'x',);
            zip.add('b', 'y',);
            zip.add('c', 'z',);
            expect(zip.size,).toBe(3,);
          },
        },),
        it({
          name: 'produces deterministic output when modifiedAt is fixed',
          fn: async () => {
            const fixed = new Date(Date.UTC(2_024, 0, 1, 0, 0, 0,),);
            const a = new ZipWriter({ modifiedAt: fixed, },);
            a.add('x.txt', 'hello',);
            const b = new ZipWriter({ modifiedAt: fixed, },);
            b.add('x.txt', 'hello',);
            const ba = a.build();
            const bb = b.build();
            expect(ba.length,).toBe(bb.length,);
            for (let loopIndex = 0; loopIndex < ba.length; loopIndex += 1)
              expect(ba[loopIndex],).toBe(bb[loopIndex],);
          },
        },),
      ],
    },),
  ],
},);
