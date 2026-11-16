// Rebuilds the recheck image after the timing runner's fix (the step for the base arms at 64,000 tokens named one
// count, which token-scaling.ts refuses), runs that step for round 1, where it failed, and then rounds 2 to 5 of
// every step. Each run is mount-free and has no network.
//
// Run: node run-remaining-timing-rounds.ts
// Writes keep/image-rebuild.log, keep/timing-round-1-large-base.jsonl and keep/timing-rounds-2-to-5.jsonl.

import { spawnSync, } from 'node:child_process';
import { writeFileSync, } from 'node:fs';

const HERE = import.meta.dirname;
const build = spawnSync('podman', ['build', '--tag', 'localhost/optique-two-remaining-copies-recheck:2026-11-13', '--file', 'Containerfile.recheck', '.',], {
  cwd: HERE,
  encoding: 'utf8',
  maxBuffer: 64 * 1024 * 1024,
},);

writeFileSync(`${HERE}/keep/image-rebuild.log`, `${build.stdout}${build.stderr}`,);

if (build.status !== 0) throw new Error('the image rebuild failed',);

for (
  const [label, ...args] of [
    ['timing-round-1-large-base', '--first-round', '1', '--rounds', '1', '--steps', 'large-base',],
    ['timing-rounds-2-to-5', '--first-round', '2', '--rounds', '4',],
  ]
) {
  const run = spawnSync('node', [`${HERE}/run-in-recheck-image.ts`, label as string, 'node', '/run-timing.ts', ...args,], { encoding: 'utf8', },);

  console.log(JSON.stringify({ label, exit: run.status, stderr: run.stderr.slice(0, 400,), },),);
}
