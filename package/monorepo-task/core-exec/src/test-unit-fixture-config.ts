/**
 Verbatim root mise configuration for the shared `test:unit` regression fixtures. @module
 */

import {
  configurationSpan,
  variableSection,
} from './build-and-test-fixture-config.ts';

//region Configuration extraction: the template and its helpers stay verbatim.

/**
 Extract one root task template together with its interpreter and usage spec.

 @param name - template whose definition is exercised

 @returns original TOML task-template section

 @example
 ```ts
 templateSection('test:unit');
 ```
 */
function templateSection(name: string,): string {
  return configurationSpan({
    start: `[task_templates."${name}"]\n`,
    end: '\n[task_templates.',
  },);
}

/**
 Root configuration under which a disposable package runs the real shared `test:unit` template.

 Every variable the template interpolates, and the template itself, is copied from the root file, so a moved boundary
 fails here instead of exercising a reconstruction. The exemption list is the repository's own, which is what makes a
 stale entry visible.

 @example
 ```ts
 writeFile(join(root, 'mise.toml'), testUnitFixtureConfig);
 ```
 */
export const testUnitFixtureConfig: string = `monorepo_root = true
[monorepo]
config_roots = ["package/*/*"]
[settings]
experimental = true
task.disable_spec_from_run_scripts = true
[vars]
${variableSection('parse_usage_args',)}
${variableSection('is_inactive_test',)}
${variableSection('run_test_files',)}
${variableSection('unit_test_exempt_packages',)}
${templateSection('test:unit',)}
[tasks."test:unit"]
extends = "test:unit"
`;

//endregion Configuration extraction
