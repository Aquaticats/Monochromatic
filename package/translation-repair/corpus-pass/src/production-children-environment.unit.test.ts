/**
 Tests that each child process production code starts holds no credential,
 no setting of the package and, for a corpus read, no repository routing
 variable, read from the child's own side.

 THE PROOF IS A REAL CHILD. Each case gives this process an invented key
 (`WHISKER_API_KEY`, never a real one), an invented setting and a plain
 variable that is the positive control (a child that cannot see the plain
 variable would pass for the wrong reason), runs one site against a
 throwaway repository, and reads which of those names the child held: for
 git from its own trace (`GIT_TRACE2_EVENT` with `GIT_TRACE2_ENV_VARS`, so
 nothing but the watched names is written), for the program the picture
 reader runs from a file the program writes. The sites are in
 `corpus-source.ts`, `corpus-run/artifact-generation.ts`,
 `corpus-run/cache-account-git.ts`, `corpus-run/run-config.ts` and
 `image-ocr.ts`. The suite runs one case at a time, since the invented
 variables are process-wide writes.

 @module
 */

import {
  readFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

import { runInstalledProgram, } from '@monochromatic-dev/translation-repair-slice-reading';
import {
  resolveCommit,
  tipContains,
} from '@monochromatic-dev/translation-repair-artifact-pool';
import {
  readHeadSha,
  resolveRunsDir,
} from '@monochromatic-dev/translation-repair-run';
import {
  listCorpusPeople,
  readCorpusBytes,
  readCorpusFile,
} from '@monochromatic-dev/translation-repair-corpus-source';
import { cacheAccountGitOutput, } from '@monochromatic-dev/translation-repair-slice-cache';
import { makeNamingArchive, } from './archive-naming.test-fixture.ts';
import {
  INVENTED_PLAIN,
  inventedEnvironment,
  namesGitHeld,
} from '@monochromatic-dev/translation-repair-test-fixture/ts/child-sight.test-fixture.ts';
import { scratchDir, } from '@monochromatic-dev/translation-repair-test-fixture/ts/scratch-dir.test-fixture.ts';

/**
 What a git child holds once the rule is kept: the plain variable alone.
 */
const PLAIN_ONLY: readonly string[] = [INVENTED_PLAIN,];

await describe({
  name: 'children of production code',
  concurrency: 1,
  children: [
    it({
      name: 'STARTS the git child of readCorpusFile with the plain variable and neither the key nor the setting',
      fn: async () => {
        await using archive = await makeNamingArchive({},);
        await using scratch = await scratchDir({ prefix: 'child-sight-', },);
        const tracePath = join(
          scratch.path,
          'trace.json',
        );
        using held = inventedEnvironment({ tracePath, },);
        void held;
        await readCorpusFile({ pin: archive.pin, relPath: archive.relPath, },);
        expect(await namesGitHeld({ tracePath, },),).toEqual(PLAIN_ONLY,);
      },
    },),
    it({
      name: 'STARTS the git child of readCorpusBytes with the plain variable and neither the key nor the setting',
      fn: async () => {
        await using archive = await makeNamingArchive({},);
        await using scratch = await scratchDir({ prefix: 'child-sight-', },);
        const tracePath = join(
          scratch.path,
          'trace.json',
        );
        using held = inventedEnvironment({ tracePath, },);
        void held;
        await readCorpusBytes({ pin: archive.pin, relPath: archive.relPath, },);
        expect(await namesGitHeld({ tracePath, },),).toEqual(PLAIN_ONLY,);
      },
    },),
    it({
      name: 'STARTS the git child of listCorpusPeople with the plain variable and neither the key nor the setting',
      fn: async () => {
        await using archive = await makeNamingArchive({},);
        await using scratch = await scratchDir({ prefix: 'child-sight-', },);
        const tracePath = join(
          scratch.path,
          'trace.json',
        );
        using held = inventedEnvironment({ tracePath, },);
        void held;
        await listCorpusPeople({ pin: archive.pin, },);
        expect(await namesGitHeld({ tracePath, },),).toEqual(PLAIN_ONLY,);
      },
    },),
    it({
      name: 'STARTS the git child of listCorpusPeople without the repository routing variable GIT_DIR '
        + 'this process holds, and lists the clone the pin names',
      fn: async () => {
        await using archive = await makeNamingArchive({},);
        await using scratch = await scratchDir({ prefix: 'child-sight-', },);
        const tracePath = join(
          scratch.path,
          'trace.json',
        );
        using held = inventedEnvironment({
          tracePath,
          extra: { GIT_DIR: join(
            scratch.path,
            'no-such-repository',
          ), },
        },);
        void held;
        expect(await listCorpusPeople({ pin: archive.pin, },),).toEqual(['starlit-cat',],);
        expect(await namesGitHeld({ tracePath, },),).toEqual(PLAIN_ONLY,);
      },
    },),
    it({
      name: 'STARTS the git child of resolveCommit with the plain variable and neither the key nor the setting',
      fn: async () => {
        await using archive = await makeNamingArchive({},);
        await using scratch = await scratchDir({ prefix: 'child-sight-', },);
        const tracePath = join(
          scratch.path,
          'trace.json',
        );
        using held = inventedEnvironment({ tracePath, },);
        void held;
        await resolveCommit({ revision: archive.pin.commitSha, repository: archive.pin.cloneDir, },);
        expect(await namesGitHeld({ tracePath, },),).toEqual(PLAIN_ONLY,);
      },
    },),
    it({
      name: 'STARTS both git children of tipContains, the ancestry check and the shallow check a clean negative '
        + 'asks, with the plain variable and neither the key nor the setting',
      fn: async () => {
        await using archive = await makeNamingArchive({},);
        await using scratch = await scratchDir({ prefix: 'child-sight-', },);
        const tracePath = join(
          scratch.path,
          'trace.json',
        );
        using held = inventedEnvironment({ tracePath, },);
        void held;
        expect(await tipContains({
          tip: archive.parentCommit,
          commit: archive.pin.commitSha,
          repository: archive.pin.cloneDir,
        },),).toBe(false,);
        expect(await namesGitHeld({ tracePath, },),).toEqual(PLAIN_ONLY,);
      },
    },),
    it({
      name: 'STARTS the git child of cacheAccountGitOutput with the plain variable and neither the key nor the setting',
      fn: async () => {
        await using archive = await makeNamingArchive({},);
        await using scratch = await scratchDir({ prefix: 'child-sight-', },);
        const tracePath = join(
          scratch.path,
          'trace.json',
        );
        using held = inventedEnvironment({ tracePath, },);
        void held;
        await cacheAccountGitOutput({ args: ['-C', archive.pin.cloneDir, 'rev-parse', 'HEAD',], },);
        expect(await namesGitHeld({ tracePath, },),).toEqual(PLAIN_ONLY,);
      },
    },),
    it({
      name: 'STARTS the git child of readHeadSha with the plain variable and neither the key nor the setting',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'child-sight-', },);
        const tracePath = join(
          scratch.path,
          'trace.json',
        );
        using held = inventedEnvironment({ tracePath, },);
        void held;
        await readHeadSha();
        expect(await namesGitHeld({ tracePath, },),).toEqual(PLAIN_ONLY,);
      },
    },),
    it({
      name: 'STARTS the git child of resolveRunsDir, asked for the worktree root when no runs directory '
        + 'setting stands, with the plain variable and neither the key nor the setting',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'child-sight-', },);
        const tracePath = join(
          scratch.path,
          'trace.json',
        );
        using held = inventedEnvironment({ tracePath, unset: ['TRANSLATION_REPAIR_RUNS_DIR',], },);
        void held;
        await resolveRunsDir();
        expect(await namesGitHeld({ tracePath, },),).toEqual(PLAIN_ONLY,);
      },
    },),
    it({
      name: 'STARTS the program runInstalledProgram runs with the plain variable and neither the key nor the setting',
      fn: async () => {
        await using scratch = await scratchDir({ prefix: 'child-sight-', },);
        const reportPath = join(
          scratch.path,
          'seen.json',
        );
        using held = inventedEnvironment({ tracePath: join(
          scratch.path,
          'unused-trace.json',
        ), },);
        void held;
        await runInstalledProgram({
          program: process.execPath,
          args: [
            '--eval',
            [
              'const names = Object.keys(process.env)',
              '  .filter((name) => name.includes(\'WHISKER\'))',
              '  .toSorted();',
              'require(\'node:fs\').writeFileSync(process.argv[1], JSON.stringify(names));',
            ].join('\n',),
            reportPath,
          ],
        },);
        /**
         Names the program wrote.
         */
        const seen = await readFile(
          reportPath,
          'utf8',
        );
        expect(JSON.parse(seen,),).toEqual([INVENTED_PLAIN,],);
      },
    },),
  ],
},);
