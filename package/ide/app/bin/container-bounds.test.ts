// Checks of the container bound that every task and inspection script of the IDE passes to podman.
// What: the default is the share of a given host, rounded down with floors; an override replaces one value, a
// malformed override throws an error naming the variable and the accepted form, and the thread count and job count
// follow the CPU limit.
// Why: a wrong bound either starves a build (an out-of-memory kill) or lets test threads outnumber the CPU quota.
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { join } from 'node:path';
import { test } from 'node:test';
import {
  ContainerBoundError,
  containerBounds,
  cpuPeriod,
  currentHost,
  defaultOpenFiles,
  hostBound,
  hostShare,
  minimumMemoryGiB,
  minimumPids,
  pidsFor,
  pidsPerCpu,
  variableNames,
} from './container-bounds.ts';

const gibibyte = 2 ** 30;
const hostOf = ({ cpus, memoryGiB }: { cpus: number; memoryGiB: number }) => ({ cpus, memoryBytes: memoryGiB * gibibyte });
// The host of the owner's decision: 28 CPUs and 61.83 GiB (66,394,247,168 bytes), a "64 GB" machine.
const ownerHost = { cpus: 28, memoryBytes: 66_394_247_168 };
// The package root, beside the module and this test, so the documented files are found from any working directory.
const packageDirectory = join(import.meta.dirname, '..');
// Every IDE_CONTAINER_* name one document lists, without repeats and sorted, so the documents and the module compare
// as sets. A rename in one document alone leaves the old name there and the new one here.
const documentedVariableNames = (file: string): string[] => {
  const text = readFileSync(join(packageDirectory, file), 'utf8');
  return [...new Set(text.match(/IDE_CONTAINER_[A-Z0-9_]+/g) ?? [])].sort();
};
// The bound the module computes from a host: the named share rounded down, with the named floors.
const sharedBound = (host: { cpus: number; memoryBytes: number }) => ({
  cpus: Math.max(1, Math.floor(host.cpus * hostShare)),
  memoryGiB: Math.max(minimumMemoryGiB, Math.floor((host.memoryBytes * hostShare) / gibibyte)),
});
// The `podman run` limits for a memory size, CPU count, and process limit, in the order the tasks pass them.
const runLimits = (memory: string, cpus: string, pids: string): string[] => [
  `--memory=${memory}`,
  `--cpus=${cpus}`,
  `--pids-limit=${pids}`,
  '--ulimit',
  `nofile=${String(defaultOpenFiles)}:${String(defaultOpenFiles)}`,
];
// The `podman build` limits for a memory size and a CPU count, in the order the tasks pass them.
const buildLimits = (memory: string, cpus: number): string[] => [
  `--memory=${memory}`,
  `--cpu-period=${String(cpuPeriod)}`,
  `--cpu-quota=${String(Math.floor(cpus * cpuPeriod))}`,
];
// The `--env` pairs that size cargo and the test runner to a whole CPU count, in the order the tasks pass them.
const followingEnvironment = (cpus: number): string[] => ['--env', `CARGO_BUILD_JOBS=${String(cpus)}`, '--env', `NEXTEST_TEST_THREADS=${String(cpus)}`];

test('the default is the host share of a 28 CPU, 64 GB host', () => {
  assert.deepEqual(hostBound(ownerHost), sharedBound(ownerHost));
});

test('a one CPU, 2 GB host keeps one CPU and the memory floor', () => {
  assert.deepEqual(hostBound({ cpus: 1, memoryBytes: 2e9 }), { cpus: 1, memoryGiB: minimumMemoryGiB });
  assert.deepEqual(hostBound(hostOf({ cpus: 1, memoryGiB: minimumMemoryGiB })), { cpus: 1, memoryGiB: minimumMemoryGiB });
});

test('the share is rounded down, never up', () => {
  for (const cpus of [3, 5]) assert.equal(hostBound(hostOf({ cpus, memoryGiB: 8 })).cpus, Math.max(1, Math.floor(cpus * hostShare)));
  for (const memoryGiB of [5, 6]) {
    assert.equal(hostBound(hostOf({ cpus: 8, memoryGiB })).memoryGiB, Math.max(minimumMemoryGiB, Math.floor(memoryGiB * hostShare)));
  }
});

test('the process limit is the larger of its floor and its per-CPU count', () => {
  for (const cpus of [1, 2, 3, 21]) assert.equal(pidsFor(cpus), Math.max(minimumPids, cpus * pidsPerCpu));
});

test('with nothing set, every value is computed from the host', () => {
  const bound = sharedBound(ownerHost);
  const cpus = String(bound.cpus);
  const memory = `${String(bound.memoryGiB)}g`;
  const bounds = containerBounds({ environment: {}, host: ownerHost });
  assert.deepEqual(bounds.limitArguments, runLimits(memory, cpus, String(pidsFor(bound.cpus))));
  assert.deepEqual(bounds.environmentArguments, followingEnvironment(bound.cpus));
  assert.deepEqual(bounds.imageBuildArguments, buildLimits(memory, bound.cpus));
});

test('a small host yields the conservative bound the repository used before', () => {
  const host = hostOf({ cpus: 2, memoryGiB: 2 });
  const bound = sharedBound(host);
  const cpus = String(bound.cpus);
  const bounds = containerBounds({ environment: {}, host });
  assert.deepEqual(bounds.limitArguments, runLimits(`${String(bound.memoryGiB)}g`, cpus, String(pidsFor(bound.cpus))));
  assert.deepEqual(bounds.environmentArguments, followingEnvironment(bound.cpus));
});

test('setting the CPU count and memory reproduces the earlier fixed bound', () => {
  const cpus = 2;
  const memory = '2g';
  const bounds = containerBounds({
    environment: { IDE_CONTAINER_CPUS: String(cpus), IDE_CONTAINER_MEMORY: memory },
    host: ownerHost,
  });
  assert.deepEqual(bounds.limitArguments, runLimits(memory, String(cpus), String(pidsFor(cpus))));
  assert.deepEqual(bounds.environmentArguments, followingEnvironment(cpus));
  assert.deepEqual(bounds.imageBuildArguments, buildLimits(memory, cpus));
});

test('the image build uses the same CPU count and memory as the run', () => {
  const cpus = 6;
  const memory = '12g';
  const bounds = containerBounds({
    environment: { IDE_CONTAINER_CPUS: String(cpus), IDE_CONTAINER_MEMORY: memory },
    host: ownerHost,
  });
  assert.ok(bounds.limitArguments.includes(`--memory=${memory}`));
  assert.ok(bounds.limitArguments.includes(`--cpus=${String(cpus)}`));
  assert.ok(bounds.imageBuildArguments.includes(`--memory=${memory}`));
  assert.ok(bounds.imageBuildArguments.includes(`--cpu-quota=${String(cpus * cpuPeriod)}`));
});

test('jobs and test threads follow the CPU limit as whole CPUs, at least one', () => {
  const half = containerBounds({ environment: { IDE_CONTAINER_CPUS: '1.5' }, host: ownerHost });
  assert.equal(half.cpus, '1.5');
  assert.equal(half.jobs, '1');
  assert.equal(half.testThreads, '1');
  assert.ok(half.imageBuildArguments.includes(`--cpu-quota=${String(1.5 * cpuPeriod)}`));
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
  const bound = sharedBound(ownerHost);
  const bounds = containerBounds({ environment: { IDE_CONTAINER_CPUS: '', IDE_CONTAINER_MEMORY: '' }, host: ownerHost });
  assert.equal(bounds.cpus, String(bound.cpus));
  assert.equal(bounds.memory, `${String(bound.memoryGiB)}g`);
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

test('the variables are the ones the README and mise.toml name', () => {
  const accepted: string[] = [...Object.values(variableNames)].sort();
  for (const file of ['README.md', 'mise.toml']) {
    assert.deepEqual(documentedVariableNames(file), accepted, `${file} names different IDE_CONTAINER_* variables than bin/container-bounds.ts`);
  }
});

test('this host yields a bound with whole, positive numbers', () => {
  const host = currentHost();
  assert.ok(Number.isInteger(host.cpus) && (host.cpus >= 1));
  assert.ok(host.memoryBytes > 0);
  const bound = hostBound(host);
  assert.ok((bound.cpus >= 1) && (bound.cpus <= host.cpus));
  assert.ok(bound.memoryGiB >= minimumMemoryGiB);
});
