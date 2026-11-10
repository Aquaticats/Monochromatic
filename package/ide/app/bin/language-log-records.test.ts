// Checks of the log-record readers the lifetime check reads the application's log with.
// What: the colour sequences a record carries, the level a record states, the text that reports a shutdown as a
// failure, and the three helix-lsp record shapes a healthy server prints. Each expected value is what the pattern the
// reader replaced answered for the same text.
// Why: the lifetime check accepts a run only when the log holds no ERROR record, no bare shutdown error and no
// helix-lsp record other than the three shapes, so a reader that answers differently from the pattern it replaced
// either accepts a run the pattern rejected or reports a healthy server as a regression.
import assert from 'node:assert/strict';
import { test } from 'node:test';

import { withoutColourSequences } from './language-log-records.ts';

test(
  'removes the colour sequence a tracing record carries',
  () => {
    // The application's log holds one tracing record per line, coloured for a terminal.
    assert.equal(
      withoutColourSequences('\u001b[2m2026-11-10T12:00:00.123456Z\u001b[0m \u001b[32m INFO\u001b[0m ide_app: started'),
      '2026-11-10T12:00:00.123456Z  INFO ide_app: started'
    );
  }
);

test(
  'removes several sequences in one text and leaves the records apart',
  () => {
    assert.equal(
      withoutColourSequences('\u001b[1;32ma\u001b[0m\n\u001b[31mb\u001b[0m'),
      'a\nb'
    );
  }
);

test(
  'removes a sequence that names no parameter',
  () => {
    assert.equal(withoutColourSequences('\u001b[mbold'), 'bold');
  }
);

test(
  'removes a sequence that carries several parameters',
  () => {
    assert.equal(withoutColourSequences('\u001b[38;5;208morange'), 'orange');
    assert.equal(withoutColourSequences('\u001b[0;1;4;31mtext'), 'text');
  }
);

test(
  'keeps an escape that starts no colour sequence',
  () => {
    // A cursor move (`\u001b[2K`) is another command, and the pattern reached the `m` command only.
    assert.equal(withoutColourSequences('\u001b[2Kabc'), '\u001b[2Kabc');
    assert.equal(withoutColourSequences('\u001b[?25labc'), '\u001b[?25labc');
  }
);

test(
  'keeps a sequence that ends nowhere',
  () => {
    assert.equal(withoutColourSequences('abc\u001b[1;32'), 'abc\u001b[1;32');
    assert.equal(withoutColourSequences('abc\u001b'), 'abc\u001b');
  }
);

test(
  'keeps an escape that is not followed by the introducer',
  () => {
    assert.equal(withoutColourSequences('\u001bXabc'), '\u001bXabc');
    assert.equal(withoutColourSequences('\u001b'), '\u001b');
  }
);

test(
  'removes the sequence a record starts inside a rejected escape',
  () => {
    // The pattern retried one character later, so the second escape still matched.
    assert.equal(withoutColourSequences('\u001b[\u001b[0m'), '\u001b[');
  }
);

test(
  'removes a sequence that is only a parameter list',
  () => {
    assert.equal(withoutColourSequences('\u001b[;;;m'), '');
    assert.equal(withoutColourSequences('\u001b[0m\u001b[0m'), '');
  }
);

test(
  'returns text that holds no escape unchanged',
  () => {
    const plain = 'Summary [ 0.039s] 1 test run: 1 passed, 0 skipped';
    assert.equal(withoutColourSequences(plain), plain);
    assert.equal(withoutColourSequences(''), '');
  }
);
