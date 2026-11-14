// Run inside the work container, with the suite fixture at /work, the staged arms at /arms and an output folder at
// /out: builds @optique/core from each named arm's source with upstream's own build (tsdown) and copies the built
// package (package.json and dist) to /out/<arm>, so that Node and Bun, which need the built package, can be timed on
// every arm from one image.
//
//   node /suite/build-arm-packages-inside.ts main base multiple-final both concat

import { spawnSync, } from 'node:child_process';
import { cpSync, mkdirSync, readFileSync, readdirSync, rmSync, } from 'node:fs';

const CORE = '/work/optique/packages/core';

for (const arm of process.argv.slice(2,)) {
  rmSync(`${CORE}/src`, { recursive: true, force: true, },);
  cpSync(`/arms/${arm}/packages/core/src`, `${CORE}/src`, { recursive: true, },);
  rmSync(`${CORE}/dist`, { recursive: true, force: true, },);

  const build = spawnSync('pnpm', ['build',], { cwd: CORE, encoding: 'utf8', maxBuffer: 256 * 1024 * 1024, },);

  rmSync(`/out/${arm}`, { recursive: true, force: true, },);
  mkdirSync(`/out/${arm}`, { recursive: true, },);
  cpSync(`${CORE}/package.json`, `/out/${arm}/package.json`,);
  cpSync(`${CORE}/dist`, `/out/${arm}/dist`, { recursive: true, },);

  const manifest = JSON.parse(readFileSync(`${CORE}/package.json`, 'utf8',),) as { dependencies?: Record<string, string>; };

  console.log(JSON.stringify({
    arm,
    build: build.status,
    tail: `${build.stdout}${build.stderr}`.trim().split('\n',).slice(-1,),
    distFiles: readdirSync(`/out/${arm}/dist`,).length,
    runtimeDependencies: Object.keys(manifest.dependencies ?? {},),
    holdsTheStateFix: readdirSync(`/out/${arm}/dist`,).some((name,) => name.startsWith('persistent-vector',)) ||
      readFileSync(`/out/${arm}/dist/modifiers.js`, 'utf8',).includes('appendTo',),
    holdsConcat: readFileSync(`/out/${arm}/dist/modifiers.js`, 'utf8',).includes('.concat([nextItemState',),
  },),);
}

console.log(JSON.stringify({ done: true, },),);
