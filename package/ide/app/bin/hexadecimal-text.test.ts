// The shape of the revision a runtime manifest names, against the commit ids the manifest holds.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { isHexadecimalText } from './hexadecimal-text.ts';

// A Helix revision as runtime/manifest.json names it: the commit id of the pinned release.
const revision = 'a1b2c3d4e5f60718293a4b5c6d7e8f9012345678';
const revisionLength = revision.length;

test('accepts a commit id of the length the manifest states', () => {
  assert.equal(revisionLength, 40);
  assert.equal(isHexadecimalText({ text: revision, length: revisionLength }), true);
});

test('accepts text of the asked length over the digits and the letters a to f', () => {
  for (const text of ['0000', 'ffff', '0123', 'abcdef', '9f9f']) assert.equal(isHexadecimalText({ text, length: text.length }), true, text);
});

test('rejects text of another length, whatever it holds', () => {
  assert.equal(isHexadecimalText({ text: revision.slice(1), length: revisionLength }), false);
  assert.equal(isHexadecimalText({ text: revision + '0', length: revisionLength }), false);
  assert.equal(isHexadecimalText({ text: '', length: revisionLength }), false);
});

test('rejects text holding a character no hexadecimal print uses', () => {
  for (const text of ['000A', '000g', '00-0', '00 0', '\u00e90f0', '000.']) assert.equal(isHexadecimalText({ text, length: text.length }), false, JSON.stringify(text));
});
