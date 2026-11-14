// Runs annotated-scaling.ts on four arms in rotating order, three rounds. One JSON line per run, then a done line.
//
// Run inside the image: node /run-annotated.ts

import { spawnSync, } from 'node:child_process';

const ARMS = ['base', 'multiple', 'both', 'views-always',] as const;
const ROUNDS = 3;

for (let round = 1; round <= ROUNDS; round += 1) {
  const offset = (round - 1) % ARMS.length;

  for (const arm of [...ARMS.slice(offset,), ...ARMS.slice(0, offset,),]) {
    const child = spawnSync('deno', ['run', '--no-config', '--import-map', `/maps/recheck-${arm}.json`, 'annotated-scaling.ts', '--counts', '1000,2000,4000,8000,16000', '--runs', '2', '--warmup', '1',], {
      cwd: '/repo',
      encoding: 'utf8',
      maxBuffer: 64 * 1024 * 1024,
    },);
    let result: unknown = null;

    try {
      result = JSON.parse(child.stdout.trim().split('\n',).at(-1,) ?? '',);
    }
    catch (error) {
      result = { unparsed: child.stdout.slice(0, 300,), reason: String(error,), };
    }

    console.log(JSON.stringify({ round, arm, exit: child.status, result, stderr: child.stderr.slice(0, 500,), },),);
  }
}

console.log(JSON.stringify({ done: true, },),);
