import { symlink } from 'node:fs/promises';
import {
  join,
  resolve,
} from 'node:path';

import {
  createFakePodman,
  type FakePodman,
} from './fake-podman-fixture.ts';
import {
  type ArgumentSet,
  scannerArgumentSets,
} from './golden-argument-sets.ts';
import {
  linterArgumentSets,
  linterEnvironment,
} from './golden-linter-sets.ts';
import {
  nativeArgumentSets,
  nativeEnvironment,
} from './golden-native-sets.ts';
import {
  type GoldenOutcome,
  normalizeRun,
  packageRoot,
  pathReplacements,
} from './golden-normalize.ts';
import { taskSource } from './golden-task-source.ts';
import {
  runInlineSource,
  runScript,
  runShellCommand,
  type ScriptRun,
} from './run-fixture-program.ts';

/**
 One run of a product's mutation script whose observable behavior is pinned.
 */
export type GoldenScenario = {
  readonly name: string;
  /**
   How the behavior is reached: a Node script, or a mise task, run as the task's `shell` setting says (TypeScript
   source or a shell command).
   */
  readonly kind: 'script' | 'task';
  /**
   Script path for `script`, mise file path for a task.
   */
  readonly script: string;
  /**
   Arguments of the script; the task name for a task.
   */
  readonly args: readonly string[];
  readonly environment: Readonly<Record<string, string>>;
};

/**
 Absolute path of a product file.

 @param relative - path below the repository's `package` directory
 
 @returns the absolute path
 
 @example
 ```ts
 productFile('git-policy/cli/bin/mutate-native-container.ts');
 ```
 */
function productFile(relative: string): string {
  return resolve(
    packageRoot,
    relative,
  );
}

/**
 Build the script scenarios of one product.

 @param product - name prefix of the scenarios
 
 @param script - path of the product script
 
 @param base - environment every scenario of the product has
 
 @param sets - the option spreads
 
 @returns one scenario per spread
 
 @example
 ```ts
 scriptScenarios({ product: 'linter', script, base: {}, sets: linterArgumentSets });
 ```
 */
function scriptScenarios({
  product,
  script,
  base,
  sets,
}: {
  readonly product: string;
  readonly script: string;
  readonly base: Readonly<Record<string, string>>;
  readonly sets: readonly ArgumentSet[];
}): readonly GoldenScenario[] {
  return sets.map(function scenarioOf([name, args, environment]) {
    return {
      kind: 'script' as const,
      name: `${product}-${name}`,
      script,
      args,
      environment: {
        ...base,
        ...environment,
      },
    };
  });
}

/**
 Every pinned scenario of the products whose campaign scripts are shared.

 @returns the scenarios, native wrapper first
 
 @example
 ```ts
 goldenScenarios();
 ```
 */
export function goldenScenarios(): readonly GoldenScenario[] {
  return [
    ...scriptScenarios({
      product: 'git-policy',
      script: productFile('git-policy/cli/bin/mutate-native-container.ts'),
      base: nativeEnvironment,
      sets: nativeArgumentSets,
    }),
    ...scriptScenarios({
      product: 'linter',
      script: productFile('linter/monochromatic-lint/bin/mutate-container.mjs'),
      base: linterEnvironment,
      sets: linterArgumentSets,
    }),
    {
      kind: 'task',
      name: 'linter-inline-processors-campaign',
      script: productFile('linter/monochromatic-lint/mise.toml'),
      args: ['mutation:processors'],
      environment: {
        ...linterEnvironment,
        FAKE_OUTCOMES: '1',
      },
    },
    {
      kind: 'task',
      name: 'linter-list-processors-task',
      script: productFile('linter/monochromatic-lint/mise.toml'),
      args: ['mutation:list:processors'],
      environment: {},
    },
    ...scriptScenarios({
      product: 'scanner',
      script: productFile('forbidden-strings/cli/bin/mutate-container.mjs'),
      base: {},
      sets: scannerArgumentSets,
    }),
  ];
}

/**
 Start a scenario by the means its kind names.

 @param fake - fixture
 
 @param scenario - the scenario
 
 @returns what it did
 
 @example
 ```ts
 await runKind({ fake, scenario });
 ```
 */
async function runKind({
  fake,
  scenario,
}: {
  readonly fake: FakePodman;
  readonly scenario: GoldenScenario;
}): Promise<ScriptRun> {
  if (scenario.kind === 'script')
    return runScript({
      fake,
      script: scenario.script,
      args: scenario.args,
      extraEnvironment: scenario.environment,
    });
  /**
   Package directory that holds the mise file.
   */
  const packageDirectory = resolve(
    scenario.script,
    '..',
  );
  /**
   The task's run text and how the task runs it.
   */
  const {
    source,
    inline
  } = await taskSource({
    miseFile: scenario.script,
    task: scenario.args[0] ?? '',
  });
  if (!inline)
    return runShellCommand({
      fake,
      command: source,
      workingDirectory: packageDirectory,
      extraEnvironment: scenario.environment,
    });
  // The task source resolves workspace packages from its directory, as a mise task does from the package.
  await symlink(
    join(
      packageDirectory,
      'node_modules',
    ),
    join(
      fake.workingDirectory,
      'node_modules',
    ),
  );
  return runInlineSource({
    fake,
    source,
    extraEnvironment: scenario.environment,
  });
}

/**
 Run one scenario in a fresh fixture and return its normalized outcome.

 @param scenario - the scenario
 
 @returns the normalized outcome
 
 @example
 ```ts
 await runScenario(goldenScenarios()[0]);
 ```
 */
export async function runScenario(scenario: GoldenScenario): Promise<GoldenOutcome> {
  /**
   Stand-in programs of this scenario, removed when the function ends.
   */
  await using fake = await createFakePodman();
  /**
   What the program did.
   */
  const run = await runKind({
    fake,
    scenario,
  });
  return normalizeRun({
    run,
    replacements: await pathReplacements(fake),
  });
}
