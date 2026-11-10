// The ten-second averages one /proc/pressure file reports, against the text the kernel prints there.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { PressureLineError, pressureShares } from './pressure-shares.ts';

// The lines a real /proc/pressure/cpu file holds, with its field names.
const cpu = [
  'some avg10=0.12 avg60=0.05 avg300=0.01 total=7543',
  'full avg10=0.00 avg60=0.00 avg300=0.00 total=0',
  '',
].join('\n');

test('reads one share per line, keyed by the resource and the line label', () => {
  assert.deepEqual(pressureShares({ resource: 'cpu', text: cpu }), { cpu_some: 0.12, cpu_full: 0 });
});

test('reads a share the kernel prints without a whole part, and a whole one', () => {
  assert.deepEqual(pressureShares({ resource: 'io', text: 'some avg10=.5 total=1\nfull avg10=12 total=2' }), { io_some: 0.5, io_full: 12 });
});

test('reads the second occurrence when the first names no number', () => {
  assert.deepEqual(pressureShares({ resource: 'cpu', text: 'some avg10= avg10=1.5 total=1' }), { cpu_some: 1.5 });
});

test('reports the line a reader cannot use, naming it', () => {
  for (const line of ['some avg60=0.00 total=0', 'some avg10= total=0', 'some avg10=x total=0']) {
    assert.throws(
      () => pressureShares({ resource: 'cpu', text: line + '\n' }),
      (error: unknown) => error instanceof PressureLineError && error.message.includes(line),
    );
  }
});
