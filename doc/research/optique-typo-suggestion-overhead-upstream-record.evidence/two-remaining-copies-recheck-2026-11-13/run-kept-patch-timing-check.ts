// Builds the image that holds the kept patch's arm and times it beside the agent's arm and the base, mount-free and
// with no network, while no other container of this recheck is going. Then runs the two behaviour probes on it.
//
// Run: node run-kept-patch-timing-check.ts
// Writes keep/kept-patch-image-build.log, keep/timing-round-kept-patch.jsonl (the name the timing summariser picks
// up) and what run-probes-on-the-kept-patch.ts writes.

import { spawnSync, } from 'node:child_process';
import { writeFileSync, } from 'node:fs';

const HERE = import.meta.dirname;
const IMAGE = 'localhost/optique-two-remaining-copies-recheck-kept-patch:2026-11-13';
const build = spawnSync('podman', ['build', '--tag', IMAGE, '--file', 'Containerfile.kept-patch', '.',], { cwd: HERE, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, },);

writeFileSync(`${HERE}/keep/kept-patch-image-build.log`, `${build.stdout}${build.stderr}`,);

if (build.status !== 0) throw new Error('the image build failed',);

const run = spawnSync('podman', ['run', '--rm', '--network', 'none', '--cpus', '2', '--memory', '8g', '--pids-limit', '1024', '--blkio-weight', '10', IMAGE,], {
  encoding: 'utf8',
  maxBuffer: 256 * 1024 * 1024,
},);

writeFileSync(`${HERE}/keep/timing-round-kept-patch.jsonl`, `${run.stdout}${JSON.stringify({ containerExit: run.status, },)}\n`,);
writeFileSync(`${HERE}/keep/timing-round-kept-patch.stderr.log`, run.stderr,);

const probes = spawnSync(process.execPath, [`${HERE}/run-probes-on-the-kept-patch.ts`,], { encoding: 'utf8', },);

console.log(JSON.stringify({ timingExit: run.status, probesExit: probes.status, probesStderr: probes.stderr.slice(0, 300,), },),);
