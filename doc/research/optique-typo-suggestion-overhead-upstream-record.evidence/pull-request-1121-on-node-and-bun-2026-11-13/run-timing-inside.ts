// Run inside the image: times a thousand paths through the `git commit`-shaped parser of the published reproduction
// script, with `--` before the paths and without it, on Node and on Bun, for each arm. Seven rounds; the runtime that
// goes first alternates and the arm order rotates by one each round, so no arm always follows the same one.
// One JSON line per run, then a done line.

import { spawnSync, } from 'node:child_process';
import { readFileSync, realpathSync, } from 'node:fs';

const RUNTIMES = ['node', 'bun',] as const;
const ARMS = ['npm-1.3.6', 'main-before-1121', 'main', 'main-again', 'base',] as const;
const ROUNDS = 7;
const STEPS = [
  { name: 'separator', args: ['--paths', '1000', '--runs', '7', '--warmup', '2', '--separator',], },
  { name: 'no-separator', args: ['--paths', '1000', '--runs', '7', '--warmup', '2',], },
] as const;

const versions = spawnSync('sh', ['-c', 'node --version; bun --version',], { encoding: 'utf8', },);

console.log(JSON.stringify({
  runtimes: versions.stdout.trim().split('\n',),
  packages: ARMS.map((arm,) => {
    const folder = realpathSync(`/timing-1121/${arm}/node_modules/@optique/core`,);
    const manifest = JSON.parse(readFileSync(`${folder}/package.json`, 'utf8',),) as { version?: string; };

    return { arm, folder, version: manifest.version, };
  },),
},),);

for (let round = 1; round <= ROUNDS; round += 1) {
  const runtimes = round % 2 === 1 ? RUNTIMES : [...RUNTIMES,].reverse();
  const offset = (round - 1) % ARMS.length;
  const arms = [...ARMS.slice(offset,), ...ARMS.slice(0, offset,),];

  for (const runtime of runtimes) {
    for (const step of STEPS) {
      for (const arm of arms) {
        const startedAt = Date.now();
        const child = spawnSync(runtime, ['commit-paths.ts', ...step.args,], { cwd: `/timing-1121/${arm}`, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, },);
        let result: unknown = null;

        try {
          result = JSON.parse(child.stdout.trim().split('\n',).at(-1,) ?? '',);
        }
        catch (error) {
          result = { unparsed: child.stdout.slice(0, 300,), reason: String(error,), };
        }

        console.log(JSON.stringify({
          round,
          runtime,
          step: step.name,
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
