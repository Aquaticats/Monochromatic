import type { CommandName, } from './command-lines.ts';

//region Command owners
// WHICH PACKAGE OWNS EACH RUNNER, because the package's task is what starts it.
//
// The family was split out of `package/module/translation-repair` into one package per part, and each runner's
// task now lives in the `mise.toml` of the package that builds the runner (`build-entries.ts`). A refusal that
// tells an operator which task to start must name that package, and the runner's own name cannot say which it is.
//
// ONE TABLE, KEYED BY THE SAME NAMES AS `COMMAND_LINES`, so a runner absent from either cannot call
// `reportingRefusals`, and the compiler holds the two to each other key for key. The table is held to the build
// entries the packages list (`source-scan/src/corpus-run/command-lines.unit.test.ts`), so an owner cannot name a
// package whose `build-entries.ts` does not list the runner.
//
// NO FALLBACK. A runner with no owner here does not compile, and a path is never built from a guess.

/**
 Directory name, under `package/translation-repair/`, of the package that owns each runner and so its task.

 @example
 ```ts
 const owner = COMMAND_OWNERS['meter-report']; // 'run'
 ```
 */
export const COMMAND_OWNERS: Readonly<Record<CommandName, string>> = {
  'audit-sensitivity': 'probe',
  'budget-sample': 'run',
  'cache-account-audit': 'slice-cache',
  'cap-census': 'run',
  'checker-sensitivity': 'bench',
  'corpus-pass': 'corpus-pass',
  'coverage-census': 'coverage-census',
  'coverage-control-probe': 'coverage-census',
  'coverage-probe': 'coverage-census',
  'damage-sample': 'probe',
  'displacement-probe': 'bench',
  'draw-sample': 'probe',
  'editor-calibrate': 'bench',
  'editor-fidelity-pool-draw': 'probe',
  'editor-fidelity-probe': 'probe',
  'editor-fidelity-read': 'probe',
  'editor-standing-read': 'bench',
  'editor-width-probe': 'bench',
  'judge-fidelity-probe': 'probe',
  'ledger-report': 'run',
  'meter-report': 'run',
  'model-catalog': 'bench',
  'model-health': 'bench',
  'probe-relabel': 'probe',
  'probe-sensitivity': 'probe',
  'probe-verify': 'probe',
  'producer-calibrate': 'bench',
  'reader-measure': 'bench',
  'recall-benchmark': 'probe',
  'repeat-census': 'run',
  'rendering-audit-settled': 'probe',
  'rendering-audit-settled-report': 'probe',
  'roster-bench': 'bench',
  'roster-card': 'bench',
  'run-timing-report': 'run',
  'score-agreement': 'probe',
  'score-attribution': 'probe',
  'score-crosscheck': 'probe',
  'score-probe': 'probe',
  'score-verify': 'probe',
  'sentinel-probe': 'bench',
  'slice-census': 'slice-cache',
  'slice-cost-report': 'slice-cache',
  'spend-report': 'run',
  'translate-probe': 'bench',
  'verify-published': 'run',
  'window-trial-probe': 'bench',
};

/**
 The task that starts a runner, written from the repository root as `mise run` takes it.

 @param command - runner's name, which names its task and, through the owner table, the package whose
 `mise.toml` holds it

 @returns The task path, such as `//package/translation-repair/run:meter-report`

 @example
 ```ts
 const task = taskPathOf({ command: 'meter-report', },); // '//package/translation-repair/run:meter-report'
 ```
 */
export function taskPathOf({ command, }: { readonly command: CommandName; },): string {
  return `//package/translation-repair/${COMMAND_OWNERS[command]}:${command}`;
}

//endregion Command owners
