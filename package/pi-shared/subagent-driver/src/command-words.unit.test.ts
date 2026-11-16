/**
 Reading a command as the shell splits it, far enough to tell which words a command runs: a program's own position, an interpreter's
 script, a wrapper's program, and a word that merely names a path.

 Every case is a command text the test writes, so nothing is run and nothing is read.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  buildToolRanIn,
  executedWordsIn,
  scriptRunsABuildTool,
} from '../dist/final/node/index.mjs';

await describe({
  name: '',
  children: [
    describe({
      name: executedWordsIn.name,
      children: [
        it({
          name: 'reads a program in its own segment, and the script an interpreter or a shell runs after it',
          fn: async () => {
            expect(executedWordsIn({ command: 'ls dist/x.js', },),).toEqual([
              { word: 'ls', role: 'program', },
            ],);
            expect(executedWordsIn({ command: 'node repro.ts', },),).toEqual([
              { word: 'node', role: 'program', },
              { word: 'repro.ts', role: 'script', },
            ],);
            expect(executedWordsIn({ command: 'bash -c "echo hi"', },),).toEqual([
              { word: 'bash', role: 'program', },
              { word: 'echo', role: 'program', },
            ],);
            expect(executedWordsIn({ command: 'node --input-type=module -e "console.log(1)"', },),).toEqual([
              { word: 'node', role: 'program', },
            ],);
          },
        },),
        it({
          name: 'reads the program after a wrapper\'s own words, including a scope and its `--`',
          fn: async () => {
            expect(executedWordsIn({ command: 'timeout 60 node repro.ts', },),).toEqual([
              { word: 'node', role: 'program', },
              { word: 'repro.ts', role: 'script', },
            ],);
            expect(executedWordsIn({
              command: 'systemd-run --user --scope --property=MemoryMax=4G -- /bin/bash run.sh',
            },),).toEqual([
              { word: '/bin/bash', role: 'program', },
              { word: 'run.sh', role: 'script', },
            ],);
            expect(executedWordsIn({ command: 'nice -n 5 ./target/debug/a --version', },),).toEqual([
              { word: './target/debug/a', role: 'program', },
            ],);
            expect(executedWordsIn({ command: 'env HOME=/tmp cargo build', },),).toEqual([
              { word: 'cargo', role: 'program', },
            ],);
          },
        },),
        it({
          name: 'reads each segment of a chain, and takes the runs after `&&`, `|` and `;` as programs too',
          fn: async () => {
            expect(executedWordsIn({ command: 'cd a && VAR=1 node repro.ts | tee out', },),).toEqual([
              { word: 'cd', role: 'program', },
              { word: 'node', role: 'program', },
              { word: 'repro.ts', role: 'script', },
              { word: 'tee', role: 'program', },
            ],);
          },
        },),
      ],
    },),
    describe({
      name: buildToolRanIn.name,
      children: [
        it({
          name: 'reads a build tool in a program position, and not one merely named',
          fn: async () => {
            expect(buildToolRanIn({ command: 'mise run //package/a:native:build', },),).toBe(true,);
            expect(buildToolRanIn({ command: 'cargo build', },),).toBe(true,);
            expect(buildToolRanIn({ command: 'rg cargo build', },),).toBe(false,);
            expect(buildToolRanIn({ command: 'ls dist/x.js', },),).toBe(false,);
          },
        },),
      ],
    },),
    describe({
      name: scriptRunsABuildTool.name,
      children: [
        it({
          name: 'reads a build tool in a script line\'s program position',
          fn: async () => {
            expect(scriptRunsABuildTool({ text: 'exec mise exec -- cargo build\n', },),).toBe(true,);
            expect(scriptRunsABuildTool({ text: 'echo cargo build\n', },),).toBe(false,);
          },
        },),
      ],
    },),
  ],
},);
