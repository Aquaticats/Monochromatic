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
  isFingerprintedName,
  sha256Buffer,
} from '../../dist/final/node/index.mjs';

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

/**
 Fingerprinted-name inputs, each paired with what the deleted
 `\.[0-9a-f]{10}\.[^.]+$` pattern answered for it.
 */
const FINGERPRINTED_NAME_CASES = [
  {
    name: 'a fingerprinted stylesheet',
    file: 'styles.a1b2c3d4ef.css',
    expected: true,
    reason: 'a digest of the insert length sits between two dots',
  },
  {
    name: 'an unfingerprinted stylesheet',
    file: 'styles.css',
    expected: false,
    reason: 'the segment before the extension is not a digest',
  },
  {
    name: 'a name with a single long segment',
    file: 'a.0123456789',
    expected: false,
    reason: 'a digest needs a dot on either side',
  },
  {
    name: 'a name whose extension is empty',
    file: 'a.0123456789.',
    expected: false,
    reason: 'the extension class needs at least one character',
  },
  {
    name: 'a name starting with the digest',
    file: '.0123456789.css',
    expected: true,
    reason: 'the leading dot serves as the dot before the digest',
  },
  {
    name: 'a dotfile with one dot',
    file: '.a1b2c3d4ef',
    expected: false,
    reason: 'one dot cannot bound a digest on both sides',
  },
  {
    name: 'a digest one character too long',
    file: 'a.a1b2c3d4efg.css',
    expected: false,
    reason: 'the length compare rejects eleven characters',
  },
  {
    name: 'a digest one character too short',
    file: 'a.a1b2c3d4e.css',
    expected: false,
    reason: 'the length compare rejects nine characters',
  },
  {
    name: 'an upper-case digest',
    file: 'a.A1B2C3D4EF.css',
    expected: false,
    reason: 'the hex alphabet is lower case only',
  },
  {
    name: 'a digest with a non-hex letter',
    file: 'a.a1b2c3d4eg.css',
    expected: false,
    reason: 'the alphabet stops at f',
  },
  {
    name: 'an upper-case extension',
    file: 'a.a1b2c3d4ef.CSS',
    expected: true,
    reason: 'the extension class is any character but a dot',
  },
  {
    name: 'an empty segment before the extension',
    file: 'a.a1b2c3d4ef..css',
    expected: false,
    reason: 'the extension class needs at least one character',
  },
  {
    name: 'an extension holding a dot',
    file: 'a.a1b2c3d4ef.c.d',
    expected: false,
    reason: 'the extension class stops at a dot, so the tail cannot end there',
  },
  {
    name: 'a zstd-compressed fingerprinted file',
    file: 'a.a1b2c3d4ef.c.zst',
    expected: false,
    reason: 'the compressed form has its own pattern, which this one does not match',
  },
  {
    name: 'a path rather than a basename',
    file: 'dist/assets/a.a1b2c3d4ef.css',
    expected: true,
    reason: 'the check reads the tail of whatever it is given, as the anchored pattern did',
  },
  {
    name: 'an empty name',
    file: '',
    expected: false,
    reason: 'no dot to find',
  },
  {
    name: 'a single dot',
    file: '.',
    expected: false,
    reason: 'a dot with nothing around it',
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
    describe({
      name: isFingerprintedName.name,
      children: FINGERPRINTED_NAME_CASES.map(function fingerprintedNameCase({ name, file, expected, reason, },) {
        return it({
          name: `reads ${name} (${reason})`,
          fn: async function readsFingerprintedName(): Promise<void> {
            expect(isFingerprintedName(file,),).toBe(expected,);
          },
        },);
      },),
    },),
  ],
},);
