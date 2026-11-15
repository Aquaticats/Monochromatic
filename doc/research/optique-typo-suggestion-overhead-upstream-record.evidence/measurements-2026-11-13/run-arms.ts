// Measures five arms on Deno with the published scripts, in rotating order so no arm always runs first:
// @optique/core 1.3.6 from npm, upstream main, upstream's pull request 1121, and the last two with the input-trace
// prototype applied. Prints one JSON line per run.

import { spawnSync, } from 'node:child_process';

const ARMS = [
  { arm: '1.3.6', map: 'maps/npm-1.3.6.json', },
  { arm: 'main', map: '/maps/src-main.json', },
  { arm: 'pr1121', map: '/maps/src-pr1121.json', },
  { arm: 'main-patched', map: '/maps/src-main-patched.json', },
  { arm: 'pr1121-patched', map: '/maps/src-pr1121-patched.json', },
] as const;
const SCRIPTS = [
  { script: 'commit-paths.ts', args: ['--separator',], },
  { script: 'commit-paths.ts', args: [], },
  { script: 'token-scaling.ts', args: [], },
] as const;
const ROUNDS = 7;

for (const { arm, map, } of ARMS) {
  const child = spawnSync('deno', ['run', '--no-config', '--import-map', map, 'mismatch-count.ts',], { cwd: '/repo', encoding: 'utf8', },);

  console.log(JSON.stringify({ runtime: 'deno', arm, script: 'mismatch-count.ts', args: [], round: 0, exit: child.status, result: JSON.parse(child.stdout.trim().split('\n',).at(-1,) ?? 'null',), stderr: child.stderr.slice(0, 300,), },),);
}

for (let round = 1; round <= ROUNDS; round += 1) {
  const offset = round % ARMS.length;
  const order = [...ARMS.slice(offset,), ...ARMS.slice(0, offset,),];

  for (const { script, args, } of SCRIPTS) {
    for (const { arm, map, } of order) {
      const child = spawnSync('deno', ['run', '--no-config', '--import-map', map, script, ...args,], {
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

      console.log(JSON.stringify({ runtime: 'deno', arm, script, args: [...args,], round, exit: child.status, result, stderr: child.stderr.slice(0, 300,), },),);
    }
  }
}
