// Builds the image for the mixed-input threshold check and runs it, mount-free and with no network, while no other
// container of this recheck is going.
//
// Run: node run-mixed-threshold-check.ts
// Writes keep/mixed-threshold-image-build.log and keep/timing-round-mixed-threshold.jsonl (the name the timing
// summariser picks up) with its stderr log.

import { spawnSync, } from 'node:child_process';
import { writeFileSync, } from 'node:fs';

const HERE = import.meta.dirname;
const IMAGE = 'localhost/optique-two-remaining-copies-recheck-mixed-threshold:2026-11-13';
const build = spawnSync('podman', ['build', '--tag', IMAGE, '--file', 'Containerfile.mixed-threshold', '.',], { cwd: HERE, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, },);

writeFileSync(`${HERE}/keep/mixed-threshold-image-build.log`, `${build.stdout}${build.stderr}`,);

if (build.status !== 0) throw new Error('the image build failed',);

const run = spawnSync('podman', ['run', '--rm', '--network', 'none', '--cpus', '2', '--memory', '8g', '--pids-limit', '1024', '--blkio-weight', '10', IMAGE,], {
  encoding: 'utf8',
  maxBuffer: 256 * 1024 * 1024,
},);

writeFileSync(`${HERE}/keep/timing-round-mixed-threshold.jsonl`, `${run.stdout}${JSON.stringify({ containerExit: run.status, },)}\n`,);
writeFileSync(`${HERE}/keep/timing-round-mixed-threshold.stderr.log`, run.stderr,);
