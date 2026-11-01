#!/usr/bin/env node
// Measure how often a Language lifetime test finds a server process left behind (running or as a zombie)
// after the worker handle was dropped, with the worker's reaping step and with the fixed pause it replaced.
// Everything runs in a disposable package copy. Each measurement starts RUNS test processes, PARALLEL at a
// time, inside one container bounded to 2 GiB, 2 CPUs, and 512 processes with no network; the parallel
// processes share the two CPUs, which is the contention that makes a killed process slow to end.
// The two variants alternate for ROUNDS rounds, so a change in machine load during the task reaches both.
// Compare rounds with each other before comparing variants: on a machine whose disk stalls, one round
// differed from the next by more than the variants did.
import { spawnSync, type SpawnSyncReturns } from 'node:child_process';
import { cpSync, existsSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, realpathSync, statSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join, relative, sep } from 'node:path';
import { containerBounds } from './container-bounds.ts';

// The bound of every container this script starts: the shared computed share of the host, or the caller's IDE_CONTAINER_* values.
const containerLimits = containerBounds();

const privateRoot = join(homedir(), 'temp', 'agent');
if ((statSync(privateRoot).mode & 0o077) !== 0) throw new Error('Scratch root must exclude group and other permissions: ' + privateRoot);
const disposable = ({ value, what }: { value: string; what: string }): string => {
  const resolved = realpathSync(value);
  if (!resolved.startsWith(realpathSync(privateRoot) + sep) || !statSync(resolved).isDirectory()) throw new Error(what + ' must be a disposable directory below ' + privateRoot);
  return resolved;
};
if (!process.env.usage_cache) throw new Error('Provide the disposable Cargo target-cache directory');
const cache = disposable({ value: process.env.usage_cache, what: 'Target cache' });
const cargoHome = process.env.usage_cargo ? disposable({ value: process.env.usage_cargo, what: 'Cargo home copy' }) : 'ide-cargo';
const runs = Number(process.env.usage_runs ?? 600);
const parallel = Number(process.env.usage_parallel ?? 16);
const rounds = Number(process.env.usage_rounds ?? 2);
if (![runs, parallel, rounds].every(value => Number.isInteger(value) && value >= 1)) throw new Error('runs, parallel, and rounds must be positive whole numbers');
const artifact = mkdtempSync(join(privateRoot, 'ide-language-reap-rate-'));
const source = join(artifact, 'package');
const origin = process.cwd();
cpSync(origin, source, { recursive: true, filter: path => !['target', '.git', 'node_modules'].includes(relative(origin, path).split(sep)[0] ?? '') });
mkdirSync(join(source, 'target'), { recursive: true });
console.log('LANGUAGE_REAP_RATE_ARTIFACT=' + artifact);

const bounded = ({ volumes, command }: { volumes: string[]; command: string[] }): SpawnSyncReturns<string> => spawnSync('podman', [
  'run', '--rm', '--network=none', ...containerLimits.limitArguments,
  '--security-opt', 'label=disable',
  '--volume', source + ':/work', '--volume', cache + ':/work/target', ...volumes,
  '--workdir', '/work', ...containerLimits.environmentArguments, '--env', 'HELIX_RUNTIME=/work/target/debug/runtime',
  'localhost/monochromatic/ide', ...command,
], { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024 });
// The tests whose last step is "no child process is left": two servers that end by themselves, and one that must be killed.
const tests = ['lifecycle::dropping_the_worker_leaves_no_child_process', 'lifecycle::server_that_ignores_exit_is_killed_and_reaped_before_the_drop_returns'];
interface ReapResult { label: string; test: string; runs: number; parallel: number; zombie: number; running: number; other: number; ioPressure: string }
const results: ReapResult[] = [];
const measure = (label: string): void => {
  const build = bounded({ volumes: ['--volume', cargoHome + ':/cargo'], command: ['cargo', 'test', '--offline', '--no-default-features', '--test', 'language', '--no-run'] });
  writeFileSync(join(artifact, label + '-build.log'), (build.stdout ?? '') + (build.stderr ?? ''));
  if (build.status !== 0) throw new Error('The test binary did not build for ' + label + '; inspect ' + artifact);
  // Cargo names the executable it just built in its last "Executable" line.
  const binary = /Executable tests\/language\.rs \((target\/debug\/deps\/language-[0-9a-f]+)\)/.exec(build.stderr ?? '')?.[1];
  if (!binary) throw new Error('Cargo did not name the language test executable for ' + label);
  for (const test of tests) {
    const out = join(artifact, label + '-' + test.split('::')[1]);
    mkdirSync(join(out, 'runs'), { recursive: true });
    // xargs replaces {} with the run number; a failed run leaves a marker beside its log.
    const one = '/work/' + binary + ' ' + test + ' --exact --nocapture > /out/runs/{}.log 2>&1 || touch /out/runs/{}.failed';
    const loop = bounded({ volumes: ['--volume', out + ':/out'], command: ['bash', '-c', 'seq ' + runs + ' | xargs --max-procs=' + parallel + " -I{} bash -c '" + one + "'"] });
    if (loop.error) throw loop.error;
    const failed = readdirSync(join(out, 'runs')).filter(name => name.endsWith('.failed')).map(name => name.slice(0, -7));
    // What a failed run left behind decides who can do something about it:
    // - zombie: the process ended and nobody collected it, which is the worker's reaping;
    // - running: the kernel had not ended the killed process when the test stopped waiting (state D or R);
    // - other: no leftover child, for example a server that was not ready in time under the load.
    const kinds = { zombie: 0, running: 0, other: 0 };
    for (const run of failed) {
      const left = /child process(?:es remain after the worker was dropped| was left when the drop returned): \[(.*)\]/.exec(readFileSync(join(out, 'runs', run + '.log'), 'utf8'))?.[1];
      kinds[left === undefined ? 'other' : (left.includes("'Z'") ? 'zombie' : 'running')] += 1;
    }
    // Machine-wide I/O pressure when the loop ended: stalls there keep a killed process from ending.
    const pressure = existsSync('/proc/pressure/io') ? readFileSync('/proc/pressure/io', 'utf8').split('\n')[0] ?? '' : 'unavailable';
    results.push({ label, test, runs, parallel, ...kinds, ioPressure: pressure });
    writeFileSync(join(artifact, 'results.json'), JSON.stringify(results, null, 2));
    console.log(JSON.stringify(results.at(-1)));
  }
};
const replaceOne = ({ text, before, after }: { text: string; before: string; after: string }): string => {
  const start = text.indexOf(before);
  if (start < 0) throw new Error('Mutation source anchor is absent: ' + before);
  if (text.indexOf(before, start + before.length) >= 0) throw new Error('Mutation source anchor is ambiguous: ' + before);
  return text.slice(0, start) + after + text.slice(start + before.length);
};
// The shutdown as it was: a fixed 50 ms pause inside the worker's runtime, and no reaping after it.
const worker = join(source, 'src/language/worker.rs');
const current = readFileSync(worker, 'utf8');
const former = replaceOne({
  text: replaceOne({ text: current, before: '            drop(runtime);\n            reap::finish(REAP_GRACE);\n', after: '' }),
  before: '        drop(self.session);\n        drop(self.registry);\n    }',
  after: '        drop(self.session);\n        drop(self.registry);\n        tokio::time::sleep(Duration::from_millis(50)).await;\n    }',
});
try {
  for (let round = 1; round <= rounds; round += 1) {
    writeFileSync(worker, current);
    measure('reaping-' + round);
    writeFileSync(worker, former);
    measure('fixed-pause-' + round);
  }
} finally { writeFileSync(worker, current); }
const total = ({ prefix, test, kind }: { prefix: string; test: string; kind: 'zombie' | 'running' | 'other' }): number => results.filter(item => item.label.startsWith(prefix) && item.test === test).reduce((sum, item) => sum + item[kind], 0);
for (const test of tests) {
  for (const variant of ['fixed-pause', 'reaping']) console.log(test + ' with ' + variant + ', of ' + runs * rounds + ' runs: ' + (['zombie', 'running', 'other'] as const).map(kind => total({ prefix: variant, test, kind }) + ' ' + kind).join(', '));
}
console.log('Language reap rates: ' + join(artifact, 'results.json'));
