/**
 Verbatim root mise sections the root test aggregate fixture exercises. @module
 */

import {
  configurationSpan,
  rootConfig,
  variableSection,
} from './build-and-test-fixture-config.ts';

//region Section extraction: keep the aggregate and its fanout as authored.

/**
 Extract one quoted root task section, boundary and all.

 @param name - task whose own quoted definition is exercised

 @returns original TOML task section including its closing delimiter

 @throws Error when the fixture's expected task boundary is absent

 @example
 ```ts
 quotedTaskSection('test:packages');
 ```
 */
function quotedTaskSection(name: string,): string {
  return configurationSpan({
    start: `[tasks."${name}"]\n`,
    end: '\n[tasks.',
  },);
}

/**
 Extract the shared TypeScript unit template, which owns the discovery rule.

 @returns original TOML template section including its closing delimiter

 @throws Error when the template boundary is absent

 @example
 ```ts
 unitTemplateSection();
 ```
 */
function unitTemplateSection(): string {
  return configurationSpan({
    start: '[task_templates."test:unit"]\n',
    end: '\n[task_templates.',
  },);
}

/**
 Extract the aggregate itself, whose suite list and phase order are the subject.

 @returns original TOML root test section including its closing delimiter

 @throws Error when the aggregate boundary is absent

 @example
 ```ts
 aggregateTaskSection();
 ```
 */
function aggregateTaskSection(): string {
  return configurationSpan({
    start: '[tasks.test]\n',
    end: '\n[tasks.',
  },);
}

//endregion Section extraction

/**
 Disposable configuration: the real aggregate, fanout and unit template over fake suites.

 The three root suites keep their real names so the aggregate dispatches them, and
 only their implementations become fixture scripts, because a Playwright container
 and a real unit suite are not what this fixture measures.

 @returns complete TOML configuration for one disposable repository

 @throws Error when a section this fixture measures is absent from the root configuration

 @example
 ```ts
 await writeFile(join(root, 'mise.toml'), fixtureConfiguration(),);
 ```
 */
export function fixtureConfiguration(): string {
  return `monorepo_root = true
[monorepo]
config_roots = ["package/*/*"]
[settings]
experimental = true
task.disable_spec_from_run_scripts = true
[vars]
${variableSection('mise_tasks_json_max_buffer',)}
${variableSection('parse_usage_args',)}
${variableSection('is_inactive_test',)}
${variableSection('run_test_files',)}
${variableSection('unit_test_exempt_packages',)}
${variableSection('fanout_packages',)}
${unitTemplateSection()}
${quotedTaskSection('test:unit',)}
${quotedTaskSection('test:packages',)}
${aggregateTaskSection()}
[tasks."test:browser"]
run = "node fixture-browser.ts"
[tasks."test:e2e"]
run = "node fixture-e2e.ts"
`;
}

/**
 Inputs independently select which suite fails.
 */
export type RootTestAggregateFixture = {
  /**
   Root browser suite failure must not stop the package suites.
   */
  readonly browserFails?: boolean;
  /**
   One package suite failing must fail the aggregate and name that package.
   */
  readonly ownedTestFails?: boolean;
};

/**
 Fixture output includes a file-backed trace independent of displayed commands.
 */
export type RootTestAggregateResult = {
  /**
   Suite identifiers sorted, because the aggregate starts its root suites together.
   */
  readonly events: readonly string[];
  /**
   Both streams retain the aggregate and child diagnostics.
   */
  readonly output: string;
  /**
   Numeric status is returned only after normal child exit, never signal termination.
   */
  readonly exitCode: number;
};
