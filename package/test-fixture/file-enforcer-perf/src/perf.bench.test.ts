/**
 * Micro-benchmarks for file-enforcer core operations using mitata.
 *
 * mitata provides automatic high-resolution timing, dead-code elimination
 * detection, GC-aware measurement, and auto-batching for fast functions.
 * For reliable end-to-end numbers, use run-e2e.ts with hyperfine instead.
 */

import {
  cat,
  classifyEvent,
  dedup,
  expandGlob,
  getJsonProperty,
  mirrorGlobPath,
  overwrite,
  reset,
  trackDest,
  trackRead,
  trackWriteTime,
} from '@monochromatic-dev/file-enforcer-core/ts';
import {
  bench,
  boxplot,
  do_not_optimize,
  run,
  summary,
} from 'mitata';
import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';

//region Fixture setup
/** Root temp directory for all benchmark fixtures. */
const tempDir = await mkdtemp(join(scratchRoot(), 'fe-perf-',),);

/** Directory with 200 files across 20 directories for glob benchmarks. */
const globDir = join(tempDir, 'glob-fixture',);

/** Number of directories in the glob fixture. */
const DIR_COUNT = 20;

/** Files per directory in the glob fixture. */
const FILES_PER_DIR = 10;

await Promise.all(
  Array.from({ length: DIR_COUNT, }, async (_unused, dirIndex,) => {
    const dir = join(globDir, `dir-${String(dirIndex,).padStart(2, '0',)}`,);
    await mkdir(dir, { recursive: true, },);
    await Promise.all(
      Array.from({ length: FILES_PER_DIR, },
        (_inner, fileIndex,) =>
          writeFile(join(dir, `file-${String(fileIndex,)}.ts`,),
            `content-${String(dirIndex,)}-${String(fileIndex,)}`,),),
    );
  },),
);

/** Directory with 20 text files for cat(string[]) benchmarks. */
const catDir = join(tempDir, 'cat-fixture',);
await mkdir(catDir, { recursive: true, },);

/** Number of files for the cat array benchmark. */
const CAT_FILE_COUNT = 20;

/** Paths to the 20 files used by the cat(string[]) benchmark. */
const catPaths = await Promise.all(
  Array.from({ length: CAT_FILE_COUNT, }, async (_, index,) => {
    const path = join(catDir, `file-${String(index,)}.txt`,);
    await writeFile(path, `content of file ${String(index,)}\n`.repeat(50,),);
    return path;
  },),
);

/** Directory with 60 files across 20 packages for cat(glob) benchmarks. */
const catGlobDir = join(tempDir, 'cat-glob-fixture',);

await Promise.all(
  Array.from({ length: DIR_COUNT, }, async (_, dirIndex,) => {
    const dir = join(catGlobDir, `pkg-${String(dirIndex,).padStart(2, '0',)}`, 'lib',);
    await mkdir(dir, { recursive: true, },);
    await Promise.all(
      ['index', 'utils', 'helpers',].map(function writeFixtureFile(name,) {
        return writeFile(join(dir, `${name}.ts`,), `pkg${String(dirIndex,)} ${name}`,);
      },),
    );
  },),
);

/** File for overwrite benchmarks. */
const overwriteFile = join(tempDir, 'overwrite-target.txt',);
const overwriteContent = 'known content '.repeat(100,);
await writeFile(overwriteFile, overwriteContent,);

/** File for classifyEvent benchmarks. */
const classifyFile = join(tempDir, 'tracked.txt',);
await writeFile(classifyFile, 'content',);
reset();
trackRead(classifyFile,);
trackDest(classifyFile,);
trackWriteTime(classifyFile,);

/** Directory with 6-level deep nested paths for deep glob benchmarks. */
const deepGlobDir = join(tempDir, 'deep-glob-fixture',);

await Promise.all(
  Array.from({ length: DIR_COUNT, }, async (_, dirIndex,) => {
    const deepDir = join(
      deepGlobDir,
      `pkg-${String(dirIndex,).padStart(2, '0',)}`,
      'lib',
      'deep',
      'nested',
      'very',
      'deep',
    );
    await mkdir(deepDir, { recursive: true, },);
    await writeFile(join(deepDir, 'module.ts',), `deep-content-${String(dirIndex,)}`,);
  },),
);

/** Directory with overlapping glob patterns across 20 packages. */
const overlapDir = join(tempDir, 'overlap-fixture',);

await Promise.all(
  Array.from({ length: DIR_COUNT, }, async (_, dirIndex,) => {
    const pkgDir = join(overlapDir, `pkg-${String(dirIndex,).padStart(2, '0',)}`,);
    await mkdir(join(pkgDir, 'lib',), { recursive: true, },);
    await mkdir(join(pkgDir, 'src',), { recursive: true, },);
    await mkdir(join(pkgDir, 'test',), { recursive: true, },);
    await Promise.all([
      writeFile(join(pkgDir, 'lib', 'index.ts',), `lib-${String(dirIndex,)}`,),
      writeFile(join(pkgDir, 'src', 'main.ts',), `src-${String(dirIndex,)}`,),
      writeFile(join(pkgDir, 'test', 'spec.ts',), `test-${String(dirIndex,)}`,),
    ],);
  },),
);

/** Five different glob patterns that partially overlap. */
const overlapPatterns = [
  join(overlapDir, 'pkg-*/lib/*.ts',),
  join(overlapDir, 'pkg-*/src/*.ts',),
  join(overlapDir, 'pkg-*/test/*.ts',),
  join(overlapDir, 'pkg-0*/lib/*.ts',),
  join(overlapDir, 'pkg-1*/lib/*.ts',),
];
//endregion

//region I/O benchmarks: glob expansion, file reading, file writing
summary(function ioBenchmarks() {
  bench('expandGlob: 200 files across 20 directories', async function expandGlobBench() {
    const matches = await expandGlob(join(globDir, '**/*.ts',),);
    do_not_optimize(matches,);
  },);

  bench('cat(string[]): concatenate 20 files', async function catArrayBench() {
    const result = await cat(catPaths,);
    do_not_optimize(result,);
  },);

  bench('cat(glob): glob-read 60 files', async function catGlobBench() {
    const result = await cat(join(catGlobDir, 'pkg-*/lib/*.ts',),);
    do_not_optimize(result,);
  },);

  bench('overwrite: skip (content unchanged)', async function overwriteSkipBench() {
    await overwrite({ dest: overwriteFile, content: overwriteContent, },);
  },);

  bench('overwrite: write (content different)', async function overwriteWriteBench() {
    await overwrite({
      dest: overwriteFile,
      content: `iteration ${String(Date.now(),)}`,
    },);
  },);

  bench('classifyEvent + stat: classification', async function classifyBench() {
    const kind = await classifyEvent({
      filename: 'tracked.txt',
      watchedDir: tempDir,
      configPath: join(tempDir, 'config.ts',),
    },);
    do_not_optimize(kind,);
  },);
},);
//endregion

//region Pure computation benchmarks: string/JSON operations
summary(function computeBenchmarks() {
  bench('dedup: 2000-line content with 50% duplicates', function dedupBench() {
    /** @see LINE_COUNT - number of lines in the dedup fixture */
    const LINE_COUNT = 2_000;
    const DEDUP_MODULO = 1_000;
    const content = Array
      .from(
        { length: LINE_COUNT, },
        (_, lineIndex,) => `line ${String(lineIndex % DEDUP_MODULO,)}`,
      )
      .join('\n',);
    const result = dedup(content,);
    do_not_optimize(result,);
  },);

  bench('getJsonProperty: extract from 1KB JSON', function getJsonPropertyBench() {
    const ARRAY_SIZE = 50;
    const jsonContent = JSON.stringify({
      deeply: { nested: { property: { value: 'found-it', }, }, },
      array: Array.from({ length: ARRAY_SIZE, }, (_, index,) => ({
        id: index,
        name: `item-${String(index,)}`,
      }),),
    }, null, 2,);
    const result = getJsonProperty({
      path: ['deeply', 'nested', 'property', 'value',],
      content: jsonContent,
    },);
    do_not_optimize(result,);
  },);
},);
//endregion

//region Path transformation benchmarks with range parameterization
boxplot(function mirrorGlobPathBoxplot() {
  bench('mirrorGlobPath', function mirrorBench() {
    const result = mirrorGlobPath({
      sourcePattern: 'package/*/src/*.ts',
      destPattern: 'output/*/lib/*.ts',
      sourcePath: 'package/pkg-00/src/index.ts',
    },);
    do_not_optimize(result,);
  },)
    .range('iterations', 1, 1024,);
},);
//endregion

//region Glob expansion benchmarks with depth and overlap variations
summary(function globVariationBenchmarks() {
  bench('deep glob: 6-level nested paths across 20 dirs', async function deepGlobBench() {
    const matches = await expandGlob(
      join(deepGlobDir, 'pkg-*/lib/deep/nested/very/deep/module.ts',),
    );
    do_not_optimize(matches,);
  },);

  bench('5 overlapping globs across 20 dirs', async function overlapGlobBench() {
    const results = await Promise.all(
      overlapPatterns.map(function expandPattern(pattern,) {
        return expandGlob(pattern,);
      },),
    );
    do_not_optimize(results,);
  },);
},);
//endregion

await run();

await rm(tempDir, { recursive: true, force: true, },);
