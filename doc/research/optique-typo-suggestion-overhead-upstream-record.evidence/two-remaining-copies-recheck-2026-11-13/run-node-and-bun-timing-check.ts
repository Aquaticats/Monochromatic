// Builds @optique/core from each arm (in the work image, no network, with this recheck's own folders mounted), bakes
// the built packages into an image, and times them on Node and Bun, mount-free and with no network.
//
// Run: node run-node-and-bun-timing-check.ts
// Writes keep/arm-packages-build.jsonl, keep/node-and-bun-image-build.log and
// keep/timing-round-node-and-bun.jsonl (the name the timing summariser picks up) with its stderr log.

import { spawnSync, } from 'node:child_process';
import { mkdirSync, rmSync, writeFileSync, } from 'node:fs';

const HERE = import.meta.dirname;
const IMAGE = 'localhost/optique-two-remaining-copies-recheck-node-and-bun:2026-11-13';

rmSync(`${HERE}/arm-packages`, { recursive: true, force: true, },);
mkdirSync(`${HERE}/arm-packages`, { recursive: true, },);

const packages = spawnSync('podman', [
  'run',
  '--rm',
  '--network',
  'none',
  '--cpus',
  '4',
  '--memory',
  '8g',
  '--pids-limit',
  '1024',
  '--blkio-weight',
  '10',
  '--security-opt',
  'label=disable',
  '--volume',
  `${HERE}/suite-fixture:/work:rw`,
  '--volume',
  `${HERE}/arms:/arms:ro`,
  '--volume',
  `${HERE}/suite:/suite:ro`,
  '--volume',
  `${HERE}/arm-packages:/out:rw`,
  '--env',
  'HOME=/work/container-home',
  '--entrypoint',
  'node',
  'localhost/optique-prototype-work-current-bun:2026-11-13',
  '/suite/build-arm-packages-inside.ts',
  'main',
  'base',
  'multiple-final',
  'both',
  'concat',
], { encoding: 'utf8', maxBuffer: 256 * 1024 * 1024, },);

writeFileSync(`${HERE}/keep/arm-packages-build.jsonl`, `${packages.stdout}${JSON.stringify({ containerExit: packages.status, stderr: packages.stderr.slice(0, 600,), },)}\n`,);

if (packages.status !== 0) throw new Error('building the arm packages failed',);

const build = spawnSync('podman', ['build', '--tag', IMAGE, '--file', 'Containerfile.node-and-bun', '.',], { cwd: HERE, encoding: 'utf8', maxBuffer: 64 * 1024 * 1024, },);

writeFileSync(`${HERE}/keep/node-and-bun-image-build.log`, `${build.stdout}${build.stderr}`,);

if (build.status !== 0) throw new Error('the image build failed',);

const run = spawnSync('podman', ['run', '--rm', '--network', 'none', '--cpus', '2', '--memory', '8g', '--pids-limit', '1024', '--blkio-weight', '10', IMAGE,], {
  encoding: 'utf8',
  maxBuffer: 256 * 1024 * 1024,
},);

writeFileSync(`${HERE}/keep/timing-round-node-and-bun.jsonl`, `${run.stdout}${JSON.stringify({ containerExit: run.status, },)}\n`,);
writeFileSync(`${HERE}/keep/timing-round-node-and-bun.stderr.log`, run.stderr,);
