import {
  mkdtempSync,
  readFileSync,
  readdirSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  fileURLToPath,
} from 'node:url';
import { join, } from 'node:path';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import spawn from 'nano-spawn';

/** Built plugin entry exercised at Oxlint consumer seam. */
const BUILT_PLUGIN_PATH = fileURLToPath(new URL(
  '../dist/final/node/index.mjs',
  import.meta.url,
),);

/** Disposable standalone project for cache-isolated Oxlint processes. */
type DisposableProject = {
  readonly path: string;
  readonly [Symbol.dispose]: () => void;
};

/** Worker selection for one Oxlint process. */
type WorkerSelection =
  | { readonly kind: 'default'; }
    | {
      readonly kind: 'fixed';
      readonly count: number;
    };

/** Captured Oxlint output whether diagnostics make process exit nonzero. */
type OxlintCapture = {
  readonly stdout: string;
  readonly stderr: string;
};

/**
 Creates standalone project removed after test scope.
 
 @returns disposable cache-isolated project.
 */
function disposableProject(): DisposableProject {
  /**
   Temporary project root outside repository lockfile ancestry.
   */
  const path = mkdtempSync(join(scratchRoot(), 'readonly-omission-diagnostics-',),);
  return {
    path,
    [Symbol.dispose](): void {
      rmSync(path, { recursive: true, force: true, },);
    },
  };
}

/**
 Captures Oxlint JSON and logger stderr for one worker selection.
 
 @param projectPath - Standalone project root.
 
 @param workers - Fixed or host-default worker policy.
 
 @returns captured stdout and stderr on success or diagnostic exit.
 */
async function runOxlint({
  projectPath,
  workers,
}: {
  readonly projectPath: string;
  readonly workers: WorkerSelection;
}): Promise<OxlintCapture> {
  /**
   Optional explicit worker arguments.
   */
  const workerArguments = workers.kind === 'fixed'
    ? [
      '--threads',
      String(workers.count,),
    ]
    : [];
  try {
    return await spawn(
      'oxlint',
      [
        ...workerArguments,
        '--format',
        'json',
        '--config',
        join(projectPath, 'oxlint.json',),
        join(projectPath, 'input.ts',),
      ],
      { cwd: projectPath, },
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
 Reads exact diagnostic records from Oxlint JSON output.
 
 @param capture - Captured Oxlint process output.
 
 @returns parsed diagnostics without timing or process metadata.
 */
function diagnosticRecords(capture: OxlintCapture,): readonly unknown[] {
  /**
   Parsed output narrowed only to consumed diagnostic array.
   */
  const parsed = JSON.parse(capture.stdout,) as { readonly diagnostics?: readonly unknown[]; };
  return parsed.diagnostics ?? [];
}

/**
 Seeds every persisted envelope under one project's effect cache root with a recorded omission.

 The compiler summarizes the fixture completely, so the record a failing scan persists is seeded
 instead: the warm and worker runs must restore it and report it identically.

 @param projectPath - Standalone project root whose cache the runs share.

 @returns number of envelopes seeded.
 */
function seedOmissionRecords(projectPath: string,): number {
  /**
   Default persistent cache root of a project with no lockfile ancestor.
   */
  const cacheRoot = join(
    projectPath,
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
   Envelope count carrying source identity the seeded record can name.
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
    if ((typeof envelope.fileName) !== 'string')
      continue;
    writeFileSync(
      entryPath,
      JSON.stringify({
        ...envelope,
        omittedCallableKeys: [`${envelope.fileName}:seeded-omitted-callable`,],
        omissionReasons: ['direct-summary-construction-failed',],
      },),
    );
    seeded += 1;
  }
  return seeded;
}

await describe({
  name: 'persistent omission diagnostic parity',
  concurrency: 1,
  children: [
    it({
      name: 'keeps cold warm and worker diagnostic fingerprints equal',
      fn: async () => {
        using project = disposableProject();
        writeFileSync(
          join(project.path, 'tsconfig.json',),
          '{"compilerOptions":{"strict":true},"include":["input.ts"]}\n',
        );
        writeFileSync(
          join(project.path, 'oxlint.json',),
          JSON.stringify({
            jsPlugins: [BUILT_PLUGIN_PATH,],
            rules: {
              'prefer-readonly-parameter-type/prefer-readonly-parameter-types': 'error',
            },
          },),
        );
        writeFileSync(
          join(project.path, 'input.ts',),
          `export function take<Fn extends (...args: never[]) => unknown,>(
  fn: Fn,
  args: Parameters<Fn>,
): void {
  void fn;
  void args;
}

export function use(): void {
  take(
    function render(): string {
      return '';
    },
    [],
  );
}

export function inspect(state: { nested: { value: string; }; },): number {
  return state.nested.value.length;
}
`,
        );
        const cold = await runOxlint({
          projectPath: project.path,
          workers: { kind: 'fixed', count: 1, },
        },);
        /* The compiler summarizes this fixture completely, so the cold run persists no
         * omission. The record a failing scan leaves behind is seeded into its entry, and
         * every later run must restore it, report it, and answer with the same
         * diagnostics. */
        if (seedOmissionRecords(project.path,) === 0)
          throw new Error('Expected persistent summary cache entry to seed.',);
        const warm = await runOxlint({
          projectPath: project.path,
          workers: { kind: 'fixed', count: 1, },
        },);
        const defaultWorkers = await runOxlint({
          projectPath: project.path,
          workers: { kind: 'default', },
        },);
        /**
         Positive-control diagnostics proving comparison can observe rule output.
         */
        const coldDiagnostics = diagnosticRecords(cold,);
        expect(coldDiagnostics.length,).toBeGreaterThan(0,);
        expect(diagnosticRecords(warm,),).toEqual(coldDiagnostics,);
        expect(diagnosticRecords(defaultWorkers,),).toEqual(coldDiagnostics,);
        expect(warm.stderr,).toContain(
          'from effect cache: direct-summary-construction-failed',
        );
        expect(defaultWorkers.stderr,).toContain(
          'from effect cache: direct-summary-construction-failed',
        );
      },
    },),
  ],
},);
