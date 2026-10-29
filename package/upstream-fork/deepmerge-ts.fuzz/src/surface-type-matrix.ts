/**
 Result types under compiler variation: type-checks this package's
 type-level files (the `type-*` tests, both soundness corpora, and the
 `surface-*` pins) with every installed TypeScript release under named flag
 sets, and prints each run's diagnostics by file and code.

 Runs inside a capped container with the scratch install from
 `./surface-toolchain.ts` mounted writable and this repo read-only:

 ```sh
 node src/surface-type-matrix.ts <scratch> <config>[,<config>...] [alias...]
 ```

 Files are copied to `<scratch>/sidecar/` with the harness import pointed at
 a declaration-only shim and `.ts` import suffixes rewritten to `.js`, so
 releases before 5.0 resolve them. The `base` config matches this package's
 strictness with library checking on; the first control is `base` on `ts60`,
 which must be clean. Findings: `doc/audit/deepmerge-ts-surface-2026-09-24.md`.

 @module
 */

import {
  mkdir,
  readdir,
  readFile,
  writeFile,
} from 'node:fs/promises';
import { join, } from 'node:path';

import {
  diagnosticCounts,
  isTypeScriptAlias,
  runChild,
  tscPrefix,
  TYPESCRIPT_RELEASES,
  type TypeScriptAlias,
} from './surface-toolchain.ts';

/**
 This package's `src` directory.
 */
const SOURCE = import.meta.dirname;

/**
 Per-run limit; the slowest cell measured takes under 10 seconds.
 */
const RUN_TIMEOUT_MS = 300_000;

/**
 Milliseconds per second, for reported durations.
 */
const MS_PER_SECOND = 1_000;

/**
 This repo's strictness flags (`package/config/typescript/tsconfig.options.json`).
 */
const STRICT = [
  '--strict',
  '--exactOptionalPropertyTypes',
  '--noUncheckedIndexedAccess',
] as const;

/**
 Named flag sets appended to {@link tscPrefix}.
 */
const CONFIGS: Readonly<Record<string, readonly string[]>> = {
  base: STRICT,
  isolated: [
    ...STRICT,
    '--isolatedModules',
  ],
  'no-exact-optional': [
    '--strict',
    '--noUncheckedIndexedAccess',
  ],
  'no-strict-null': [
    '--strict',
    '--strictNullChecks',
    'false',
    '--noUncheckedIndexedAccess',
  ],
  'no-unchecked-index': [
    '--strict',
    '--exactOptionalPropertyTypes',
  ],
  node10: [
    ...STRICT,
    '--moduleResolution',
    'node10',
    '--ignoreDeprecations',
    '6.0',
  ],
  node16: [
    ...STRICT,
    '--module',
    'node16',
    '--moduleResolution',
    'node16',
  ],
  nodenext: [
    ...STRICT,
    '--module',
    'nodenext',
    '--moduleResolution',
    'nodenext',
  ],
  'strict-false': [
    '--strict',
    'false',
  ],
  verbatim: [
    ...STRICT,
    '--verbatimModuleSyntax',
  ],
};

/**
 Declaration-only stand-in for `@monochromatic-dev/test-harness-core/ts`: the type
 checks need only `expectTypeOf`, which the harness re-exports from `expect-type`.
 */
const SHIM = `export { expectTypeOf } from "expect-type";
export declare function describe(options: { name: string; children: unknown[] }): Promise<void>;
export declare function it(options: { name: string; fn: () => Promise<void> }): unknown;
export declare function expect(value: unknown): any;
`;

/**
 Whether a source file belongs to the type-level set.

 @param name - File name in `src`.

 @returns True for the type tests, both corpora, the surface pins, and their imports.

 @example
 ```ts
 isTypeLevelFile('type-leaf.unit.test.ts',); // true
 ```
 */
function isTypeLevelFile(name: string,): boolean {
  return (name.startsWith('type-',) && name.endsWith('.unit.test.ts',))
    || (name.startsWith('surface-',) && name.endsWith('.unit.test.ts',)
      && (!name.includes('.local.',)))
    || [
      'declared-type-soundness.generated.ts',
      'surface-collection-view.ts',
      'target.ts',
      'type-soundness.generated.ts',
    ].includes(name,);
}

/**
 Suffix of this repo's unit test files, which `test:unit` collects anywhere
 outside `node_modules`, including under `dist/`.
 */
const UNIT_TEST_SUFFIX = '.unit.test.ts';

/**
 Name of a file's copy: unit test files lose their suffix so `test:unit` never
 collects the copies from the scratch directory.

 @param name - Source file name.

 @returns Copy name, such as `type-leaf.types.ts`.

 @example
 ```ts
 copyName('type-leaf.unit.test.ts',); // 'type-leaf.types.ts'
 ```
 */
function copyName(name: string,): string {
  return name.endsWith(UNIT_TEST_SUFFIX,)
    ? `${name.slice(
      0,
      -UNIT_TEST_SUFFIX.length,
    )}.types.ts`
    : name;
}

/**
 Copy the type-level files into `<scratch>/sidecar/`, rewriting imports.

 @param scratch - Scratch directory.

 @returns Paths of the copies, relative to `scratch`.

 @example
 ```ts
 const files = await copySidecar('/scratch',);
 ```
 */
async function copySidecar(scratch: string,): Promise<readonly string[]> {
  /**
   Destination directory.
   */
  const destination = join(
    scratch,
    'sidecar',
  );
  await mkdir(
    destination,
    { recursive: true, },
  );
  /**
   Names of the copied files.
   */
  const names = (await readdir(SOURCE,)).filter(isTypeLevelFile,);
  await Promise.all(names.map(async function copy(name,) {
    /**
     File text with the harness import and `.ts` suffixes rewritten.
     */
    const text = names.reduce(
      function rewrite(
        current,
        other,
      ) {
        return current.replaceAll(
          `'./${other}'`,
          `'./${other.slice(
            0,
            -'.ts'.length,
          )}.js'`,
        );
      },
      (await readFile(
        join(
          SOURCE,
          name,
        ),
        'utf8',
      )).replaceAll(
        "'@monochromatic-dev/test-harness-core/ts'",
        "'./module-test-shim.js'",
      ),
    );
    await writeFile(
      join(
        destination,
        copyName(name,),
      ),
      text,
    );
  },),);
  await writeFile(
    join(
      destination,
      'module-test-shim.ts',
    ),
    SHIM,
  );
  return [
    ...names,
    'module-test-shim.ts',
  ].map(function relative(name,) {
    return join(
      'sidecar',
      copyName(name,),
    );
  },);
}

/**
 Type-check the copies once per requested config and alias.

 @param scratch - Scratch directory holding the install.

 @param configs - Names from {@link CONFIGS}.

 @param aliases - Releases to run.

 @example
 ```ts
 await runMatrix({ aliases: ['ts60',], configs: ['base',], scratch: '/scratch', });
 ```
 */
async function runMatrix(
  {
    scratch,
    configs,
    aliases,
  }: {
    readonly scratch: string;
    readonly configs: readonly string[];
    readonly aliases: readonly TypeScriptAlias[];
  },
): Promise<void> {
  /**
   Files handed to every run.
   */
  const files = await copySidecar(scratch,);
  for (const config of configs) {
    for (const alias of aliases) {
      /**
       Start time, for the reported duration.
       */
      const started = performance.now();
      /* oxlint-disable eslint/no-await-in-loop -- runs are sequential so each gets the container's whole CPU budget. */
      /**
       Compiler run for this release and config.
       */
      const outcome = await runChild({
        args: [
          ...tscPrefix({
            alias,
            scratch,
          },),
          ...(CONFIGS[config] ?? []),
          ...files,
        ],
        command: 'node',
        cwd: scratch,
        timeoutMs: RUN_TIMEOUT_MS,
      },);
      /* oxlint-enable eslint/no-await-in-loop */
      /**
       Diagnostics grouped by file and code.
       */
      const counts = diagnosticCounts(outcome.output,);
      console.log(`${alias} ${config}: exit ${outcome.exit}, ${String(counts.length,)} file/code groups in ${((performance.now() - started) / MS_PER_SECOND).toFixed(1,)}s`,);
      for (const [key, count,] of counts)
        console.log(`  ${key} x${String(count,)}`,);
    }
  }
}

if (import.meta.main) {
  /**
   Scratch directory, comma-separated configs, and optional aliases.
   */
  const [scratch, configList = 'base', ...aliasArgs] = process.argv
    .slice(2,);
  /**
   Requested configs.
   */
  const configs = configList.split(',',);
  /**
   Requested aliases, or every release.
   */
  const aliases = (aliasArgs.length === 0) ? Object.keys(TYPESCRIPT_RELEASES,) : aliasArgs;
  if ((scratch === undefined) || configs.some(function unknownConfig(config,) {
    return !Object.hasOwn(
      CONFIGS,
      config,
    );
  },)
    || (!aliases.every(function known(alias,) {
      return isTypeScriptAlias(alias,);
    },)))
    throw new Error(`usage: surface-type-matrix.ts <scratch> <${Object.keys(CONFIGS,)
      .join('|',)}>[,...] [alias...]`,);
  await runMatrix({
    aliases: aliases.filter(function known(alias,) {
      return isTypeScriptAlias(alias,);
    },),
    configs,
    scratch,
  },);
}
