/**
 Whether the external channel survives oxlint's default worker count.
 
 The only test here that runs oxlint to reach the external channel, and it exists because no other shape
 can fail when this breaks. Measured rather than argued: deleting the eager initializer from `index.ts`
 kills this test and leaves `external-capture-channel.unit.test.ts` passing. That test drives the effect
 summaries from plain `node`, where the second native child spawns without trouble, so every external
 assertion in it holds whether or not the channel works under the linter that actually ships.
 
 ## What broke, and why it was invisible
 
 A native TypeScript child was created per generated implementation project, on demand, mid-lint. Under
 oxlint that spawn fails:
 
 ```text
 external effect inference failed for ... retainCallback: Error: spawn ENOMEM
 ```
 
 `externalCallableEffect` catches it, logs at debug, and answers with the same sentinel it uses for a
 package it cannot resolve. Every call then falls through to the unresolved boundary, which withholds, so
 the entire channel disappeared without a single failing assertion anywhere. Four separate investigations
 read that silence as evidence that no dependency's shipped implementation resolves.
 
 ## Why the worker count, and why not merely the ordering
 
 Measured at every thread count on a 16-core host with 63 GiB RAM and `/proc/sys/vm/overcommit_memory`
 at 0, counting reports and how many carried external provenance:
 
 ```text
 threads=1    4 / 4        threads=8    4 / 4
 threads=2    4 / 4        threads=16   5 / 0
 threads=4    4 / 4
 ```
 
 Reservations exist at eight workers too and the spawn succeeds there, so "started after the
 reservations" cannot be the criterion. What decides it is the aggregate reserved size at spawn time.
 
 That is why this test passes no `--threads` flag. Pinning a specific count would pin the wrong quantity,
 and a count below this host's would pass while the shipped default failed.
 
 ## What it asserts, and what it deliberately does not
 
 External provenance in the output. Only `applyExternalEffect` and the capture channel beside it write a
 package name and version into a message, so its presence proves the shipped implementation was loaded
 and its summary applied.
 
 It does not assert the readonly offer that the falsification used, because that needs `type-fest`
 resolvable from the consumer and this package does not depend on it. Measured without it: the reports
 arrive and carry provenance, and only the suggestion changes. The offer belongs to
 `external-capture-channel.unit.test.ts`, which drives it through the summaries instead.
 
 @module
 */

import {
  mkdirSync,
  mkdtempSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import { join, } from 'node:path';
import { fileURLToPath, } from 'node:url';

import spawn from 'nano-spawn';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';

/**
 Authored dependency name, which must not name anything installed.
 */
const PACKAGE_NAME = 'worker-retainer-probe';

/**
 Authored dependency version, matched exactly by the fixture lockfile key.
 */
const PACKAGE_VERSION = '1.0.0';

/**
 Provenance every assertion here looks for, written only by the external channel.
 */
const EXTERNAL_PROVENANCE = `${PACKAGE_NAME}@${PACKAGE_VERSION}`;

/**
 Built plugin the linted run loads, rather than this package's source.
 */
const PLUGIN_ENTRY = fileURLToPath(new URL(
  '../dist/final/node/index.mjs',
  import.meta.url,
),);

/**
 Repository oxlint binary, resolved from this file rather than from a working directory.
 */
const OXLINT_BINARY = fileURLToPath(new URL(
  '../../../../node_modules/.bin/oxlint',
  import.meta.url,
),);

/**
 Shipped implementation whose store past its own return is what the summary proves.
 */
const IMPLEMENTATION_SOURCE = `const heldCallbacks = [];

export function retainCallback(callback) {
  heldCallbacks.push(callback);
  return heldCallbacks.length;
}
`;

/**
 Shipped declarations the consumer's checker resolves the call against.
 */
const DECLARATION_SOURCE = `export declare function retainCallback(callback: () => void): number;
`;

/**
 Consumer handing the retaining export a closure that hands back caller state.
 */
const CONSUMER_SOURCE = `import { retainCallback, } from '${PACKAGE_NAME}';

export type Row = { label: string; };

export type Config = { row: Row; };

export function handRowProducerToExternalRetainer(config: Config,): number {
  return retainCallback(function produce(): Row {
    return config.row;
  },);
}
`;

/**
 Disposable consumer workspace owning its own lockfile, project, dependency and linter config.
 */
type WorkerFixture = {
  readonly consumerPath: string;
  readonly configPath: string;
  readonly [Symbol.dispose]: () => void;
};

/**
 Materializes a consumer oxlint can lint, with the dependency it resolves against.
 
 @returns fixture removed when the enclosing scope ends.
 
 @example
 ```ts
 using fixture = workerFixture();
 ```
 */
function workerFixture(): WorkerFixture {
  /**
   Consumer root, outside every repository lockfile's reach.
   */
  const root = mkdtempSync(join(
    scratchRoot(),
    'readonly-external-workers-',
  ),);
  /**
   Installed dependency root, whose `node_modules` segment is what package identity reads.
   */
  const packageRoot = join(
    root,
    'node_modules',
    PACKAGE_NAME,
  );
  mkdirSync(
    packageRoot,
    { recursive: true, },
  );
  writeFileSync(
    join(
      packageRoot,
      'package.json',
    ),
    `${JSON.stringify(
      {
        name: PACKAGE_NAME,
        version: PACKAGE_VERSION,
        type: 'module',
        exports: {
          types: './index.d.ts',
          default: './index.js',
        },
      },
      null,
      2,
    )}\n`,
  );
  writeFileSync(
    join(
      packageRoot,
      'index.js',
    ),
    IMPLEMENTATION_SOURCE,
  );
  writeFileSync(
    join(
      packageRoot,
      'index.d.ts',
    ),
    DECLARATION_SOURCE,
  );
  /* Key shape copied from this repository's `pnpm-lock.yaml`. The eligibility gate scans for keys at
   * two-space indentation without regard to section, so the surrounding structure is for readers. */
  writeFileSync(
    join(
      root,
      'pnpm-lock.yaml',
    ),
    `lockfileVersion: '9.0'

packages:

  ${PACKAGE_NAME}@${PACKAGE_VERSION}:
    resolution: {integrity: sha512-${'A'.repeat(86,)}==}

snapshots:

  ${PACKAGE_NAME}@${PACKAGE_VERSION}: {}
`,
  );
  writeFileSync(
    join(
      root,
      'tsconfig.json',
    ),
    `${JSON.stringify(
      {
        compilerOptions: {
          strict: true,
          module: 'nodenext',
          moduleResolution: 'nodenext',
          target: 'esnext',
          noEmit: true,
        },
        include: ['consumer.ts',],
      },
      null,
      2,
    )}\n`,
  );
  /**
   Linter configuration loading the built plugin by absolute path.
   */
  const configPath = join(
    root,
    '.oxlintrc.json',
  );
  writeFileSync(
    configPath,
    `${JSON.stringify(
      {
        jsPlugins: [PLUGIN_ENTRY,],
        ignorePatterns: [
          '**/dist',
          '**/node_modules',
        ],
        rules: { 'prefer-readonly-parameter-type/no-opaque-parameter-effects': 'error', },
      },
      null,
      2,
    )}\n`,
  );
  /**
   Consumer source path, whose ancestors supply the project and the lockfile.
   */
  const consumerPath = join(
    root,
    'consumer.ts',
  );
  writeFileSync(
    consumerPath,
    CONSUMER_SOURCE,
  );
  return {
    consumerPath,
    configPath,
    [Symbol.dispose](): void {
      rmSync(
        root,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Tests whether a caught value carries the captured output of a finished subprocess.
 
 @param value - Value thrown by the spawn helper.
 
 @returns whether standard output can be read from it.
 
 @example
 ```ts
 if (carriesOutput(error)) return error.stdout;
 ```
 */
function carriesOutput(value: unknown,): value is { readonly stdout: string; } {
  return ((typeof value) === 'object')
    && (value !== null)
    && ('stdout' in value)
    && ((typeof value.stdout) === 'string');
}

/**
 Lints one consumer at oxlint's own default worker count.
 
 No `--threads` argument, deliberately. The failure this guards depends on how much the workers
 reserved, so choosing a count for them would pin a quantity that is not the shipped one.
 
 @param fixture - Disposable consumer to lint.
 
 @returns standard output of the run, reports included.
 
 @throws when the run failed for a reason that produced no output to read.
 
 @example
 ```ts
 const output = await lintConsumer(fixture);
 ```
 */
async function lintConsumer(fixture: WorkerFixture,): Promise<string> {
  try {
    /**
     Completed run, reached only when the linter reported nothing.
     */
    const clean = await spawn(
      OXLINT_BINARY,
      [
        '--config',
        fixture.configPath,
        fixture.consumerPath,
      ],
    );
    return clean.stdout;
  }
  catch (error) {
    /* A reporting run exits nonzero, and its output is the whole subject of this test, so a thrown
     * result carrying output is the expected path rather than a failure. */
    if (carriesOutput(error,))
      return error.stdout;
    throw error;
  }
}

await describe({
  name: 'external channel under oxlint workers',
  concurrency: 1,
  children: [
    it({
      name: 'loads a shipped implementation at the default worker count',
      fn: async () => {
        using fixture = workerFixture();
        /**
         Reports from a run using oxlint's own worker default.
         */
        const output = await lintConsumer(fixture,);
        /* The whole assertion. Provenance naming the package and its version can only come from the
         * external channel, so its presence proves the shipped implementation loaded and its summary
         * applied. Before the shared child was started early, this run produced reports that named the
         * call and carried no provenance at all. */
        expect(output.includes(EXTERNAL_PROVENANCE,),).toBe(true,);
        /* And the capture channel beside it, which is the fact this consumer was written to produce: the
         * retained closure hands back caller state, and the parameter is withheld for that reason rather
         * than for the generic unresolved one. */
        expect(output.includes('handed callable capture',),).toBe(true,);
      },
    },),
  ],
},);
