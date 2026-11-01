/**
 Whether a warm run reports an installed package's recorded omission exactly once.

 A consumer's cached summaries already carry the effects applied from an installed package's shipped
 implementation, so a warm run restores the consumer from its entry and never opens that
 implementation again. The report then comes from the omission record the consumer's entry carries,
 while the implementation's own entry stays unread. The in-source case is held by
 `persistent-omission-diagnostics.unit.test.ts`.

 @module
 */

import {
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  realpathSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  dirname,
  join,
} from 'node:path';
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
const PACKAGE_NAME = 'omission-probe';

/**
 Authored dependency version, matched exactly by the fixture lockfile key.
 */
const PACKAGE_VERSION = '1.0.0';

/**
 Bounded category the shipped implementation's recorded omission is reported under.
 */
const OMISSION_REASON = 'direct-summary-construction-failed';

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
 Shipped implementation whose helper call instantiates its generic's tuple parameter type.
 */
const IMPLEMENTATION_SOURCE = `/**
 * @template {(...args: never[]) => unknown} Fn
 * @param {Fn} fn
 * @param {Parameters<Fn>} args
 * @returns {void}
 */
function take(fn, args) {
  void fn;
  void args;
}

/**
 * @param {() => string} callback
 * @returns {number}
 */
export function run(callback) {
  take(function render() { return ''; }, []);
  return callback().length;
}
`;

/**
 Shipped declarations the consumer's checker resolves the call against.
 */
const DECLARATION_SOURCE = 'export declare function run(callback: () => string): number;\n';

/**
 Builds a consumer source handing the shipped export a closure over caller state.

 @param name - Exported function name of the consumer, distinct per file.

 @returns consumer source text.
 */
function consumerSource(name: string,): string {
  return `import { run, } from '${PACKAGE_NAME}';\n\nexport function ${name}(config: { label: string; },): number {\n  return run(function produce(): string {\n    return config.label;\n  },);\n}\n`;
}

/**
 Disposable consumer workspace owning its own lockfile, project, dependency and linter config.
 */
type OmissionFixture = {
  readonly consumerPaths: readonly string[];
  readonly configPath: string;
  readonly implementationPath: string;
  readonly [Symbol.dispose]: () => void;
};

/**
 Materializes two consumers of one installed package whose entries the test seeds with an
 implementation omission record.

 @returns fixture removed when the enclosing scope ends.
 */
function omissionFixture(): OmissionFixture {
  /**
   Consumer root, outside every repository lockfile's reach and held under its real path.
   */
  const root = realpathSync(mkdtempSync(
    join(
      scratchRoot(),
      'readonly-external-omission-',
    ),
  ),);
  /**
   Installed dependency root, whose `node_modules` segment is what package identity reads.
   */
  const packageRoot = join(root, 'node_modules', PACKAGE_NAME,);
  mkdirSync(packageRoot, { recursive: true, },);
  writeFileSync(
    join(packageRoot, 'package.json',),
    `${JSON.stringify({
      name: PACKAGE_NAME,
      version: PACKAGE_VERSION,
      type: 'module',
      exports: {
        types: './index.d.ts',
        default: './index.js',
      },
    }, null, 2,)}\n`,
  );
  writeFileSync(join(packageRoot, 'index.js',), IMPLEMENTATION_SOURCE,);
  writeFileSync(join(packageRoot, 'index.d.ts',), DECLARATION_SOURCE,);
  /* Key shape copied from this repository's `pnpm-lock.yaml`. The eligibility gate scans for keys at
   * two-space indentation without regard to section, so the surrounding structure is for readers. */
  writeFileSync(
    join(root, 'pnpm-lock.yaml',),
    `lockfileVersion: '9.0'\n\npackages:\n\n  ${PACKAGE_NAME}@${PACKAGE_VERSION}:\n    resolution: {integrity: sha512-${'A'.repeat(86,)}==}\n\nsnapshots:\n\n  ${PACKAGE_NAME}@${PACKAGE_VERSION}: {}\n`,
  );
  writeFileSync(
    join(root, 'tsconfig.json',),
    `${JSON.stringify({
      compilerOptions: {
        strict: true,
        module: 'nodenext',
        moduleResolution: 'nodenext',
        target: 'esnext',
        noEmit: true,
      },
      include: [
        'first.ts',
        'second.ts',
      ],
    }, null, 2,)}\n`,
  );
  /**
   Linter configuration loading the built plugin by absolute path.
   */
  const configPath = join(root, '.oxlintrc.json',);
  writeFileSync(
    configPath,
    `${JSON.stringify({
      jsPlugins: [PLUGIN_ENTRY,],
      ignorePatterns: [
        '**/dist',
        '**/node_modules',
      ],
      rules: { 'prefer-readonly-parameter-type/prefer-readonly-parameter-types': 'error', },
    }, null, 2,)}\n`,
  );
  /**
   Consumers whose summaries both come from the same shipped export.
   */
  const consumerPaths = [
    join(root, 'first.ts',),
    join(root, 'second.ts',),
  ];
  writeFileSync(consumerPaths[0] ?? '', consumerSource('first',),);
  writeFileSync(consumerPaths[1] ?? '', consumerSource('second',),);
  return {
    consumerPaths,
    configPath,
    implementationPath: join(packageRoot, 'index.js',),
    [Symbol.dispose](): void {
      rmSync(root, { recursive: true, force: true, },);
    },
  };
}

/**
 Captured oxlint output whether diagnostics make the process exit nonzero.
 */
type OxlintCapture = {
  readonly stdout: string;
  readonly stderr: string;
};

/**
 Lints both consumers in one process at one worker.

 @param fixture - Disposable consumer workspace.

 @returns captured standard output and logger standard error.
 */
async function lintConsumers(fixture: OmissionFixture,): Promise<OxlintCapture> {
  try {
    return await spawn(
      OXLINT_BINARY,
      [
        '--threads',
        '1',
        '--format',
        'json',
        '--config',
        fixture.configPath,
        ...fixture.consumerPaths,
      ],
    );
  }
  catch (error: unknown) {
    if (((typeof error) !== 'object')
      || (error === null)
      || (!('stdout' in error))
      || (!('stderr' in error))
      || ((typeof error.stdout) !== 'string')
      || ((typeof error.stderr) !== 'string'))
      throw error;
    return {
      stdout: error.stdout,
      stderr: error.stderr,
    };
  }
}

/**
 Reads exact diagnostic records from oxlint JSON output.

 @param capture - Captured oxlint process output.

 @returns parsed diagnostics without timing or process metadata.
 */
function diagnosticRecords(capture: OxlintCapture,): readonly unknown[] {
  /**
   Parsed output narrowed only to the consumed diagnostic array.
   */
  const parsed = JSON.parse(capture.stdout,) as { readonly diagnostics?: readonly unknown[]; };
  return parsed.diagnostics ?? [];
}

/**
 Counts non-overlapping occurrences of a message in captured logger output.

 @param text - Captured logger output.

 @param message - Message text searched for.

 @returns number of occurrences.
 */
function occurrences({
  text,
  message,
}: {
  readonly text: string;
  readonly message: string;
},): number {
  return text.split(message,).length - 1;
}

/**
 Seeds each consumer entry with the implementation omission record its scan would persist.

 The compiler summarizes the shipped implementation completely, so the record a failing scan
 persists is seeded instead: the warm run must report it from the record without reopening the
 implementation.

 @param fixture - Disposable consumer workspace owning the cache root.

 @returns number of consumer entries seeded.
 */
function seedExternalOmissions(fixture: OmissionFixture,): number {
  /**
   Default persistent cache root under the fixture's own lockfile.
   */
  const cacheRoot = join(
    dirname(fixture.configPath,),
    'node_modules',
    '.cache',
    'prefer-readonly-parameter-type',
  );
  /**
   Relative envelope paths discovered under the cache root.
   */
  const entries = readdirSync(cacheRoot, {
    recursive: true,
    encoding: 'utf8',
  },)
    .filter(function jsonEntry(entry,): boolean {
      return entry.endsWith('.json',);
    },);
  /**
   Consumer source identities whose entries carry the record.
   */
  const consumerFileNames = new Set(fixture.consumerPaths,);
  /**
   Envelope count seeded with the implementation omission record.
   */
  let seeded = 0;
  for (const entry of entries) {
    /**
     Exact envelope path rewritten in place.
     */
    const entryPath = join(cacheRoot, entry,);
    /**
     Parsed envelope narrowed to the fields the seed extends.
     */
    const envelope = JSON.parse(readFileSync(entryPath, 'utf8',),) as {
      readonly fileName?: unknown;
    };
    if (((typeof envelope.fileName) !== 'string')
      || (!consumerFileNames.has(envelope.fileName,)))
      continue;
    writeFileSync(
      entryPath,
      JSON.stringify({
        ...envelope,
        externalOmissions: [{
          sourceFileName: fixture.implementationPath,
          omittedCount: 1,
          reasons: [OMISSION_REASON,],
        },],
      },),
    );
    seeded += 1;
  }
  return seeded;
}

await describe({
  name: 'omissions inside an installed package implementation',
  concurrency: 1,
  children: [
    it({
      name: 'reports a recorded implementation omission in a warm run exactly once',
      fn: async () => {
        using fixture = omissionFixture();
        /** Cold run, which analyses the shipped implementation completely. */
        const cold = await lintConsumers(fixture,);
        /* The compiler summarizes the shipped implementation completely, so the cold run records
         * no omission. The record a failing scan leaves in each consumer's entry is seeded
         * instead: the warm run must report it exactly once even though two consumers restore
         * it, and must never re-analyse the implementation to say it. */
        if (seedExternalOmissions(fixture,) === 0)
          throw new Error('Expected consumer cache entries to seed.',);
        /** Warm run over the cache the cold run wrote. */
        const warm = await lintConsumers(fixture,);
        /** Message a fresh scan of the shipped implementation logs. */
        const omitted = `omitted 1 callable summaries for ${fixture.implementationPath}: ${OMISSION_REASON}`;
        /** Message a restore of the implementation's omission record logs. */
        const restored = `restored 1 omitted callable summaries for ${fixture.implementationPath} from effect cache: ${OMISSION_REASON}`;
        /* The complete scan records nothing, and the restore reports the record without ever
         * re-analysing the implementation to say it. */
        expect(occurrences({ text: cold.stderr, message: omitted, },),).toBe(0,);
        expect(occurrences({ text: warm.stderr, message: restored, },),).toBe(1,);
        expect(occurrences({ text: warm.stderr, message: omitted, },),).toBe(0,);
        /** Diagnostics of the cold run, which prove the comparison can observe rule output. */
        const coldDiagnostics = diagnosticRecords(cold,);
        expect(coldDiagnostics.length,).toBeGreaterThan(0,);
        expect(diagnosticRecords(warm,),).toEqual(coldDiagnostics,);
      },
    },),
  ],
},);
