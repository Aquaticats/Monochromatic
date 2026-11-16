// Builds the image for the annotated-parse check and runs it, mount-free and with no network.
//
// Run: node run-annotated-check.ts
// Writes keep/annotated-image-build.log, keep/annotated-parse-timing.jsonl and its stderr log.

import { spawnSync, } from 'node:child_process';
import { writeFileSync, } from 'node:fs';

const HERE = import.meta.dirname;
const IMAGE = 'localhost/optique-two-remaining-copies-recheck-annotated:2026-11-13';
const build = spawnSync('podman', ['build', '--tag', IMAGE, '--file', 'Containerfile.annotated', '.',], { cwd: HERE, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, },);

writeFileSync(`${HERE}/keep/annotated-image-build.log`, `${build.stdout}${build.stderr}`,);

if (build.status !== 0) throw new Error('the image build failed',);

const run = spawnSync('podman', ['run', '--rm', '--network', 'none', '--cpus', '2', '--memory', '8g', '--pids-limit', '1024', '--blkio-weight', '10', IMAGE,], {
  encoding: 'utf8',
  maxBuffer: 256 * 1024 * 1024,
},);

writeFileSync(`${HERE}/keep/annotated-parse-timing.jsonl`, `${run.stdout}${JSON.stringify({ containerExit: run.status, },)}\n`,);
writeFileSync(`${HERE}/keep/annotated-parse-timing.stderr.log`, run.stderr,);
