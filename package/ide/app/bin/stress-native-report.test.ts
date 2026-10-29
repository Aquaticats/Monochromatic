// Tests of the stress tally against nextest's real status-line format (cargo-nextest 0.9.143 output, 2026-10-20).
import assert from 'node:assert/strict';
import { test } from 'node:test';
import { summarize } from './stress-native-report.ts';

const pass = (name: string): string => `        PASS [   0.114s] [1/3] (1/2) ide::bin/monochromatic-ide ${name}`;

test('counts passes per test over all logs', () => {
  const tally = summarize([`${pass('native::a::one')  }\n${  pass('native::a::one')}`, pass('native::a::one')]);
  assert.equal(tally.failures, 0);
  assert.match(tally.report, /native::a::one: 0 failed, 0 passed after a retry \(FLAKY\), 3 passed, of 3 runs/);
});

test('keeps the first failure output and stops at the next status line', () => {
  const log = [
    pass('native::a::one'),
    '        FAIL [   2.000s] [1/3] (2/2) ide::bin/monochromatic-ide native::a::two',
    '  stderr ───',
    "    thread 'native::a::two' panicked at src/native/a.rs:9:5:",
    '    the frame moved',
    pass('native::a::three'),
  ].join('\n');
  const tally = summarize([log]);
  assert.equal(tally.failures, 1);
  assert.match(tally.report, /native::a::two: 1 failed/);
  assert.match(tally.report, /the frame moved/);
  assert.doesNotMatch(tally.report, /first failure:[^]*native::a::three: /);
  assert.doesNotMatch(tally.report, /first failure:[^]*monochromatic-ide native::a::three/);
});

test('a pass after a retry is counted apart from a pass, and a retry attempt is not a result', () => {
  const log = [
    '  TRY 1 FAIL [   0.500s] [1/1] (───) ide::bin/monochromatic-ide native::a::flaky',
    '  TRY 2 PASS [   0.700s] [1/1] (1/1) ide::bin/monochromatic-ide native::a::flaky',
  ].join('\n');
  const tally = summarize([log]);
  assert.equal(tally.failures, 0);
  assert.match(tally.report, /native::a::flaky: 0 failed, 1 passed after a retry \(FLAKY\), 0 passed, of 1 runs/);
});

test('a test that fails every attempt is one failure, with its first attempt kept', () => {
  const log = [
    '  TRY 1 FAIL [   0.500s] [1/1] (───) ide::bin/monochromatic-ide native::a::broken',
    '    first attempt output',
    '  TRY 2 FAIL [   0.500s] [1/1] (1/1) ide::bin/monochromatic-ide native::a::broken',
    '    second attempt output',
  ].join('\n');
  const tally = summarize([log]);
  assert.equal(tally.failures, 1);
  assert.match(tally.report, /native::a::broken: 1 failed, 0 passed after a retry \(FLAKY\), 0 passed, of 1 runs \(2 failed attempts/);
  assert.match(tally.report, /first attempt output/);
});

test('a test counter padded with spaces, as in a run of more than nine tests, still marks a final result', () => {
  const log = [
    '        FAIL [   3.427s] [28/40] (15/36) ide::bin/monochromatic-ide native::a::padded',
    '        FAIL [   0.994s] [18/40] ( 7/36) ide::bin/monochromatic-ide native::a::padded',
    '        PASS [   0.994s] [19/40] ( 8/36) ide::bin/monochromatic-ide native::a::padded',
  ].join('\n');
  const tally = summarize([log]);
  assert.equal(tally.failures, 2);
  assert.match(tally.report, /native::a::padded: 2 failed, 0 passed after a retry \(FLAKY\), 1 passed, of 3 runs/);
});

test('a log with no result is a failure, not a pass', () => {
  assert.equal(summarize(['nothing here']).failures, 1);
});
