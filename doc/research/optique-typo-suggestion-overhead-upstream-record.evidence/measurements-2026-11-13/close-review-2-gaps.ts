// Closes three evidence gaps the second review pass named, and prints one JSON line per step:
// - a Deno run of the same command in a copy of the folder without its package.json;
// - the output of the second failing Deno invocation, which was seen once and not kept;
// - whether the committed patch applies to upstream main and to pull request 1121's head.
//
// Run: node close-review-2-gaps.ts

import { spawnSync, } from 'node:child_process';

const HOME = process.env['HOME'];

if (HOME === undefined) throw new Error('HOME is not set',);

const CLONE = `${HOME}/temp/agent/upstream-prototype.O1iPjcqr/optique`;
const PATCH = '/var/mnt/encrypted/repository/Monochromatic/doc/troubleshooting/optique-input-trace-copies-its-map-for-each-recorded-token.patch';
const STRIP_COLOUR = /\u001b\[[0-9;]*m/g;

function container(name: string, shell: string,): void {
  const child = spawnSync('podman', [
    'run',
    '--rm',
    '--network',
    'none',
    '--cpus',
    '2',
    '--memory',
    '2g',
    '--pids-limit',
    '256',
    '--blkio-weight',
    '10',
    '--entrypoint',
    'sh',
    'localhost/optique-cross-runtime:2026-11-13',
    '-c',
    shell,
  ], { encoding: 'utf8', },);

  console.log(JSON.stringify({ step: name, exit: child.status, output: `${child.stdout}${child.stderr}`.replace(STRIP_COLOUR, '',).trim().split('\n',).map((line,) => line.slice(0, 420,)), },),);
}

container(
  'deno-run-without-package-json',
  'mkdir /no-package && cp -r /repo/maps /repo/commit-paths.ts /no-package/ && cd /no-package && ls && deno run --import-map maps/npm-1.3.6.json commit-paths.ts --paths 2 --runs 1 --warmup 0; echo "exit=$?"',
);
container(
  'deno-cache-beside-package-json',
  'cd /repo && deno cache --import-map maps/npm-1.3.6.json mismatch-count.ts commit-paths.ts token-scaling.ts; echo "exit=$?"',
);

function git(args: string[], cwd = CLONE,): { status: number | null; text: string; } {
  const child = spawnSync('git', args, { cwd, encoding: 'utf8', },);

  return { status: child.status, text: `${child.stdout}${child.stderr}`.trim(), };
}

for (const [name, ref,] of [['main', 'refs/remotes/origin/main',], ['pr1121', 'refs/remotes/origin/perf/option-scope-overhead',],] as const) {
  const worktree = `${HOME}/temp/agent/optique-2026-11-13/apply-check-${name}`;
  const sha = git(['rev-parse', ref,],).text;
  const added = git(['worktree', 'add', '--detach', worktree, sha,],);
  const check = git(['apply', '--check', '--verbose', PATCH,], worktree,);
  const removed = git(['worktree', 'remove', '--force', worktree,],);

  console.log(JSON.stringify({ step: `apply-check-${name}`, sha, added: added.status, applies: check.status === 0, detail: check.text.split('\n',).slice(-6,), removed: removed.status, },),);
}
