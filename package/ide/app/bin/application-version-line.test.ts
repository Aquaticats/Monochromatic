// Checks of the version line read a lone copy of the application is recognised by.
// What: which output opens with the application's version line, over the line the build prints and over the near
// misses: another program's name, a part that is missing or empty, and a number that does not open the output.
// Why: the check demands this line from a copy of the executable alone in its own folder, with no environment, so a
// read that accepted other output would pass a copy that never started, and one that refused a real line would fail a
// build that works.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { namesApplicationVersion } from './application-version-line.ts';

test('reads the version line the build prints', () => {
  assert.equal(namesApplicationVersion('monochromatic-ide 0.1.0\n'), true);
  assert.equal(namesApplicationVersion('monochromatic-ide 1.2.3'), true);
  assert.equal(namesApplicationVersion('monochromatic-ide 10.20.30\n'), true);
});

test('reads a line whose last part is followed by more text', () => {
  assert.equal(namesApplicationVersion('monochromatic-ide 1.2.3-beta.1\n'), true);
  assert.equal(namesApplicationVersion('monochromatic-ide 1.2.3 (commit 0123abc)\n'), true);
});

test('reads no version line from another program', () => {
  assert.equal(namesApplicationVersion(''), false);
  assert.equal(namesApplicationVersion('monochromatic 1.2.3\n'), false);
  assert.equal(namesApplicationVersion('Monochromatic-ide 1.2.3\n'), false);
  assert.equal(namesApplicationVersion('monochromatic-ide\n'), false);
  assert.equal(namesApplicationVersion('ide 1.2.3\n'), false);
});

test('reads no version line from a number with a part missing or empty', () => {
  assert.equal(namesApplicationVersion('monochromatic-ide 1.2\n'), false);
  assert.equal(namesApplicationVersion('monochromatic-ide 1\n'), false);
  assert.equal(namesApplicationVersion('monochromatic-ide 1.2.\n'), false);
  assert.equal(namesApplicationVersion('monochromatic-ide .1.2\n'), false);
  assert.equal(namesApplicationVersion('monochromatic-ide 1..2\n'), false);
  assert.equal(namesApplicationVersion('monochromatic-ide 1.2.x\n'), false);
});

test('reads no version line when the number does not open the output', () => {
  assert.equal(namesApplicationVersion(' warning: something\nmonochromatic-ide 1.2.3\n'), false);
  assert.equal(namesApplicationVersion('\nmonochromatic-ide 1.2.3\n'), false);
  assert.equal(namesApplicationVersion(' monochromatic-ide 1.2.3\n'), false);
});

test('reads no version line from digits of another script', () => {
  assert.equal(namesApplicationVersion('monochromatic-ide \u0661.\u0662.\u0663\n'), false);
  assert.equal(namesApplicationVersion('monochromatic-ide 1.\uFF12.3\n'), false);
});
