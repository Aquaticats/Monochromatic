/**
 Verbatim root mise configuration used by disposable mise regression fixtures. @module
 */

import { readFile, } from 'node:fs/promises';

/**
 Root configuration consumed by mise, not a reconstructed implementation.
 */
export const rootConfig: string = await readFile(
  new URL(
    '../../../../mise.toml',
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

 @param start - unique opening text in the root configuration

 @param end - closing boundary excluded from the result

 @returns original text without reimplementing the task

 @throws Error if the fixture's expected configuration shape is absent

 @example
 ```ts
 configurationSpan({ start: '[tasks.try]\n', end: '\n[tasks.', });
 ```
 */
export function configurationSpan({
  start,
  end,
}: {
  readonly start: string;
  readonly end: string
},): string {
  /**
   Offset of the exact task or variable definition.
   */
  const startIndex = rootConfig.indexOf(start,);
  /**
   Closing boundary after the opening text, not inside it.
   */
  const endIndex = rootConfig.indexOf(
    end,
    startIndex + start.length,
  );
  if ((startIndex === (-1)) || (endIndex === (-1)))
    throw new Error(`Missing mise fixture configuration boundary: ${start} to ${end}`,);
  return rootConfig.slice(
    startIndex,
    endIndex,
  );
}

/**
 Extract a root task together with its original interpreter and usage spec.

 @param name - task whose definition is exercised

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
export function variableSection(name: string,): string {
  return `${configurationSpan({
    start: `${name} = """\n`,
    end: '\n"""',
  },)}\n"""`;
}

/**
 Extract a root variable whose value fits on one line, quotes included.

 `variableSection` expects a triple-quoted body and would swallow the rest of the file for a one-line
 assignment, but a fixture section can interpolate such a variable too: the forwarded-arguments marker is
 declared that way and `parse_usage_args` interpolates it.

 @param name - single-line variable a copied root section interpolates

 @returns the original TOML assignment line

 @throws Error when the root configuration has no single-line assignment of that name

 @example
 ```ts
 oneLineVariable('forwarded_arguments_marker');
 ```
 */
export function oneLineVariable(name: string,): string {
  /**
   The opening of a one-line declaration, up to and including its first quote.
   */
  const prefix = `${name} = "`;
  /**
   The first line that starts like a declaration of this name, whichever form it takes.
   */
  const line = rootConfig
    .split('\n',)
    .find(function declares(entry,): boolean {
      return entry.startsWith(prefix,);
    },);
  if (line === undefined) throw new Error(`Missing one-line root variable: ${name}`,);
  /**
   A triple-quoted declaration opens with a second quote right after the prefix, and its closing delimiter sits on
   its own later line, so a line of that form would not end with a quote.
   */
  if ((line[prefix.length] === '"') || (!line.endsWith('"',)))
    throw new Error(`Root variable is not a one-line assignment: ${name}`,);
  return line;
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
${variableSection('mise_tasks_json_max_buffer',)}
${oneLineVariable('forwarded_arguments_marker',)}
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
