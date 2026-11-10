// Checks of the terminal color removal the bundle checks read captured output through.
// What: which sequences are color escapes and which are not, over the shapes a captured session writes and over the
// near misses: a lone escape, an escape cut short, another escape form, and two escapes where one starts inside the
// other's parameter run.
// Why: the checks look for phrases inside the compositor's and the application's combined output; a sequence left in
// place splits a phrase and fails a good session, and dropping an escape that is not a color would change the text the
// checks quote back in their diagnostics.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { withoutTerminalColors } from './terminal-color-escapes.ts';

// A sequence of the color form: the escape, the bracket, the parameters and the terminator.
const color = (...parameters: (string | number)[]): string => '\u001b[' + parameters.join(';') + 'm';

test('drops the color sequences a session wrote and keeps the text between them', () => {
  assert.equal(withoutTerminalColors('pass: 3\n'), 'pass: 3\n');
  assert.equal(withoutTerminalColors(color(32) + 'pass: 3' + color(0) + '\n'), 'pass: 3\n');
  assert.equal(withoutTerminalColors(color(1, 32) + 'bold green' + color(0, 0) + ' then plain'), 'bold green then plain');
});

test('drops a sequence whose parameters are empty', () => {
  assert.equal(withoutTerminalColors(color() + 'plain'), 'plain');
  assert.equal(withoutTerminalColors(color('', '', '') + 'plain'), 'plain');
});

test('drops a run of sequences with text before and after', () => {
  assert.equal(withoutTerminalColors('a' + color(0) + 'b' + color(31) + 'c'), 'abc');
});

test('keeps an escape that is not a color sequence', () => {
  // Erasing from the cursor to the end of the line ends with K, not m.
  assert.equal(withoutTerminalColors('\u001b[1Kkept'), '\u001b[1Kkept');
  assert.equal(withoutTerminalColors('\u001b[2Jkept'), '\u001b[2Jkept');
  assert.equal(withoutTerminalColors('\u001b[?25lkept'), '\u001b[?25lkept');
});

test('keeps an escape cut short before its terminator', () => {
  assert.equal(withoutTerminalColors('\u001b'), '\u001b');
  assert.equal(withoutTerminalColors('\u001b['), '\u001b[');
  assert.equal(withoutTerminalColors('\u001b[0'), '\u001b[0');
  assert.equal(withoutTerminalColors('text\u001b[12;34'), 'text\u001b[12;34');
});

test('keeps an escape whose bracket is missing', () => {
  assert.equal(withoutTerminalColors('\u001b0m'), '\u001b0m');
  assert.equal(withoutTerminalColors('\u001b(32m'), '\u001b(32m');
});

test('resumes after the terminator, so an escape written inside another one is kept', () => {
  // The first sequence ends at its own terminator; what follows is read again, and an escape that opened inside the
  // parameters of a sequence that never ended is text.
  assert.equal(withoutTerminalColors('\u001b[\u001b[m'), '\u001b[');
  assert.equal(withoutTerminalColors('\u001b[3\u001b[1mx'), '\u001b[3x');
});

test('keeps text that holds no escape at all, including an m or a bracket on its own', () => {
  assert.equal(withoutTerminalColors(''), '');
  assert.equal(withoutTerminalColors('m[0;32m'), 'm[0;32m');
  assert.equal(withoutTerminalColors('cat'), 'cat');
});
