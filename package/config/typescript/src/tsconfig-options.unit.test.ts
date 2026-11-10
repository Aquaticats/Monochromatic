/**
 The preset's `exclude` entries must filter the consuming package's own
 directories. TypeScript resolves a bare relative `exclude` path against the
 file that declares it, so an unprefixed entry protects only this package and
 silently leaves a consumer's generated, backup, and retired trees type-checked.

 The checks run the workspace compiler's `--showConfig` on a disposable
 consumer, because the resolved file list is the promise the preset makes.
 Nothing is imported from a built artifact: the artifact under test is
 `tsconfig.options.json` itself.

 @module
 */

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { execFile, } from 'node:child_process';
import {
  mkdir,
  mkdtemp,
  rm,
  writeFile,
} from 'node:fs/promises';
import { createRequire, } from 'node:module';
import {
  dirname,
  join,
} from 'node:path';
import { fileURLToPath, } from 'node:url';
import { promisify, } from 'node:util';

/**
 Specialized promisified `execFile`, typed through Node's own adapter contract.
 The generic `promisify(execFile)` form trips `strict-void-return`;
 see `doc/troubleshooting/oxlint-promisify-void-return.md`.
 */
const promisifyExecFile: (original: typeof execFile) => typeof execFile.__promisify__ = promisify;

const execFileAsync = promisifyExecFile(execFile,);

/** The preset under test, whose `exclude` entries these checks resolve. */
const PRESET_PATH = fileURLToPath(new URL('../tsconfig.options.json', import.meta.url,),);

/** The workspace compiler, so a consumer's configuration resolves through the TypeScript the presets are written for. */
const COMPILER_PATH = join(
  dirname(createRequire(import.meta.url,).resolve('typescript/package.json',),),
  'bin',
  'tsc',
);

/** One consumer source per directory the preset claims to exclude, plus the source it must keep. */
const CONSUMER_SOURCES = [
  'src/retained.ts',
  'dist/generated.ts',
  'bak/copied.ts',
  'deprecated/retired.ts',
  'src/deprecated/retired.ts',
] as const;

/** The one consumer source the preset's own `include` reaches, and so the only file a check may see. */
const RETAINED_SOURCE = 'src/retained.ts';

/** Content of every fixture source, which no check loads: only the resolved file list matters. */
const FIXTURE_SOURCE = 'export const fixture = 1;\n';

/**
 Include specs of a consumer that type-checks each directory it names, the
 case the preset must still filter. A consumer writes these relative to its own
 file, the same way a package in this repository lists `src`.
 */
const WIDENED_INCLUDE = [
  'src/**/*.ts',
  'dist/**/*.ts',
  'bak/**/*.ts',
  'deprecated/**/*.ts',
] as const;

type Consumer = {
  readonly path: string;
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

type ConsumerOptions = {
  /** The consumer's own `include`; absent to inherit the preset's. */
  readonly include?: readonly string[];
};

type ResolvedConfiguration = {
  readonly files: readonly string[];
  readonly exclude: readonly string[];
};

/**
 Creates a disposable consuming package, holding one source file in each
 directory the preset claims to exclude.

 @param options - the `include` the consumer declares, when it declares one
 @returns the fixture's root, removed when its scope ends
 */
async function makeConsumer(options: ConsumerOptions,): Promise<Consumer> {
  const { include, } = options;
  const path = await mkdtemp(join(scratchRoot(), 'config-typescript-consumer-',),);
  await Promise.all(CONSUMER_SOURCES.map(async function writeSource(source,) {
    await mkdir(join(path, dirname(source,),), { recursive: true, },);
    await writeFile(join(path, source,), FIXTURE_SOURCE, 'utf8',);
  },),);
  const configuration = include === undefined
    ? { extends: PRESET_PATH, }
    : { extends: PRESET_PATH, include, };
  await writeFile(
    join(path, 'tsconfig.json',),
    `${JSON.stringify(configuration, undefined, 2,)}\n`,
    'utf8',
  );
  return {
    path,
    async [Symbol.asyncDispose]() {
      await rm(path, { force: true, recursive: true, },);
    },
  };
}

/**
 Reads the configuration the compiler resolves for a consumer.

 @param consumerPath - fixture root the compiler runs from
 @returns the resolved file list relative to the fixture, and every resolved `exclude` entry
 */
async function resolvedConfiguration(consumerPath: string,): Promise<ResolvedConfiguration> {
  const { stdout, } = await execFileAsync(
    process.execPath,
    [COMPILER_PATH, '--showConfig',],
    { cwd: consumerPath, encoding: 'utf8', },
  );
  const parsed = JSON.parse(stdout,) as {
    files?: readonly string[];
    exclude?: readonly string[];
  };
  const files = (parsed.files ?? [])
    .map(function withoutCurrentDirectoryPrefix(file,) {
      return file.startsWith('./',) ? file.slice('./'.length,) : file;
    },)
    .toSorted();
  return { exclude: parsed.exclude ?? [], files, };
}

await describe({
  name: '',
  concurrency: 1,
  children: [
    describe({
      name: 'tsconfig.options.json exclude entries',
      children: [
        it({
          name: 'resolve inside the consuming package',
          fn: async () => {
            await using consumer = await makeConsumer({},);
            const resolved = await resolvedConfiguration(consumer.path,);
            expect(resolved.exclude.length,).toBeGreaterThan(0,);
            const consumerPrefix = `${consumer.path}/`;
            const outside = resolved.exclude.filter(function outsideConsumer(entry,) {
              return !entry.startsWith(consumerPrefix,);
            },);
            expect(outside,).toEqual([],);
          },
        },),
        it({
          name: 'filter generated, backup, and retired sources a consumer includes',
          fn: async () => {
            await using consumer = await makeConsumer({ include: WIDENED_INCLUDE, },);
            const resolved = await resolvedConfiguration(consumer.path,);
            expect(resolved.files,).toEqual([RETAINED_SOURCE,],);
          },
        },),
        it({
          name: 'filter generated, backup, and retired sources under the preset include',
          fn: async () => {
            await using consumer = await makeConsumer({},);
            const resolved = await resolvedConfiguration(consumer.path,);
            expect(resolved.files,).toEqual([RETAINED_SOURCE,],);
          },
        },),
      ],
    },),
  ],
},);
