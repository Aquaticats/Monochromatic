// Stages four source arms of @optique/core for a second measurement image, from the disposable prototype clone:
// upstream main as it is today, the head of upstream's pull request 1121, and each of those with the input-trace
// prototype applied. Also saves the prototype's patch, and records whether it applies to each base.
//
// Nothing is committed or pushed in the clone; the patched trees are made in throwaway worktrees that are removed.
//
// Run: node stage-upstream-arms.ts

import { spawnSync, } from 'node:child_process';
import { cpSync, mkdirSync, rmSync, writeFileSync, } from 'node:fs';

const HOME = process.env['HOME'];

if (HOME === undefined) throw new Error('HOME is not set',);

const CLONE = `${HOME}/temp/agent/upstream-prototype.O1iPjcqr/optique`;
const OUT = `${HOME}/temp/agent/optique-2026-11-13/upstream-arms`;
const PATCH = `${HOME}/temp/agent/optique-2026-11-13/input-trace-persistent-map.patch`;

function git(args: string[], cwd = CLONE,): { status: number | null; stdout: string; stderr: string; } {
  const child = spawnSync('git', args, { cwd, encoding: 'utf8', maxBuffer: 256 * 1024 * 1024, },);

  return { status: child.status, stdout: child.stdout, stderr: child.stderr, };
}

function must(args: string[], cwd = CLONE,): string {
  const result = git(args, cwd,);

  if (result.status !== 0) throw new Error(`git ${args.join(' ',)} failed: ${result.stderr.slice(0, 400,)}`,);

  return result.stdout;
}

rmSync(OUT, { recursive: true, force: true, },);
mkdirSync(OUT, { recursive: true, },);

// The patch as upstream would receive it: everything but the generated changelog mirror, which `sacho sync` rebuilds.
writeFileSync(PATCH, must(['diff', 'HEAD', '--binary', '--', '.', ':(exclude)CHANGES.md',],),);

must(['fetch', '--no-tags', 'origin', '+refs/heads/main:refs/remotes/origin/main', '+refs/heads/perf/option-scope-overhead:refs/remotes/origin/perf/option-scope-overhead',],);

const bases = {
  main: must(['rev-parse', 'refs/remotes/origin/main',],).trim(),
  pr1121: must(['rev-parse', 'refs/remotes/origin/perf/option-scope-overhead',],).trim(),
};
const record: Record<string, unknown> = { prototypeBase: must(['rev-parse', 'HEAD',],).trim(), bases, };

for (const [name, sha,] of Object.entries(bases,)) {
  const worktree = `${OUT}/worktree-${name}`;

  must(['worktree', 'add', '--detach', worktree, sha,],);
  cpSync(`${worktree}/packages/core/src`, `${OUT}/${name}/packages/core/src`, { recursive: true, },);

  const applied = git(['apply', '--index', PATCH,], worktree,);

  record[`patchAppliesTo_${name}`] = applied.status === 0 ? true : applied.stderr.slice(0, 400,);

  if (applied.status === 0) cpSync(`${worktree}/packages/core/src`, `${OUT}/${name}-patched/packages/core/src`, { recursive: true, },);

  must(['worktree', 'remove', '--force', worktree,],);
}

console.log(JSON.stringify(record,),);
