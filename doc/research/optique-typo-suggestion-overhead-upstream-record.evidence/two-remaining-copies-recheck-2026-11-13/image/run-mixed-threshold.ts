// Ten rounds of base against the multiple() state fix on alternating option-and-argument input around the size at
// which the fix starts handing out views (1,024 items in a state; this input fills two states, so they pass it at
// about 6,000 tokens), with the two arms taking turns to go first, and with the base run twice per round so that
// the spread between two runs of identical code is measured beside the difference between the arms.
//
// Run inside the image: node /run-mixed-threshold.ts

import { spawnSync, } from 'node:child_process';

const ROUNDS = 10;
const ARGS = ['--counts', '1500,3000,4500,6000,7500,9000,12000', '--runs', '5', '--warmup', '2',];

function run({ round, arm, label, }: { round: number; arm: string; label: string; },): void {
  const child = spawnSync('deno', ['run', '--no-config', '--import-map', `/maps/recheck-${arm}.json`, 'mixed-scaling.ts', ...ARGS,], {
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

  console.log(JSON.stringify({ round, step: 'mixed-threshold', arm: label, exit: child.status, result, stderr: child.stderr.slice(0, 400,), },),);
}

for (let round = 1; round <= ROUNDS; round += 1) {
  const order = round % 2 === 1
    ? [{ arm: 'base', label: 'base', }, { arm: 'multiple', label: 'multiple', }, { arm: 'base', label: 'base-again', },]
    : [{ arm: 'multiple', label: 'multiple', }, { arm: 'base', label: 'base-again', }, { arm: 'base', label: 'base', },];

  for (const { arm, label, } of order) run({ round, arm, label, },);
}

console.log(JSON.stringify({ done: true, },),);
