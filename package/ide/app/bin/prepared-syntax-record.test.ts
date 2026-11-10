// Checks of the prepared-syntax record read the bundle checks settle a session by.
// What: the count a session prints for the file the check opened, over the shapes the application's record really
// takes and over the near misses: another file, a missing count, a path that only ends in the opened file's name, and
// the whitespace that ends a path.
// Why: one check settles a session when the record appears and another demands a count from it, so a read that misses
// a real record fails a good session, and one that reads another file's record lets a copy pass on highlighting it
// never did.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { preparedSyntaxSpanCount } from './prepared-syntax-record.ts';

// The record the application prints, as a line among the session's other output.
const record = ({ path, count }: { path: string; count: string }): string =>
  'debug: source syntax prepared path=' + path + ' spans=' + count + '\nstarting window\n';

test('reads the count a session printed for the opened file', () => {
  assert.equal(preparedSyntaxSpanCount(record({ path: '/home/cat/project/fixture.sql', count: '42' })), 42);
  assert.equal(preparedSyntaxSpanCount(record({ path: 'fixture.sql', count: '1' })), 1);
  assert.equal(preparedSyntaxSpanCount(record({ path: '/home/cat/project/fixture.sql', count: '007' })), 7);
});

test('reads a record of no prepared spans as the count zero', () => {
  assert.equal(preparedSyntaxSpanCount(record({ path: '/home/cat/project/fixture.sql', count: '0' })), 0);
});

test('reads the record wherever the line stands in the session output', () => {
  const line = record({ path: '/a/b/fixture.sql', count: '9' });
  assert.equal(preparedSyntaxSpanCount(line), 9);
  assert.equal(preparedSyntaxSpanCount('first line\n' + line + 'last line\n'), 9);
  assert.equal(preparedSyntaxSpanCount('preamble ' + line), 9);
});

test('reads the record of the opened file when another file was recorded first', () => {
  const output = record({ path: '/a/b/other.sql', count: '3' }) + record({ path: '/a/b/fixture.sql', count: '12' });
  assert.equal(preparedSyntaxSpanCount(output), 12);
});

test('reads no record from output that holds none', () => {
  assert.equal(preparedSyntaxSpanCount(''), undefined);
  assert.equal(preparedSyntaxSpanCount('starting window\nhighlighting unavailable; retaining readable source\n'), undefined);
  assert.equal(preparedSyntaxSpanCount('source syntax prepared path=/a/b/other.sql spans=3\n'), undefined);
});

test('reads no record from a line whose count is missing', () => {
  assert.equal(preparedSyntaxSpanCount('source syntax prepared path=/a/b/fixture.sql spans=\n'), undefined);
  assert.equal(preparedSyntaxSpanCount('source syntax prepared path=/a/b/fixture.sql spans=x\n'), undefined);
  assert.equal(preparedSyntaxSpanCount('source syntax prepared path=/a/b/fixture.sql span=3\n'), undefined);
});

test('reads no record from a path that only ends in a name resembling the opened file', () => {
  assert.equal(preparedSyntaxSpanCount('source syntax prepared path=/a/b/fixture.sql.bak spans=3\n'), undefined);
  assert.equal(preparedSyntaxSpanCount('source syntax prepared path=/a/b/xfixture.sql spans=3\n'), 3);
  assert.equal(preparedSyntaxSpanCount('source syntax prepared path=fixture.sq spans=3\n'), undefined);
});

test('ends the path at any character the pattern held as whitespace', () => {
  // A tab, a no-break space and a line separator each end a path, and none of them is the space the record states its
  // count with, so the line is not a record for this file.
  assert.equal(preparedSyntaxSpanCount('source syntax prepared path=/a/b/fixture.sql\tspans=3\n'), undefined);
  assert.equal(preparedSyntaxSpanCount('source syntax prepared path=/a/b/fixture.sql\u00A0spans=3\n'), undefined);
  assert.equal(preparedSyntaxSpanCount('source syntax prepared path=/a/b/fixture.sql\u3000spans=3\n'), undefined);
  // A path of no characters cannot end in the file name.
  assert.equal(preparedSyntaxSpanCount('source syntax prepared path= fixture.sql spans=3\n'), undefined);
});

test('reads the count as far as the digits run and no further', () => {
  assert.equal(preparedSyntaxSpanCount('source syntax prepared path=/a/fixture.sql spans=12abc\n'), 12);
  assert.equal(preparedSyntaxSpanCount('source syntax prepared path=/a/fixture.sql spans=12\n'), 12);
});
