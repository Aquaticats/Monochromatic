// Independent recheck of the input-trace prototype, run inside the work container on 2026-11-13 by a session that
// did not write the patch. Each step's full output goes to a log beside this file; one JSON line per step goes to
// stdout with the exit status and the last lines.
//
// Run inside the work container, with the prototype folder mounted at /work:
//   node /work/logs/recheck-2026-11-13.ts

import { spawnSync, } from 'node:child_process';
import { readFileSync, writeFileSync, } from 'node:fs';

const REPOSITORY = '/work/optique';
const CORE = `${REPOSITORY}/packages/core`;
const TRIE = `${CORE}/src/internal/immutable-string-map.ts`;
const NEW_TESTS = ['src/internal/immutable-string-map.test.ts', 'src/input-trace.test.ts',];
const STRIP_COLOUR = /\u001b\[[0-9;]*m/g;

function step(
  { name, command, args, cwd, }: { name: string; command: string; args: string[]; cwd: string; },
): { exit: number | null; output: string; } {
  const child = spawnSync(command, args, { cwd, encoding: 'utf8', maxBuffer: 256 * 1024 * 1024, },);
  const output = `${child.stdout}${child.stderr}`.replace(STRIP_COLOUR, '',);

  writeFileSync(`/work/logs/13-recheck-${name}.log`, output,);
  console.log(JSON.stringify({
    step: name,
    exit: child.status,
    signal: child.signal,
    tail: output.trim().split('\n',).slice(-4,).map((line,) => line.slice(0, 220,)),
  },),);

  return { exit: child.status, output, };
}

const git = (name: string, args: string[],) =>
  step({ name, command: 'git', args: ['-c', 'safe.directory=*', ...args,], cwd: REPOSITORY, },);

// CHANGES.md is a generated mirror of changes.d; the patch's run was stopped part way through regenerating it.
git('changes-restore', ['checkout', 'HEAD', '--', 'CHANGES.md',],);
step({ name: 'sacho-sync', command: '/work/tools/sacho', args: ['sync',], cwd: REPOSITORY, },);
step({ name: 'sacho-fmt', command: '/work/tools/sacho', args: ['fmt',], cwd: REPOSITORY, },);
step({ name: 'sacho-check', command: '/work/tools/sacho', args: ['check',], cwd: REPOSITORY, },);
git('status', ['status', '--short',],);
git('changes-diff', ['diff', 'HEAD', '--', 'CHANGES.md',],);

const denoNewTests = ['test', '--allow-read', '--allow-env', ...NEW_TESTS,];

step({ name: 'deno-new-tests', command: 'deno', args: denoNewTests, cwd: CORE, },);

// Guard check: each mutation breaks one property the patch relies on, and the new tests must fail on it.
const MUTATIONS = [
  {
    name: 'overwrite-mutates-shared-leaf',
    from: '    const entries = leaf.entries.slice();\n    entries[writtenIndex] = written;',
    to: '    const entries = leaf.entries as unknown as LeafEntry<V>[];\n    entries[writtenIndex] = written;',
  },
  { name: 'rank-ignores-lower-slots', from: '  let remaining = bitmap & (bit - 1);', to: '  let remaining = bitmap & bit;', },
  {
    name: 'delete-keeps-cleared-bit',
    from: '    return branchOf(node.bitmap & ~bit, children);',
    to: '    return branchOf(node.bitmap, children);',
  },
  {
    name: 'set-copies-nothing-in-branch',
    from: '  const children = node.children.slice();\n  if ((node.bitmap & bit) === 0) {',
    to: '  const children = node.children as unknown as TrieNode<V>[];\n  if ((node.bitmap & bit) === 0) {',
  },
] as const;
const original = readFileSync(TRIE, 'utf8',);

for (const mutation of MUTATIONS) {
  const occurrences = original.split(mutation.from,).length - 1;

  if (occurrences !== 1) throw new Error(`mutation ${mutation.name} matched ${occurrences} times`,);

  writeFileSync(TRIE, original.replace(mutation.from, mutation.to,),);

  const { exit, } = step({ name: `mutation-${mutation.name}`, command: 'deno', args: denoNewTests, cwd: CORE, },);

  writeFileSync(TRIE, original,);
  console.log(JSON.stringify({ mutation: mutation.name, caught: exit !== 0, },),);
}

if (readFileSync(TRIE, 'utf8',) !== original) throw new Error('the mutated file was not restored',);

step({ name: 'deno-core-suite', command: 'deno', args: ['test', '--allow-read', '--allow-env',], cwd: CORE, },);
step({ name: 'bun-new-tests', command: 'bun', args: ['test', ...NEW_TESTS,], cwd: CORE, },);
step({ name: 'core-build', command: 'pnpm', args: ['build',], cwd: CORE, },);
step({ name: 'node-core-suite', command: 'node', args: ['--test',], cwd: CORE, },);
git('status-after', ['status', '--short',],);
