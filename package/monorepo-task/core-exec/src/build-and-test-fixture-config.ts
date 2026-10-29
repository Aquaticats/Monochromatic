/**
 Verbatim generated configuration used by disposable mise regression fixtures. @module
 */

import { readFile, } from 'node:fs/promises';

/**
 Generated configuration consumed by mise, not a reconstructed implementation.
 */
export const generatedConfig: string = await readFile(
  new URL(
    '../../../../mise.toml',
    import.meta.url,
  ),
  'utf8',
);

/**
 Source owner used to detect stale generated task definitions.
 */
export const canonicalConfig: string = await readFile(
  new URL(
    '../../../../mise.no-env.toml',
    import.meta.url,
  ),
  'utf8',
);

/**
 Inputs independently select execution shape and phase outcomes.
 */
export type BuildAndTestFixture = {
  /**
   Repository-relative file arguments exercise the real usage parser.
   */
  readonly args: readonly string[];
  /**
   Build failure must survive passing tests.
   */
  readonly buildFails: boolean;
  /**
   Missing package build tasks retain their existing skip semantics.
   */
  readonly packageBuild?: boolean;
  /**
   Preparation failure is exercised only by the explicitly tolerant aggregate.
   */
  readonly prepareFails?: boolean;
  /**
   Entry point distinguishes ordinary orchestration from intentional tolerance.
   */
  readonly task?: string;
  /**
   Test failure must fail the aggregate independently of build status.
   */
  readonly testFails: boolean;
};

/**
 Fixture output includes an execution trace independent of displayed commands.
 */
export type BuildAndTestResult = {
  /**
   File-backed phase order proves which subprocesses actually executed.
   */
  readonly events: readonly string[];
  /**
   Both streams retain the aggregate and original child diagnostics.
   */
  readonly output: string;
  /**
   Numeric status is returned only after normal child exit, never signal termination.
   */
  readonly exitCode: number;
};

//region Configuration extraction: preserve task and helper bodies verbatim.

/**
 Extract one required configuration span, failing if its boundary changes.

 @param start - unique opening text in generated configuration

 @param end - closing boundary excluded from the result

 @returns original text without reimplementing the task

 @throws Error if the fixture's expected configuration shape is absent

 @example
 ```ts
 configurationSpan({ start: '[tasks.try]\n', end: '\n[tasks.', });
 ```
 */
function configurationSpan({
  start,
  end,
}: {
  readonly start: string;
  readonly end: string
},): string {
  /**
   Offset of the exact task or variable definition.
   */
  const startIndex = generatedConfig.indexOf(start,);
  /**
   Closing boundary after the opening text, not inside it.
   */
  const endIndex = generatedConfig.indexOf(
    end,
    startIndex + start.length,
  );
  if ((startIndex === (-1)) || (endIndex === (-1)))
    throw new Error(`Missing mise fixture configuration boundary: ${start} to ${end}`,);
  return generatedConfig.slice(
    startIndex,
    endIndex,
  );
}

/**
 Extract a root task together with its original interpreter and usage spec.

 @param name - task whose generated definition is exercised

 @returns original TOML task section

 @example
 ```ts
 taskSection('buildAndTest');
 ```
 */
function taskSection(name: string,): string {
  return configurationSpan({
    start: `[tasks.${name}]\n`,
    end: '\n[tasks.',
  },);
}

/**
 Extract a shared triple-quoted helper including its closing delimiter.

 @param name - helper referenced by the task's Tera template

 @returns original TOML variable assignment

 @example
 ```ts
 variableSection('parse_usage_args');
 ```
 */
function variableSection(name: string,): string {
  return `${configurationSpan({
    start: `${name} = """\n`,
    end: '\n"""',
  },)}\n"""`;
}

/**
 Shared root config replaces only build/test implementations, not orchestration.
 */
export const fixtureConfig: string = `experimental_monorepo_root = true
[monorepo]
config_roots = ["package/*/*"]
[settings]
experimental = true
task.disable_spec_from_run_scripts = true
[vars]
${variableSection('parse_usage_args',)}
${variableSection('run_test_files',)}
${taskSection('try',)}
${taskSection('buildAndTest',)}
${taskSection('prepareAndBuild--allowFailure',)}
[tasks.build]
run = "node root-build.ts"
[tasks.prepare]
run = "node prepare.ts"
[tasks.test]
run = "node test.ts"
`;

//endregion Configuration extraction
