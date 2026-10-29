import {
  copyFile,
  mkdtemp,
  rm,
} from 'node:fs/promises';
import { scratchRoot, } from '@monochromatic-dev/test-harness-core/ts/scratch-root.ts';
import {
  isAbsolute,
  join,
  resolve,
} from 'node:path';

import {
  findRoot,
  MISE_MONOREPO,
} from '@monochromatic-dev/fs-path-core/ts';
import spawn from 'nano-spawn';
import dedent from 'string-dedent';

/**
 Shared dev-only fixtures for the oxlint plugin unit tests.
 
 This file intentionally lives outside the published plugin packages. It is
 imported only by colocated unit tests, so shared test process setup stays in
 one place without adding runtime code to any shipped plugin.
 
 @module
 */

/**
 Workspace root resolved from this shared test-support directory.
 
 @example
 ```ts
 OXLINT_PLUGIN_TEST_ROOT;
 ```
 */
export const OXLINT_PLUGIN_TEST_ROOT: string = await findRoot({
  cwd: import.meta.dirname,
  marker: MISE_MONOREPO,
},);

/**
 Single diagnostic from oxlint JSON output after plugin filtering.
 */
export type OxlintRuleDiagnostic = {
  /**
   Human-readable diagnostic message.
   */
  readonly message: string;
  /**
   Rule identifier in `plugin(rule-name)` format.
   */
  readonly code: string;
  /**
   Diagnostic severity reported by oxlint.
   */
  readonly severity: string;
  /**
   Source file path reported by oxlint.
   */
  readonly filename: string;
  /**
   Label spans oxlint attaches to the diagnostic, each locating the offending token.
   */
  readonly labels?: readonly {
    /**
     Exact source range the label names.
     */
    readonly span: {
      /**
       UTF-16 offset of the range start in the linted source.
       */
      readonly offset: number;
      /**
       UTF-16 length of the range.
       */
      readonly length: number;
      /**
       One-based line of the range start.
       */
      readonly line: number;
      /**
       One-based column of the range start.
       */
      readonly column: number;
    };
  }[];
};

/**
 Raw diagnostic shape emitted by oxlint before runner-level diagnostics are filtered out.
 */
type OxlintDiagnostic = Omit<OxlintRuleDiagnostic, 'code'> & {
  /**
   Rule identifier, absent for runner-level diagnostics.
   */
  readonly code?: string;
};

/**
 Top-level oxlint `--format json` output.
 */
type OxlintOutput = {
  /**
   Diagnostics emitted for the target file.
   */
  readonly diagnostics: readonly OxlintDiagnostic[];
};

/**
 Disposable temp copy of a fixture file.
 */
export type TempFixtureFile = {
  /**
   Absolute path to copied fixture file.
   */
  readonly filePath: string;
  /**
   Removes temp directory that contains fixture copy.
   */
  readonly [Symbol.asyncDispose]: () => Promise<void>;
};

/**
 Options for locating one test-fixture package.
 */
export type FixturePackageRootParams = {
  /**
   Directory name under `package/test-fixture`.
   */
  readonly fixturePackageName: string;
};

/**
 Options for locating a fixture config file.
 */
export type FixtureConfigPathParams = FixturePackageRootParams & {
  /**
   Config filename inside the fixture package.
   */
  readonly fileName: string;
};

/**
 Options for resolving a lint target.
 */
export type ResolveFixtureTargetParams = {
  /**
   Source fixture root used for relative fixture paths.
   */
  readonly fixtureSourceRoot: string;
  /**
   Relative fixture path or absolute temp fixture path.
   */
  readonly fixturePath: string;
};

/**
 Which fixes an oxlint run writes back to the target file.
 
 `fixes` passes `--fix`; `fixes-and-suggestions` also passes `--fix-suggestions`.
 */
export type OxlintFixMode = 'fixes' | 'fixes-and-suggestions';

/**
 Options for running oxlint against a fixture target.
 */
export type RunOxlintFixtureParams = {
  /**
   Diagnostic code prefix to keep, including opening parenthesis.
   */
  readonly codePrefix: `${string}(`;
  /**
   Config flag spelling accepted by the plugin's fixture setup.
   */
  readonly configFlag: '-c' | '--config';
  /**
   Absolute path to fixture oxlint config.
   */
  readonly fixtureConfig: string;
  /**
   Absolute path to target source file.
   */
  readonly target: string;
  /**
   Optional Oxlint worker count for resource-sensitive plugin tests.
   */
  readonly threads?: number;
  /**
   Fixes to write back to the target. When absent the run only lints and leaves the target untouched.
   */
  readonly fix?: OxlintFixMode;
  /**
   Variables added to the environment of the oxlint process, such as a fault injection loaded through
   `NODE_OPTIONS`. Absent variables keep the environment of the test process.
   */
  readonly environment?: Readonly<Record<string, string>>;
};

/**
 Options for copying a source fixture into a disposable temp directory.
 */
export type CreateTempFixtureFileParams = {
  /**
   Basename for copied temp file.
   */
  readonly fileName: string;
  /**
   Source fixture path to copy into temp directory.
   */
  readonly sourcePath: string;
  /**
   Temp directory prefix passed to `mkdtempSync`.
   */
  readonly tempPrefix: string;
};

/**
 Returns absolute package root for one oxlint test fixture package.
 
 @param params - fixture package name to resolve
 
 @returns absolute path under `package/test-fixture`
 
 @example
 ```ts
 fixturePackageRoot({ fixturePackageName: 'oxlint-tsdoc' });
 ```
 */
export function fixturePackageRoot(params: FixturePackageRootParams,): string {
  /**
   Fixture package name to resolve under `package/test-fixture`.
   */
  const { fixturePackageName, } = params;
  return resolve(
    OXLINT_PLUGIN_TEST_ROOT,
    'package',
    'test-fixture',
    fixturePackageName,
  );
}

/**
 Returns absolute `src` root for one oxlint test fixture package.
 
 @param params - fixture package name to resolve
 
 @returns absolute path to fixture source directory
 
 @example
 ```ts
 fixtureSourceRoot({ fixturePackageName: 'oxlint-stylistic' });
 ```
 */
export function fixtureSourceRoot(params: FixturePackageRootParams,): string {
  return resolve(
    fixturePackageRoot(params,),
    'src',
  );
}

/**
 Returns absolute path to a fixture oxlint config file.
 
 @param params - fixture package and config filename
 
 @returns absolute config path
 
 @example
 ```ts
 fixtureConfigPath({ fixturePackageName: 'oxlint-tsdoc', fileName: '.oxlintrc.fixture.json' });
 ```
 */
export function fixtureConfigPath(params: FixtureConfigPathParams,): string {
  /**
   Fixture package name and config filename to resolve.
   */
  const {
    fixturePackageName,
    fileName,
  } = params;
  return resolve(
    fixturePackageRoot({ fixturePackageName, },),
    fileName,
  );
}

/**
 Resolves relative fixture names while preserving absolute temp paths.
 
 @param params - fixture source root and user-supplied path
 
 @returns absolute lint target path
 
 @example
 ```ts
 resolveFixtureTarget({ fixtureSourceRoot: FIXTURES, fixturePath: 'invalid/file.ts' });
 ```
 */
export function resolveFixtureTarget(params: ResolveFixtureTargetParams,): string {
  /**
   Fixture source root and target path to resolve.
   */
  const {
    fixtureSourceRoot: sourceRoot,
    fixturePath,
  } = params;
  return isAbsolute(fixturePath,)
    ? fixturePath
    : resolve(
      sourceRoot,
      fixturePath,
    );
}

/**
 Text oxlint printed, and the command line that printed it.
 */
type OxlintProcessOutput = {
  /**
   Command line oxlint was started with, as nano-spawn reports it.
   */
  readonly command: string;
  /**
   Captured standard output.
   */
  readonly stdout: string;
  /**
   Captured standard error.
   */
  readonly stderr: string;
};

/**
 Error thrown when oxlint prints text that is not the JSON the runner asked for.

 Oxlint prints a failure text instead of JSON when it cannot load a plugin or read its config, such as when a
 plugin's `createOnce` throws, so the message carries the whole output for the reader to act on.
 */
export class OxlintOutputParseError extends Error {
  /**
   Builds the error from the run that printed the text.

   @param params - target, config and process output of the run, and the JSON parse failure

   @example
   ```ts
   new OxlintOutputParseError({ target, fixtureConfig, output, cause });
   ```
   */
  constructor(
    params: {
      readonly target: string;
      readonly fixtureConfig: string;
      readonly output: OxlintProcessOutput;
      readonly cause: unknown;
    },
  ) {
    /**
     Run details the message reports.
     */
    const {
      target,
      fixtureConfig,
      output,
      cause,
    } = params;
    super(
      dedent`
        oxlint did not print JSON while linting ${target} with the config ${fixtureConfig}.
        Command: ${output.command}
        Standard output:
        ${output.stdout}
        Standard error:
        ${output.stderr}
      `,
      { cause, },
    );
    this.name = 'OxlintOutputParseError';
  }
}

/**
 Returns the output nano-spawn stores on errors when oxlint exits non-zero.

 @param error - thrown value from `spawn`

 @returns captured stdout, with stderr and the command line when nano-spawn stored them

 @throws original thrown value when stdout is not available

 @example
 ```ts
 const output = outputFromSpawnError({ error });
 ```
 */
function outputFromSpawnError({ error, }: { readonly error: unknown; },): OxlintProcessOutput {
  if (((typeof error) !== 'object') || (error === null)) {
    throw new Error(
      'oxlint spawn failure did not expose stdout.',
      { cause: error, },
    );
  }
  if (!('stdout' in error)) {
    throw new Error(
      'oxlint spawn failure did not expose stdout.',
      { cause: error, },
    );
  }
  /**
   Captured process stdout attached by nano-spawn.
   */
  const { stdout, } = error;
  if ((typeof stdout) !== 'string') {
    throw new Error(
      'oxlint spawn failure stdout was not a string.',
      { cause: error, },
    );
  }
  return {
    command: ('command' in error) && ((typeof error.command) === 'string') ? error.command : 'oxlint',
    stdout,
    stderr: ('stderr' in error) && ((typeof error.stderr) === 'string') ? error.stderr : '',
  };
}

/**
 Parses oxlint JSON output into typed diagnostics.
 
 @param params - target, config and process output of the run
 
 @returns parsed oxlint output
 
 @throws {@link OxlintOutputParseError} naming the target and config and carrying the stdout, stderr and command
 when the stdout is not JSON, with the JSON parse failure as `cause`
 
 @example
 ```ts
 const output = parseOxlintOutput({ target, fixtureConfig, output: processOutput });
 ```
 */
function parseOxlintOutput(
  params: {
    readonly target: string;
    readonly fixtureConfig: string;
    readonly output: OxlintProcessOutput;
  },
): OxlintOutput {
  /**
   Run details the failure message reports.
   */
  const {
    target,
    fixtureConfig,
    output: processOutput,
  } = params;
  try {
    /* oxlint-disable typescript/no-unsafe-assignment -- JSON.parse returns unknown JSON; OxlintOutput validates the consumed shape for tests. */
    /**
     Parsed oxlint output trusted by fixture tests.
     */
    const output: OxlintOutput = JSON.parse(processOutput.stdout,);
    /* oxlint-enable typescript/no-unsafe-assignment */
    return output;
  }
  catch (error: unknown) {
    throw new OxlintOutputParseError({
      target,
      fixtureConfig,
      output: processOutput,
      cause: error,
    },);
  }
}

/**
 Returns the oxlint flags that write fixes back to the target.
 
 @param fix - requested fix mode
 
 @returns the flags of the mode
 
 @example
 ```ts
 fixFlags('fixes-and-suggestions');
 ```
 */
function fixFlags(fix: OxlintFixMode,): readonly string[] {
  if (fix === 'fixes')
    return ['--fix',];
  return [
    '--fix',
    '--fix-suggestions',
  ];
}

/**
 Runs oxlint with a fixture config and filters diagnostics to one plugin.

 Oxlint reports a plugin that threw, and a source it could not parse, as a diagnostic with no rule code. Keeping
 only the diagnostics of one plugin would drop those silently, and a fixture expecting no findings would pass
 while the plugin crashed, so such a diagnostic fails the run instead.

 A fix run (the `fix` option) rewrites the target and applies the same rule. A plugin that crashes fixes
 nothing, so a test that reads the target back to show a source was left alone would otherwise pass through
 the crash.

 @param params - oxlint fixture invocation details

 @returns plugin diagnostics left after the run, sorted exactly as oxlint emitted them

 @throws {@link Error} naming the target and every message when oxlint emitted a diagnostic with no rule code

 @throws {@link OxlintOutputParseError} when oxlint printed text instead of JSON, such as when a plugin failed to load

 @example
 ```ts
 await runOxlintFixture({ codePrefix: 'tsdoc(', configFlag: '--config', fixtureConfig, target });
 ```
 */
export async function runOxlintFixture(
  params: RunOxlintFixtureParams,
): Promise<readonly OxlintRuleDiagnostic[]> {
  /**
   Oxlint invocation details for this fixture run.
   */
  const {
    codePrefix,
    configFlag,
    fixtureConfig,
    target,
    threads,
    fix,
    environment,
  } = params;
  /**
   Captures the output of oxlint whether diagnostics make the process exit non-zero or not.
   
   @returns captured oxlint output
   */
  async function captureOutput(): Promise<OxlintProcessOutput> {
    try {
      /**
       Successful oxlint process output.
       */
      const result = await spawn(
        'oxlint',
        [
          ...threads === undefined ? [] : [
            '--threads',
            String(threads,),
          ],
          ...fix === undefined ? [] : fixFlags(fix,),
          '--format',
          'json',
          configFlag,
          fixtureConfig,
          target,
        ],
        {
          cwd: OXLINT_PLUGIN_TEST_ROOT,
          ...environment === undefined ? {} : { env: environment, },
        },
      );
      return {
        command: result.command,
        stdout: result.stdout,
        stderr: result.stderr,
      };
    }
    catch (error: unknown) {
      return outputFromSpawnError({ error, },);
    }
  }

  /**
   Parsed oxlint JSON output for the fixture run.
   */
  const output = parseOxlintOutput({
    target,
    fixtureConfig,
    output: await captureOutput(),
  },);
  /**
   Diagnostics emitted by oxlint before plugin filtering.
   */
  const { diagnostics, } = output;
  /**
   Diagnostics oxlint emitted for something other than a rule, such as a plugin that threw.
   */
  const runnerFailures = diagnostics
    .filter(function hasNoRuleCode(diagnostic,): boolean {
      return (typeof diagnostic.code) !== 'string';
    },);
  if (runnerFailures.length > 0) {
    throw new Error(
      [
        `oxlint emitted ${String(runnerFailures.length,)} diagnostic(s) with no rule code while linting ${target},`,
        'which is how it reports a plugin that threw or a source it could not parse:',
        ...runnerFailures.map(function describeFailure(diagnostic,): string {
          return diagnostic.message;
        },),
      ]
        .join('\n',),
    );
  }
  /**
   Diagnostics belonging to the requested plugin only.
   */
  const pluginDiagnostics = diagnostics
    .filter(
    function keepPluginDiagnostic(
      diagnostic,
    ): diagnostic is OxlintRuleDiagnostic {
      if ((typeof diagnostic.code) !== 'string')
        return false;
      return diagnostic.code
        .startsWith(codePrefix,);
    },
    );
  return pluginDiagnostics;
}

/**
 Creates a temp fixture copy with disposal-backed directory cleanup.
 
 @param params - fixture source, temp basename, and directory prefix
 
 @returns copied temp fixture file handle
 
 @example
 ```ts
 using fixture = createTempFixtureFile({ fileName: 'case.ts', sourcePath, tempPrefix: 'oxlint-case-' });
 ```
 */
export async function createTempFixtureFile(
  params: CreateTempFixtureFileParams,
): Promise<TempFixtureFile> {
  /**
   Temp fixture source, basename, and directory prefix.
   */
  const {
    fileName,
    sourcePath,
    tempPrefix,
  } = params;
  /**
   Unique temp directory owning this fixture copy.
   */
  const dirPath = await mkdtemp(join(
    scratchRoot(),
    tempPrefix,
  ),);
  /**
   Absolute path to temp fixture copy.
   */
  const filePath = resolve(
    dirPath,
    fileName,
  );
  await copyFile(
    sourcePath,
    filePath,
  );

  return {
    filePath,
    [Symbol.asyncDispose]: async function cleanup(): Promise<void> {
      await rm(
        dirPath,
        {
          recursive: true,
          force: true,
        },
      );
    },
  };
}

/**
 Extracts sorted unique rule codes from diagnostics.
 
 @param diagnostics - plugin diagnostics
 
 @returns sorted unique diagnostic codes
 
 @example
 ```ts
 uniqueRuleCodes(diagnostics);
 ```
 */
export function uniqueRuleCodes(
  diagnostics: readonly OxlintRuleDiagnostic[],
): readonly string[] {
  return [
    ...new Set(diagnostics.map(
      function pickCode(diagnostic,): string {
        return diagnostic.code;
      },
    ),),
  ].toSorted();
}
