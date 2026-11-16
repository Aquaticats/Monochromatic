// Times the arm the kept patch produces beside the agent's state-fix arm, on which the five rounds were run, and the
// base, so that the figures quoted for the state fix can be said to hold for the patch as kept. Five rounds, the
// order of the arms rotated. One JSON line per run, then a done line.
//
// Run inside the image: node /run-kept-patch-timing.ts

import { spawnSync, } from 'node:child_process';

const ARMS = ['base', 'multiple', 'multiple-final',] as const;
const ROUNDS = 5;
const STEPS = [
  { name: 'kept-patch-sweep', script: 'token-scaling.ts', args: ['--counts', '2000,4000,8000,16000,32000', '--runs', '2', '--warmup', '1',], },
  { name: 'kept-patch-commit-paths', script: 'commit-paths.ts', args: ['--paths', '1000', '--runs', '7', '--warmup', '2',], },
  { name: 'kept-patch-commit-paths-long', script: 'commit-paths.ts', args: ['--paths', '20000', '--runs', '2', '--warmup', '1',], },
  { name: 'kept-patch-mixed', script: 'mixed-scaling.ts', args: ['--counts', '3000,4500,6000,9000,12000', '--runs', '5', '--warmup', '2',], },
] as const;

for (let round = 1; round <= ROUNDS; round += 1) {
  const offset = (round - 1) % ARMS.length;

  for (const step of STEPS) {
    for (const arm of [...ARMS.slice(offset,), ...ARMS.slice(0, offset,),]) {
      const child = spawnSync('deno', ['run', '--no-config', '--import-map', `/maps/recheck-${arm}.json`, step.script, ...step.args,], {
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

      console.log(JSON.stringify({ round, step: step.name, arm, exit: child.status, result, stderr: child.stderr.slice(0, 400,), },),);
    }
  }
}

console.log(JSON.stringify({ done: true, },),);
