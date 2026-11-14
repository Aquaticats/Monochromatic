// Runs the three published scripts on Node against the older release lines the image installed, with 1.3.6 beside
// them as the reference, in rotating order for the two timed scripts. Prints one JSON line per run.

import { spawnSync, } from 'node:child_process';
import { readFileSync, } from 'node:fs';

const older = readFileSync('/arm/older-lines.txt', 'utf8',).trim().split('\n',);
const arms = [...older, '1.3.6',];
const ROUNDS = 5;

console.log(JSON.stringify({ arms, sha256: readFileSync('/arm/older-lines.sha256', 'utf8',).trim().split('\n',), },),);

function run({ arm, script, args, round, }: { arm: string; script: string; args: string[]; round: number; },): void {
  const child = spawnSync('node', [script, ...args,], { cwd: `/arm/${arm}`, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, },);
  let result: unknown = null;

  try {
    result = JSON.parse(child.stdout.trim().split('\n',).at(-1,) ?? '',);
  }
  catch (error) {
    result = { unparsed: child.stdout.slice(0, 300,), reason: String(error,), };
  }

  console.log(JSON.stringify({ runtime: 'node', arm, script, args, round, exit: child.status, result, stderr: child.stderr.slice(0, 400,), },),);
}

for (const arm of arms) run({ arm, script: 'mismatch-count.ts', args: [], round: 0, },);

for (let round = 1; round <= ROUNDS; round += 1) {
  const offset = round % arms.length;
  const order = [...arms.slice(offset,), ...arms.slice(0, offset,),];

  for (const arm of order) run({ arm, script: 'token-scaling.ts', args: [], round, },);
  for (const arm of order) run({ arm, script: 'commit-paths.ts', args: ['--separator',], round, },);
}
