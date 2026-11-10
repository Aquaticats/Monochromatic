// The watch count of one inotify descriptor, against the fdinfo text the kernel prints for it.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { inotifyWatchCount } from './inotify-fdinfo.ts';

// One line per watch, as /proc/<pid>/fdinfo/<fd> of an inotify descriptor holds it.
const watchLine = (watch: number): string => 'inotify wd:' + watch.toString(16) + ' ino:1a2b3c sdev:fd obj:monochromatic_ide\tmask:3ce owned:0';

test('counts one watch per line the descriptor reports', () => {
  const fdinfo = ['pos:\t0', watchLine(1), watchLine(2), watchLine(3), ''].join('\n');
  assert.equal(inotifyWatchCount({ fdinfo }), 3);
});

test('counts nothing in a descriptor that holds no watch', () => {
  assert.equal(inotifyWatchCount({ fdinfo: '' }), 0);
  assert.equal(inotifyWatchCount({ fdinfo: 'pos:\t0\nflags:\t02\nino:\t4d2\n' }), 0);
});

test('counts only a watch line that starts the line', () => {
  assert.equal(inotifyWatchCount({ fdinfo: 'x inotify wd:1\n  inotify wd:2\n' }), 0);
  assert.equal(inotifyWatchCount({ fdinfo: 'x inotify wd:1\ninotify wd:2\n' }), 1);
});

test('counts a watch line after each line terminator a line start reaches', () => {
  // A bare carriage return, a line separator, and a paragraph separator each start a line for the pattern this
  // read replaced, and none of them is a newline.
  for (const terminator of ['\n', '\r', '\u2028', '\u2029']) {
    assert.equal(inotifyWatchCount({ fdinfo: watchLine(1) + terminator + watchLine(2) }), 2, JSON.stringify(terminator));
  }
});

test('counts a watch line the text ends with, without a final line terminator', () => {
  assert.equal(inotifyWatchCount({ fdinfo: watchLine(1) }), 1);
  assert.equal(inotifyWatchCount({ fdinfo: 'pos:\t0\n' + watchLine(7) }), 1);
  assert.equal(inotifyWatchCount({ fdinfo: 'pos:\t0\n' + watchLine(7) + '\n' }), 1);
});
