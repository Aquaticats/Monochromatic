// Checks of the runtime cache folder name read the bundle checks find a build's unpacked parsers by.
// What: which names have the shape the application gives a key folder, over the names earlier runs left on this
// machine and over the near misses: a name of the wrong length, an upper-case digit, and a character that is no digit.
// Why: the checks look for the unpacked parsers below the one key folder of the build under test and count the other
// files there as leftovers, so a folder refused by mistake fails a good build and one accepted by mistake reads
// another build's cache as this one's.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { isRuntimeCacheKeyFolderName } from './runtime-cache-key.ts';

test('reads the key folders earlier runs wrote below the runtime cache', () => {
  for (const name of ['0000000000000031', '0123456789abcdef', 'ffffffffffffffff']) {
    assert.equal(isRuntimeCacheKeyFolderName(name), true);
  }
});

test('reads a name with one digit of another kind as no key folder', () => {
  assert.equal(isRuntimeCacheKeyFolderName('0123456789abcdeF'), false);
  assert.equal(isRuntimeCacheKeyFolderName('0123456789abcdeA'), false);
  assert.equal(isRuntimeCacheKeyFolderName('0123456789abcdeg'), false);
  assert.equal(isRuntimeCacheKeyFolderName('0123456789abcde-'), false);
  assert.equal(isRuntimeCacheKeyFolderName(' 123456789abcdef'), false);
});

test('reads a name of another length as no key folder', () => {
  assert.equal(isRuntimeCacheKeyFolderName('0123456789abcde'), false);
  assert.equal(isRuntimeCacheKeyFolderName('0123456789abcdef0'), false);
  assert.equal(isRuntimeCacheKeyFolderName(''), false);
});

test('reads the names the runtime folder holds beside its key folders as no key folder', () => {
  assert.equal(isRuntimeCacheKeyFolderName('last-used'), false);
  assert.equal(isRuntimeCacheKeyFolderName('.00000000000000aa.removing-1-0'), false);
  assert.equal(isRuntimeCacheKeyFolderName('..'), false);
  assert.equal(isRuntimeCacheKeyFolderName('0123456789abcde\u0661'), false);
});
