import {
  mkdtempSync,
  rmSync,
  writeFileSync,
} from 'node:fs';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  join,
  resolve,
} from 'node:path';
import { pathToFileURL, } from 'node:url';

import {
  describe,
  expect,
  it,
} from '@monochromatic-dev/test-harness-core/ts';
import {
  fixtureConfigPath,
  OXLINT_PLUGIN_TEST_ROOT as ROOT,
} from '@monochromatic-dev/oxlint-plugin-test-support/ts';
import spawn from 'nano-spawn';

//region Helpers

/** Spread and copy rules that ask the TypeScript bridge for evidence. */
type SpreadRule = 'no-useless-spread' | 'prefer-spread';

/** One diagnostic of an Oxlint JSON report, as far as these cases read it. */
type ReportedDiagnostic = {
  readonly message: string;
  readonly severity: string;
  readonly filename: string;
  readonly code?: string;
};

/** What one Oxlint process left behind. */
type OxlintOutcome = {
  readonly exitCode: number;
  readonly diagnostics: readonly ReportedDiagnostic[];
  readonly stderr: string;
};

/** Fault injection that makes every native TypeScript spawn fail as an out-of-memory spawn does. */
const REFUSE_NATIVE_SPAWN = pathToFileURL(resolve(
  import.meta.dirname,
  'refuse-native-api-spawn.test-fixture.ts',
),).href;

/** Declarations the scratch sources share. */
const DECLARATIONS = 'declare const toppings: readonly string[];\n';

/**
 Writes a typed scratch project holding one source, so the bridge could resolve it were it running.

 @param source - Text of the one source file.

 @returns the source path, and a disposer that removes the project.
 */
function scratchProject(source: string,): { readonly filePath: string; readonly [Symbol.dispose]: () => void; } {
  /** Unique temp directory holding the project. */
  const directory = mkdtempSync(join(scratchRoot(), 'oxlint-spread-bridge-failure-',),);
  writeFileSync(
    join(directory, 'tsconfig.json',),
    JSON.stringify({
      compilerOptions: {
        lib: ['ESNext',],
        target: 'esnext',
        module: 'preserve',
        strict: true,
        noEmit: true,
        types: [],
      },
      files: ['case.ts',],
    },),
  );
  writeFileSync(join(directory, 'case.ts',), `${DECLARATIONS}${source}\n`,);
  return {
    filePath: join(directory, 'case.ts',),
    [Symbol.dispose]: function removeProject(): void {
      rmSync(directory, { recursive: true, force: true, },);
    },
  };
}

/**
 Lints one source with one spread rule at a single worker, optionally with native spawns refused.

 @param rule - Rule enabled alone.

 @param target - Source to lint.

 @param refuseSpawn - Whether every native TypeScript spawn fails.

 @returns exit code, parsed diagnostics and standard error.
 */
async function lint({
  rule,
  target,
  refuseSpawn,
}: {
  readonly rule: SpreadRule;
  readonly target: string;
  readonly refuseSpawn: boolean;
},): Promise<OxlintOutcome> {
  /** Arguments of the one-worker JSON run. */
  const args = [
    '--threads',
    '1',
    '--format',
    'json',
    '-c',
    fixtureConfigPath({
      fixturePackageName: 'oxlint-no-restricted-syntax',
      fileName: `.oxlintrc.${rule}.fixture.json`,
    },),
    target,
  ];
  /** Environment of the run, with the fault injection loaded first when asked for. */
  const environment = refuseSpawn
    ? { NODE_OPTIONS: `--import=${REFUSE_NATIVE_SPAWN}`, }
    : {};
  /** Process result whether Oxlint exits zero or not. */
  const settled = await spawn('oxlint', args, { cwd: ROOT, env: environment, },)
    .then(
    function succeeded(result,): { stdout: string; stderr: string; exitCode: number; } {
      return { stdout: result.stdout, stderr: result.stderr, exitCode: 0, };
    },
    function failed(error: unknown,): { stdout: string; stderr: string; exitCode: number; } {
      if (((typeof error) !== 'object') || (error === null) || (!('stdout' in error)) || (!('stderr' in error))
        || (!('exitCode' in error)))
        throw error;
      return {
        stdout: String(error.stdout,),
        stderr: String(error.stderr,),
        exitCode: Number(error.exitCode,),
      };
    },
    );
  /** Parsed report, whose diagnostics array is all these cases read. */
  const report = JSON.parse(settled.stdout,) as { readonly diagnostics: readonly ReportedDiagnostic[]; };
  return {
    exitCode: settled.exitCode,
    diagnostics: report.diagnostics,
    stderr: settled.stderr,
  };
}

/**
 Finds the diagnostics that carry no rule code, which is how Oxlint reports a plugin that threw.

 @param outcome - Result of one run.

 @returns diagnostics without a rule code.
 */
function pluginFailures(outcome: OxlintOutcome,): readonly ReportedDiagnostic[] {
  return outcome.diagnostics.filter(function hasNoCode(diagnostic,): boolean {
    return diagnostic.code === undefined;
  },);
}

/**
 Sources on which the rules answer differently depending on whether the bridge could type them.
 */
const CASES: readonly {
  readonly name: string;
  readonly rule: SpreadRule;
  readonly source: string;
  readonly typedReport: string;
}[] = [
  {
    name: 'an Array.from the rule drops when it has no type',
    rule: 'prefer-spread',
    source: 'export const fromArray = Array.from(toppings,);',
    typedReport: 'no-restricted-syntax(prefer-spread)',
  },
  {
    name: 'a slice the rule reports as ambiguous when it has no type',
    rule: 'prefer-spread',
    source: 'export const copy = toppings.slice();',
    typedReport: 'no-restricted-syntax(prefer-spread)',
  },
  {
    name: 'a spread the rule reports as ambiguous when it has no type',
    rule: 'no-useless-spread',
    source: 'export const firstTwo = [...toppings.slice(0, 2,),];',
    typedReport: 'no-restricted-syntax(no-useless-spread)',
  },
];

//endregion Helpers

await describe({
  name: 'spread rules when the TypeScript bridge cannot start',
  concurrency: 1,
  children: CASES.map(function describeCase(testCase,) {
    return describe({
      name: testCase.name,
      concurrency: 1,
      children: [
        it({
          name: 'reports through the bridge when it starts',
          fn: async () => {
            using scratch = scratchProject(testCase.source,);
            const outcome = await lint({
              rule: testCase.rule,
              target: scratch.filePath,
              refuseSpawn: false,
            },);

            expect(pluginFailures(outcome,),).toEqual([],);
            expect(outcome.diagnostics.map(function codeOf(diagnostic,): string {
              return diagnostic.code ?? 'no rule code';
            },),).toEqual([testCase.typedReport,],);
            expect(outcome.stderr,).not.toContain('refused',);
          },
        },),
        it({
          name: 'fails the lint with a diagnostic naming the spawn failure and how to lower the worker count',
          fn: async () => {
            using scratch = scratchProject(testCase.source,);
            const outcome = await lint({
              rule: testCase.rule,
              target: scratch.filePath,
              refuseSpawn: true,
            },);
            const failures = pluginFailures(outcome,);

            // The fault injection reached the native spawn, so the failure below is the one under test.
            expect(outcome.stderr,).toContain('refused 1 native spawns',);
            expect(outcome.exitCode,).not.toBe(0,);
            expect(failures.length,).toBe(1,);
            expect(failures[0]?.severity,).toBe('error',);
            expect(failures[0]?.message,).toContain(scratch.filePath,);
            expect(failures[0]?.message,).toContain('TypeScript',);
            expect(failures[0]?.message,).toContain('failed to start',);
            expect(failures[0]?.message,).toContain('spawn ENOMEM',);
            expect(failures[0]?.message,).toContain('--threads',);
            // A degraded report would name the rule and hide the cause.
            expect(outcome.diagnostics.filter(function ownRule(diagnostic,): boolean {
              return diagnostic.code === testCase.typedReport;
            },),).toEqual([],);
          },
        },),
      ],
    },);
  },),
},);
