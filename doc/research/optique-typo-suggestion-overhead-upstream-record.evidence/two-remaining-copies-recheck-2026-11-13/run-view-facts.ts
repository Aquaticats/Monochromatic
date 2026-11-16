// Runs view-facts.ts on Deno, Node and Bun against the arm with plain arrays (base) and the arm whose state and
// buffer are always views, each in its own mount-free container, and writes one JSON line per run.
//
// Run: node run-view-facts.ts > keep/view-facts.jsonl

import { spawnSync, } from 'node:child_process';

const RUNTIMES = [
  { runtime: 'deno', command: ['deno', 'run', '--allow-read', '/view-facts.ts',], },
  { runtime: 'node', command: ['node', '/view-facts.ts',], },
  { runtime: 'bun', command: ['bun', '/view-facts.ts',], },
] as const;

for (const arm of ['base', 'views-always',]) {
  for (const { runtime, command, } of RUNTIMES) {
    const [entrypoint, ...rest] = command;
    const child = spawnSync('podman', [
      'run',
      '--rm',
      '--network',
      'none',
      '--cpus',
      '1',
      '--memory',
      '2g',
      '--pids-limit',
      '256',
      '--blkio-weight',
      '10',
      '--entrypoint',
      entrypoint,
      'localhost/optique-two-remaining-copies-recheck:2026-11-13',
      ...rest,
      `/arms/${arm}`,
    ], { encoding: 'utf8', },);
    let facts: unknown = null;

    try {
      facts = JSON.parse(child.stdout.trim().split('\n',).at(-1,) ?? '',);
    }
    catch (error) {
      facts = { unparsed: child.stdout.slice(0, 400,), reason: String(error,), };
    }

    console.log(JSON.stringify({ arm, runtime, exit: child.status, facts, stderr: child.stderr.slice(0, 600,), },),);
  }
}
