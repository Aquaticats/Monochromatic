#!/usr/bin/env node
// Real toolchain worktrees against the production setup: servers found through the `mpm-shadow bin --json` lookup and started
// confined, a lookup that fails and falls back to PATH with the notice, a folder without the toolchain that keeps the
// PATH lookup and its message, a worktree whose toolchain is not installed, a hung lookup program that is stopped, and
// project trees compared before and after every run. Everything runs on disposable worktrees below the private agent
// scratch root. The tools are the ones this worktree's `mise run toolchain:install` provisioned (read with the same
// `mpm-shadow bin --json` lookup the IDE uses); nothing is installed, trusted, or written into a project, and no server
// touches this repository.
import { spawnSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { chmodSync, cpSync, existsSync, lstatSync, mkdirSync, mkdtempSync, readFileSync, readdirSync, readlinkSync, realpathSync, rmSync, statSync, symlinkSync, writeFileSync } from 'node:fs';
import { homedir, tmpdir } from 'node:os';
import { dirname, join, resolve } from 'node:path';

const binary = resolve('target/debug/ide-language-inspect');
// The private agent scratch root: its ancestors may hold other sessions' toolchain files, so the project without the
// toolchain lives below the system temporary directory instead (the sandbox binds a project below /tmp again).
const privateRoot = join(homedir(), 'temp', 'agent');
if ((statSync(privateRoot).mode & 0o077) !== 0) throw new Error('Scratch root must exclude group and other permissions: ' + privateRoot);
const base = realpathSync(mkdtempSync(join(privateRoot, 'ide-language-toolchain-')));
const outside = realpathSync(mkdtempSync(join(tmpdir(), 'ide-language-toolchain-')));
const results = join(base, 'results');
mkdirSync(results);
console.log('LANGUAGE_TOOLCHAIN_ARTIFACT=' + base);

const write = ({ root, relative, text }: { root: string; relative: string; text: string }): void => {
  const target = join(root, relative);
  mkdirSync(join(target, '..'), { recursive: true });
  writeFileSync(target, text);
};
const offsetOf = ({ text, needle, within = 0 }: { text: string; needle: string; within?: number }): number => {
  const index = text.indexOf(needle);
  if (index < 0) throw new Error('fixture text lacks ' + needle);
  return [...text.slice(0, index)].length + within;
};

// The worktree's own `mpm-shadow` link and the pinned answer of the repository: where its provisioned tools are. This
// file lives at package/ide/app/bin, so the repository root is four directories up. The lookup is run by the link path
// itself, as the IDE does, because the launcher behind the link learns the tree from the path it was started under.
const repoRoot = resolve(dirname(import.meta.filename), '../../../..');
const programGiven = join(repoRoot, '.cache/toolchain-bin/mpm-shadow');
if (!existsSync(programGiven)) throw new Error('This check needs the installed mpm-shadow link at ' + programGiven + '; run `mise run toolchain:install` first');
const planText = readFileSync(join(repoRoot, 'toolchain.plan.jsonc'), 'utf8');
const answerOf = (program: string): { exposureDir: string; pathDirs: string[]; bins: Record<string, string> } => {
  const run = spawnSync(program, ['bin', '--json'], { encoding: 'utf8', cwd: repoRoot });
  if (run.status !== 0) throw new Error('the bin lookup failed: ' + (run.stderr || run.stdout));
  return JSON.parse(run.stdout);
};
const provisioned = answerOf(programGiven).bins;
const wanted = ['node', 'taplo', 'marksman', 'bash-language-server'];
for (const name of wanted) {
  if (!provisioned[name]) throw new Error('the provisioned toolchain has no ' + name + '; run `mise run toolchain:install` first; this check never installs anything');
}

// What the desktop launcher has: the PATH line of the systemd user session, read for that line only.
const sessionPath = (() => {
  const shown = spawnSync('systemctl', ['--user', 'show-environment'], { encoding: 'utf8' });
  const line = (shown.stdout ?? '').split('\n').find(text => text.startsWith('PATH='));
  const found = line ? line.slice('PATH='.length) : process.env.PATH;
  if (found === undefined) throw new Error('Neither the systemd user session nor this process has a PATH');
  return found;
})();
console.log('launcher-like PATH entries: ' + sessionPath.split(':').length);

const script = [
  '#!/bin/sh',
  '# Greet one name.',
  'greet() {',
  '  echo "hello $1"',
  '}',
  '',
  'greet "world"',
  '',
].join('\n');
const notes = ['# Notes', '', 'See [[other]] and [the other note](other.md).', ''].join('\n');
const dataToml = ['[package]', 'name = "example"', 'version = "0.1.0"', ''].join('\n');

// A disposable toolchain worktree: the plan, the `mpm-shadow` link (or a fixture program in its place), and the tool
// links of one exposure directory, plus the sample files the servers need.
const worktree = ({ root, tools = wanted, program }: { root: string; tools?: string[]; program?: { text: string } | null }): void => {
  write({ root, relative: 'toolchain.plan.jsonc', text: planText });
  mkdirSync(join(root, '.cache/toolchain-bin'), { recursive: true });
  for (const name of tools) symlinkSync(provisioned[name], join(root, '.cache/toolchain-bin', name));
  const link = join(root, '.cache/toolchain-bin/mpm-shadow');
  if (program === undefined) symlinkSync(programGiven, link);
  else if (program !== null) {
    // A fixture program replaces the real one: it fails or hangs the way a damaged toolchain would.
    writeFileSync(link, program.text);
    chmodSync(link, 0o755);
  }
  write({ root, relative: 'run.sh', text: script });
  write({ root, relative: 'data.toml', text: dataToml });
  write({ root, relative: 'notes.md', text: notes });
  // marksman roots its workspace at a folder with this marker (or a .git folder).
  write({ root, relative: '.marksman.toml', text: '[core]\n' });
  write({ root, relative: 'other.md', text: '# Other note\n\nSome text.\n' });
};
const withToolchain = join(base, 'toolchain-project');
worktree({ root: withToolchain });
// A worktree whose lookup program is damaged: it fails and the notice quotes the lookup.
const broken = join(base, 'broken-project');
worktree({ root: broken, program: { text: '#!/bin/sh\necho "mpm-shadow: toolchain.lock.json is unreadable" >&2\nexit 1\n' } });
const hung = join(base, 'hung-project');
worktree({ root: hung, program: { text: '#!/bin/sh\nexec sleep 600\n' } });
// A worktree that has the plan and no mpm-shadow link: nothing is installed yet.
const uninstalled = join(base, 'uninstalled-project');
worktree({ root: uninstalled, program: null });
write({ root: outside, relative: 'a.lua', text: 'local value = 1\nprint(value)\n' });

interface PlanStep {
  label: string;
  do: string;
  [field: string]: unknown;
}
const plan = ({ root, steps }: { root: string; steps: PlanStep[] }): { project: string; steps: PlanStep[] } => ({ project: root, steps });
const bashHover = (at: number): PlanStep => ({ label: 'hover', do: 'request', kind: 'hover', at, until: 'hover', seconds: 60 });
interface LookupCase {
  name: string;
  root: string;
  path: string;
  plan: { project: string; steps: PlanStep[] };
}
const cases: LookupCase[] = [
  {
    name: 'toolchain-project-node-server',
    root: withToolchain,
    path: sessionPath,
    plan: plan({ root: withToolchain, steps: [
      { label: 'open', do: 'open', file: 'run.sh' },
      { label: 'ready', do: 'ready', seconds: 60 },
      bashHover(offsetOf({ text: script, needle: 'greet "world"', within: 1 })),
      { label: 'diagnostics', do: 'diagnostics', minimum: 0, maximum: 0, seconds: 2 },
      { label: 'close', do: 'close' },
    ] }),
  },
  {
    name: 'toolchain-project-binary-server',
    root: withToolchain,
    path: sessionPath,
    plan: plan({ root: withToolchain, steps: [
      { label: 'open', do: 'open', file: 'data.toml' },
      { label: 'ready', do: 'ready', seconds: 60 },
      { label: 'diagnostics', do: 'diagnostics', minimum: 0, maximum: 0, seconds: 2 },
      { label: 'close', do: 'close' },
    ] }),
  },
  {
    name: 'toolchain-project-markdown-server',
    root: withToolchain,
    path: sessionPath,
    plan: plan({ root: withToolchain, steps: [
      { label: 'open', do: 'open', file: 'notes.md' },
      { label: 'ready', do: 'ready', seconds: 60 },
      { label: 'hover', do: 'request', kind: 'hover', at: offsetOf({ text: notes, needle: '[[other]]', within: 3 }), until: 'hover', seconds: 60 },
      { label: 'close', do: 'close' },
    ] }),
  },
  {
    name: 'broken-project',
    root: broken,
    path: sessionPath,
    plan: plan({ root: broken, steps: [
      { label: 'open', do: 'open', file: 'run.sh' },
      { label: 'sleep', do: 'sleep', milliseconds: 1500 },
      { label: 'notice', do: 'diagnostics', minimum: 1, maximum: 100, seconds: 30 },
      { label: 'close', do: 'close' },
    ] }),
  },
  {
    name: 'uninstalled-project',
    root: uninstalled,
    path: sessionPath,
    plan: plan({ root: uninstalled, steps: [
      { label: 'open', do: 'open', file: 'run.sh' },
      { label: 'sleep', do: 'sleep', milliseconds: 1500 },
      { label: 'notice', do: 'diagnostics', minimum: 1, maximum: 100, seconds: 30 },
      { label: 'close', do: 'close' },
    ] }),
  },
  {
    name: 'folder-without-toolchain',
    root: outside,
    path: sessionPath,
    plan: plan({ root: outside, steps: [
      { label: 'open', do: 'open', file: 'a.lua' },
      { label: 'sleep', do: 'sleep', milliseconds: 1500 },
      { label: 'notice', do: 'diagnostics', minimum: 1, maximum: 100, seconds: 30 },
      { label: 'close', do: 'close' },
    ] }),
  },
  {
    name: 'hung-lookup-program',
    root: hung,
    path: sessionPath,
    plan: plan({ root: hung, steps: [
      { label: 'open', do: 'open', file: 'run.sh' },
      { label: 'sleep', do: 'sleep', milliseconds: 1500 },
      { label: 'notice', do: 'diagnostics', minimum: 1, maximum: 100, seconds: 60 },
      { label: 'close', do: 'close' },
    ] }),
  },
];

// Every path with type, size, mode, times, link count, inode, content hash, and link target.
interface SnapshotEntry {
  mode: number;
  size: number;
  mtime: string;
  ctime: string;
  nlink: number;
  ino: string;
  target?: string;
  sha256?: string;
}
const snapshot = (root: string): Record<string, SnapshotEntry> => {
  const entries: Record<string, SnapshotEntry> = {};
  const walk = (directory: string): void => {
    for (const name of readdirSync(directory).sort()) {
      const path = join(directory, name);
      const stat = lstatSync(path, { bigint: true });
      const entry: SnapshotEntry = { mode: Number(stat.mode), size: Number(stat.size), mtime: String(stat.mtimeNs), ctime: String(stat.ctimeNs), nlink: Number(stat.nlink), ino: String(stat.ino) };
      if (stat.isSymbolicLink()) entry.target = readlinkSync(path);
      else if (stat.isFile()) entry.sha256 = createHash('sha256').update(readFileSync(path)).digest('hex');
      entries[path.slice(root.length + 1)] = entry;
      if (stat.isDirectory()) walk(path);
    }
  };
  walk(root);
  return entries;
};
interface TreeDifference { added: string[]; removed: string[]; changed: string[] }
const difference = ({ before, after }: { before: Record<string, SnapshotEntry>; after: Record<string, SnapshotEntry> }): TreeDifference => ({
  added: Object.keys(after).filter(key => !(key in before)),
  removed: Object.keys(before).filter(key => !(key in after)),
  changed: Object.keys(after).filter(key => key in before && JSON.stringify(before[key]) !== JSON.stringify(after[key])),
});
const unchanged = (tree: TreeDifference): boolean => tree.added.length === 0 && tree.removed.length === 0 && tree.changed.length === 0;

// Processes of the fixtures: a working directory inside a fixture.
const leftovers = () => readdirSync('/proc').filter(name => /^\d+$/.test(name)).flatMap(pid => {
  try {
    const cwd = readlinkSync(join('/proc', pid, 'cwd'));
    const command = readFileSync(join('/proc', pid, 'cmdline'), 'utf8').split('\0').join(' ').trim();
    const inside = cwd.startsWith(base) || cwd.startsWith(outside);
    return inside ? [pid + ' ' + cwd + ' ' + command] : [];
  } catch { return []; }
});

const summary: { base: string; outside: string; tools: string[]; cases: unknown[] } = { base, outside, tools: wanted, cases: [] };
let failed = false;
const report = ({ name, passed }: { name: string; passed: boolean }): void => {
  if (!passed) failed = true;
  console.log((passed ? 'PASS ' : 'FAIL ') + name);
};
// The application's private state goes below the fixture. The launcher-like PATH replaces the shell's.
const environment = (path: string): NodeJS.ProcessEnv => ({ ...process.env, PATH: path, XDG_CACHE_HOME: join(base, 'app-cache') });
// The records of the inspection binary's event stream, one JSON object per line; only the fields read here are named.
interface LookupEvent {
  step?: unknown;
  result?: { ready?: boolean; matched?: boolean };
  reply?: { outcome: { text?: string } };
  diagnostics?: { items?: { source?: string; message?: string }[] };
}
const noticeText = (events: LookupEvent[]): string => events.flatMap(event => (event.diagnostics ? [event.diagnostics] : [])).flatMap(diagnostics => diagnostics.items ?? [])
  .filter(item => item.source === 'language support').map(item => item.message).join('\n');

for (const item of cases) {
  const planPath = join(results, item.name + '.plan.json');
  writeFileSync(planPath, JSON.stringify(item.plan, null, 2));
  const before = snapshot(item.root);
  const started = Date.now();
  const run = spawnSync(binary, [planPath], { encoding: 'utf8', env: environment(item.path), maxBuffer: 64 * 1024 * 1024 });
  const seconds = (Date.now() - started) / 1000;
  const tree = difference({ before, after: snapshot(item.root) });
  writeFileSync(join(results, item.name + '.events.jsonl'), run.stdout ?? '');
  writeFileSync(join(results, item.name + '.stderr.txt'), run.stderr ?? '');
  if (run.error) throw run.error;
  const events = (run.stdout ?? '').split('\n').filter(Boolean).map((line): LookupEvent => JSON.parse(line));
  const steps = events.filter(event => event.step !== undefined);
  const result = (label: string): { ready?: boolean; matched?: boolean } => steps[item.plan.steps.findIndex(step => step.label === label)]?.result ?? {};
  const replies = events.flatMap(event => (event.reply ? [event.reply] : []));
  const hover = replies.map(reply => reply.outcome.text ?? '').join('\n');
  const notice = noticeText(events);
  const stderr = (run.stderr ?? '').replace(/\u001b\[[0-9;]*m/g, '');
  const elapsed = [...stderr.matchAll(/elapsed_ms=(\d+)/g)].map(found => Number(found[1]));
  const checks: [string, boolean][] = [['process exited cleanly', run.status === 0], ['every step was recorded', steps.length === item.plan.steps.length]];
  if (item.name === 'toolchain-project-node-server') {
    checks.push(['the toolchain provides tool directories for the project', /the toolchain provides tool directories for the project/.test(stderr)]);
    checks.push(['the Node server became ready through the toolchain', result('ready').ready === true]);
    checks.push(['hover names the function', result('hover').matched === true && hover.includes('greet')]);
    checks.push(['the notice does not mention the toolchain', !/toolchain/i.test(notice)]);
  }
  if (item.name === 'toolchain-project-markdown-server') {
    checks.push(['the Markdown server became ready through the toolchain', result('ready').ready === true]);
    checks.push(['hover on a wiki link shows the linked note', result('hover').matched === true && hover.includes('Other note')]);
  }
  if (item.name === 'toolchain-project-binary-server') {
    checks.push(['the binary server became ready through the toolchain', result('ready').ready === true]);
    checks.push(['taplo is not reported missing and the lookup has nothing to say', !notice.includes('taplo is not available') && !notice.includes('toolchain lookup said') && !notice.includes('toolchain lookup reported')]);
  }
  if (item.name === 'broken-project') {
    checks.push(['the notice quotes the lookup', notice.includes('the toolchain lookup said:')]);
    checks.push(['the notice names the lookup the user runs', notice.includes('mpm-shadow bin --json')]);
    checks.push(['the server falls back to PATH and its shim is not started', /bash-language-server is not available: the program 'bash-language-server' for bash-language-server was not found on PATH/.test(notice)]);
  }
  if (item.name === 'uninstalled-project') {
    checks.push(['the notice says the toolchain is not installed and names the install task', notice.includes('mise run toolchain:install')]);
    checks.push(['the server falls back to PATH', /bash-language-server is not available/.test(notice)]);
  }
  if (item.name === 'folder-without-toolchain') {
    checks.push(['the notice keeps the PATH message', /lua-language-server is not available: the program 'lua-language-server' for lua-language-server was not found on PATH\./.test(notice)]);
    checks.push(['the toolchain is not mentioned', !/toolchain/i.test(notice)]);
    checks.push(['the lookup was not asked to provide anything', !/the toolchain provides tool directories/.test(stderr)]);
  }
  if (item.name === 'hung-lookup-program') {
    checks.push(['the hung lookup program was stopped at the time limit', notice.includes('did not finish within 10 s and was stopped')]);
    checks.push(['the whole run stayed near the limit', seconds < 40]);
  }
  checks.push(['the project tree is unchanged', unchanged(tree)]);
  const left = leftovers();
  checks.push(['no process is left in the fixtures', left.length === 0]);
  summary.cases.push({ name: item.name, seconds, lookupMilliseconds: elapsed, tree, leftovers: left, notice, hover: hover.slice(0, 300), checks: checks.map(([name, passed]) => ({ name, passed })) });
  for (const [name, passed] of checks) report({ name: item.name + ': ' + name, passed });
}

// Positive control: the comparison sees a created file.
const control = join(base, 'control-project');
cpSync(withToolchain, control, { recursive: true });
const controlBefore = snapshot(control);
write({ root: control, relative: 'scratch-note.txt', text: '# written by the control\n' });
const controlTree = difference({ before: controlBefore, after: snapshot(control) });
report({ name: 'control: the tree comparison sees a created file', passed: controlTree.added.includes('scratch-note.txt') });
rmSync(control, { recursive: true, force: true });
report({ name: 'no scratch note exists in any project', passed: [withToolchain, broken, hung, uninstalled, outside].every(root => !existsSync(join(root, 'scratch-note.txt'))) });

writeFileSync(join(results, 'results.json'), JSON.stringify(summary, null, 2));
console.log('Language toolchain inspection ' + (failed ? 'FAILED' : 'passed') + ': ' + join(results, 'results.json'));
if (failed) process.exitCode = 1;
