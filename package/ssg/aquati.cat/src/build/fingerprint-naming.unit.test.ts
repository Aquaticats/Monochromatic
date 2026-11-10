/**
 Tests for the fingerprinting name and digest primitives.
 
 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import {
  insertHash,
  sha256Buffer,
} from './fingerprint-naming.ts';

/**
 Hash long enough to exercise the truncation.
 */
const LONG_HASH = 'a1b2c3d4ef9876543210';

/**
 Hash shorter than the inserted prefix length.
 */
const SHORT_HASH = 'abc';

/**
 SHA-256 of an empty buffer, a fixed value defined by the algorithm.
 */
const EMPTY_SHA256 = 'e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855';

/**
 Cases for hash insertion, each pairing an input name with its expected output.
 */
const INSERT_HASH_CASES = [
  {
    name: 'styles.css',
    hash: LONG_HASH,
    expected: 'styles.a1b2c3d4ef.css',
    reason: 'one extension',
  },
  {
    name: 'archive.tar.gz',
    hash: LONG_HASH,
    expected: 'archive.tar.a1b2c3d4ef.gz',
    reason: 'a compound extension, where only the last part is kept',
  },
  {
    name: 'favicon',
    hash: LONG_HASH,
    expected: 'favicon.a1b2c3d4ef',
    reason: 'no extension at all',
  },
  {
    name: 'styles.css',
    hash: SHORT_HASH,
    expected: 'styles.abc.css',
    reason: 'a hash shorter than the inserted prefix',
  },
  {
    name: '.hidden',
    hash: LONG_HASH,
    expected: '.a1b2c3d4ef.hidden',
    reason: 'a dotfile, where the leading dot is read as the extension',
  },
] as const;

await describe({
  name: '',
  children: [
    describe({
      name: insertHash.name,
      children: INSERT_HASH_CASES.map(function insertCase({ name, hash, expected, reason, },) {
        return it({
          name: `inserts the hash for ${name} (${reason})`,
          fn: async function insertsHash(): Promise<void> {
            expect(insertHash({
              name,
              hash,
            },),).toBe(expected,);
          },
        },);
      },),
    },),
    describe({
      name: sha256Buffer.name,
      children: [
        it({
          name: 'digests an empty buffer',
          fn: async function digestsEmptyBuffer(): Promise<void> {
            /**
             Digest of zero bytes of input.
             */
            const digest = sha256Buffer(Buffer.alloc(0,),);

            expect(digest,).toBe(EMPTY_SHA256,);
          },
        },),
        it({
          name: 'digests distinct buffers to distinct hashes',
          fn: async function digestsDistinctBuffers(): Promise<void> {
            /**
             Digest of a single `a` byte.
             */
            const first = sha256Buffer(Buffer.from('a',),);
            /**
             Digest of a single `b` byte.
             */
            const second = sha256Buffer(Buffer.from('b',),);

            expect(first,).not.toBe(second,);
          },
        },),
      ],
    },),
  ],
},);
