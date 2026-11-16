// Times the built @optique/core of each arm on Node and on Bun, with the published reproduction scripts and the
// alternating-input script, in five rounds whose runtime order and arm order rotate. Every figure this recheck had
// until now was taken on Deno from source; Node and Bun are what this repository runs on. One JSON line per run,
// then a done line.
//
// Each arm has a folder /timing/<arm> that holds the scripts and a node_modules/@optique/core link to that arm's
// built package, so a script resolves the library the way an installed package is resolved.
//
// Run inside the image: node /run-node-and-bun-timing.ts

import { spawnSync, } from 'node:child_process';

const RUNTIMES = ['node', 'bun',] as const;
const ROUNDS = 5;
// A run that takes longer than this is recorded as such and not waited for.
const RUN_LIMIT_MILLISECONDS = 15 * 60 * 1000;

interface Step {
  readonly name: string;
  readonly script: string;
  readonly args: readonly string[];
  readonly arms: readonly string[];
}

const STEPS: readonly Step[] = [
  { name: 'sweep-with-main', script: 'token-scaling.ts', args: ['--counts', '2000,4000,8000,16000', '--runs', '2', '--warmup', '1',], arms: ['main', 'base',], },
  { name: 'sweep', script: 'token-scaling.ts', args: ['--counts', '2000,4000,8000,16000,32000', '--runs', '2', '--warmup', '1',], arms: ['base', 'multiple-final', 'both', 'concat',], },
  { name: 'large', script: 'token-scaling.ts', args: ['--counts', '64000,128000', '--runs', '1', '--warmup', '0',], arms: ['multiple-final', 'both',], },
  { name: 'mixed', script: 'mixed-scaling.ts', args: ['--counts', '3000,4500,6000,12000,24000,48000', '--runs', '2', '--warmup', '1',], arms: ['base', 'multiple-final', 'both',], },
  { name: 'commit-paths', script: 'commit-paths.ts', args: ['--paths', '1000', '--runs', '7', '--warmup', '2',], arms: ['base', 'multiple-final', 'both', 'concat',], },
  { name: 'commit-paths-long', script: 'commit-paths.ts', args: ['--paths', '20000', '--runs', '2', '--warmup', '1',], arms: ['base', 'multiple-final', 'both', 'concat',], },
];

const versions = spawnSync('sh', ['-c', 'node --version; bun --version',], { encoding: 'utf8', },);

console.log(JSON.stringify({ runtimes: versions.stdout.trim().split('\n',), },),);

for (let round = 1; round <= ROUNDS; round += 1) {
  const runtimes = round % 2 === 1 ? RUNTIMES : [...RUNTIMES,].reverse();

  for (const runtime of runtimes) {
    for (const step of STEPS) {
      const offset = (round - 1) % step.arms.length;

      for (const arm of [...step.arms.slice(offset,), ...step.arms.slice(0, offset,),]) {
        const startedAt = Date.now();
        const child = spawnSync(runtime, [step.script, ...step.args,], { cwd: `/timing/${arm}`, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, timeout: RUN_LIMIT_MILLISECONDS, },);
        let result: unknown = null;

        try {
          result = JSON.parse(child.stdout.trim().split('\n',).at(-1,) ?? '',);
        }
        catch (error) {
          result = { unparsed: child.stdout.slice(0, 300,), reason: String(error,), };
        }

        console.log(JSON.stringify({
          round,
          step: `${runtime}-${step.name}`,
          arm,
          exit: child.status,
          signal: child.signal,
          wallMs: Date.now() - startedAt,
          result,
          stderr: child.stderr.slice(0, 400,),
        },),);
      }
    }
  }
}

console.log(JSON.stringify({ done: true, },),);
