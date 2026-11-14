// Runs upstream's core suite on Bun twice inside the work container: on an unpatched worktree of the prototype's
// base commit, and on the patched working tree, then compares the names of the failing tests. One JSON line per
// step, and one final line with the comparison.
//
//   node /work/logs/bun-suite-with-and-without-the-patch.ts

import { spawnSync, } from 'node:child_process';
import { writeFileSync, } from 'node:fs';

const CLONE = '/work/optique';
const BASELINE = '/work/bun-baseline';
const STRIP_COLOUR = /\u001b\[[0-9;]*m/g;

function step(
  { name, command, args, cwd, }: { name: string; command: string; args: string[]; cwd: string; },
): { exit: number | null; output: string; } {
  const child = spawnSync(command, args, { cwd, encoding: 'utf8', maxBuffer: 512 * 1024 * 1024, },);
  const output = `${child.stdout}${child.stderr}`.replace(STRIP_COLOUR, '',);

  writeFileSync(`/work/logs/17-bun-${name}.log`, output,);
  console.log(JSON.stringify({ step: name, exit: child.status, tail: output.trim().split('\n',).slice(-4,).map((line,) => line.slice(0, 200,)), },),);

  return { exit: child.status, output, };
}

const git = (name: string, args: string[],) => step({ name, command: 'git', args: ['-c', 'safe.directory=*', ...args,], cwd: CLONE, },);

git('worktree-add', ['worktree', 'add', '--detach', BASELINE, 'HEAD',],);

let install = step({ name: 'baseline-install-offline', command: 'pnpm', args: ['install', '--offline', '--frozen-lockfile',], cwd: BASELINE, },);

if (install.exit !== 0) install = step({ name: 'baseline-install', command: 'pnpm', args: ['install', '--frozen-lockfile',], cwd: BASELINE, },);

step({ name: 'baseline-build', command: 'pnpm', args: ['build',], cwd: `${BASELINE}/packages/core`, },);

const failures = (output: string,): string[] =>
  [...new Set(output.split('\n',).filter((line,) => line.startsWith('(fail)',)).map((line,) => line.replace(/\s\[[0-9.]+ms\]$/, '',)),),].sort();
// Baseline first, then patched, then baseline again: a test that fails in one baseline run and not the other is
// unstable on this host, whatever the patch does.
const baselineFirst = step({ name: 'baseline-suite-first', command: 'bun', args: ['test',], cwd: `${BASELINE}/packages/core`, },);
const patched = step({ name: 'patched-suite', command: 'bun', args: ['test',], cwd: `${CLONE}/packages/core`, },);
const baselineSecond = step({ name: 'baseline-suite-second', command: 'bun', args: ['test',], cwd: `${BASELINE}/packages/core`, },);

git('worktree-remove', ['worktree', 'remove', '--force', BASELINE,],);

const first = failures(baselineFirst.output,);
const second = failures(baselineSecond.output,);
const withPatch = failures(patched.output,);
const inEitherBaseline = new Set([...first, ...second,],);

console.log(JSON.stringify({
  comparison: {
    baselineFirstFailures: first.length,
    baselineSecondFailures: second.length,
    patchedFailures: withPatch.length,
    failOnlyWithPatch: withPatch.filter((name,) => !inEitherBaseline.has(name,)),
    failInBothBaselinesNotWithPatch: first.filter((name,) => second.includes(name,) && !withPatch.includes(name,)),
    summaries: [baselineFirst, patched, baselineSecond,].map((run,) => run.output.trim().split('\n',).slice(-5,)),
  },
},),);
