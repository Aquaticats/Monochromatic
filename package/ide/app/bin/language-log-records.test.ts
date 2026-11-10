// Checks of the log-record readers the lifetime check reads the application's log with.
// What: the colour sequences a record carries, the level a record states, the text that reports a shutdown as a
// failure, and the three helix-lsp record shapes a healthy server prints. Each expected value is what the pattern the
// reader replaced answered for the same text.
// Why: the lifetime check accepts a run only when the log holds no ERROR record, no bare shutdown error and no
// helix-lsp record other than the three shapes, so a reader that answers differently from the pattern it replaced
// either accepts a run the pattern rejected or reports a healthy server as a regression.
import assert from 'node:assert/strict';
import { test } from 'node:test';

import { isBareShutdownError, isServerStandardErrorLine, levelOf, withoutColourSequences } from './language-log-records.ts';

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

test(
  'reads the level a tracing record states',
  () => {
    // One record of the application's log: a timestamp, a level, the target and the message.
    assert.equal(levelOf('2026-11-10T12:00:00.123456Z  INFO ide_app::language: started'), 'INFO');
    assert.equal(levelOf('2026-11-10T12:00:00.123456Z  WARN ide_app::language: slow'), 'WARN');
    assert.equal(levelOf('2026-11-10T12:00:00.123456Z ERROR helix_lsp::transport: refused'), 'ERROR');
    assert.equal(levelOf('2026-11-10T12:00:00.123456Z DEBUG helix_lsp::transport: -> initialized'), 'DEBUG');
    assert.equal(levelOf('2026-11-10T12:00:00.123456Z TRACE helix_lsp::transport: <- shutdown'), 'TRACE');
  }
);

test(
  'reads the level of a record the worker wrote',
  () => {
    // The worker's own records carry a process name rather than a timestamp.
    assert.equal(levelOf('language-worker  INFO ide_app::language: started'), 'INFO');
  }
);

test(
  'reads a level followed by a tab',
  () => {
    assert.equal(levelOf('2026-11-10T12:00:00Z  INFO\tide_app: started'), 'INFO');
  }
);

test(
  'reads the first level of a line that states two',
  () => {
    assert.equal(levelOf('a INFO b WARN c'), 'INFO');
  }
);

test(
  'reads the level a record states after the whitespace the pattern reached',
  () => {
    assert.equal(levelOf('2026-11-10T12:00:00Z\u00a0INFO ide_app: started'), 'INFO');
    assert.equal(levelOf('2026-11-10T12:00:00Z\u3000WARN ide_app: slow'), 'WARN');
  }
);

test(
  'reads no level from a line that states none',
  () => {
    assert.equal(levelOf('no level here'), undefined);
    assert.equal(levelOf(''), undefined);
    assert.equal(levelOf('2026-11-10T12:00:00Z  NOTICE ide_app: started'), undefined);
    assert.equal(levelOf('2026-11-10T12:00:00Z  info ide_app: started'), undefined);
    assert.equal(levelOf('2026-11-10T12:00:00Z INFOMORE ide_app: started'), undefined);
  }
);

test(
  'reads no level from a line that states one where the pattern found no field',
  () => {
    // An anchored `\S+` needs a field before the level, and one whitespace character after it.
    assert.equal(levelOf(' INFO ide_app: started'), undefined);
    assert.equal(levelOf('INFO ide_app: started'), undefined);
    assert.equal(levelOf('2026-11-10T12:00:00Z  INFO'), undefined);
    assert.equal(levelOf('abc'), undefined);
  }
);

test(
  'reads no level across a character the whitespace set never reached',
  () => {
    // A zero-width space is part of the field, so the level no longer follows a run of whitespace.
    assert.equal(levelOf('2026-11-10T12:00:00Z\u200bINFO ide_app: started'), undefined);
  }
);

test(
  'reads the phrase a server reports its own shutdown with',
  () => {
    // helix-lsp forwards each line a server writes to standard error into the log, so a server that reports its own
    // shutdown as a failure reaches the log as one of these phrases, in the case the server wrote it in.
    assert.equal(isBareShutdownError('scripted-ls err <- "context canceled"'), true);
    assert.equal(isBareShutdownError('helix_lsp::transport: 123 err: <- StreamClosed'), true);
    assert.equal(isBareShutdownError('write failed: broken pipe'), true);
  }
);

test(
  'reads a phrase in any case the pattern reached',
  () => {
    assert.equal(isBareShutdownError('CONTEXT CANCELED'), true);
    assert.equal(isBareShutdownError('Context Canceled'), true);
    assert.equal(isBareShutdownError('StreamClosed'), true);
    assert.equal(isBareShutdownError('streamclosed'), true);
    assert.equal(isBareShutdownError('Broken Pipe'), true);
    assert.equal(isBareShutdownError('BROKEN PIPE'), true);
  }
);

test(
  'reads a phrase anywhere in a longer text',
  () => {
    assert.equal(isBareShutdownError('a context canceled b'), true);
    assert.equal(isBareShutdownError('Broken pipeline'), true);
  }
);

test(
  'reads no phrase from text that states one with something other than a space between its words',
  () => {
    assert.equal(isBareShutdownError('context  canceled'), false);
    assert.equal(isBareShutdownError('contextcanceled'), false);
    assert.equal(isBareShutdownError('context\tcanceled'), false);
    assert.equal(isBareShutdownError('stream closed'), false);
    assert.equal(isBareShutdownError('brokenpipe'), false);
  }
);

test(
  'reads no phrase from text that states another word or nothing at all',
  () => {
    assert.equal(isBareShutdownError('context cancelled'), false);
    assert.equal(isBareShutdownError('StreamClosures'), false);
    assert.equal(isBareShutdownError('no shutdown error'), false);
    assert.equal(isBareShutdownError(''), false);
  }
);

test(
  'reads no phrase across a character that uppercases to an ASCII letter',
  () => {
    // The `i` flag leaves a character above ASCII unfolded when its uppercase form is an ASCII letter, so a long s
    // never stands for an `s` and a Kelvin sign never stands for a `k`.
    assert.equal(isBareShutdownError('\u017ftreamclosed'), false);
    assert.equal(isBareShutdownError('\u212aey'), false);
  }
);

test(
  'reads the record helix-lsp writes for a line of a server standard error',
  () => {
    // The shapes are the ones doc/troubleshooting/helix-lsp-transport-error-level-records.md captured from real servers.
    assert.equal(isServerStandardErrorLine('helix_lsp::transport: scripted-ls err <- "context canceled\\n"'), true);
    assert.equal(
      isServerStandardErrorLine(
        'helix_lsp::transport: rust-analyzer err <- "2026-10-05T14:50:17.468643919-04:00  WARN notify error: No path was found.\\n"'
      ),
      true
    );
  }
);

test(
  'reads a quoted line that holds quotation marks and no text',
  () => {
    assert.equal(isServerStandardErrorLine('helix_lsp::transport: scripted-ls err <- ""'), true);
    assert.equal(isServerStandardErrorLine('helix_lsp::transport: scripted-ls err <- "a "quoted" b"'), true);
  }
);

test(
  'reads no standard error line from a record that quotes nothing or never closes its quote',
  () => {
    assert.equal(isServerStandardErrorLine('helix_lsp::transport: scripted-ls err <- "unterminated'), false);
    assert.equal(isServerStandardErrorLine('helix_lsp::transport: scripted-ls err <- "a" and more'), false);
    assert.equal(isServerStandardErrorLine('helix_lsp::transport: scripted-ls err <- "a" '), false);
  }
);

test(
  'reads no standard error line from a record that carries a line terminator after the source name',
  () => {
    // What the pattern's `.` reached ends at a line terminator, so a quote after one closes nothing.
    assert.equal(isServerStandardErrorLine('helix_lsp::transport: scripted-ls err <- "a\nb"'), false);
    assert.equal(isServerStandardErrorLine('helix_lsp::transport: scripted-ls err <- "a\rb"'), false);
    assert.equal(isServerStandardErrorLine('helix_lsp::transport: scripted-ls err <- "a\u2028b"'), false);
    assert.equal(isServerStandardErrorLine('helix_lsp::transport: scripted-ls err <- "a\u2029b"'), false);
  }
);

test(
  'reads the standard error line of a record that carries its level in front of the module path',
  () => {
    // The pattern is anchored at the end of the record and not at its start, because a log line states its level and
    // target before the module path, so the shape is found anywhere the line holds it.
    assert.equal(isServerStandardErrorLine('ERROR helix_lsp::transport: scripted-ls err <- "context canceled\\n"'), true);
    assert.equal(isServerStandardErrorLine('2026-11-10T12:00:00.123456Z  WARN helix_lsp::transport: scripted-ls err <- "a"'), true);
  }
);

test(
  'reads the standard error line of a record that holds the module path twice',
  () => {
    // The first place the module path stands is followed by something else, so the scan has to read the second.
    assert.equal(
      isServerStandardErrorLine('helix_lsp::transport: a helix_lsp::transport: b err <- "x"'),
      true
    );
  }
);

test(
  'reads no standard error line from another transport record or another text',
  () => {
    assert.equal(isServerStandardErrorLine('helix_lsp::transport: typescript-native err: <- StreamClosed'), false);
    assert.equal(isServerStandardErrorLine('helix_lsp::transport: rust-analyzer <- ServerError(-32801): content modified'), false);
    assert.equal(isServerStandardErrorLine('helix_lsp::transport: scripted-ls ERR <- "a"'), false);
    assert.equal(isServerStandardErrorLine('helix_lsp::transport:  err <- "a"'), false);
    assert.equal(isServerStandardErrorLine(''), false);
  }
);
