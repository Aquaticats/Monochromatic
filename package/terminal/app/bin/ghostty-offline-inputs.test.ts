// Controls of the pure rules in ghostty-offline-inputs.ts, run by `mise run //package/terminal/app:test:bin`.

import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { test } from 'node:test';
import { missingInputs, parseManifest, pinnedGhosttyCommit, readyMarkerIsCurrent, readyMarkerFor, sha256Of, verifyArchive } from './ghostty-offline-inputs.ts';

const commit = '0123456789abcdef0123456789abcdef01234567'
const digest = 'a'.repeat(64)
const entry = { name: 'one', source: 'https://example.invalid/one.tar.gz', archive: 'https://example.invalid/one.tar.gz', sha256: digest, zigHash: 'one-0.1.0-abc' }
const manifestWith = (changes: object) => JSON.stringify({ ghosttyRepository: 'https://example.invalid/g.git', ghosttyCommit: commit, packages: [entry], ...changes })

test('the checked-in manifest passes every rule', () => {
  const manifest = parseManifest(readFileSync(new URL('../ghostty-offline-inputs.json', import.meta.url), 'utf8'))
  assert.ok(manifest.packages.length > 0)
})

test('a manifest with an https repository, a full commit and checksummed packages is accepted', () => {
  assert.equal(parseManifest(manifestWith({})).packages[0]?.name, 'one')
})

test('a package without a checksum is refused', () => {
  assert.throws(() => parseManifest(manifestWith({ packages: [{ ...entry, sha256: undefined }] })), /never fetched/)
  assert.throws(() => parseManifest(manifestWith({ packages: [{ ...entry, sha256: 'ABC' }] })), /sha256/)
})

test('a plain http archive is refused', () => {
  assert.throws(() => parseManifest(manifestWith({ packages: [{ ...entry, archive: 'http://example.invalid/one.tar.gz' }] })), /https/)
})

test('a movable Ghostty name is refused', () => {
  assert.throws(() => parseManifest(manifestWith({ ghosttyCommit: 'main' })), /full 40-digit commit/)
})

test('a zig hash that is a path is refused', () => {
  for (const zigHash of ['../escape', 'a/b', '.hidden', '']) {
    assert.throws(() => parseManifest(manifestWith({ packages: [{ ...entry, zigHash }] })), /zigHash/)
  }
})

test('two packages with one zig hash are refused', () => {
  assert.throws(() => parseManifest(manifestWith({ packages: [entry, { ...entry, name: 'two' }] })), /twice/)
})

test('the pinned commit is read from the crate build script', () => {
  assert.equal(pinnedGhosttyCommit(`const GHOSTTY_COMMIT: &str = "${commit}";`), commit)
  assert.throws(() => pinnedGhosttyCommit('const OTHER: &str = "x";'), /no longer declares/)
})

test('an archive that matches its pin passes and a changed archive is refused with both digests', () => {
  const bytes = new TextEncoder().encode('archive')
  const pinned = { ...entry, sha256: sha256Of(bytes) }
  verifyArchive({ bytes, pinned })
  assert.throws(() => verifyArchive({ bytes: new TextEncoder().encode('other'), pinned }), (error: Error) => error.message.includes(pinned.sha256) && error.message.includes(sha256Of(new TextEncoder().encode('other'))))
})

test('the ready marker changes with the manifest text', () => {
  assert.notEqual(readyMarkerFor('a'), readyMarkerFor('b'))
  assert.equal(readyMarkerFor('a'), readyMarkerFor('a'))
})

test('the ready marker counts only when it holds the current pins', () => {
  const expectedMarkerText = readyMarkerFor('a')
  assert.equal(readyMarkerIsCurrent({ markerText: expectedMarkerText, expectedMarkerText }), true)
  assert.equal(readyMarkerIsCurrent({ markerText: undefined, expectedMarkerText }), false)
  assert.equal(readyMarkerIsCurrent({ markerText: readyMarkerFor('b'), expectedMarkerText }), false)
})

// The defect of issue 6271: a marker left behind by a fetch whose source tree was then removed (a fresh worktree
// that kept the gitignored marker) must not pass as current, because libghostty-vt-sys asserts build.zig inside
// GHOSTTY_SOURCE_DIR and the build stops.
test('a matching marker over a missing Ghostty source tree is not current', () => {
  const manifest = parseManifest(manifestWith({}))
  const packageDirectory = `packages/${entry.zigHash}`
  const present = new Set([packageDirectory])
  const isPresent = (relativePath: string) => present.has(relativePath)
  assert.deepEqual(missingInputs({ manifest, isPresent }), ['source/build.zig'])
  present.add('source/build.zig')
  assert.deepEqual(missingInputs({ manifest, isPresent }), [])
  present.delete(packageDirectory)
  assert.deepEqual(missingInputs({ manifest, isPresent }), [packageDirectory])
})
