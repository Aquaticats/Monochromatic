// Checks of the copyright-line read the bundle check asks of a font notice.
// What: which lines count as a copyright line naming the project authors, over the notices the bundle ships and over
// the folds, separators and mid-line phrases a text can hold.
// Why: the check demands a copyright line from each font notice, so a read that misses a folded notice passes a file
// that names nobody, and one that counts a sentence about copyright passes a file that replaced the notice.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { hasProjectAuthorsCopyrightLine, mentionsCopyright } from './copyright-lines.ts';

// The first lines of the two font notices the bundle ships, as they stand beside the IDE.
const notices = [
  'Copyright (c) 2016 The Inter Project Authors (https://github.com/rsms/inter)',
  'Copyright 2020 The JetBrains Mono Project Authors (https://github.com/JetBrains/JetBrainsMono)',
];

test('reads the copyright line of each shipped font notice', () => {
  for (const notice of notices) {
    assert.equal(hasProjectAuthorsCopyrightLine(notice), true);
    assert.equal(hasProjectAuthorsCopyrightLine('Some title\n\n' + notice + '\n\nSIL OPEN FONT LICENSE\n'), true);
  }
});

test('reads a line that puts the phrase directly after the label', () => {
  assert.equal(hasProjectAuthorsCopyrightLine('Copyright Project Authors'), true);
});

test('reads the phrase only on the line the label opens', () => {
  assert.equal(hasProjectAuthorsCopyrightLine('Copyright (c) 2016\nThe Inter Project Authors'), false);
  assert.equal(hasProjectAuthorsCopyrightLine('Copyright (c) 2016\nThe Inter Project Authors\nCopyright 2016 The Project Authors'), true);
});

test('reads a line of the text that opens after any line terminator', () => {
  assert.equal(hasProjectAuthorsCopyrightLine('title\rCopyright 2016 The Project Authors'), true);
  assert.equal(hasProjectAuthorsCopyrightLine('title\u2028Copyright 2016 The Project Authors'), true);
  assert.equal(hasProjectAuthorsCopyrightLine('title\u2029Copyright 2016 The Project Authors'), true);
});

test('refuses a notice folded by a carriage return or a paragraph separator', () => {
  assert.equal(hasProjectAuthorsCopyrightLine('Copyright 2016\rThe Project Authors'), false);
  assert.equal(hasProjectAuthorsCopyrightLine('Copyright 2016\u2028The Project Authors'), false);
  assert.equal(hasProjectAuthorsCopyrightLine('Copyright 2016\u2029The Project Authors'), false);
});

test('refuses a phrase that occurs mid-line rather than where the line opens', () => {
  assert.equal(hasProjectAuthorsCopyrightLine('The Copyright Holder and the Project Authors'), false);
  assert.equal(hasProjectAuthorsCopyrightLine('See the Copyright notice\n The Project Authors'), false);
});

test('refuses a label that is not the one the line opens with', () => {
  assert.equal(hasProjectAuthorsCopyrightLine('copyright 2016 The Project Authors'), false);
  assert.equal(hasProjectAuthorsCopyrightLine('Copyright-2016 The Project Authors'), false);
  assert.equal(hasProjectAuthorsCopyrightLine('Copyright2016 The Project Authors'), false);
});

test('refuses text that names holders other than the project authors', () => {
  assert.equal(hasProjectAuthorsCopyrightLine('Copyright 2016 The Inter Authors'), false);
  assert.equal(hasProjectAuthorsCopyrightLine('Copyright 2016 The Inter Project Holder'), false);
  assert.equal(hasProjectAuthorsCopyrightLine(''), false);
});

test('reads the word copyright in any case of its letters', () => {
  assert.equal(mentionsCopyright('Copyright (c) 2016 The Inter Project Authors'), true);
  assert.equal(mentionsCopyright('copyright'), true);
  assert.equal(mentionsCopyright('COPYRIGHT'), true);
  assert.equal(mentionsCopyright('licensed under the CoPyRiGhT holder'), true);
});

test('refuses a word that only resembles copyright', () => {
  assert.equal(mentionsCopyright('copyrigth'), false);
  assert.equal(mentionsCopyright('copy right'), false);
  assert.equal(mentionsCopyright(''), false);
});

test('refuses a letter outside ASCII that resembles one of the word', () => {
  // A dotted capital I and a long s each look like an ASCII letter and are not one, and a pattern without the `u` flag
  // does not fold either to it.
  assert.equal(mentionsCopyright('COPYR\u0130GHT'), false);
  assert.equal(mentionsCopyright('copyri\u017fht'), false);
  assert.equal(mentionsCopyright('copyright\u0130'), true);
});
