// Run inside the work container: runs one test file of upstream's packages/core on Bun several times, alternating
// between two arms, to see whether a failure seen once in the full suite follows the arm or comes and goes on its
// own. The file is completion-bash-quoting.test.ts, whose tests drive an interactive bash; one of them failed once,
// in the full Bun suite of the arm `multiple-final`, with the bash it started ended by SIGPIPE.
//
//   node /suite/repeat-one-bun-test-file.ts

import { spawnSync, } from 'node:child_process';
import { cpSync, rmSync, } from 'node:fs';

const CORE = '/work/optique/packages/core';
const FILE = 'src/completion-bash-quoting.test.ts';
const ROUNDS = 5;
const STRIP_COLOUR = /\u001b\[[0-9;]*m/gu;

for (let round = 1; round <= ROUNDS; round += 1) {
  for (const arm of ['base', 'multiple-final',]) {
    rmSync(`${CORE}/src`, { recursive: true, force: true, },);
    cpSync(`/arms/${arm}/packages/core/src`, `${CORE}/src`, { recursive: true, },);

    const build = spawnSync('pnpm', ['build',], { cwd: CORE, encoding: 'utf8', },);
    const startedAt = Date.now();
    const run = spawnSync('bun', ['test', FILE,], { cwd: CORE, encoding: 'utf8', maxBuffer: 256 * 1024 * 1024, },);
    const output = `${run.stdout}${run.stderr}`.replace(STRIP_COLOUR, '',);

    console.log(JSON.stringify({
      round,
      arm,
      build: build.status,
      exit: run.status,
      seconds: Math.round((Date.now() - startedAt) / 1000,),
      summary: output.split('\n',).filter((line,) => /^\s*\d+ (pass|fail|skip)$|^Ran \d+ tests/u.test(line,)).map((line,) => line.trim()),
      failing: output.split('\n',).filter((line,) => line.startsWith('(fail)',)).map((line,) => line.slice(0, 200,)),
      signals: [...new Set(output.match(/signal: "[A-Z]+"/gu,) ?? [],),],
    },),);
  }
}

console.log(JSON.stringify({ done: true, },),);
