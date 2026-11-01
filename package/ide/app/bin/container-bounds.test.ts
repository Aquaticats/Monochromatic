// Checks of the container bound that every task and inspection script of the IDE passes to podman.
// What: the default is the share of a given host, rounded down with floors; an override replaces one value, a
// malformed override throws an error naming the variable and the accepted form, and the thread count and job count
// follow the CPU limit.
// Why: a wrong bound either starves a build (an out-of-memory kill) or lets test threads outnumber the CPU quota.
import assert from 'node:assert/strict';
import { test } from 'node:test';
import {
  ContainerBoundError,
  containerBounds,
  currentHost,
  hostBound,
  pidsFor,
  variableNames,
} from './container-bounds.ts';

const gibibyte = 2 ** 30;
const hostOf = ({ cpus, memoryGiB }: { cpus: number; memoryGiB: number }) => ({ cpus, memoryBytes: memoryGiB * gibibyte });
// The host of the owner's decision: 28 CPUs and 61.83 GiB (66,394,247,168 bytes), a "64 GB" machine.
const ownerHost = { cpus: 28, memoryBytes: 66_394_247_168 };

test('nine tenths of a 28 CPU, 64 GB host is 25 CPUs and 55 GiB', () => {
  assert.deepEqual(hostBound(ownerHost), { cpus: 25, memoryGiB: 55 });
});

test('a one CPU, 2 GB host keeps one CPU and the 2 GiB floor', () => {
  assert.deepEqual(hostBound({ cpus: 1, memoryBytes: 2e9 }), { cpus: 1, memoryGiB: 2 });
  assert.deepEqual(hostBound(hostOf({ cpus: 1, memoryGiB: 2 })), { cpus: 1, memoryGiB: 2 });
});

test('the share is rounded down, never up', () => {
  assert.equal(hostBound(hostOf({ cpus: 3, memoryGiB: 8 })).cpus, 2);
  assert.equal(hostBound(hostOf({ cpus: 5, memoryGiB: 8 })).cpus, 4);
  assert.equal(hostBound(hostOf({ cpus: 8, memoryGiB: 5 })).memoryGiB, 4);
  assert.equal(hostBound(hostOf({ cpus: 8, memoryGiB: 6 })).memoryGiB, 5);
});

test('the process limit keeps a floor of 512 and grows with the CPU count', () => {
  assert.equal(pidsFor(1), 512);
  assert.equal(pidsFor(2), 512);
  assert.equal(pidsFor(3), 768);
  assert.equal(pidsFor(21), 5_376);
});

test('with nothing set, every value is computed from the host', () => {
  const bounds = containerBounds({ environment: {}, host: ownerHost });
  assert.deepEqual(bounds.limitArguments, ['--memory=55g', '--cpus=25', '--pids-limit=6400', '--ulimit', 'nofile=4096:4096']);
  assert.deepEqual(bounds.environmentArguments, ['--env', 'CARGO_BUILD_JOBS=25', '--env', 'NEXTEST_TEST_THREADS=25']);
  assert.deepEqual(bounds.imageBuildArguments, ['--memory=55g', '--cpu-period=100000', '--cpu-quota=2500000']);
});

test('a small host yields the conservative bound the repository used before', () => {
  const bounds = containerBounds({ environment: {}, host: hostOf({ cpus: 2, memoryGiB: 2 }) });
  assert.deepEqual(bounds.limitArguments, ['--memory=2g', '--cpus=1', '--pids-limit=512', '--ulimit', 'nofile=4096:4096']);
  assert.deepEqual(bounds.environmentArguments, ['--env', 'CARGO_BUILD_JOBS=1', '--env', 'NEXTEST_TEST_THREADS=1']);
});

test('setting the CPU count and memory reproduces the earlier fixed bound', () => {
  const bounds = containerBounds({
    environment: { IDE_CONTAINER_CPUS: '2', IDE_CONTAINER_MEMORY: '2g' },
    host: ownerHost,
  });
  assert.deepEqual(bounds.limitArguments, ['--memory=2g', '--cpus=2', '--pids-limit=512', '--ulimit', 'nofile=4096:4096']);
  assert.deepEqual(bounds.environmentArguments, ['--env', 'CARGO_BUILD_JOBS=2', '--env', 'NEXTEST_TEST_THREADS=2']);
  assert.deepEqual(bounds.imageBuildArguments, ['--memory=2g', '--cpu-period=100000', '--cpu-quota=200000']);
});

test('the image build uses the same CPU count and memory as the run', () => {
  const bounds = containerBounds({
    environment: { IDE_CONTAINER_CPUS: '6', IDE_CONTAINER_MEMORY: '12g' },
    host: ownerHost,
  });
  assert.ok(bounds.limitArguments.includes('--memory=12g'));
  assert.ok(bounds.limitArguments.includes('--cpus=6'));
  assert.ok(bounds.imageBuildArguments.includes('--memory=12g'));
  assert.ok(bounds.imageBuildArguments.includes('--cpu-quota=600000'));
});

test('jobs and test threads follow the CPU limit as whole CPUs, at least one', () => {
  const half = containerBounds({ environment: { IDE_CONTAINER_CPUS: '1.5' }, host: ownerHost });
  assert.equal(half.cpus, '1.5');
  assert.equal(half.jobs, '1');
  assert.equal(half.testThreads, '1');
  assert.ok(half.imageBuildArguments.includes('--cpu-quota=150000'));
  const fraction = containerBounds({ environment: { IDE_CONTAINER_CPUS: '0.5' }, host: ownerHost });
  assert.equal(fraction.jobs, '1');
  assert.equal(fraction.testThreads, '1');
});

test('jobs, test threads, processes, and open files can each be overridden', () => {
  const bounds = containerBounds({
    environment: {
      IDE_CONTAINER_CPUS: '4',
      IDE_CONTAINER_JOBS: '3',
      IDE_CONTAINER_TEST_THREADS: '2',
      IDE_CONTAINER_PIDS: '900',
      IDE_CONTAINER_NOFILE: '8192',
    },
    host: ownerHost,
  });
  assert.equal(bounds.jobs, '3');
  assert.equal(bounds.testThreads, '2');
  assert.ok(bounds.limitArguments.includes('--pids-limit=900'));
  assert.ok(bounds.limitArguments.includes('nofile=8192:8192'));
});

test('an empty variable counts as unset', () => {
  const bounds = containerBounds({ environment: { IDE_CONTAINER_CPUS: '', IDE_CONTAINER_MEMORY: '' }, host: ownerHost });
  assert.equal(bounds.cpus, '25');
  assert.equal(bounds.memory, '55g');
});

const malformed: [string, string][] = [
  ['IDE_CONTAINER_CPUS', 'many'],
  ['IDE_CONTAINER_CPUS', '0'],
  ['IDE_CONTAINER_CPUS', '0.0'],
  ['IDE_CONTAINER_CPUS', '-2'],
  ['IDE_CONTAINER_CPUS', '2 '],
  ['IDE_CONTAINER_CPUS', '1e3'],
  ['IDE_CONTAINER_CPUS', '1.234'],
  ['IDE_CONTAINER_MEMORY', '2'],
  ['IDE_CONTAINER_MEMORY', 'lots'],
  ['IDE_CONTAINER_MEMORY', '2gb'],
  ['IDE_CONTAINER_MEMORY', '0g'],
  ['IDE_CONTAINER_MEMORY', '-1g'],
  ['IDE_CONTAINER_MEMORY', '2g --privileged'],
  ['IDE_CONTAINER_PIDS', '0'],
  ['IDE_CONTAINER_PIDS', '-1'],
  ['IDE_CONTAINER_PIDS', '5.5'],
  ['IDE_CONTAINER_NOFILE', '4096:4096'],
  ['IDE_CONTAINER_JOBS', 'all'],
  ['IDE_CONTAINER_TEST_THREADS', 'num-cpus'],
];

for (const [name, value] of malformed) {
  test(`${name}="${value}" is refused with the variable named`, () => {
    assert.throws(
      () => containerBounds({ environment: { [name]: value }, host: ownerHost }),
      error => (error instanceof ContainerBoundError) && error.message.includes(name) && error.message.includes(`"${value}"`) && error.message.includes('Use '),
    );
  });
}

test('the error for memory states the accepted form', () => {
  assert.throws(
    () => containerBounds({ environment: { IDE_CONTAINER_MEMORY: 'big' }, host: ownerHost }),
    /IDE_CONTAINER_MEMORY is "big".*a whole number with the unit k, m, or g, such as 512m or 8g/,
  );
});

test('the variables are the ones the README names', () => {
  assert.deepEqual(Object.values(variableNames).sort(), [
    'IDE_CONTAINER_CPUS',
    'IDE_CONTAINER_JOBS',
    'IDE_CONTAINER_MEMORY',
    'IDE_CONTAINER_NOFILE',
    'IDE_CONTAINER_PIDS',
    'IDE_CONTAINER_TEST_THREADS',
  ]);
});

test('this host yields a bound with whole, positive numbers', () => {
  const host = currentHost();
  assert.ok(Number.isInteger(host.cpus) && (host.cpus >= 1));
  assert.ok(host.memoryBytes > 0);
  const bound = hostBound(host);
  assert.ok((bound.cpus >= 1) && (bound.cpus <= host.cpus));
  assert.ok(bound.memoryGiB >= 2);
});
