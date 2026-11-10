// The columns of one `strace --summary-only` row, against the rows a real strace run prints.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { isPrintedDecimal, reportedColumns } from './strace-summary.ts';

// A row of `strace --follow-forks --summary-only` output: percent of time, seconds, microseconds per call, calls,
// errors, and the system call's name.
const row = '% time     seconds  usecs/call     calls    errors syscall';
const call = ' 65.42    0.000123           4       123           openat';

test('reads the columns of a summary header row and of a call row', () => {
  assert.deepEqual(reportedColumns(row), ['%', 'time', 'seconds', 'usecs/call', 'calls', 'errors', 'syscall']);
  assert.deepEqual(reportedColumns(call), ['65.42', '0.000123', '4', '123', 'openat']);
});

test('reads no empty column between columns several characters apart, or around the row', () => {
  assert.deepEqual(reportedColumns('   a \t\t b   c  '), ['a', 'b', 'c']);
  assert.deepEqual(reportedColumns('a\u00a0b'), ['a', 'b']);
});

test('reads no columns from a row that holds only whitespace', () => {
  // A pattern split answers this row with one empty word; the caller asks for at least five columns, so neither
  // answer reaches its counters.
  assert.deepEqual(reportedColumns(''), []);
  assert.deepEqual(reportedColumns(' \t '), []);
});

test('keeps a column that holds a character whitespace never separates', () => {
  assert.deepEqual(reportedColumns('a/b.c-d_e'), ['a/b.c-d_e']);
});

test('accepts the seconds column of a real summary row, and a run of digits and points', () => {
  for (const column of ['0.000123', '0.000144', '12', '.5', '5.', '..', '0']) assert.equal(isPrintedDecimal(column), true, JSON.stringify(column));
});

test('rejects a column that holds anything but digits and points, and an empty one', () => {
  for (const column of ['', ' ', '-1', '1,2', '1e3', '1 2', '0x1', '\u0667', '1.2\t']) assert.equal(isPrintedDecimal(column), false, JSON.stringify(column));
});
