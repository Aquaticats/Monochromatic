// Run inside the work container after the timing-threshold test was removed from the prototype on 2026-11-13:
// formats and lints the touched files with upstream's own tools, reruns the new tests on Deno and Bun, the mutation
// guards, and the core suites on Deno and Node. One JSON line per step.
//
//   node /work/logs/recheck-after-test-removal.ts

import { spawnSync, } from 'node:child_process';
import { readFileSync, writeFileSync, } from 'node:fs';

const REPOSITORY = '/work/optique';
const CORE = `${REPOSITORY}/packages/core`;
const TRIE = `${CORE}/src/internal/immutable-string-map.ts`;
const TOUCHED = [
  'src/input-trace.ts',
  'src/input-trace.test.ts',
  'src/internal/immutable-string-map.ts',
  'src/internal/immutable-string-map.test.ts',
];
const NEW_TESTS = ['src/internal/immutable-string-map.test.ts', 'src/input-trace.test.ts',];
const STRIP_COLOUR = /\u001b\[[0-9;]*m/g;

function step({ name, command, args, cwd, }: { name: string; command: string; args: string[]; cwd: string; },): number | null {
  const child = spawnSync(command, args, { cwd, encoding: 'utf8', maxBuffer: 256 * 1024 * 1024, },);
  const output = `${child.stdout}${child.stderr}`.replace(STRIP_COLOUR, '',);

  writeFileSync(`/work/logs/16-recheck-${name}.log`, output,);
  console.log(JSON.stringify({ step: name, exit: child.status, tail: output.trim().split('\n',).slice(-3,).map((line,) => line.slice(0, 200,)), },),);

  return child.status;
}

step({ name: 'deno-fmt-check', command: 'deno', args: ['fmt', '--check', ...TOUCHED,], cwd: CORE, },);
step({ name: 'deno-lint', command: 'deno', args: ['lint', ...TOUCHED,], cwd: CORE, },);
step({ name: 'deno-check', command: 'deno', args: ['check', ...TOUCHED,], cwd: CORE, },);

const denoNewTests = ['test', '--allow-read', '--allow-env', ...NEW_TESTS,];

step({ name: 'deno-new-tests', command: 'deno', args: denoNewTests, cwd: CORE, },);
step({ name: 'bun-new-tests', command: 'bun', args: ['test', ...NEW_TESTS,], cwd: CORE, },);

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
  if (original.split(mutation.from,).length !== 2) throw new Error(`mutation ${mutation.name} does not match once`,);

  writeFileSync(TRIE, original.replace(mutation.from, mutation.to,),);

  const exit = step({ name: `mutation-${mutation.name}`, command: 'deno', args: denoNewTests, cwd: CORE, },);

  writeFileSync(TRIE, original,);
  console.log(JSON.stringify({ mutation: mutation.name, caught: exit !== 0, },),);
}

if (readFileSync(TRIE, 'utf8',) !== original) throw new Error('the mutated file was not restored',);

step({ name: 'deno-core-suite', command: 'deno', args: ['test', '--allow-read', '--allow-env',], cwd: CORE, },);
step({ name: 'core-build', command: 'pnpm', args: ['build',], cwd: CORE, },);
step({ name: 'node-core-suite', command: 'node', args: ['--test',], cwd: CORE, },);
step({ name: 'sacho-check', command: '/work/tools/sacho', args: ['check',], cwd: REPOSITORY, },);
