/**
 In-container sweep of Stryker's surviving mutants through the sidecar, run
 by the `mutation:sweep` mise task after `mutation`.

 `baseline` mode bundles the unmutated checkout with esbuild and requires
 every sidecar `src/*.unit.test.ts` file to pass against it through
 `DEEPMERGE_FUZZ_TARGET`: the control that lets an esbuild bundle stand in
 for the npm release. `sweep` mode splices each selected mutant from
 `/out/mutation.json` into a scratch copy of the source, bundles it to
 `/out/mutants/<id>.mjs` (reused by `./mutation-differential.ts`), runs every
 sidecar file against it two at a time, and appends one JSON line per mutant
 to `/out/sweep.jsonl` with its verdict (`./mutation-mutant.ts`).

 A file that times out counts as detecting the mutant, as Stryker counts
 timeouts.

 @module
 */

import {
  appendFile,
  cp,
  mkdir,
  readFile,
  rm,
  writeFile,
} from 'node:fs/promises';
import { createRequire, } from 'node:module';
import { join, } from 'node:path';

import {
  MutationStepError,
  OUT,
  stageCheckout,
  UPSTREAM,
} from './mutation-container.ts';
import {
  runSidecar,
  sidecarFiles,
} from './mutation-sidecar.ts';
import {
  applyMutant,
  parseMutationReport,
  selectMutants,
  sweepVerdict,
} from './mutation-mutant.ts';

/**
 Stryker statuses the sweep revisits: what upstream's suite did not detect.
 */
const SWEEP_STATUSES = [
  'Survived',
  'NoCoverage',
] as const;

/**
 Scratch copy of the checkout source that mutants are spliced into.
 */
const MUTANT_SOURCE = '/tmp/mutant-src';

/**
 Prefix of every mutated file in the report (Stryker mutates `src/**` only).
 */
const SOURCE_PREFIX = 'src/';

/**
 The part of esbuild's API the sweep calls.
 */
type Esbuild = {
  readonly build: (options: Readonly<Record<string, unknown>>,) => Promise<unknown>;
};

/**
 Whether a loaded module exposes esbuild's `build`.

 @param value - Module from the image's install.

 @returns True when `build` is a function.

 @example
 ```ts
 isEsbuild(require('esbuild',),);
 ```
 */
function isEsbuild(value: unknown,): value is Esbuild {
  return ((typeof value) === 'object') && (value !== null)
    && ((typeof Reflect.get(
      value,
      'build',
    )) === 'function');
}

/**
 esbuild from the image's pinned install, not this repo's.

 @returns esbuild's API.

 @throws {@link MutationStepError} When the image has no usable esbuild.

 @example
 ```ts
 const esbuild = loadEsbuild();
 ```
 */
function loadEsbuild(): Esbuild {
  /**
   Loaded module.
   */
  const loaded: unknown = createRequire(join(
    UPSTREAM,
    'package.json',
  ),)('esbuild',);
  if (!isEsbuild(loaded,))
    throw new MutationStepError('esbuild in /upstream has no build function; rebuild the image with mutation:image',);
  return loaded;
}

/**
 Bundle a source tree's `index.ts` into one ES module.

 @param srcDir - Directory holding the deepmerge-ts source.

 @param outfile - Bundle path.

 @example
 ```ts
 await bundle({ outfile: '/out/mutants/baseline.mjs', srcDir: '/upstream/src', });
 ```
 */
async function bundle(
  {
    srcDir,
    outfile,
  }: {
    readonly srcDir: string;
    readonly outfile: string;
  },
): Promise<void> {
  await loadEsbuild()
    .build({
      bundle: true,
      entryPoints: [join(
        srcDir,
        'index.ts',
      ),],
      format: 'esm',
      logLevel: 'error',
      outfile,
      platform: 'node',
    },);
}

/**
 Bundle the unmutated checkout and require every sidecar file to pass.

 @throws {@link MutationStepError} When a file fails, so the bundle cannot stand in for the npm release.

 @example
 ```ts
 await sweepBaseline();
 ```
 */
export async function sweepBaseline(): Promise<void> {
  await stageCheckout();
  await mkdir(
    join(
      OUT,
      'mutants',
    ),
    { recursive: true, },
  );
  /**
   Unmutated bundle.
   */
  const target = join(
    OUT,
    'mutants',
    'baseline.mjs',
  );
  await bundle({
    outfile: target,
    srcDir: join(
      UPSTREAM,
      'src',
    ),
  },);
  /**
   Outcome per sidecar file.
   */
  const outcomes = await runSidecar({
    files: await sidecarFiles(),
    target,
  },);
  await writeFile(
    join(
      OUT,
      'baseline.json',
    ),
    `${JSON.stringify(
      outcomes,
      undefined,
      2,
    )}\n`,
  );
  /**
   Files that failed against the unmutated bundle.
   */
  const failed = outcomes.filter(function didFail(outcome,) {
    return !outcome.passed;
  },);
  console.log(`baseline: ${String(outcomes.length - failed.length,)} of ${String(outcomes.length,)} sidecar files pass`,);
  if (failed.length > 0) {
    throw new MutationStepError(`baseline bundle fails ${failed.map(function fileOf(outcome,) {
      return outcome.file;
    },)
      .join(', ',)}; see ${join(
        OUT,
        'baseline.json',
      )}`,);
  }
}

/**
 Sweep the selected mutants through the sidecar.

 @param ids - Mutant ids to sweep; empty sweeps every survivor.

 @example
 ```ts
 await sweepMutants(['226',]);
 ```
 */
export async function sweepMutants(ids: readonly string[],): Promise<void> {
  await stageCheckout();
  await rm(
    MUTANT_SOURCE,
    {
      force: true,
      recursive: true,
    },
  );
  await cp(
    join(
      UPSTREAM,
      'src',
    ),
    MUTANT_SOURCE,
    { recursive: true, },
  );
  await mkdir(
    join(
      OUT,
      'mutants',
    ),
    { recursive: true, },
  );
  /**
   Mutants to sweep.
   */
  const selected = selectMutants({
    ids,
    report: parseMutationReport(await readFile(
      join(
        OUT,
        'mutation.json',
      ),
      'utf8',
    ),),
    statuses: SWEEP_STATUSES,
  },);
  /**
   Sidecar files, fixed for the whole sweep.
   */
  const files = await sidecarFiles();
  console.log(`sweeping ${String(selected.length,)} mutants through ${String(files.length,)} sidecar files`,);
  for (const entry of selected) {
    if (!entry.file
      .startsWith(SOURCE_PREFIX,))
      throw new MutationStepError(`mutant ${entry.mutant
        .id} is in ${entry.file}, outside src/`,);
    /**
     Mutated file inside the scratch source copy.
     */
    const mutatedPath = join(
      MUTANT_SOURCE,
      entry.file
        .slice(SOURCE_PREFIX.length,),
    );
    /**
     Bundle for this mutant.
     */
    const target = join(
      OUT,
      'mutants',
      `${entry.mutant
        .id}.mjs`,
    );
    // Mutants run one at a time: each rewrites the shared scratch copy.
    // oxlint-disable-next-line eslint/no-await-in-loop -- sequential by design, see the comment above.
    await writeFile(
      mutatedPath,
      applyMutant(entry,),
    );
    // oxlint-disable-next-line eslint/no-await-in-loop -- sequential by design, see the comment above.
    await bundle({
      outfile: target,
      srcDir: MUTANT_SOURCE,
    },);
    // oxlint-disable-next-line eslint/no-await-in-loop -- restores the original before the next mutant.
    await writeFile(
      mutatedPath,
      entry.source,
    );
    /**
     Outcome per sidecar file.
     */
    // oxlint-disable-next-line eslint/no-await-in-loop -- sequential by design, see the comment above.
    const outcomes = await runSidecar({
      files,
      target,
    },);
    /**
     Files that failed or timed out against this mutant.
     */
    const failed = outcomes
      .filter(function didFail(outcome,) {
        return !outcome.passed;
      },)
      .map(function fileOf(outcome,) {
        return outcome.file;
      },);
    // oxlint-disable-next-line eslint/no-await-in-loop -- one log line per mutant, in sweep order.
    await appendFile(
      join(
        OUT,
        'sweep.jsonl',
      ),
      `${JSON.stringify({
        failed,
        file: entry.file,
        id: entry.mutant
          .id,
        line: entry.mutant
          .location
          .start
          .line,
        mutator: entry.mutant
          .mutatorName,
        replacement: entry.mutant
          .replacement,
        status: entry.mutant
          .status,
        verdict: sweepVerdict(failed,),
      },)}\n`,
    );
    console.log(`${entry.mutant
      .id}\t${entry.file}:${String(entry.mutant
        .location
        .start
        .line,)}\t${sweepVerdict(failed,)}\t${failed.join(' ',)}`,);
  }
}

if (import.meta.main) {
  /**
   Mode and optional mutant ids from the task.
   */
  const [mode, ...ids] = process.argv
    .slice(2,);
  if (mode === 'baseline')
    await sweepBaseline();
  else if (mode === 'sweep')
    await sweepMutants(ids,);
  else
    throw new MutationStepError(`unknown mode ${String(mode,)}; expected baseline or sweep`,);
}
