// Times upstream's pull request 1121 on Node and Bun, which until now was measured from source on Deno only.
//
// Arms, each a built @optique/core resolved as an installed package:
//   npm-1.3.6          the published 1.3.6 tarball (already in the base image, SHA-256 checked when that was built)
//   main-before-1121   upstream main just before the merge, commit 84f95710, built here with upstream's own build
//   main               upstream main at the merge, d26f1a24, whose tree is the pull request's head (built earlier)
//   main-again         the very same package as main under a second name: the same-code control for the spread
//   base               main with the input-trace patch (built earlier)
//
// Steps: stage the one new source arm from the disposable upstream clone, build it in the work image with no network,
// bake it into an image on top of the earlier Node-and-Bun image, run the timing mount-free with no network.
//
// Run: node run-pull-request-1121-on-node-and-bun.ts
// Writes keep/staging.json, keep/arm-package-build.jsonl, keep/image-build.log, keep/timing-rounds.jsonl and its
// stderr log.

import { spawnSync, } from 'node:child_process';
import { cpSync, mkdirSync, rmSync, writeFileSync, } from 'node:fs';
import { homedir, } from 'node:os';

const HERE = import.meta.dirname;
const HOME = homedir();
const CLONE = `${HOME}/temp/agent/upstream-prototype.O1iPjcqr/optique`;
const RECHECK = `${HOME}/temp/agent/optique-two-remaining-copies-recheck-2026-11-13`;
const BEFORE = '84f95710';
const MERGE = 'd26f1a24';
const IMAGE = 'localhost/optique-pull-request-1121-on-node-and-bun:2026-11-13';

function run(command: string, args: readonly string[], cwd = HERE,): { status: number | null; stdout: string; stderr: string; } {
  const child = spawnSync(command, args, { cwd, encoding: 'utf8', maxBuffer: 512 * 1024 * 1024, },);

  return { status: child.status, stdout: child.stdout, stderr: child.stderr, };
}

function must(command: string, args: readonly string[], cwd = HERE,): string {
  const result = run(command, args, cwd,);

  if (result.status !== 0) throw new Error(`${command} ${args.join(' ',)} failed: ${result.stderr.slice(0, 600,)}`,);

  return result.stdout;
}

mkdirSync(`${HERE}/keep`, { recursive: true, },);

//region Stage the source of main just before the merge, read from the clone without changing it

const armSource = `${HERE}/arms/main-before-1121`;

rmSync(`${HERE}/arms`, { recursive: true, force: true, },);
mkdirSync(armSource, { recursive: true, },);

const archive = `${HERE}/main-before-1121.tar`;

must('git', ['archive', '--format=tar', `--output=${archive}`, BEFORE, 'packages/core/src',], CLONE,);
must('tar', ['--extract', `--file=${archive}`, `--directory=${armSource}`,],);
rmSync(archive,);

writeFileSync(
  `${HERE}/keep/staging.json`,
  `${JSON.stringify({
    before: must('git', ['rev-parse', `${BEFORE}^{commit}`,], CLONE,).trim(),
    merge: must('git', ['rev-parse', `${MERGE}^{commit}`,], CLONE,).trim(),
    mergeParents: must('git', ['rev-parse', `${MERGE}^1`, `${MERGE}^2`,], CLONE,).trim().split('\n',),
    mergeTree: must('git', ['rev-parse', `${MERGE}^{tree}`,], CLONE,).trim(),
    sourceFilesChangedByTheMerge: must('git', ['diff', '--stat', BEFORE, MERGE, '--', 'packages/core/src', 'packages/core/package.json', 'packages/core/tsdown.config.ts',], CLONE,).trim().split('\n',),
  }, null, 2,)}\n`,
);

//endregion

//region Build the staged arm as a package, in the work image, with no network

rmSync(`${HERE}/arm-packages`, { recursive: true, force: true, },);
mkdirSync(`${HERE}/arm-packages`, { recursive: true, },);

const build = run('podman', [
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
  `${RECHECK}/suite-fixture:/work:rw`,
  '--volume',
  `${HERE}/arms:/arms:ro`,
  '--volume',
  `${RECHECK}/suite:/suite:ro`,
  '--volume',
  `${HERE}/arm-packages:/out:rw`,
  '--env',
  'HOME=/work/container-home',
  '--entrypoint',
  'node',
  'localhost/optique-prototype-work-current-bun:2026-11-13',
  '/suite/build-arm-packages-inside.ts',
  'main-before-1121',
],);

writeFileSync(`${HERE}/keep/arm-package-build.jsonl`, `${build.stdout}${JSON.stringify({ containerExit: build.status, stderr: build.stderr.slice(0, 600,), },)}\n`,);

if (build.status !== 0 || !build.stdout.includes('"build":0',)) throw new Error('building the arm package failed',);

//endregion

//region Bake and run

cpSync(`${HERE}/run-timing-inside.ts`, `${HERE}/image-context-run-timing-inside.ts`,);

const image = run('podman', ['build', '--tag', IMAGE, '--file', 'Containerfile', '.',],);

writeFileSync(`${HERE}/keep/image-build.log`, `${image.stdout}${image.stderr}`,);

if (image.status !== 0) throw new Error('the image build failed',);

const timing = run('podman', ['run', '--rm', '--network', 'none', '--cpus', '2', '--memory', '4g', '--pids-limit', '512', '--blkio-weight', '10', IMAGE,],);

writeFileSync(`${HERE}/keep/timing-rounds.jsonl`, `${timing.stdout}${JSON.stringify({ containerExit: timing.status, },)}\n`,);
writeFileSync(`${HERE}/keep/timing-rounds.stderr.log`, timing.stderr,);
console.log(JSON.stringify({ containerExit: timing.status, lines: timing.stdout.split('\n',).length, },),);

//endregion
