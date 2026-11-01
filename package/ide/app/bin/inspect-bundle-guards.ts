#!/usr/bin/env node
// Observe the bundle checks failing on damaged copies of a single application executable.
// Each case damages its own copy below the private agent scratch root; the checked executable is only read.
// These cases damage the file itself. Checks of run-time behavior (the cache comparison, ignoring a user Helix
// runtime, the digest check of embedded parts) need an executable built without that behavior; the package
// README, under "Bundle checks", records those controls, run with inspect:bundle on a deliberately altered build.
import { spawnSync } from 'node:child_process';
import { chmodSync, copyFileSync, mkdtempSync, readFileSync, realpathSync, statSync, writeFileSync } from 'node:fs';
import { homedir } from 'node:os';
import { join, resolve } from 'node:path';

const privateRoot = join(homedir(), 'temp', 'agent');
if ((statSync(privateRoot).mode & 0o077) !== 0) throw new Error('Guard scratch root must exclude group and other permissions: ' + privateRoot);
const executable = realpathSync(resolve(process.env.usage_file || 'dist/monochromatic-ide'));
const runtime = realpathSync(resolve(process.env.usage_runtime || 'target/release/runtime'));
const artifact = realpathSync(mkdtempSync(join(privateRoot, 'ide-bundle-guard-')));
console.log('BUNDLE_GUARD_ARTIFACT=' + artifact);

// Change one byte in the middle of an embedded file's only occurrence in the copy.
const flipInside = (sourceFile: string) => (copy: string): void => {
  const bytes = readFileSync(copy);
  const needle = readFileSync(sourceFile);
  const at = bytes.indexOf(needle);
  if (at < 0) throw new Error(sourceFile + ' is not embedded in ' + copy);
  if (bytes.indexOf(needle, at + 1) >= 0) throw new Error(sourceFile + ' occurs twice in ' + copy + '; damaging one occurrence would be ambiguous');
  const middle = at + Math.floor(needle.length / 2);
  bytes[middle] = (bytes[middle] ?? 0) ^ 0x01;
  writeFileSync(copy, bytes);
};
// `checks` are the checks run on the damaged copy; `fails` maps each check expected to fail to a phrase of its message.
// Checks named in `checks` but not in `fails` must still pass, which shows a failure is specific to its damage.
interface GuardCase {
  name: string;
  damage: (copy: string) => void;
  checks: string[];
  fails: Record<string, string>;
}
const cases: GuardCase[] = [
  { name: 'executable-bit-cleared', damage: copy => chmodSync(copy, 0o644), checks: ['inventory', 'license-texts'],
    fails: { inventory: 'is not executable', 'license-texts': 'could not run' } },
  { name: 'embedded-query-damaged', damage: flipInside(join(runtime, 'queries/sql/highlights.scm')), checks: ['inventory', 'license-texts', 'lone-copy-highlights'],
    fails: { inventory: 'runtime/queries/sql/highlights.scm', 'lone-copy-highlights': 'runtime/queries/sql/highlights.scm embedded in' } },
  { name: 'embedded-grammar-damaged', damage: flipInside(join(runtime, 'grammars/sql.so')), checks: ['inventory', 'license-texts', 'lone-copy-highlights'],
    fails: { inventory: 'runtime/grammars/sql.so', 'lone-copy-highlights': 'runtime/grammars/sql.so embedded in' } },
  { name: 'embedded-grammar-notice-damaged', damage: flipInside(join(runtime, 'licenses/rust/LICENSE')), checks: ['inventory', 'license-texts'],
    fails: { inventory: 'runtime/licenses/rust/LICENSE', 'license-texts': 'runtime/licenses/rust/LICENSE' } },
  { name: 'embedded-font-notice-damaged', damage: flipInside(resolve('asset/font/Inter-LICENSE.txt')), checks: ['inventory', 'license-texts'],
    fails: { inventory: 'LICENSES/font/Inter-LICENSE.txt', 'license-texts': 'LICENSES/font/Inter-LICENSE.txt' } },
  { name: 'embedded-application-license-damaged', damage: flipInside(resolve('LICENSES/LGPL-3.0-or-later.txt')), checks: ['inventory', 'license-texts'],
    fails: { inventory: 'LICENSES/LGPL-3.0-or-later.txt', 'license-texts': 'LICENSES/LGPL-3.0-or-later.txt' } },
  { name: 'embedded-helix-license-damaged', damage: flipInside(join(runtime, 'Helix-LICENSE')), checks: ['inventory', 'license-texts'],
    fails: { inventory: 'runtime/Helix-LICENSE', 'license-texts': 'runtime/Helix-LICENSE' } },
  { name: 'embedded-crate-licenses-damaged', damage: flipInside(resolve(runtime, '..', '..', 'crate-licenses.json')), checks: ['inventory', 'license-texts', 'crate-licenses'],
    fails: { inventory: 'LICENSES/crates.json', 'license-texts': 'LICENSES/crates.json', 'crate-licenses': 'LICENSES/crates.json' } },
];
const only = process.env.usage_only ? new Set(process.env.usage_only.split(',')) : undefined;
const selected = only ? cases.filter(item => only.has(item.name)) : cases;
if (only && selected.length !== only.size) throw new Error('Unknown guard name in: ' + process.env.usage_only);

interface CheckRecord { name: string; passed: boolean; detail?: unknown }
const results: unknown[] = [];
const inspect = ({ name, file, checks }: { name: string; file: string; checks: string[] }): { status: number | null; records: CheckRecord[] } => {
  const run = spawnSync(process.execPath, ['bin/inspect-bundle.ts'], { encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, env: { ...process.env, usage_file: file, usage_runtime: runtime, usage_only: checks.join(',') } });
  const output = (run.stdout ?? '') + (run.stderr ?? '');
  writeFileSync(join(artifact, name + '.log'), output);
  if (run.error) throw run.error;
  const records = output.split('\n').filter(line => line.startsWith('{"name":')).map(line => JSON.parse(line));
  if (records.length !== checks.length) throw new Error('Expected ' + checks.length + ' check records for ' + name + ', saw ' + records.length + '; inspect ' + artifact);
  return { status: run.status, records };
};
const note = (entry: unknown): void => { results.push(entry); writeFileSync(join(artifact, 'results.json'), JSON.stringify(results, null, 2)); console.log(JSON.stringify(entry)); };

// The undamaged executable passes every check any case runs; this is the baseline for all of them.
const used = [...new Set(selected.flatMap(item => item.checks))];
const baseline = inspect({ name: 'baseline', file: executable, checks: used });
const baselinePassed = baseline.status === 0 && baseline.records.every(record => record.passed);
note({ name: 'baseline', phase: 'intact', checks: used, accepted: baselinePassed });
if (!baselinePassed) throw new Error('The intact executable does not pass its checks; inspect ' + artifact);

for (const item of selected) {
  const copy = join(artifact, item.name + '-monochromatic-ide');
  copyFileSync(executable, copy);
  chmodSync(copy, 0o755);
  item.damage(copy);
  const { status, records } = inspect({ name: item.name, file: copy, checks: item.checks });
  const verdicts = records.map(record => {
    const phrase = item.fails[record.name];
    const asExpected = phrase === undefined ? record.passed : !record.passed && String(record.detail).includes(phrase);
    return { check: record.name, expected: phrase === undefined ? 'pass' : 'fail: ' + phrase, passed: record.passed, asExpected, detail: record.passed ? undefined : String(record.detail).slice(0, 300) };
  });
  const accepted = status !== 0 && verdicts.every(verdict => verdict.asExpected);
  note({ name: item.name, phase: 'damaged', status, accepted, verdicts });
  if (!accepted) throw new Error('Unexpected result for ' + item.name + '; inspect ' + artifact);
}
console.log('Bundle guard controls passed: ' + artifact + ' (' + selected.length + ' of ' + cases.length + ' cases)');
