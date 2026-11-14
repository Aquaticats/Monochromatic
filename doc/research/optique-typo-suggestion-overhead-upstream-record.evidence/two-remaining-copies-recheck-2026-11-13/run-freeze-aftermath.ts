// Runs probe/freeze-aftermath.ts and probe/write-aftermath.ts on Deno, Node and Bun against three arms, each in
// its own container with no network: plain arrays (base), the agent's diffs with every state and buffer a view
// (views-always), and the kept patch with every state a view (multiple-final-all-views). The probe and arms folders
// are mounted read-only; these are behaviour checks, not timings.
//
// Run: node run-freeze-aftermath.ts
// Writes keep/freeze-aftermath.jsonl and keep/write-aftermath.jsonl.

import { spawnSync, } from 'node:child_process';
import { writeFileSync, } from 'node:fs';

const HERE = import.meta.dirname;
const RUNTIMES = [
  { runtime: 'deno', command: ['deno', 'run', '--allow-read',], },
  { runtime: 'node', command: ['node',], },
  { runtime: 'bun', command: ['bun',], },
] as const;

for (const probe of ['freeze-aftermath', 'write-aftermath',]) {
  const lines: string[] = [];

  for (const arm of ['base', 'views-always', 'multiple-final-all-views',]) {
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
        '--security-opt',
        'label=disable',
        '--volume',
        `${HERE}/probe:/extra:ro`,
        '--volume',
        `${HERE}/arms:/staged:ro`,
        '--entrypoint',
        entrypoint,
        'localhost/optique-two-remaining-copies-recheck:2026-11-13',
        ...rest,
        `/extra/${probe}.ts`,
        `/staged/${arm}`,
      ], { encoding: 'utf8', },);
      let facts: unknown = null;

      try {
        facts = JSON.parse(child.stdout.trim().split('\n',).at(-1,) ?? '',);
      }
      catch (error) {
        facts = { unparsed: child.stdout.slice(0, 400,), reason: String(error,), };
      }

      lines.push(JSON.stringify({ arm, runtime, exit: child.status, facts, stderr: child.stderr.slice(0, 600,), },),);
    }
  }

  writeFileSync(`${HERE}/keep/${probe}.jsonl`, `${lines.join('\n',)}\n`,);
}
